# N3/N4 local implementation review

Prepared 2026-09-17 on `codex/n3-n4-local-evidence`. Created from local main `5f1dcb6`, then fast-forwarded to N1/N2 `976aa35` before editing; main was not merged or pushed. The original W3 branch/worktree at `30e52c8` is preserved. This is a review candidate, not a release or human cryptography sign-off.

## Delivered scope

N3 reconciles the reviewed source changes from W3-M `bc2cc42` with C1 and N1/N2. `CredentialSummary.declared_watermark` adds action labels, signed soft-binding identifiers, and the exact descriptive note. Only an explicit bound, unbound, or legacy watermark action under `c2pa.actions`/`c2pa.actions.v2` qualifies. Label-prefix lookalikes are rejected. Malformed optional assertions are skipped without fabricating context. Algorithms remain verbatim and are labeled as soft-binding identifiers, **not necessarily the watermark**. A fingerprint can coexist with a watermark action; the code does not infer an association. No declaration changes Layer 2 or the verdict.

The current self-verifying generator, not old W3 fixture binaries, generated all twelve JPEG/PNG/SVG vectors under one fresh test-only CA. Matching fuzz seeds were regenerated together. The generator normalizes only its own unsigned SVG source fixture to LF before signing, avoiding Windows checkout conversion in generated input. Signed output and hard-binding bytes are preserved by the existing `-text` rules; no user specimen is normalized. No private signing key is persisted. Tiny corpus pixels can now return NotEvaluated under N1, so declaration tests require no independent hit rather than pretending a pixel check completed.

The declaration is rendered in CLI, JSON, popup, and local inspector. Shared browser claim rendering uses `textContent`. `docs/CLAUDE_MARKS.md` delivers the useful C4 matrix, separating format support, generic fixtures, genuine vendor evidence, and detector availability. C0-positive/C2/C3, conditional C4 text/release acceptance, and actual SynthID detection remain open. No vendor API, account, issuer-name heuristic, or upload is introduced.

N4 delivers all three requested increments: local file picker/drop handling (one or exactly two files), unsigned JSON receipts previewed before downloading, and side-by-side reports with explicit evidence differences. There is no inline SVG/HTML preview and no receipt import. Comparison cannot prove derivation. Native buttons/file inputs support keyboard use. Selection generations prevent delayed reads from restoring cleared results. Reads enforce the shared 25 MiB ceiling; oversized inputs are errors with no downloadable invented report.

The extension shares `lib/engine.js` between URL examination and the inspector. It initializes the WASM from the exact bytes whose digest appears in the receipt and loads the bundled production anchor bytes. Missing anchors now fail explicitly instead of silently proceeding without trust. Loaded immutable resources are held per extension context; a missing-engine startup can retry. Receipts carry format/version, basename only, exact asset SHA-256/length, MIME-hint source, examination time, package version, actual WASM digest/profile/SDK version, enabled checks/model absence, anchor-bundle digest/date, and the unchanged report. They omit source URLs and local absolute paths, do not copy document text, and identify themselves as unsigned observations—not Content Credentials or legal attestations. Signed claim text is included in the visible preview. Nothing is stored persistently by the inspector and no new manifest permission is requested.

## Trust and dependency boundary

Human sign-off is required before merge for reconciled `layers/c2pa.rs` and the shared engine/anchor initialization path. Prior C1 approval is not approval of this diff. N0's exact dependency pin and N1's execution limit remain prerequisite review items.

No runtime/test dependency or resolved version was added. C2PA remains exactly 0.90.13. Cargo.lock SHA-256 remains `08570dfc2372bedc69dee9da93f3e5895c7457061cf61a1b9e1e0ed64c62b639`. Production anchors remain byte-identical: `bd884642cf69151739833b0e4489332f7b30d49a0545abaafa982927dc7f739d`. The extension manifest and canonical verdict phrases are unchanged. JSON gains only optional descriptive credential context; `verify_bytes` and CLI exits stay unchanged.

## Validation and reproduction

Run from the repository root with the pinned toolchain and dependency cache:

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --locked --offline -- -D warnings
    cargo test --workspace --locked --offline
    cargo test -p provenance-core --no-default-features --locked --offline
    node scripts/scan_support_test.mjs
    node scripts/background_test.mjs
    node scripts/inspect_test.mjs
    wasm-pack build crates/provenance-wasm --target web --out-dir ../../target/n3-n4-wasm --locked --offline
    node scripts/wasm_smoke.mjs target/n3-n4-wasm

Observed: workspace **66 passed, one optional model test ignored**; reduced-feature core **43 passed**; format and strict lint pass. Native tests cover actions, duplicates, unknown/missing algorithms, mixed fingerprint context, malformed assertions, label lookalikes, JSON escaping, unsigned/untrusted/invalid absence, certificate policy, six unsigned Claude specimens, and the twelve-vector corpus. The optional model was not re-calibrated or rerun in this task.

Real standalone WASM passes all twelve vectors, explicit declaration/findings checks, untrusted/invalid declaration absence, oversized-input recovery, and nested/truncated SVG recovery. Release-profile artifact before separate wasm-opt: **6,968,175 raw / 2,341,807 gzip bytes**, SHA-256 `9a79afddc92ee6ec29ad9632bffba511420b07a9821b3a8157e66895b23edf69`, within the 7,000,000/2,500,000 budgets. This is a test artifact in ignored `target/`, not an installed or released engine.

Browser setup uses the existing Playwright 1.62.1 instructions in `N1_N2_REVIEW.md`. With its Node module and installed browsers discoverable:

    node scripts/scan_browser_test.mjs target/n3-n4-wasm
    node scripts/inspect_browser_test.mjs target/n3-n4-wasm

Observed passes: Chromium 151.0.7922.34 and Firefox 153.0. Existing page-generation/consent lifecycle regressions pass. The inspector harness loads the real page modules and real WASM from an ephemeral loopback server substituting only packaged resource URLs, then disables networking. Inspection and receipt downloads issue **zero requests** after initialization. It checks both unchanged production anchors (synthetic signed files are untrusted) and an isolated test-root response (Verified versus stripped Inconclusive). It covers the three labeled flower SVGs, exact asset/engine/anchor digests, receipt download matching its preview, keyboard activation, no DOM execution from SVG/claim strings, too many files, oversized files and recovery, text/drop input, clear-during-read, and missing-engine retry. A screenshot of the two-file flow was visually inspected.

The Node background test still substitutes browser APIs and the engine for authorization/queue checks; the browser inspector test does not prove store installation or real extension permission prompts. Those release checks, new full fuzz/advisory runs, independent C2PA-tool cross-checks, genuine vendor specimens, and publication remain outstanding. Existing fuzz target coverage and new seeds are retained.

## Publication status

The user-authorized N1/N2 push succeeded; GitHub [CI run 35261458076](https://github.com/gwamoniak/provenance-lens/actions/runs/35261458076) completed successfully for exact commit `976aa35d084570074de04e422b5d09d3873b7fd2`. This does not claim remote CI for N3/N4, which is a separate local candidate. No main push, merge, store submission, or npm publication is authorized by this work.
