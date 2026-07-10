---
name: wasm-packaging
description: The WASM build pipeline, size discipline, and JS interop patterns for crates/provenance-wasm. Load for Milestone 2 work or any change to the WASM boundary or its build.
---

# WASM packaging

## Build

    wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg

`--target web` produces an ES module the MV3 extension imports directly; `extension/pkg/` is build output, gitignored, never committed. The workspace `[profile.release]` already sets `opt-level = "s"`, `lto = true`, `strip = true`.

## Size discipline

The artifact ships inside a browser extension; measure gzipped size after every dependency change and record it in the ExecPlan (M2 sets the budget from evidence; placeholder ≤ 4 MB gzipped).

- Check what's inside before optimizing blind: `twiggy top extension/pkg/*.wasm`.
- The `c2pa` crate is the big rock: build it with default features off and enable only what Layer 1 validation needs; crypto backends matter for wasm32 (pure-Rust backends link; native/OpenSSL ones don't). Record the exact feature set in the Decision Log.
- `wasm-opt -Os` (bundled with wasm-pack) stays enabled.
- No `serde` until the JSON shape outgrows the hand-rolled encoder (Decision Log gate).

## Interop patterns

- One exported function, bytes in / JSON string out. Keep it that way as long as possible: every additional exported type multiplies glue code and bundle size.
- Pass `&[u8]`, not JS objects; let wasm-bindgen copy once. Never hold references to JS memory across calls.
- Errors cross the boundary as data (the JSON report or a thrown JS `Error` from wasm-bindgen), never as a panic — see rust-quality's no-panic policy; consider `console_error_panic_hook` in debug builds only.
- wasm32 has no entropy/clock by default: if a dependency drags in `getrandom`, wire the `js` feature; record it.

## npm package (post-wedge)

The proposal's `verify-wasm` deliverable — typed JS/TS bindings on npm — is post-wedge. When it happens: `--target bundler` build variant, hand-written `.d.ts` for the report shape, README with the honesty rules. Do not start it before the wedge ships.
