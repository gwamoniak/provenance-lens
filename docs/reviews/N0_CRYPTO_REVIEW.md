# N0 dependency pin review packet

Status: **maintainer review pending** (2026-09-16). Branch: `codex/n0-reproducible-baseline`, created from local `main` at `5f1dcb6`. This packet requests review of the dependency-resolution change; it does not claim approval.

## Exact trust-surface change

In `crates/provenance-core/Cargo.toml`, the direct dependency requirement changes from `c2pa = { version = "0.90.13", default-features = false, features = ["rust_native_crypto"] }` to the same feature configuration with `version = "=0.90.13"`. The requirement can therefore resolve only the SDK version reviewed for C1. `.gitignore` no longer excludes `Cargo.lock`, and the existing root lockfile is included unchanged, SHA-256 `30ae9b4dbf53f1da9fc0a659076f735c40147ae76bd3593ca229707f73d752ff`. It is 91,029 bytes. `cargo tree -p c2pa --depth 0 --locked --offline` reports `c2pa v0.90.13`.

No C2PA validation code, status mapping, signature/trust policy, production anchor, vector, generated certificate, or remote manifest feature was changed. The reviewed SDK still has default features disabled and `rust_native_crypto` enabled. The exact version and lockfile constrain future resolution; they do not approve a later SDK bump. `.gitattributes` now marks `Cargo.lock -text` so Windows checkouts preserve its reviewed bytes.

## Local validation

Run from `D:\sandbox\projects\provenance-lens`:

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --locked --offline -- -D warnings
    cargo test --workspace --locked --offline
    cargo test -p provenance-core --no-default-features --locked --offline
    cargo check -p provenance-wasm --target wasm32-unknown-unknown --locked --offline
    node scripts/scan_support_test.mjs
    wasm-pack build crates/provenance-wasm --target web --out-dir ../../target/n0-wasm --locked --offline
    node scripts/wasm_smoke.mjs target/n0-wasm

Observed: formatter and strict lint pass. The workspace reports 54 passing tests plus one explicitly ignored model-inference test; the reduced-feature core suite passes 34 tests. The fresh actual WASM artifact passes all eleven JPEG/PNG/SVG vectors and has SHA-256 `44cdeef03b739bd1e20772119b2ce29fc1c99cd9a99e98538f22bb54aa2efd9f` (6,946,939 raw / 2,333,698 gzip bytes). The opt-in model test exits nonzero if run with `--ignored` and no model environment variable. No model inference was performed.

`git diff -- crates/provenance-core/src/layers/c2pa.rs extension/trust/anchors.pem` is empty. A local clean clone of commit `fce9ca2` has the same 91,029-byte lockfile and SHA-256 as above; `cargo test --workspace --locked --offline` passes there (54 passed, one explicitly ignored), using a pre-populated offline crate/toolchain cache. The first clone revealed Windows checkout newline conversion and was replaced by the committed `.gitattributes` fix. The new CI checks have not yet run online because the branch has not been pushed.

## Reviewer decision requested

Confirm that narrowing the reviewed 0.90.13 dependency to an exact requirement and tracking its existing lockfile preserve the approved C1 validation surface. Review the staged Cargo.lock identity and any unexpected resolved-package change. Record approval or requested changes with date and reviewer name in `PROVENANCE_LENS_EXECPLAN.md` before this branch can merge. C1's previous approval is evidence for the selected SDK, not a new sign-off for this branch.
