---
name: wasm-packaging
description: The WASM build pipeline, size discipline, and JS interop patterns for crates/provenance-wasm. Load for Milestone 2 work or any change to the WASM boundary or its build.
---

# WASM packaging

## Build (three steps, all from the repo root)

    wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg
    wasm-opt extension/pkg/provenance_wasm_bg.wasm -Os \
        --enable-bulk-memory --enable-bulk-memory-opt --enable-sign-ext \
        --enable-mutable-globals --enable-nontrapping-float-to-int --enable-reference-types \
        -o extension/pkg/provenance_wasm_bg.wasm
    node scripts/wasm_smoke.mjs        # runs the vector corpus through the REAL artifact

`--target web` produces an ES module the MV3 extension imports directly; `extension/pkg/` is build output, gitignored, never committed. The workspace `[profile.release]` already sets `opt-level = "s"`, `lto = true`, `strip = true`.

wasm-pack's **bundled** wasm-opt predates bulk-memory operations (which Rust emits by default) and fails — it is disabled via `[package.metadata.wasm-pack.profile.release] wasm-opt = false` in `crates/provenance-wasm/Cargo.toml`; the explicit system-binaryen step above (Homebrew `binaryen`, needs the feature flags shown) replaces it. The smoke script is the acceptance check: every committed vector must produce its recorded verdict through the compiled artifact.

## Size discipline

Measured at M2 (c2pa 0.89.2, rust_native_crypto): **6.41 MB raw / ~2.15 MB gzipped** after wasm-opt. Budget: **≤ 7 MB raw, ≤ 2.5 MB gzipped** — measure after every dependency change and record in the ExecPlan; exceeding the budget is a Decision Log event, not a silent bump.

- Check what's inside before optimizing blind: `twiggy top extension/pkg/*.wasm`.
- The `c2pa` crate is the big rock: default features off (no OpenSSL, no HTTP clients — also the sans-IO rule). Record any feature change in the Decision Log.
- No `serde` in the wrapper until the JSON shape outgrows the hand-rolled encoder (Decision Log gate).

## Interop patterns

- One exported function, bytes in / JSON string out. Keep it that way as long as possible: every additional exported type multiplies glue code and bundle size.
- Pass `&[u8]`, not JS objects; let wasm-bindgen copy once. Never hold references to JS memory across calls.
- Errors cross the boundary as data (the JSON report or a thrown JS `Error` from wasm-bindgen), never as a panic — see rust-quality's no-panic policy; consider `console_error_panic_hook` in debug builds only.
- wasm32 has no entropy/clock by default: if a dependency drags in `getrandom`, wire the `js` feature; record it.

## npm package (post-wedge)

The proposal's `verify-wasm` deliverable — typed JS/TS bindings on npm — is post-wedge. When it happens: `--target bundler` build variant, hand-written `.d.ts` for the report shape, README with the honesty rules. Do not start it before the wedge ships.
