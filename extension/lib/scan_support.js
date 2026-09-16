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
      title: entry.error || `not examined — no access to ${host || "this image's host"}`,
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
export function makeLimiter(max, maxWaiting = 50) {
  let inFlight = 0;
  const waiting = [];
  const runNext = () => {
    const next = waiting.shift();
    if (next) next();
  };
  return function run(task, signal) {
    return new Promise((resolve, reject) => {
      const cancel = () => {
        const index = waiting.indexOf(start);
        if (index >= 0) waiting.splice(index, 1);
        reject(signal.reason || new Error("examination cancelled"));
      };
      const start = () => {
        signal?.removeEventListener("abort", cancel);
        if (signal?.aborted) { cancel(); runNext(); return; }
        inFlight++;
        Promise.resolve()
          .then(task)
          .then(resolve, reject)
          .finally(() => {
            inFlight--;
            runNext();
          });
      };
      if (signal?.aborted) cancel();
      else if (inFlight < max) start();
      else if (waiting.length >= maxWaiting) reject(new Error("page examination queue is full; not examined — try Verify again"));
      else {
        waiting.push(start);
        signal?.addEventListener("abort", cancel, { once: true });
      }
    });
  };
}

export const MAX_ASSET_BYTES = 25 * 1024 * 1024;
export const FETCH_TIMEOUT_MS = 15_000;

// Exact scheme/host scope. Access to a CDN is not consent to scan its pages.
export function sitePattern(value) {
  try {
    const url = new URL(value);
    return ["http:", "https:"].includes(url.protocol) && !url.username && !url.password
      ? `${url.protocol}//${url.hostname}/*` : null;
  } catch { return null; }
}

// Only pending requests are shared. Settled results are never URL-cached.
export function makeInFlight() {
  const pending = new Map();
  return function run(key, task) {
    if (pending.has(key)) return pending.get(key);
    const promise = Promise.resolve().then(task).finally(() => pending.delete(key));
    pending.set(key, promise);
    return promise;
  };
}

// Bound actual streamed bytes, including responses without Content-Length.
export async function fetchBytes(url, {
  signal, fetchImpl = fetch, maxBytes = MAX_ASSET_BYTES, timeoutMs = FETCH_TIMEOUT_MS,
} = {}) {
  const controller = new AbortController();
  const abort = () => controller.abort(signal.reason || new Error("examination cancelled"));
  if (signal?.aborted) abort();
  signal?.addEventListener("abort", abort, { once: true });
  const timer = setTimeout(() => controller.abort(new Error("image fetch exceeded the 15-second deadline; not examined")), timeoutMs);
  let rejectAborted;
  const aborted = new Promise((_, reject) => { rejectAborted = reject; });
  const onAbort = () => rejectAborted(controller.signal.reason);
  controller.signal.addEventListener("abort", onAbort, { once: true });
  let reader;
  try {
    if (controller.signal.aborted) throw controller.signal.reason;
    const response = await Promise.race([
      fetchImpl(url, { signal: controller.signal, cache: "no-store" }), aborted,
    ]);
    if (!response.ok) throw new Error(`image fetch failed (HTTP ${response.status}); not examined`);
    if (Number(response.headers.get("content-length")) > maxBytes)
      throw new Error("asset exceeds the 25 MiB examination limit; not examined");
    if (!response.body) throw new Error("image response has no readable body; not examined");
    reader = response.body.getReader();
    let size = 0;
    const chunks = [];
    while (true) {
      const { done, value } = await Promise.race([reader.read(), aborted]);
      if (done) break;
      size += value.byteLength;
      if (size > maxBytes) throw new Error("asset exceeds the 25 MiB examination limit; not examined");
      chunks.push(value);
    }
    const bytes = new Uint8Array(size);
    let offset = 0;
    for (const chunk of chunks) { bytes.set(chunk, offset); offset += chunk.byteLength; }
    return { bytes, mediaType: (response.headers.get("content-type") || "").split(";")[0].trim() || undefined,
      retrievedAt: new Date().toISOString() };
  } finally {
    clearTimeout(timer);
    signal?.removeEventListener("abort", abort);
    controller.signal.removeEventListener("abort", onAbort);
    if (reader) reader.cancel().catch(() => {});
    controller.abort();
  }
}
