#!/bin/sh
# Build the installable extension zip: engine build → true-artifact smoke →
# zip extension/ into dist/. Run from the repo root. The maintainer uploads
# the zip; this script never publishes anything.
set -eu

VERSION="$(python3 -c "import json; print(json.load(open('extension/manifest.json'))['version'])")"
OUT="dist/provenance-lens-$VERSION.zip"

wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg
wasm-opt extension/pkg/provenance_wasm_bg.wasm -Os \
    --enable-bulk-memory --enable-bulk-memory-opt --enable-sign-ext \
    --enable-mutable-globals --enable-nontrapping-float-to-int --enable-reference-types \
    -o extension/pkg/provenance_wasm_bg.wasm
node scripts/wasm_smoke.mjs

grep -q "BEGIN CERTIFICATE" extension/trust/anchors.pem || {
    echo "refusing to package: extension/trust/anchors.pem has no certificates (run scripts/update_trust_list.sh)" >&2
    exit 1
}
if ! grep -q "conformance-public" extension/trust/anchors.pem; then
    echo "WARNING: anchors.pem does not look like the official C2PA list — do not ship a test CA" >&2
fi

mkdir -p dist
rm -f "$OUT"
(cd extension && zip -qr "../$OUT" . -x "pkg/.gitignore")
SHA="$(shasum -a 256 "$OUT" | cut -d' ' -f1)"
SIZE="$(stat -f %z "$OUT")"
echo "packaged $OUT  ($SIZE bytes, sha256 $SHA)"
