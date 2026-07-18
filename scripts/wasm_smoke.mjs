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
for (const row of rows) {
  const [file, expected] = row.split("\t");
  const bytes = readFileSync(new URL(file, vectors));
  // No content-type hint: the artifact must sniff the container (JPEG + PNG
  // corpus since U3), same as the corpus test does natively.
  const report = JSON.parse(verify_bytes(bytes, undefined, caPem));
  const ok = report.verdict === expected;
  if (!ok) failures++;
  console.log(`${ok ? "PASS" : "FAIL"} ${file}: ${report.verdict} (expected ${expected})`);
}
console.log(failures === 0 ? "wasm smoke: all vectors match" : `wasm smoke: ${failures} FAILURES`);
process.exit(failures === 0 ? 0 : 1);
