// Pure helpers for the U7 scan service. This is an ES module the background
// pulls in via dynamic import (the background file itself must stay free of
// static imports — Firefox event-page requirement), which also makes the
// logic directly testable in Node: scripts/scan_support_test.mjs.

/// Tier presentation, single-sourced: the action badge (background) and the
/// page pills (content script, via the pl-verify response) both render from
/// this table, so the colors cannot drift. Text, never icons — iconography
/// smuggles safety semantics the wording rules exist to prevent.
/// (popup.css keeps its own copy of the tier colors for the legend.)
export const TIER_BADGE = {
  verified: { text: "VER", color: "#2e7d32" },
  indicated: { text: "IND", color: "#e09b00" },
  inconclusive: { text: "INC", color: "#757575" },
  tampered: { text: "TAM", color: "#c62828" },
};
export const ERROR_BADGE = { text: "ERR", color: "#000000" };

/// What the toolbar action badge shows for a report entry.
export function actionBadge(entry) {
  return (entry.report && TIER_BADGE[entry.report.verdict]) || ERROR_BADGE;
}

/// What a page pill shows for a report entry. Three kinds:
/// - "tier": solid pill, tier color, tooltip = the verbatim approved phrase;
/// - "not-examined": neutral dashed marker for images whose bytes could not
///   be fetched (usually a non-granted image host) — honest absence, never
///   silently skipped;
/// - "error": black ERR, tooltip = the honest error text.
export function pillSpec(entry) {
  if (entry.report && TIER_BADGE[entry.report.verdict]) {
    const tier = TIER_BADGE[entry.report.verdict];
    return { kind: "tier", text: tier.text, color: tier.color, title: entry.report.phrase };
  }
  if (entry.notFetched) {
    let host = "";
    try {
      host = new URL(entry.srcUrl).host;
    } catch {
      /* srcUrl may be data:/malformed; the fallback text covers it */
    }
    return {
      kind: "not-examined",
      text: "· · ·",
      color: "#757575",
      title: `not examined — no access to ${host || "this image's host"}`,
    };
  }
  return {
    kind: "error",
    text: ERROR_BADGE.text,
    color: ERROR_BADGE.color,
    title: entry.error || "verification failed",
  };
}

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
