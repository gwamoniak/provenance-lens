#!/bin/sh
# Build the installable extension zip: engine build → true-artifact smoke →
# zip extension/ into dist/. Run from the repo root. The maintainer uploads
# the zip; this script never publishes anything. Portable across the dev
# machines (macOS: zip/shasum; Windows Git Bash: bsdtar/sha256sum; version
# read via node, which every dev flow here already requires).
set -eu

VERSION="$(node -p "require('./extension/manifest.json').version")"
OUT="dist/provenance-lens-$VERSION.zip"

wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg

# --enable-bulk-memory-opt exists only in newer binaryen (older versions
# fold those ops under --enable-bulk-memory); probe instead of pinning.
BULK_OPT=""
if wasm-opt --help 2>&1 | grep -q "bulk-memory-opt"; then
    BULK_OPT="--enable-bulk-memory-opt"
fi
# The npm wasm-opt wrapper exits 0 even on failure, so write to a temp file
# and require it to exist before trusting the run.
wasm-opt extension/pkg/provenance_wasm_bg.wasm -Os \
    --enable-bulk-memory $BULK_OPT --enable-sign-ext \
    --enable-mutable-globals --enable-nontrapping-float-to-int --enable-reference-types \
    -o extension/pkg/provenance_wasm_bg.wasm.opt
if [ -s extension/pkg/provenance_wasm_bg.wasm.opt ]; then
    mv extension/pkg/provenance_wasm_bg.wasm.opt extension/pkg/provenance_wasm_bg.wasm
else
    rm -f extension/pkg/provenance_wasm_bg.wasm.opt
    echo "error: wasm-opt produced no output (it can exit 0 on failure)" >&2
    exit 1
fi
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
if command -v zip >/dev/null 2>&1; then
    (cd extension && zip -qr "../$OUT" . -x "pkg/.gitignore")
elif [ -x "${SYSTEMROOT:-C:/Windows}/System32/tar.exe" ]; then
    # Windows ships bsdtar, which writes zip when told to (-a + .zip name).
    (cd extension && "${SYSTEMROOT:-C:/Windows}/System32/tar.exe" -a -cf "../$OUT" --exclude "pkg/.gitignore" -- *)
else
    echo "no zip tool found (need zip or Windows bsdtar)" >&2
    exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
    SHA="$(sha256sum "$OUT" | cut -d' ' -f1)"
else
    SHA="$(shasum -a 256 "$OUT" | cut -d' ' -f1)"
fi
SIZE="$(wc -c < "$OUT" | tr -d ' ')"
echo "packaged $OUT  ($SIZE bytes, sha256 $SHA)"
