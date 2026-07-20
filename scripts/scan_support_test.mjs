// Node test for extension/lib/scan_support.js (the U7 scan service logic):
// the concurrency limiter and the capped FIFO session cache. Run:
//
//   node scripts/scan_support_test.mjs
//
// assert-based, no framework; exits non-zero on failure.

import assert from "node:assert/strict";
import {
  actionBadge,
  makeLimiter,
  makeSessionCache,
  pillSpec,
  TIER_BADGE,
} from "../extension/lib/scan_support.js";

// --- limiter: cap respected, FIFO order, failures release the slot ---------
{
  const limit = makeLimiter(2);
  let running = 0;
  let peak = 0;
  const order = [];
  const task = (name) => async () => {
    running++;
    peak = Math.max(peak, running);
    order.push(name);
    await new Promise((r) => setTimeout(r, 10));
    running--;
    return name;
  };
  const results = await Promise.all([
    limit(task("a")),
    limit(task("b")),
    limit(task("c")),
    limit(task("d")),
  ]);
  assert.equal(peak, 2, "at most 2 tasks in flight");
  assert.deepEqual(order, ["a", "b", "c", "d"], "FIFO start order");
  assert.deepEqual(results, ["a", "b", "c", "d"]);

  // A rejecting task must release its slot for the queue.
  const outcome = await Promise.allSettled([
    limit(async () => {
      throw new Error("boom");
    }),
    limit(async () => "after-failure"),
  ]);
  assert.equal(outcome[0].status, "rejected");
  assert.equal(outcome[1].value, "after-failure");
}

// --- cache: get/put roundtrip, FIFO eviction at cap, re-put refreshes ------
{
  // Fake storage area with the storage.get/set object shape.
  const backing = {};
  const area = {
    async get(key) {
      return key in backing ? { [key]: backing[key] } : {};
    },
    async set(obj) {
      Object.assign(backing, obj);
    },
  };
  const cache = makeSessionCache(area, "verdictCache", 3);

  assert.equal(await cache.get("u1"), undefined, "miss before put");
  await cache.put("u1", 1);
  await cache.put("u2", 2);
  await cache.put("u3", 3);
  assert.equal(await cache.get("u1"), 1);

  await cache.put("u1", 11); // re-put: refresh, no eviction (still 3 keys)
  await cache.put("u4", 4); // cap exceeded → evict the OLDEST (u2)
  assert.equal(await cache.get("u2"), undefined, "oldest evicted at cap");
  assert.equal(await cache.get("u1"), 11, "refreshed key survived");
  assert.equal(await cache.get("u3"), 3);
  assert.equal(await cache.get("u4"), 4);
}

// --- presentation: pillSpec / actionBadge cover every entry shape ----------
{
  const verified = {
    srcUrl: "https://cdn.example/a.jpg",
    report: { verdict: "verified", phrase: "Verified: …chain." },
  };
  const pill = pillSpec(verified);
  assert.equal(pill.kind, "tier");
  assert.equal(pill.text, "VER");
  assert.equal(pill.color, TIER_BADGE.verified.color);
  assert.equal(pill.title, "Verified: …chain.", "tooltip is the verbatim phrase");
  assert.deepEqual(actionBadge(verified), TIER_BADGE.verified);

  const blocked = { srcUrl: "https://cdn.example/b.jpg", notFetched: true, error: "could not fetch…" };
  const marker = pillSpec(blocked);
  assert.equal(marker.kind, "not-examined");
  assert.ok(marker.title.includes("cdn.example"), "tooltip names the image host");
  assert.equal(actionBadge(blocked).text, "ERR", "action badge stays honest on errors");

  const broken = { srcUrl: "not a url", error: "engine is not bundled…" };
  const err = pillSpec(broken);
  assert.equal(err.kind, "error");
  assert.equal(err.title, "engine is not bundled…");

  // An unknown verdict id (future tier, corrupted entry) must fall through
  // to ERR, never invent a tier pill.
  const unknown = { srcUrl: "x", report: { verdict: "trusted-ish", phrase: "?" } };
  assert.equal(pillSpec(unknown).kind, "error");
}

console.log("scan_support tests: all assertions passed");
