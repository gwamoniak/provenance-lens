// Install the real extension in a disposable Chromium/Brave profile. No import,
// messaging, verifier or browser-API substitutes. Production anchors stay intact.
// Usage: node scripts/extension_worker_test.mjs <wasm-pkg> [extension-directory]
// Optional: PROVENANCE_TEST_BROWSER=<Brave executable>.
import assert from "node:assert/strict";
import { cpSync, mkdirSync, mkdtempSync, readFileSync } from "node:fs";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import { resolve, join, relative, sep } from "node:path";
const { chromium } = createRequire(import.meta.url)("playwright");
const source = resolve(process.argv[3] || "extension"), pkg = resolve(process.argv[2] || "target/ci-wasm");
mkdirSync("target", { recursive: true });
const stage = mkdtempSync(resolve("target/worker-test-")), extension = join(stage, "extension");
cpSync(source, extension, { recursive: true, filter: (path) => relative(source, path).split(sep)[0] !== "pkg" });
cpSync(pkg, join(extension, "pkg"), { recursive: true });
const image = readFileSync("crates/provenance-core/tests/vectors/stripped.png");
const signed = readFileSync("crates/provenance-core/tests/vectors/valid_signed.png");
const server = createServer((req, res) => {
  res.setHeader("cache-control", "no-store");
  // Public fixture permits explicit verification before a host grant.
  res.setHeader("access-control-allow-origin", "*");
  if (req.url === "/") {
    res.writeHead(200, { "content-type": "text/html" });
    res.end('<!doctype html><img src="/plain.png" width="128" height="128" alt="test image">');
  } else { res.writeHead(200, { "content-type": "image/png" }); res.end(req.url === "/signed.png" ? signed : image); }
});
await new Promise((done) => server.listen(0, "127.0.0.1", done));
let context;
try {
  context = await chromium.launchPersistentContext(join(stage, "profile"), {
    headless: true, channel: "chromium",
    ...(process.env.PROVENANCE_TEST_BROWSER ? { executablePath: process.env.PROVENANCE_TEST_BROWSER } : {}),
    args: ["--enable-unsafe-extension-debugging",
      `--disable-extensions-except=${extension}`, `--load-extension=${extension}`],
  });
  const worker = context.serviceWorkers()[0] || await context.waitForEvent("serviceworker", { timeout: 15_000 });
  // WHATWG URL.origin is "null" for chrome-extension: URLs in Node.
  const base = `chrome-extension://${new URL(worker.url()).host}`;
  const page = await context.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const cdp = await context.browser().newBrowserCDPSession();
  const pageCdp = await context.newCDPSession(page);
  const versions = new Map();
  pageCdp.on("ServiceWorker.workerVersionUpdated", ({ versions: updates }) => {
    for (const version of updates) versions.set(version.versionId, version);
  });
  await pageCdp.send("ServiceWorker.enable");
  const tabs = await cdp.send("Target.getTargets", { filter: [{ type: "tab" }] });
  const targetInfo = tabs.targetInfos.find((t) => t.url === page.url());
  assert.ok(targetInfo, "fixture tab must exist");
  await cdp.send("Extensions.triggerAction", { id: new URL(worker.url()).host, targetId: targetInfo.targetId });
  let popupTarget;
  for (let attempt = 0; attempt < 100 && !popupTarget; attempt++) {
    const { targetInfos } = await cdp.send("Target.getTargets");
    popupTarget = targetInfos.find((t) => t.url === `${base}/popup/popup.html`);
    if (!popupTarget) await new Promise((done) => setTimeout(done, 50));
  }
  assert.ok(popupTarget, "toolbar action must open the actual popup");
  const { sessionId } = await cdp.send("Target.attachToTarget", { targetId: popupTarget.targetId });
  let sequence = 0;
  const evaluate = (expression) => new Promise((resolve, reject) => {
    const id = ++sequence;
    const timer = setTimeout(() => { cdp.off("Target.receivedMessageFromTarget", receive); reject(new Error("popup evaluation timed out")); }, 15_000);
    function receive(event) {
      if (event.sessionId !== sessionId) return;
      const response = JSON.parse(event.message);
      if (response.id !== id) return;
      clearTimeout(timer); cdp.off("Target.receivedMessageFromTarget", receive);
      if (response.error || response.result?.exceptionDetails) reject(new Error(JSON.stringify(response)));
      else resolve(response.result.result.value);
    }
    cdp.on("Target.receivedMessageFromTarget", receive);
    cdp.send("Target.sendMessageToTarget", { sessionId, message: JSON.stringify({ id, method: "Runtime.evaluate",
      params: { expression, awaitPromise: true, returnByValue: true, userGesture: true } }) }).catch((error) => {
      clearTimeout(timer); cdp.off("Target.receivedMessageFromTarget", receive); reject(error);
    });
  });
  const url = `http://127.0.0.1:${server.address().port}/plain.png`;
  const verify = () => evaluate(`chrome.runtime.sendMessage(${JSON.stringify({ type: "pl-reverify", url })})`);
  const result = await verify();
  assert.equal(result.error, undefined, JSON.stringify(result));
  assert.equal(result.report?.verdict, "inconclusive");
  assert.equal(result.anchorsLoaded, true);
  assert.ok(result.retrievedAt);
  const untrusted = await evaluate(`chrome.runtime.sendMessage(${JSON.stringify({ type: "pl-reverify", url: url.replace("plain.png", "signed.png") })})`);
  assert.equal(untrusted.error, undefined, JSON.stringify(untrusted));
  assert.equal(untrusted.report?.verdict, "tampered", "production anchors must not trust the synthetic test signer");
  assert.equal(untrusted.report.credentials, undefined);
  const enabled = await evaluate('chrome.runtime.sendMessage({ type: "pl-set-scan", site: "http://127.0.0.1/*", enabled: true })');
  assert.deepEqual(enabled, { error: "site access is required", stopped: true }, "scanning must still require host consent");
  const scanning = await evaluate('chrome.runtime.sendMessage({ type: "pl-set-scan", site: "http://127.0.0.1/*", enabled: false })');
  assert.deepEqual(scanning, { enabled: false });
  const version = [...versions.values()].find((v) => v.scriptURL === worker.url() && v.runningStatus === "running");
  assert.ok(version, "installed worker must be running before restart test");
  await pageCdp.send("ServiceWorker.stopWorker", { versionId: version.versionId });
  for (let attempt = 0; attempt < 100 && versions.get(version.versionId)?.runningStatus !== "stopped"; attempt++)
    await new Promise((done) => setTimeout(done, 50));
  assert.equal(versions.get(version.versionId)?.runningStatus, "stopped", "worker must really stop before recovery is tested");
  const restarted = await verify();
  assert.equal(restarted.error, undefined, JSON.stringify(restarted));
  assert.equal(restarted.report?.verdict, "inconclusive");
  assert.ok(restarted.retrievedAt);
  assert.notEqual(restarted.retrievedAt, result.retrievedAt, "restart must fetch and examine again");
  console.log(`${context.browser().version()}: installed extension, real popup/worker/WASM, scan consent gate and worker restart passed`);
} finally {
  await context?.close();
  server.closeAllConnections();
  await new Promise((done) => server.close(done));
}
