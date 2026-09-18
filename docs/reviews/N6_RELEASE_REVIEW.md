# N6 measurement and package preparation — 2026-09-18

This is a development candidate, not release approval. `codex/n5-n6-registry-release` carries unmerged N0–N5. Version 0.3.0 is aligned across Cargo, extension and generated npm; only four workspace versions changed in Cargo.lock. No dependency resolution, main, production anchors, store/npm publication or deployment changed.

## Delivered

`docs/CALIBRATION.md` preserves historical counts but corrects certainty/vendor-comparison claims and explains exact one-sided 95% bounds and correlated source families. `calibration_audit.mjs` checks byte identities, source grouping, split leakage and source-level outcomes, including nonzero-hit intervals and missing/unavailable results. It audits supplied production measurement records, not source truth or detector execution. Independent holdout collection, human near-duplicate/label review and genuine bzh-generated positives remain outstanding; proxy photographs remain proxies.

One shared Node packager replaces two divergent shell implementations; shell commands remain wrappers. Fresh `target/package-*` staging never overwrites `extension/pkg`. It checks versions and hard raw/gzip limits, copies both full licenses, verifies the exact reviewed certificate-content digest (not a header), archives/extracts, compares required-file hashes and runs real WASM vector/hostile-input smoke on extracted bytes. Unknown/test anchors require `--development` and separate output. No new dependency or permission. The archive test caught and fixed a missing `provenance_wasm_bg.wasm.d.ts` in wasm-pack's npm files list.

Each archive's `build-evidence.json` records commit/dirty flag, lock hash, toolchains, engine hash/sizes, anchors where bundled and model absence. Adjacent `artifact-evidence.json` records final archive hash/size and content hashes. Optional separate wasm-opt output must exist; without it the release-profile result must still pass both size gates. This machine lacks the separate optimizer. Archive timestamps are not promised bit-reproducible. wasm-pack's crate-local license warning persists because licenses live at the workspace root; both texts are explicitly copied and verified in the archives.

## Reproduce

Run from the repository root:

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --locked --offline -- -D warnings
    cargo test --workspace --locked --offline
    cargo test -p provenance-core --no-default-features --locked --offline
    cargo build -p provenance-log --bin plog --locked --offline
    node scripts/registry_test.mjs
    node scripts/calibration_audit_test.mjs
    node scripts/package_checks_test.mjs
    node scripts/package.mjs extension --development
    node scripts/package.mjs npm --development

Use the printed extracted directory for `node scripts/inspect_browser_test.mjs <extracted-extension>/pkg <extracted-extension>` and `node scripts/scan_browser_test.mjs <extracted-extension>/pkg`, with existing Playwright setup. Non-development packaging requires a clean committed checkout but is still only preparation, never publication or approval.

## Evidence and remaining gates

Format, strict lint, workspace (69 passed, one optional model ignored), reduced-feature core (43 passed), Node helper/background/inspector/calibration/package tests and independent Node Ed25519/unbalanced-tree verification pass. Extracted extension/npm archives pass twelve WASM vectors and oversized/nested/truncated recovery. Chromium 151.0.7922.34 and Firefox 153.0 pass offline inspector/receipt/keyboard/escaping/bounds and image lifecycle checks with the extracted development engine. Installed-extension permission dialogs and worker restart still need manual release smoke; DOM/module tests are not that evidence.

WASM: **6,968,175 raw / 2,341,807 gzip bytes**, SHA-256 `9a79afddc92ee6ec29ad9632bffba511420b07a9821b3a8157e66895b23edf69`; within 7,000,000/2,500,000 limits. Lock SHA-256 `21a75b5917ea2a1dd5a3794d2c8ff1965abc3dc7fedda191eecad2032ae74e20`. Explicit model inference also passed: `PROVENANCE_LENS_BZH_ONNX=D:\sandbox\corpus\bzh\stable-signature-bzh-resnet18.onnx`, `cargo test -p provenance-core --features stable-signature --test stable_signature --locked --offline -- --ignored`, one known-answer pair test passed in 4.36 seconds. The production loader checked the pinned model digest; this is not a full-corpus rerun or a reproduction of the original export environment.

N3/N4 `77ff1e7` remote CI `35362161502` succeeded. Latest scheduled audit/fuzz inspected: `34835684611`, 2026-09-14, successful steps on **old main `4445872`**, not this candidate. Local cargo-audit is absent and cargo-fuzz initially returned access denied; neither is a clean audit/fuzz result. The candidate adds registry fuzzing and permits existing Ubuntu audit/fuzz on manual workflow dispatch as well as schedule, retaining advisory-only audit. Candidate Ubuntu execution needs a later authorized push/run.

Read-only upstream trust check at **2026-09-18T15:29:03Z** found **30 upstream certificates versus 28 bundled**, with differing content. Anchors and their fetched date were not modified. New certificates and later SDK branches require separate review; release readiness is not established.

Remaining: human N0/N1/N3/N5 trust reviews, changed-anchor review, candidate Ubuntu CI/audit/fuzz, installed-extension smoke, at least 3,000 provenance-reviewed independent unseen clean sources and genuine bzh positives. G3 still needs explicit local-first sequencing approval, operator trust and measured quality controls. This packet does not grant approvals or authorize publication/deployment.

## Final clean-source artifacts

Both development archives were rebuilt from clean source commit `7003a9b69e2fb593839a74920c44bd196e2ccdf3` (`source_dirty: false`) and passed extracted-WASM smoke. Evidence files are next to each archive; outputs are ignored build artifacts, not committed binaries.

- `dist/development-extension-0.3.0-7003a9b6-5Kb6tS/provenance-lens-0.3.0.zip`: 2,223,398 bytes; SHA-256 `7adde88ff1058d9f98e41760f1c6d7627299a8b0cb497f52c56ce339fb930dcd`. Extracted at `target/package-Ro1qfy/extracted` for browser checks.
- `dist/development-npm-0.3.0-7003a9b6-Z8zihK/provenance-lens-verify-wasm-0.3.0.tgz`: 2,347,300 bytes; SHA-256 `e1617f34922df70d6167b86f3018941e7b8443a7b725c3715bebca0d97aa3a16`.

`cargo +nightly check --manifest-path fuzz/Cargo.toml --bin registry_parsing --offline` passed. This compile check used the fuzz workspace's separate locally resolved lockfile; it is not a fuzz run or evidence for the root production lockfile. Ubuntu runtime fuzzing remains pending. Remote readback confirms N3/N4 at `77ff1e7`, remote main unchanged at `4445872`, local main unchanged at `5f1dcb6`. N5/N6 remain local.
