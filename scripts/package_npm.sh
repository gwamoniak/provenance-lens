#!/bin/sh
# Backwards-compatible entry point; candidate preparation only, never publication.
set -eu
exec node scripts/package.mjs npm "$@"
