// Exercise the real message boundary with copied-value storage, permissions,
// tab identities, and abortable requests. No browser or network dependency.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";
import * as lib from "../extension/lib/scan_support.js";

const event = () => ({ listeners: [], addListener(fn) { this.listeners.push(fn); },
  fire(...args) { return Promise.all(this.listeners.map((fn) => fn(...args))); } });
function storage() {
  const data = {};
  return { data, async get(key) { return structuredClone({ [key]: data[key] }); },
    async set(values) { Object.assign(data, structuredClone(values)); },
    async remove(key) { delete data[key]; } };
}
const local = storage(), session = storage();
const grants = new Set(["https://page.example/*", "https://cdn.example/*"]);
const controls = [], registrations = [];
const tab = { id: 7, url: "https://page.example/article" };
const api = {
  runtime: { id: "test-extension", getURL: (p) => `chrome-extension://test-extension/${p}`,
    onInstalled: event(), onStartup: event(), onMessage: event() },
  contextMenus: { create() {}, onClicked: event() },
  storage: { local, session },
  permissions: { async contains({ origins }) { return origins.every((p) => grants.has(p)); },
    onAdded: event(), onRemoved: event() },
  scripting: { async getRegisteredContentScripts() { return structuredClone(registrations); },
    async registerContentScripts(scripts) { registrations.splice(0, registrations.length, ...scripts); },
    async updateContentScripts(scripts) { registrations.splice(0, registrations.length, ...scripts); },
    async unregisterContentScripts() { registrations.length = 0; }, async executeScript() {} },
  tabs: { async get(id) { assert.equal(id, tab.id); return { ...tab }; }, async query() { return [{ ...tab }]; },
    async sendMessage(id, message) { controls.push({ id, ...message }); }, onRemoved: event(), onUpdated: event() },
  action: { async setBadgeText() {}, async setBadgeBackgroundColor() {}, async openPopup() {} },
};
let bytes = 1, fetches = 0, hold = false, aborted = 0, failFetch = false;
const source = readFileSync(new URL("../extension/background.js", import.meta.url), "utf8");
assert.ok(source.includes('import("./lib/scan_support.js")'));
assert.ok(source.includes('import("./pkg/provenance_wasm.js")'));
vm.runInNewContext(source.replace('import("./lib/scan_support.js")', "loadLib()")
  .replace('import("./pkg/provenance_wasm.js")', "loadEngine()"), {
  chrome: api, URL, AbortController, console,
  loadLib: async () => ({ ...lib, fetchBytes: (url, options) => lib.fetchBytes(url, { ...options,
    fetchImpl: async (_url, { signal }) => {
      fetches++;
      if (failFetch) throw new TypeError("Failed to fetch");
      if (hold) return new Promise((_, reject) => signal.addEventListener("abort", () => {
        aborted++; reject(signal.reason);
      }, { once: true }));
      await new Promise((resolve) => setTimeout(resolve, 5));
      return new Response(new Uint8Array([bytes]));
    } }) }),
  loadEngine: async () => ({ default: async () => {}, verify_bytes: (input) => JSON.stringify({
    verdict: input[0] === 1 ? "verified" : "inconclusive", phrase: "computed test phrase", findings: [],
  }) }),
  fetch: async () => new Response("-----BEGIN CERTIFICATE-----"),
});
const sender = { id: api.runtime.id, tab: { ...tab }, frameId: 0, documentId: "doc-1", url: tab.url };
const popup = { id: api.runtime.id, url: api.runtime.getURL("popup/popup.html") };
const send = (message, from = sender) => new Promise((resolve) => {
  assert.equal(api.runtime.onMessage.listeners[0](message, from, resolve), true);
});
const start = (token = "token-1") => send({ type: "pl-scan-start", token });
const verify = (url = "https://cdn.example/image", generation = 1, token = "token-1") =>
  send({ type: "pl-verify", url, generation, token });
const enable = (enabled) => send({ type: "pl-set-scan", site: "https://page.example/*", enabled }, popup);

