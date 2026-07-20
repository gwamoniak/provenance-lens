// Pure helpers for the U7 scan service. This is an ES module the background
// pulls in via dynamic import (the background file itself must stay free of
// static imports — Firefox event-page requirement), which also makes the
// logic directly testable in Node: scripts/scan_support_test.mjs.

/// A concurrency limiter: at most `max` tasks run at once, the rest wait in
/// FIFO order. `run(task)` resolves/rejects with the task's outcome.
export function makeLimiter(max) {
  let inFlight = 0;
  const waiting = [];
  const runNext = () => {
    const next = waiting.shift();
    if (next) next();
  };
  return function run(task) {
    return new Promise((resolve, reject) => {
      const start = () => {
        inFlight++;
        Promise.resolve()
          .then(task)
          .then(resolve, reject)
          .finally(() => {
            inFlight--;
            runNext();
          });
      };
      if (inFlight < max) start();
      else waiting.push(start);
    });
  };
}

/// A capped FIFO cache persisted in an extension storage area (the caller
/// passes api.storage.session; tests pass a fake). One storage key holds a
/// plain object; JS object insertion order gives FIFO eviction.
// ponytail: single-key read-modify-write — fine while one background context
// is the only writer; revisit if that ever changes.
export function makeSessionCache(storageArea, storageKey, cap) {
  return {
    async get(key) {
      const { [storageKey]: cache = {} } = await storageArea.get(storageKey);
      return cache[key];
    },
    async put(key, value) {
      const { [storageKey]: cache = {} } = await storageArea.get(storageKey);
      delete cache[key]; // re-inserting moves the key to the newest slot
      cache[key] = value;
      const keys = Object.keys(cache);
      for (let i = 0; i < keys.length - cap; i++) delete cache[keys[i]];
      await storageArea.set({ [storageKey]: cache });
    },
  };
}
