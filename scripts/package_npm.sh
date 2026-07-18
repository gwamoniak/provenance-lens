#!/bin/sh
# Build the WASM engine as an npm package (U6). Publish prep ONLY —
# `npm publish` is the maintainer's action, like every release. Run from
# the repo root; output lands in dist/ (gitignored).
set -eu

OUT="dist/npm-pkg"
rm -rf "$OUT"
wasm-pack build crates/provenance-wasm --target web --out-dir "../../$OUT"

# Size optimization is best-effort here: the corpus smoke below is the real
# gate, and CI/dev machines without binaryen still produce a working package.
if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt "$OUT/provenance_wasm_bg.wasm" -Os --enable-bulk-memory --enable-bulk-memory-opt \
    --enable-sign-ext --enable-mutable-globals --enable-nontrapping-float-to-int \
    --enable-reference-types -o "$OUT/provenance_wasm_bg.wasm"
else
  echo "warning: wasm-opt (binaryen) not found - packing the unoptimized artifact" >&2
fi

# Patch the generated package.json: publishable scoped name (the maintainer
# may rename BEFORE first publish; never after), honest description, keywords.
node -e '
const fs = require("fs");
const path = process.argv[1];
const p = JSON.parse(fs.readFileSync(path, "utf8"));
p.name = "@provenance-lens/verify-wasm";
p.description = "Honest C2PA provenance verdicts in WebAssembly: Verified, Indicated, Inconclusive, or Tampered. No provenance data does NOT mean authentic.";
p.keywords = ["c2pa", "content-credentials", "provenance", "wasm", "verification"];
fs.writeFileSync(path, JSON.stringify(p, null, 2) + "\n");
' "$OUT/package.json"

# The corpus must pass through THIS artifact before anything is packed.
node scripts/wasm_smoke.mjs "$OUT"

(cd "$OUT" && npm pack --pack-destination ..)
echo "npm tarball ready under dist/ - publishing is the maintainer's action"
