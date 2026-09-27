// Chrome MV3 worker / Firefox event page: listeners register synchronously.
import { loadVerifier } from "./lib/engine.js";
import * as scanSupport from "./lib/scan_support.js";

const api = typeof browser !== "undefined" ? browser : chrome;
const MENU_ID = "provenance-lens-verify";
const SCAN_SCRIPT_ID = "provenance-lens-scan";
const SCAN_SITES = "scanEnabledSites";
const documents = new Map(); // current top-level documents, never reports
let policyRevision = 0;
let policyChanging = false;
let syncQueue = Promise.resolve();

const service = { lib: scanSupport, limit: scanSupport.makeLimiter(2, 50), pending: scanSupport.makeInFlight() };
async function examineUrl(srcUrl, signal) {
  const entry = { srcUrl, at: new Date().toISOString(), evidenceScope: "fetched-file" };
  try {
    const { lib } = service;
    if (!lib.sitePattern(srcUrl)) throw new Error("only HTTP(S) image URLs can be fetched; not examined");
    let mod;
    try { mod = await loadVerifier(); }
    catch (err) { throw new Error(`verification engine or trust bundle could not start: ${err?.message || err}; not examined`); }
    let fetched;
    try { fetched = await lib.fetchBytes(srcUrl, { signal }); }
    catch (err) {
      entry.notFetched = true;
      // Explicit right-click checks can fail before a CDN grant exists.
      // Offer access without replacing the actual network/limit error.
      entry.needsHostAccess = !signal?.aborted
        && !await api.permissions.contains({ origins: [lib.sitePattern(srcUrl)] });
      throw err;
    }
    if (signal?.aborted) throw signal.reason;
    entry.retrievedAt = fetched.retrievedAt;
    entry.anchorsLoaded = true;
    entry.report = mod.verify(fetched.bytes, fetched.mediaType);
  } catch (err) { entry.error = String(err?.message || err); }
  return entry;
}
async function showEntry(entry, signal) {
  if (signal?.aborted) return;
  const { lib } = service;
  await api.storage.session.set({ lastResult: entry });
  if (signal?.aborted) {
    const { lastResult } = await api.storage.session.get("lastResult");
    if (lastResult?.owner?.token === entry.owner?.token) await api.storage.session.remove("lastResult");
    return;
  }
  const badge = lib.actionBadge(entry);
  await api.action.setBadgeText({ text: badge.text });
  await api.action.setBadgeBackgroundColor({ color: badge.color });
  try { await api.action.openPopup(); } catch { /* toolbar click is the fallback */ }
}
async function verifyImage(url) {
  const { limit } = service;
  const entry = await limit(() => examineUrl(url)).catch((err) => ({ srcUrl: url, error: err.message }));
  await showEntry(entry);
  return entry;
}
function stopDocument(tabId) {
  documents.get(tabId)?.controller.abort(new Error("page scanning stopped; not examined"));
  documents.delete(tabId);
}
async function enabledFor(url) {
  const { lib } = service;
  const pattern = lib.sitePattern(url);
  if (!pattern || policyChanging) return false;
  const { [SCAN_SITES]: sites = [] } = await api.storage.local.get(SCAN_SITES);
  return sites.includes(pattern) && await api.permissions.contains({ origins: [pattern] }) && !policyChanging;
}
// Consent is explicit and separate from host grants. Serialized reconciliation
// prevents an older permission snapshot from re-registering a revoked site.
function invalidatePolicy() {
  const revision = ++policyRevision;
  policyChanging = true;
  for (const tabId of documents.keys()) stopDocument(tabId);
  // Clear existing page UI even while an earlier registration call is pending.
  api.tabs.query({}).then((tabs) => {
    if (revision !== policyRevision || !policyChanging) return;
    return Promise.all(tabs.map((tab) => api.tabs.sendMessage(tab.id,
      { type: "pl-scan-control", enabled: false }).catch(() => {})));
  }).catch(() => {});
  return revision;
}
function syncScanRegistration() {
  const revision = invalidatePolicy();
  syncQueue = syncQueue.catch(() => {}).then(async () => {
    const { lib } = service;
    const tabs = await api.tabs.query({});
    await Promise.all(tabs.map((tab) => api.tabs.sendMessage(tab.id, { type: "pl-scan-control", enabled: false }).catch(() => {})));
    const { lastResult } = await api.storage.session.get("lastResult");
    if (lastResult?.owner) {
      await api.storage.session.remove("lastResult");
      await api.action.setBadgeText({ text: "" });
    }
    await api.storage.session.remove("verdictCache");
    const { [SCAN_SITES]: stored = [] } = await api.storage.local.get(SCAN_SITES);
    const sites = [];
    for (const site of stored) {
      if (lib.sitePattern(site) === site && await api.permissions.contains({ origins: [site] })) sites.push(site);
    }
    if (revision !== policyRevision) return;
    const existing = await api.scripting.getRegisteredContentScripts({ ids: [SCAN_SCRIPT_ID] });
    if (revision !== policyRevision) return;
    if (sites.length) {
      const script = { id: SCAN_SCRIPT_ID, js: ["content/scan.js"], matches: sites,
        runAt: "document_idle", persistAcrossSessions: true };
      if (existing.length) await api.scripting.updateContentScripts([script]);
      else await api.scripting.registerContentScripts([script]);
    } else if (existing.length) await api.scripting.unregisterContentScripts({ ids: [SCAN_SCRIPT_ID] });
    if (revision !== policyRevision) return;
    policyChanging = false;
    for (const tab of tabs) {
      if (revision !== policyRevision) return;
      if (!sites.includes(lib.sitePattern(tab.url))) continue;
      await api.scripting.executeScript({ target: { tabId: tab.id }, files: ["content/scan.js"] }).catch(() => {});
      if (revision === policyRevision)
        await api.tabs.sendMessage(tab.id, { type: "pl-scan-control", enabled: true }).catch(() => {});
    }
  }).catch((err) => { console.warn("Page scanning remains stopped:", err); });
  return syncQueue;
}
function isPopup(sender) {
  return sender.id === api.runtime.id && sender.url === api.runtime.getURL("popup/popup.html") && !sender.tab;
}
async function currentDocument(sender, token, start = false) {
  const revision = policyRevision;
  if (sender.id !== api.runtime.id || !Number.isInteger(sender.tab?.id) || sender.frameId !== 0
      || typeof token !== "string" || token.length > 100 || !token.length)
    throw new Error("unauthorized page request");
  const tab = await api.tabs.get(sender.tab.id);
  const withoutHash = (url) => new URL(url).href.split("#")[0];
  if (!sender.url || withoutHash(tab.url) !== withoutHash(sender.url) || !await enabledFor(sender.url)
      || revision !== policyRevision) throw new Error("page scanning is not authorized for this document");
  let doc = documents.get(tab.id);
  if (start && (!doc || doc.token !== token || doc.documentId !== sender.documentId)) {
    stopDocument(tab.id);
    doc = { token, documentId: sender.documentId, url: withoutHash(sender.url), controller: new AbortController() };
    documents.set(tab.id, doc);
  }
  if (!doc || doc.token !== token || doc.documentId !== sender.documentId || doc.url !== withoutHash(sender.url)
      || doc.controller.signal.aborted) throw new Error("page request is no longer current");
  return doc;
}
async function scanRequest(message, sender) {
  const doc = await currentDocument(sender, message.token, !documents.has(sender.tab?.id));
  const { lib, limit, pending } = service;
  if (!Number.isSafeInteger(message.generation) || message.generation < 0)
    throw new Error("invalid image request");
  if (typeof message.url !== "string" || message.url.length > 8192 || !lib.sitePattern(message.url)) {
    const entry = { srcUrl: message.url, notFetched: true, error: "not examined — requires an HTTP(S) image URL of at most 8192 characters" };
    return { entry, pill: lib.pillSpec(entry) };
  }
  const signal = doc.controller.signal;
  const key = JSON.stringify([sender.tab.id, doc.token, message.url, message.generation]);
  const entry = await pending(key, () => limit(async () => {
    await currentDocument(sender, message.token);
    if (!await api.permissions.contains({ origins: [lib.sitePattern(message.url)] }))
      return { srcUrl: message.url, notFetched: true, needsHostAccess: true,
        error: `not examined — no access to ${new URL(message.url).host}` };
    return examineUrl(message.url, signal);
  }, signal)).catch((err) => ({ srcUrl: message.url, notFetched: true, error: String(err?.message || err) }));
  await currentDocument(sender, message.token);
  if (message.type === "pl-show") await showEntry({ ...entry, owner: { tabId: sender.tab.id, token: doc.token } }, signal);
  return { entry, pill: lib.pillSpec(entry) };
}
let consentQueue = Promise.resolve();
function permissionsRemoved() {
  // All consent writes share the popup's queue. Registration reconciliation
  // only reads it, so it cannot overwrite a newer Stop scanning choice.
  invalidatePolicy();
  consentQueue = consentQueue.catch(() => {}).then(async () => {
    const { [SCAN_SITES]: sites = [] } = await api.storage.local.get(SCAN_SITES);
    const retained = [];
    for (const site of sites) if (await api.permissions.contains({ origins: [site] })) retained.push(site);
    await api.storage.local.set({ [SCAN_SITES]: retained });
    await syncScanRegistration();
  });
  return consentQueue;
}
async function handleMessage(message, sender) {
  const { lib } = service;
  if (isPopup(sender)) {
    if (message.type === "pl-set-scan") {
      const site = message.site;
      if (typeof message.enabled !== "boolean" || lib.sitePattern(site) !== site) throw new Error("invalid site choice");
      consentQueue = consentQueue.catch(() => {}).then(async () => {
        if (message.enabled && !await api.permissions.contains({ origins: [site] })) throw new Error("site access is required");
        const { [SCAN_SITES]: sites = [] } = await api.storage.local.get(SCAN_SITES);
        const updated = sites.filter((value) => value !== site);
        if (message.enabled) updated.push(site);
        await api.storage.local.set({ [SCAN_SITES]: updated });
        await syncScanRegistration();
        return { enabled: message.enabled };
      });
      return consentQueue;
    }
    if (message.type === "pl-reverify") {
      const { lastResult } = await api.storage.session.get("lastResult");
      const url = message.url || lastResult?.srcUrl;
      if (!lib.sitePattern(url)) throw new Error("no HTTP(S) file is selected for verification");
      if (lastResult?.owner) await api.tabs.sendMessage(lastResult.owner.tabId,
        { type: "pl-scan-refresh", url }).catch(() => {});
      return verifyImage(url);
    }
  }
  if (message.type === "pl-scan-start") {
    await currentDocument(sender, message.token, true);
    return { enabled: true };
  }
  if (message.type === "pl-verify" || message.type === "pl-show") return scanRequest(message, sender);
  throw new Error("unauthorized extension request");
}
api.runtime.onInstalled.addListener(() => {
  api.contextMenus.create({ id: MENU_ID, title: "Verify provenance with Provenance Lens", contexts: ["image"] });
  syncScanRegistration();
});
api.contextMenus.onClicked.addListener((info) => {
  if (info.menuItemId === MENU_ID && info.srcUrl) verifyImage(info.srcUrl).catch(console.warn);
});
api.permissions.onAdded.addListener(syncScanRegistration);
api.permissions.onRemoved.addListener(permissionsRemoved);
api.runtime.onStartup.addListener(syncScanRegistration);
api.tabs.onRemoved.addListener(stopDocument);
api.tabs.onUpdated.addListener((tabId, change) => { if (change.status === "loading") stopDocument(tabId); });
api.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (!message || typeof message.type !== "string" || !message.type.startsWith("pl-")) return false;
  handleMessage(message, sender).then(sendResponse, (err) => sendResponse({ error: String(err?.message || err), stopped: true }));
  return true;
});
