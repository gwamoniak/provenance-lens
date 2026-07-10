---
name: test-vectors
description: How to generate, catalogue, and maintain the known-answer corpus in tests/vectors/ — c2patool signing, stripped and corrupted variants, the vector manifest format, and the test-time trust-anchor policy. Load for M1/M2 test work or whenever adding a vector.
---

# Test vectors — the known-answer corpus

Layer 1 is validated against assets whose correct verdict is known *by construction*. Every vector's expected verdict is asserted in tests, never eyeballed; the same corpus drives the native/WASM parity suite (M2) and seeds the fuzzer.

## Sources

1. **C2PA public test files** — the spec consortium publishes conformance assets (valid manifests, deliberately malformed ones). Check their license before vendoring; record origin + upstream commit/URL in the manifest file described below.
2. **Self-generated via c2patool** — the CAI's CLI (`cargo install c2patool` or a release binary). This is the workhorse because we control exactly what's wrong with each file.

## Generating the M1 set

Signed/valid (c2patool signs with its bundled test certificates when given a manifest definition):

    c2patool source.jpg -m manifest.json -o valid_signed.jpg

Stripped (the platform-laundering case — remove metadata by re-encoding or explicit strip; verify afterwards that c2patool no longer finds a manifest):

    # any re-encode that drops APP11/JUMBF works; document the exact tool+flags used
    c2patool valid_signed.jpg          # → should now report no manifest on the stripped copy

Tampered (corrupt the manifest store, not the whole file): locate the JUMBF segment and flip bytes *inside it* so the file still parses as an image but validation fails. Script the corruption (offset + original/new byte recorded in the vector manifest) so it is reproducible, not a binary blob of unknown provenance. Also keep one *content*-tampered vector: re-save pixels after signing so the hard binding mismatches.

Plain (no provenance ever): an ordinary camera JPEG → expected `inconclusive`.

## The trust-anchor trap (M1's known pitfall)

c2patool's test certificates are NOT on the production C2PA trust list. If tests validate against production anchors, the "valid" vectors come back untrusted and the suite lies to you. Policy: tests inject the test-CA anchor explicitly (the `c2pa` crate accepts configured trust anchors); production builds never include it. Assert both directions — the valid vector is `verified` with the test anchor loaded AND degrades to tamper/untrusted without it. Record the chosen mechanism in the ExecPlan Decision Log at M1.

## Catalogue format

`tests/vectors/manifest.tsv` — one row per vector: filename, sha256, origin (upstream URL/commit or generation command), expected verdict id (`verified`/`indicated`/`inconclusive`/`tampered`), notes. A test iterates the manifest and asserts every row; a vector on disk but absent from the manifest (or vice versa) fails the suite. Keep vectors small (tiny source images); the corpus is committed, so no file over ~200 KB without a recorded reason.