await api.runtime.onStartup.fire();
assert.equal(registrations.length, 0, "CDN/page host grants alone never authorize scanning");
assert.equal((await start()).stopped, true);
assert.equal((await enable(true)).enabled, true);
assert.deepEqual(Array.from(registrations[0].matches), ["https://page.example/*"]);
assert.equal((await start()).enabled, true);
const pair = await Promise.all([verify(), verify()]);
assert.equal(fetches, 1, "same document/source generation shares only the pending fetch");
assert.ok(pair.every((result) => result.entry.report.verdict === "verified"));
bytes = 0;
assert.equal((await verify()).entry.report.verdict, "inconclusive", "completed URL reports are never reused");
const different = await Promise.all([verify("https://cdn.example/a"), verify("https://cdn.example/b")]);
assert.ok(different.every((result) => result.entry.report), "concurrent URLs cannot lose results through copied storage writes");
assert.equal(session.data.verdictCache, undefined);
assert.equal((await send({ type: "pl-verify", url: "https://cdn.example/image", generation: 1, token: "token-1" },
  { ...sender, frameId: 1 })).stopped, true);
assert.equal((await send({ type: "pl-set-scan", site: "https://cdn.example/*", enabled: true })).stopped, true);
assert.equal((await send({ type: "pl-verify", url: "https://cdn.example/image", generation: 1, token: "token-1" },
  { ...sender, documentId: "old-doc" })).stopped, true);
const denied = await verify("https://ungranted.example/a");
assert.equal(denied.entry.needsHostAccess, true);
assert.equal(denied.pill.kind, "not-examined");
assert.equal((await verify("data:image/png;base64,AA")).pill.kind, "not-examined");

const shown = await send({ type: "pl-show", url: "https://cdn.example/image", generation: 2, token: "token-1" });
assert.equal(shown.entry.report.verdict, "inconclusive");
assert.equal(session.data.lastResult.evidenceScope, "fetched-file");
assert.ok(session.data.lastResult.retrievedAt);
hold = true;
const before = fetches;
const pending = [verify("https://cdn.example/slow-a"), verify("https://cdn.example/slow-b"), verify("https://cdn.example/queued")];
for (let i = 0; i < 100 && fetches < before + 2; i++) await new Promise((resolve) => setTimeout(resolve, 5));
assert.equal(fetches, before + 2);
grants.delete("https://page.example/*");
await api.permissions.onRemoved.fire();
assert.ok((await Promise.all(pending)).every((result) => result.stopped));
assert.equal(aborted, 2);
assert.equal(fetches, before + 2, "revoked queued work never fetches");
assert.equal(session.data.lastResult, undefined);
assert.equal(registrations.length, 0);
assert.ok(controls.some((message) => message.enabled === false));
grants.add("https://page.example/*");
await api.permissions.onAdded.fire();
assert.equal(registrations.length, 0, "a later file-host grant cannot restore revoked scanning consent");
hold = false;
await enable(true); await start("token-2");
assert.ok((await verify("https://cdn.example/recovery", 1, "token-2")).entry.report);
await enable(false);
assert.equal((await verify("https://cdn.example/recovery", 2, "token-2")).stopped, true);
assert.ok(grants.has("https://page.example/*"), "stopping scanning leaves file access independent");
await Promise.all([enable(true), api.permissions.onAdded.fire(), enable(false)]);
assert.equal(local.data.scanEnabledSites.length, 0, "registration reconciliation cannot overwrite the latest consent write");
assert.equal(registrations.length, 0);
failFetch = true;
const explicitFailure = await send({ type: "pl-reverify", url: "https://ungranted.example/image" }, popup);
assert.equal(explicitFailure.needsHostAccess, true, "explicit file checks retain the host-grant affordance");
assert.equal(explicitFailure.error, "Failed to fetch", "the real fetch error is preserved");
const grantedFailure = await send({ type: "pl-reverify", url: "https://cdn.example/image" }, popup);
assert.equal(grantedFailure.needsHostAccess, false, "network errors with access do not ask for another grant");
console.log("background tests: consent, document identity, fresh fetches, copied storage, queue cancellation and recovery passed");
