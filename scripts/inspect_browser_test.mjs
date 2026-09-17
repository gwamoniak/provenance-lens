// Actual inspector modules and WASM in Chromium/Firefox; no extension APIs
// are needed. Local packaged-resource serving is substituted, not verification.
import assert from "node:assert/strict";
import { createServer } from "node:http";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { resolve, extname, sep } from "node:path";
const { chromium, firefox } = createRequire(import.meta.url)("playwright");
const extension = resolve("extension"), pkg = resolve(process.argv[2] || "target/n3-n4-wasm");
const vectors = resolve("crates/provenance-core/tests/vectors");
const fixture = (name) => ({ name, mimeType: name.endsWith(".svg") ? "image/svg+xml" : "image/jpeg", buffer: readFileSync(resolve(vectors, name)) });
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
let testRoot = false, missingEngine = false;
const server = createServer((req, res) => {
  const path = decodeURIComponent(new URL(req.url, "http://localhost").pathname);
  const root = path.startsWith("/pkg/") ? pkg : extension;
  const file = resolve(root, "." + (root === pkg ? path.slice(4) : path));
  if (!file.startsWith(root + sep)) { res.writeHead(403).end(); return; }
  try {
    if (missingEngine && path === "/pkg/provenance_wasm_bg.wasm") { res.writeHead(404).end(); return; }
    const bytes = testRoot && path === "/trust/anchors.pem" ? readFileSync(resolve(vectors, "test_ca.pem")) : readFileSync(file);
    res.writeHead(200, { "content-type": ({ ".html": "text/html", ".js": "text/javascript", ".css": "text/css", ".wasm": "application/wasm", ".json": "application/json" })[extname(file)] || "text/plain", "cache-control": "no-store" });
    res.end(bytes);
  } catch { res.writeHead(404).end(); }
});
await new Promise((done) => server.listen(0, "127.0.0.1", done));
const origin = `http://127.0.0.1:${server.address().port}`;
try {
  for (const type of [chromium, firefox]) {
    const browser = await type.launch({ headless: true, timeout: 20_000 });
    try {
      for (const isolatedTestRoot of [false, true]) {
        testRoot = isolatedTestRoot;
        const context = await browser.newContext({ acceptDownloads: true });
        const page = await context.newPage(); page.setDefaultTimeout(10_000);
        const errors = []; page.on("pageerror", (err) => errors.push(err.message));
        await page.goto(`${origin}/inspect/inspect.html`);
        await page.waitForFunction(() => document.getElementById("status").textContent.startsWith("Ready."));
        await context.setOffline(true);
        const requests = []; page.on("request", (request) => requests.push(request.url()));
        const select = async (files) => {
          await page.locator("#files").setInputFiles(files);
          await page.waitForFunction(() => document.getElementById("status").textContent.startsWith("Local examination finished."));
          return page.locator("#results pre").evaluateAll((nodes) => nodes.map((n) => JSON.parse(n.textContent)));
        };
        let receipts = await select([fixture("valid_signed.jpg"), fixture("stripped.jpg")]);
        assert.equal(receipts[0].report.verdict, isolatedTestRoot ? "verified" : "tampered");
        assert.equal(receipts[1].report.verdict, "inconclusive");
        assert.equal(receipts[0].asset.sha256, digest(fixture("valid_signed.jpg").buffer));
        assert.equal(receipts[0].engine.wasm_sha256, digest(readFileSync(resolve(pkg, "provenance_wasm_bg.wasm"))));
        assert.equal(receipts[0].trust_anchors.sha256, digest(readFileSync(isolatedTestRoot ? resolve(vectors, "test_ca.pem") : resolve(extension, "trust/anchors.pem"))));
        if (isolatedTestRoot) assert.match(await page.locator("#changes").textContent(), /present in A, absent in B/);
        if (isolatedTestRoot && type.name() === "chromium") await page.screenshot({ path: "target/n4-inspector-chromium.png", fullPage: true });
        receipts = await select(fixture("manifest_declares_watermark.jpg"));
        assert.equal(Boolean(receipts[0].report.credentials?.declared_watermark), isolatedTestRoot);
        if (isolatedTestRoot) assert.match(await page.locator("#results ul").textContent(), /not necessarily the watermark/);
        // Native keyboard button saves exactly the previewed unsigned receipt.
        const saved = page.waitForEvent("download");
        await page.getByRole("button", { name: "Save unsigned JSON receipt" }).focus();
        await page.keyboard.press("Enter");
        const download = await saved;
        assert.deepEqual(JSON.parse(readFileSync(await download.path(), "utf8")), receipts[0]);
        for (const flower of ["flower-1-daisy.svg", "flower-2-tulip.svg", "flower-3-sunflower.svg"]) {
          const buffer = readFileSync(resolve(vectors, "claude/raw", flower.replace(".svg", "-labeled.svg")));
          receipts = await select({ name: flower, mimeType: "", buffer });
          assert.equal(receipts[0].report.findings.find((f) => f.layer === "c2pa").status, "no_signal");
          assert.equal(receipts[0].asset.mime_hint.source, "filename-extension");
        }
        receipts = await select({ name: '<img src=x onerror="alert(1)">.svg', mimeType: "image/svg+xml",
          buffer: Buffer.from('<svg xmlns="http://www.w3.org/2000/svg"><script>throw new Error("executed")</script></svg>') });
        assert.equal(await page.locator("#results img, #results svg, #results script").count(), 0);
        // Exercise the shared claim renderer with hostile signed text too.
        await page.evaluate(async () => {
          const { credentialLines, appendLines } = await import("/lib/report_view.js");
          appendLines(document.querySelector("#results ul"), credentialLines({ verdict: "verified", credentials: {
            declared_watermark: { note: "test", action_labels: ["<script>throw 1</script>"], algorithms: ['<img src=x onerror="alert(1)">'] },
          } }));
        });
        assert.equal(await page.locator("#results img, #results script").count(), 0);
        await page.locator("#files").setInputFiles([fixture("plain.jpg"), fixture("stripped.jpg"), fixture("plain.svg")]);
        assert.match(await page.locator("#status").textContent(), /exactly one or two/);
        assert.equal(await page.locator("#results article").count(), 0);
        await select({ name: "large.jpg", mimeType: "image/jpeg", buffer: Buffer.alloc(25 * 1024 * 1024 + 1) });
        assert.match(await page.locator("#results").textContent(), /25 MiB/);
        assert.equal(await page.locator("#results pre").count(), 0);
        await select(fixture("plain.jpg")); // recovery
        // Drop input uses the same bounded path and never executes the file.
        await page.evaluate(() => {
          const transfer = new DataTransfer(); transfer.items.add(new File(["ordinary text"], "text.txt", { type: "text/plain" }));
          document.getElementById("drop-zone").dispatchEvent(new DragEvent("drop", { bubbles: true, cancelable: true, dataTransfer: transfer }));
        });
        await page.waitForFunction(() => document.querySelector("#results h2")?.textContent.includes("text.txt"));
        assert.match(await page.locator("#results ul").textContent(), /not evaluated/);
        // Clearing while a file read is pending must not resurrect its result.
        await page.evaluate(() => {
          const original = Blob.prototype.arrayBuffer;
          Blob.prototype.arrayBuffer = function () {
            Blob.prototype.arrayBuffer = original;
            return new Promise((done) => { globalThis.finishRead = () => original.call(this).then(done); });
          };
        });
        await page.locator("#files").setInputFiles(fixture("plain.jpg"));
        await page.waitForFunction(() => typeof globalThis.finishRead === "function");
        await page.getByRole("button", { name: "Clear files and results" }).focus();
        await page.keyboard.press("Enter");
        await page.evaluate(async () => { await globalThis.finishRead(); await new Promise((done) => setTimeout(done, 50)); });
        assert.equal(await page.locator("#results article").count(), 0);
        assert.deepEqual(requests, [], "inspection/export must work offline without network requests");
        assert.deepEqual(errors, []);
        await context.close();
      }
      missingEngine = true;
      const recovery = await browser.newPage();
      recovery.setDefaultTimeout(10_000);
      await recovery.goto(`${origin}/inspect/inspect.html`);
      await recovery.waitForFunction(() => document.getElementById("status").textContent.startsWith("Not examined:"));
      missingEngine = false;
      await recovery.locator("#files").setInputFiles(fixture("plain.jpg"));
      await recovery.waitForFunction(() => document.getElementById("status").textContent.startsWith("Local examination finished."));
      assert.match(await recovery.locator("#results").textContent(), /Inconclusive:/);
      await recovery.close();
      console.log(`${type.name()} ${browser.version()}: offline local inspector, comparison, receipts, keyboard, escaping, bounds, production/test trust passed`);
    } finally { await browser.close(); }
  }
} finally { await new Promise((done) => server.close(done)); }
