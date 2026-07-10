---
name: test-vectors
description: How to generate, catalogue, and maintain the known-answer corpus in tests/vectors/ — c2patool signing, stripped and corrupted variants, the vector manifest format, and the test-time trust-anchor policy. Load for M1/M2 test work or whenever adding a vector.
---

# Test vectors — the known-answer corpus

Layer 1 is validated against assets whose correct verdict is known *by construction*. Every vector's expected verdict is asserted in tests, never eyeballed; the same corpus drives the native/WASM parity suite (M2) and seeds the fuzzer.

## Sources

1. **C2PA public test files** — the spec consortium publishes conformance assets (valid manifests, deliberately malformed ones). Check their license before vendoring; record origin + upstream commit/URL in the manifest file described below.
2. **Self-generated via c2patool** — the CAI's CLI (`cargo install c2patool` or a release binary). This is the workhorse because we control exactly what's wrong with each file.

## Generating the corpus (implemented in M1)

The corpus is produced by a **self-verifying generator** — from the repo root:

    cargo run -p provenance-core --example gen_vectors

It signs `tests/fixtures/plain.jpg` with a fresh `c2pa::EphemeralSigner` chain (keys exist only in memory, never persisted), derives the variants (APP11-stripped, manifest byte-flip, post-signing content byte-flip), **runs every vector through the real pipeline and aborts on any expected-verdict mismatch**, then writes the vectors, `manifest.tsv`, and the public CA root `test_ca.pem` to `crates/provenance-core/tests/vectors/`. Commit all outputs together (signatures are fresh per run — the corpus and CA must match). A lying corpus cannot be committed: the generator refuses to write one, and `tests/vectors.rs` re-asserts every row on every test run.

c2patool remains useful only as an independent cross-check of our vectors (`c2patool valid_signed.jpg` should report the manifest; the stripped copy should report none). Never hand-craft a vector: every byte change is scripted and recorded in the `notes` column.

## The trust-anchor trap (still the #1 pitfall)

The ephemeral CA is NOT on any production trust list. Tests and CLI acceptance must inject it explicitly (`Pipeline::with_trust_anchors` / `lens verify --trust-anchors tests/vectors/test_ca.pem`); production configurations never include it. Assert both directions — the valid vector is `verified` with the anchor loaded AND degrades to `tampered` (unverifiable provenance) without it; `wrong_anchor_does_not_verify` in `tests/c2pa_layer.rs` pins the cross-anchor case.

## Catalogue format

`tests/vectors/manifest.tsv` — one row per vector: filename, expected verdict id (`verified`/`indicated`/`inconclusive`/`tampered`), notes (for derived vectors: the exact byte offset and XOR applied). `tests/vectors.rs` iterates the catalogue and asserts every row, and fails on any `.jpg` on disk that the catalogue doesn't list (and vice versa). Keep vectors small (the source fixture is a 945-byte JPEG; signed vectors ≈ 14 KB); no file over ~200 KB without a recorded reason.
