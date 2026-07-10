---
name: rust-quality
description: Rust project conventions for this workspace — error handling, the no-panic policy on WASM paths, dependency discipline, and fuzzing expectations. Load before writing any Rust in crates/.
---

# Rust quality conventions

## The loop (before declaring anything done)

    cargo fmt --all
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

## No-panic policy

Everything in `provenance-core` and `provenance-wasm` runs on attacker-controlled bytes, often inside a browser. A panic in WASM aborts the module and reads as "the verifier crashed on this image" — which adversaries will engineer deliberately.

- No `unwrap()`/`expect()`/indexing/`panic!` on any path reachable from `Pipeline::examine` or `verify_bytes`. Parse errors are values: map them to `TamperEvidence` (malformed provenance structures) or `NotEvaluated` (couldn't run), per the layer's contract.
- Watch for hidden panics: slicing with untrusted offsets, `usize` arithmetic on length fields from parsed data (use `checked_*`), allocation sized by an attacker-supplied length (cap it).
- The CLI may `expect` only on programmer errors, never on file contents.

## Error handling

Until M1 the core is std-only and errors are folded into `LayerFinding` variants. When the `c2pa` crate lands, introduce `thiserror` for internal error types (proposal convention) — but the `Layer` trait keeps returning `LayerFinding`, never `Result`: a layer's failure is itself a finding.

## Dependency discipline

Every dependency is a milestone decision recorded in the ExecPlan Decision Log (M1: `c2pa`; that's the whole list today). Before adding one: who maintains it, what does it pull in transitively, does it build for wasm32? Pin versions; upgrades get a Decision Log entry.

## Fuzzing

Every parser touching untrusted bytes gets a `cargo fuzz` target when it lands (manifest parsing first, M1). A new parser without a fuzz target is a review finding. Corpus seeds come from `tests/vectors/`.
