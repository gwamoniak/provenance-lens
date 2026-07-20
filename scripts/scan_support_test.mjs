// Node test for extension/lib/scan_support.js (the U7 scan service logic):
// the concurrency limiter and the capped FIFO session cache. Run:
//
//   node scripts/scan_support_test.mjs
//
// assert-based, no framework; exits non-zero on failure.

import assert from "node:assert/strict";
import { makeLimiter, makeSessionCache } from "../extension/lib/scan_support.js";

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

console.log("scan_support tests: all assertions passed");
