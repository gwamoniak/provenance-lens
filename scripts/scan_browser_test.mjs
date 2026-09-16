// Real Chromium/Firefox DOM lifecycle, actual content script and WASM engine.
// Only the extension message transport is substituted; background_test.mjs
// separately checks browser API authorization and storage semantics.
// Requires Playwright + installed browsers. Usage: node scripts/scan_browser_test.mjs [wasm-pkg]
import assert from "node:assert/strict";
import { createServer } from "node:http";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { fetchBytes, pillSpec } from "../extension/lib/scan_support.js";
const { chromium, firefox } = createRequire(import.meta.url)("playwright");
const pkg = pathToFileURL(resolve(process.argv[2] || "target/n1-n2-wasm") + "/");
const engine = await import(new URL("provenance_wasm.js", pkg));
await engine.default({ module_or_path: readFileSync(new URL("provenance_wasm_bg.wasm", pkg)) });
const vectors = new URL("../crates/provenance-core/tests/vectors/", import.meta.url);
const ca = readFileSync(new URL("test_ca.pem", vectors), "utf8");
const signed = readFileSync(new URL("valid_signed.png", vectors));
const unsigned = readFileSync(new URL("stripped.png", vectors));
const content = readFileSync(new URL("../extension/content/scan.js", import.meta.url), "utf8");
let sameUrlSigned = true;
const server = createServer((req, res) => {
  const path = new URL(req.url, "http://localhost").pathname;
  res.setHeader("cache-control", "no-store");
  if (req.url === "/") {
    res.writeHead(200, { "content-type": "text/html" });
    res.end('<!doctype html><html><body><img id="subject" alt="test specimen" style="width:128px;height:128px" src="/valid_signed.png"></body></html>');
  } else {
    res.writeHead(200, { "content-type": "image/png" });
    res.end(path === "/stripped.png" || (path === "/same.png" && !sameUrlSigned) ? unsigned : signed);
  }
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
const waitFor = async (condition) => {
  for (let n = 0; n < 200; n++) { if (condition()) return; await new Promise((resolve) => setTimeout(resolve, 10)); }
  throw new Error("timed out waiting for held response");
};
try {
  for (const browserType of [chromium, firefox]) {
    const browser = await browserType.launch({ headless: true, timeout: 20_000 });
    try {
      const page = await browser.newPage({ viewport: { width: 800, height: 600 } });
      page.setDefaultTimeout(8_000);
      const errors = [];
      page.on("pageerror", (err) => errors.push(err.message));
      let enabled = true;
      let hold = false;
      const waiting = [];
      const requests = [];
      sameUrlSigned = true;
      await page.exposeFunction("extensionMessage", async (message) => {
        requests.push(message);
        if (message.type === "pl-scan-start") return { enabled };
        const fetched = await fetchBytes(message.url);
        const entry = { srcUrl: message.url, retrievedAt: fetched.retrievedAt, evidenceScope: "fetched-file",
          report: JSON.parse(engine.verify_bytes(fetched.bytes, fetched.mediaType, ca)) };
        if (hold && message.url.endsWith("/delayed.png")) await new Promise((resolve) => waiting.push(resolve));
        return { entry, pill: pillSpec(entry) };
      });
      await page.addInitScript(() => {
        const listeners = [];
        window.chrome = { runtime: { id: "test-extension", sendMessage: (message) => window.extensionMessage(message),
          onMessage: { addListener: (fn) => listeners.push(fn) } } };
        window.control = (message) => listeners.forEach((fn) => fn(message, { id: "test-extension" }));
        // Capture closed roots for assertions without changing their mode.
        const attachShadow = Element.prototype.attachShadow;
        window.pillRoots = [];
        Element.prototype.attachShadow = function (options) {
          const root = attachShadow.call(this, options);
          window.pillRoots.push(root); return root;
        };
        window.pills = () => window.pillRoots.filter((root) => root.host.isConnected).map((root) => root.querySelector("button")).filter(Boolean);
      });
      await page.goto(origin);
      await page.addScriptTag({ content });
      const expectPill = async (text) => {
        try { await page.waitForFunction((text) => window.pills().length === 1 && window.pills()[0].textContent === text, text); }
        catch (err) {
          console.error(await page.evaluate(() => ({ source: document.getElementById("subject")?.currentSrc,
            complete: document.getElementById("subject")?.complete, pills: window.pills().map((p) => p.textContent) })));
          console.error("recent requests", requests.slice(-4)); throw err;
        }
      };
      const noPills = () => page.waitForFunction(() => window.pills().length === 0);
      const frames = () => page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))));
      await expectPill("VER");
      await page.evaluate(() => { document.getElementById("subject").src = "/stripped.png"; });
      await expectPill("INC");

      hold = true;
      await page.evaluate(() => { document.getElementById("subject").src = "/delayed.png"; });
      await waitFor(() => waiting.length > 0);
      await noPills();
      await page.evaluate(() => { document.getElementById("subject").src = "/stripped.png"; });
      await expectPill("INC");
      waiting.splice(0).forEach((done) => done());
      await frames(); await expectPill("INC");
      hold = false;

      await page.evaluate(() => {
        const img = document.getElementById("subject");
        // Distinct URLs avoid Chromium deliberately retaining a cached larger candidate.
        img.sizes = "300px"; img.srcset = "/valid_signed.png?small 400w, /stripped.png?large 800w";
      });
      await expectPill("VER");
      await page.evaluate(() => { document.getElementById("subject").sizes = "900px"; });
      await expectPill("INC");
      await page.evaluate(() => {
        const img = document.getElementById("subject"); img.removeAttribute("srcset"); img.removeAttribute("sizes");
        const picture = document.createElement("picture");
        const source = document.createElement("source"); source.media = "(min-width: 750px)"; source.srcset = "/valid_signed.png";
        img.before(picture); picture.append(source, img); img.src = "/stripped.png";
      });
      await expectPill("VER");
      await page.setViewportSize({ width: 600, height: 600 });
      await expectPill("INC");
      await page.evaluate(() => {
        document.querySelector("source").remove(); document.getElementById("subject").src = "/same.png";
      });
      await expectPill("VER");
      sameUrlSigned = false;
      await page.evaluate(() => { window.pills()[0].focus(); });
      await page.keyboard.press("Enter");
      await expectPill("INC");
      assert.ok(requests.some((r) => r.type === "pl-show"), "native button works with keyboard activation");
      assert.ok(await page.evaluate(() => window.pills()[0].title.includes("displayed pixels were not verified")));

      await page.evaluate(() => { window.saved = document.getElementById("subject"); window.saved.remove(); });
      await noPills();
      await page.evaluate(() => { document.body.append(window.saved); });
      await expectPill("INC");
      hold = true;
      await page.evaluate(() => { document.getElementById("subject").src = "/delayed.png"; });
      await waitFor(() => waiting.length > 0);
      enabled = false;
      await page.evaluate(() => window.control({ type: "pl-scan-control", enabled: false }));
      await noPills();
      waiting.splice(0).forEach((done) => done());
      await frames(); await noPills();
      const count = requests.length;
      await page.evaluate(() => { document.getElementById("subject").src = "/valid_signed.png"; });
      await frames();
      assert.equal(requests.length, count, "disabled scanning ignores new image loads");
      enabled = true; hold = false;
      await page.evaluate(() => window.control({ type: "pl-scan-control", enabled: true }));
      await expectPill("VER");
      assert.deepEqual(errors, []);
      console.log(`${browserType.name()} ${browser.version()}: source/srcset/sizes/viewport, delayed results, fresh URL fetch, detach/reinsert, disable/resume and keyboard passed`);
    } finally { await browser.close(); }
  }
} finally { server.closeAllConnections(); await new Promise((resolve) => server.close(resolve)); }
