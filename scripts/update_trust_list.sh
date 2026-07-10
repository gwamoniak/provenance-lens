#!/bin/sh
# Refresh extension/trust/anchors.pem from the official C2PA conformance
# trust list. Run from the repo root; commit the result. The provenance
# header (source, date, sha256) makes every refresh auditable.
set -eu

URL="https://raw.githubusercontent.com/c2pa-org/conformance-public/main/trust-list/C2PA-TRUST-LIST.pem"
OUT="extension/trust/anchors.pem"
TMP="$(mktemp)"

curl -sSf "$URL" -o "$TMP"
COUNT="$(grep -c 'BEGIN CERTIFICATE' "$TMP")"
[ "$COUNT" -ge 1 ] || { echo "refusing: no certificates in download" >&2; exit 1; }
SHA="$(shasum -a 256 "$TMP" | cut -d' ' -f1)"

{
  echo "# Provenance Lens trust anchors — official C2PA conformance trust list."
  echo "# Source: $URL"
  echo "# Fetched: $(date -u +%Y-%m-%dT%H:%M:%SZ)   Certificates: $COUNT   sha256: $SHA"
  echo "# Refresh with: sh scripts/update_trust_list.sh   (commit the result)"
  cat "$TMP"
} > "$OUT"
rm -f "$TMP"
echo "wrote $OUT ($COUNT certificates, sha256 $SHA)"
