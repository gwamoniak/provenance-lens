// True-WASM smoke: load a built artifact through the wasm-pack JS glue in
// Node and run the committed vector corpus through it — the same call path
// the extension uses. Exit 0 only if every vector produces its recorded
// verdict.
//
//   node scripts/wasm_smoke.mjs [pkg-dir]
//
// pkg-dir defaults to extension/pkg (the extension engine); pass dist/npm-pkg
// to smoke the npm package build (U6, scripts/package_npm.sh does this).

import { readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

const pkgDir = process.argv[2]
  ? pathToFileURL(resolve(process.argv[2]) + "/")
  : new URL("../extension/pkg/", import.meta.url);
const { default: init, verify_bytes } = await import(new URL("provenance_wasm.js", pkgDir));

const vectors = new URL("../crates/provenance-core/tests/vectors/", import.meta.url);
const wasmBytes = readFileSync(new URL("provenance_wasm_bg.wasm", pkgDir));
await init({ module_or_path: wasmBytes });

const caPem = readFileSync(new URL("test_ca.pem", vectors), "utf8");
const rows = readFileSync(new URL("manifest.tsv", vectors), "utf8").trim().split("\n").slice(1);

let failures = 0;
let slowestMs = 0;
for (const row of rows) {
  const [file, expected] = row.split("\t");
  const bytes = readFileSync(new URL(file, vectors));
  // Raster vectors stay hint-free; SVG requires an explicit MIME hint.
  const mediaType = file.endsWith(".svg") ? "image/svg+xml" : undefined;
  const started = performance.now();
  const report = JSON.parse(verify_bytes(bytes, mediaType, caPem));
  slowestMs = Math.max(slowestMs, performance.now() - started);
  const ok = report.verdict === expected;
  if (!ok) failures++;
  console.log(`${ok ? "PASS" : "FAIL"} ${file}: ${report.verdict} (expected ${expected})`);
}
console.log(failures === 0 ? "wasm smoke: all vectors match" : `wasm smoke: ${failures} FAILURES`);
const oversized = JSON.parse(verify_bytes(new Uint8Array(25 * 1024 * 1024 + 1), "image/jpeg", caPem));
assert.equal(oversized.verdict, "inconclusive");
assert.ok(oversized.findings.every((finding) => finding.status === "not_evaluated" && finding.detail.includes("25 MiB")));
assert.equal(oversized.credentials, undefined);
const recovered = JSON.parse(verify_bytes(readFileSync(new URL("valid_signed.jpg", vectors)), undefined, caPem));
assert.equal(recovered.verdict, "verified", "oversized direct WASM input must not break later verification");
console.log("wasm smoke: oversized input refused and next signed image verified");
for (const xml of [
  '<svg xmlns="http://www.w3.org/2000/svg">' + '<g>'.repeat(10_000) + '</g>'.repeat(10_000) + '</svg>',
  '<svg xmlns="http://www.w3.org/2000/svg"><metadata>' + '<g>'.repeat(10_000),
]) {
  const started = performance.now();
  const report = JSON.parse(verify_bytes(new TextEncoder().encode(xml), "image/svg+xml", caPem));
  slowestMs = Math.max(slowestMs, performance.now() - started);
  assert.equal(report.verdict, "inconclusive", "hostile unsigned SVG must never mint proof");
}
assert.equal(JSON.parse(verify_bytes(readFileSync(new URL("valid_signed.jpg", vectors)), undefined, caPem)).verdict, "verified");
console.log(`wasm smoke: nested/truncated SVG recovery passed; slowest sampled examination ${slowestMs.toFixed(1)} ms (not a worst-case bound)`);
process.exit(failures === 0 ? 0 : 1);
