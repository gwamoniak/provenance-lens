// True-WASM smoke: load the built artifact (extension/pkg/) through the
// wasm-pack JS glue in Node and run the committed vector corpus through it —
// the same call path the extension will use in M3. Exit 0 only if every
// vector produces its recorded verdict.
//
//   wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg
//   wasm-opt <flags per .claude/skills/wasm-packaging/SKILL.md>
//   node scripts/wasm_smoke.mjs

import { readFileSync } from "node:fs";
import init, { verify_bytes } from "../extension/pkg/provenance_wasm.js";

const vectors = new URL("../crates/provenance-core/tests/vectors/", import.meta.url);
const wasmBytes = readFileSync(new URL("../extension/pkg/provenance_wasm_bg.wasm", import.meta.url));
await init({ module_or_path: wasmBytes });

const caPem = readFileSync(new URL("test_ca.pem", vectors), "utf8");
const rows = readFileSync(new URL("manifest.tsv", vectors), "utf8").trim().split("\n").slice(1);

let failures = 0;
for (const row of rows) {
  const [file, expected] = row.split("\t");
  const bytes = readFileSync(new URL(file, vectors));
  const report = JSON.parse(verify_bytes(bytes, "image/jpeg", caPem));
  const ok = report.verdict === expected;
  if (!ok) failures++;
  console.log(`${ok ? "PASS" : "FAIL"} ${file}: ${report.verdict} (expected ${expected})`);
}
console.log(failures === 0 ? "wasm smoke: all vectors match" : `wasm smoke: ${failures} FAILURES`);
process.exit(failures === 0 ? 0 : 1);
