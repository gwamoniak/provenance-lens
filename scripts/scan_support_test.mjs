// Node test for extension/lib/scan_support.js (the U7 scan service logic):
// bounded fetching/queueing, pending deduplication, and presentation. Run:
//
//   node scripts/scan_support_test.mjs
//
// assert-based, no framework; exits non-zero on failure.

import assert from "node:assert/strict";
import {
  actionBadge,
  makeLimiter,
  makeInFlight,
  fetchBytes,
  sitePattern,
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

// No completed report cache: concurrent URLs cannot overwrite one another,
// and reusing a URL after completion always examines the new response.
{
  const pending = makeInFlight();
  let calls = 0;
  const task = async () => ++calls;
  assert.deepEqual(await Promise.all([pending("doc:a:1", task), pending("doc:a:1", task), pending("doc:b:1", task)]), [1, 1, 2]);
  assert.equal(await pending("doc:a:1", task), 3, "same URL is fetched again after completion");
  await assert.rejects(pending("failure", () => { throw new Error("failed"); }));
  assert.equal(await pending("failure", task), 4, "failed jobs leave no cached state");
}

{
  const limit = makeLimiter(1, 1);
  let release;
  const active = limit(() => new Promise((resolve) => { release = resolve; }));
  const controller = new AbortController();
  let ran = false;
  const waiting = limit(() => { ran = true; }, controller.signal);
  const rejection = assert.rejects(waiting, /cancelled/);
  await assert.rejects(limit(() => {}), /queue is full/);
  controller.abort(new Error("cancelled"));
  await rejection;
  release(); await active;
  assert.equal(ran, false, "revoked queued job never starts");
  assert.equal(await limit(() => "recovered"), "recovered");
}

{
  let cancelled = false;
  const stream = new ReadableStream({
    pull(controller) { controller.enqueue(new Uint8Array(8)); },
    cancel() { cancelled = true; },
  });
  await assert.rejects(fetchBytes("https://example.com/a", { maxBytes: 10,
    fetchImpl: async () => new Response(stream) }), /25 MiB/);
  assert.equal(cancelled, true, "missing-length oversized stream is cancelled");
  await assert.rejects(fetchBytes("https://example.com/a", { timeoutMs: 20,
    fetchImpl: async () => new Promise(() => {}) }), /deadline/);
  await assert.rejects(fetchBytes("https://example.com/a", { timeoutMs: 20,
    fetchImpl: async () => new Response(new ReadableStream({})) }), /deadline/);
  const next = await fetchBytes("https://example.com/a", {
    fetchImpl: async (_url, options) => {
      assert.equal(options.cache, "no-store");
      return new Response(new Uint8Array([1, 2, 3]), { headers: { "content-type": "image/png; charset=binary" } });
    },
  });
  assert.deepEqual(next.bytes, new Uint8Array([1, 2, 3]));
  assert.equal(next.mediaType, "image/png");
  assert.ok(Date.parse(next.retrievedAt));
  assert.equal(sitePattern("https://cdn.example:8443/a"), "https://cdn.example/*");
  assert.equal(sitePattern("file:///a.jpg"), null);
  assert.equal(sitePattern("https://user:password@example.com/a"), null);
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
  assert.equal(marker.title, blocked.error, "do not mislabel every network failure as missing permission");
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
