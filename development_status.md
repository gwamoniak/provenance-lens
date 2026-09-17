# Development status — Provenance Lens

Snapshot: **2026-09-17**, updated for N3/N4 on `codex/n3-n4-local-evidence`, carrying N0–N2; local main remains `5f1dcb6`. This is not a new release. The consolidated next-step plan is [PROVENANCE_LENS_NEXT_EXECPLAN.md](PROVENANCE_LENS_NEXT_EXECPLAN.md). Historical approvals and trust rules remain in [PROVENANCE_LENS_EXECPLAN.md](PROVENANCE_LENS_EXECPLAN.md); this dashboard is a derived view.

## Current position

**Latest:** N1/N2 `976aa35` was pushed at the user's request and [GitHub CI passed](https://github.com/gwamoniak/provenance-lens/actions/runs/35261458076). N3/N4 is implemented locally: signed watermark declarations (not detection), Claude capability documentation, local-file inspection, previewed unsigned receipts, and two-file comparison. See [N3/N4 review evidence](docs/reviews/N3_N4_REVIEW.md). Main and production trust anchors are unchanged. Human review remains required before merge; N3/N4 has not been pushed.

C1's generic SVG/C2PA 0.90.13 implementation is approved and already merged into **local main**. A read-only remote check on this date confirmed remote main at `4445872`; local main is seven commits ahead. No push, merge, release, or runtime-code change was performed during the September planning review. The new plan is on `codex/development-roadmap-refresh`, created from local main in the real project folder.

The latest code has a working cryptographic verifier, native watermark detectors, a browser extension, and a registry prototype. It is not a universal AI-content detector. Current priorities are a clean reproducible build, honest detector failure reporting, and browser badges that cannot retain results for an earlier image.

N0 is now implemented locally on `codex/n0-reproducible-baseline` (from local main, carrying the reviewed plan). Its locked Rust/wasm-pack/C2PA baseline, formatter, strict lint, reduced-feature suite, and actual WASM smoke pass. The C2PA version did not change: its compatible range became an exact requirement for reviewed 0.90.13. A fresh clone passed the locked offline workspace tests and retained the pre-N0 lockfile SHA-256 after an explicit line-ending rule fixed Windows checkout conversion. These changes passed remote CI with N1/N2 on 2026-09-17; maintainer review remains required before merging the dependency-manifest change.

## Capabilities and evidence boundaries

N1/N2 are implemented, pushed, and tested on `codex/n1-n2-evidence-reliability`. Detector failures are explicit, the recovered model is identity-pinned, input/fetch/queue limits are enforced, and browser evidence follows current image generations with fresh fetches and explicit scan consent. See [the review packet](docs/reviews/N1_N2_REVIEW.md). The shared C2PA execution cutoff still needs human sign-off; no merge has occurred.

| Area | Verified current state | Remaining boundary |
|---|---|---|
| C2PA | Local main resolves c2pa 0.90.13; eleven JPEG/PNG/SVG corpus vectors pass. SVG requires an explicit MIME/filename hint. | Only trusted validated provenance becomes Proof; valid-but-untrusted remains Tampered under existing policy. Dependency/trust changes need human sign-off. |
| Claude files | Six original/labeled SVG negative controls contain no C2PA credentials. | No genuine Anthropic-signed positive specimen has been validated. Visible labels/free-form metadata are not signatures. |
| Claude text / actual SynthID | No implemented detector in this tree. | Older research snapshots are dated August; usable vendor contract, fixtures/calibration, and any privacy exception must be established before implementation. |
| Native watermark analysis | SD DWT-DCT and optional identity-pinned IMATAG bzh; explicit detected/negative/unavailable outcomes. Recovered model passes its positive/negative known-answer pair. | Full calibration was not rerun. Missing model or failed probe is NotEvaluated, not a negative. Neither scheme covers AI images generally. |
| WASM / extension | Eleven vectors plus oversized-input/recovery checks pass. Chromium/Firefox DOM lifecycle tests and background consent/sender tests pass. No pixel-watermark detector is compiled into the standalone build. | Installed-extension permission dialogs and remote CI remain untested for this candidate. Limits do not guarantee total memory or adversarial parser time. |
| W3-M signed watermark declarations | N3 reconciled the existing source with C1/N1/N2; twelve vectors and declaration checks pass. Original W3 branch/worktree preserved. | Fresh maintainer sign-off required. Signed soft-binding identifiers do not establish the watermark's scheme. |
| Local evidence workbench | N4 picker/drop, unsigned receipt preview/download, and two-file comparison pass offline in Chromium/Firefox with real WASM. | No receipt import or signed audit claim; no proof of derivation; store-installed extension/release smoke remains. |
| Registry | G0 approved; G1 PDQ implemented/calibrated; `provenance-log`/`plog` prototype exists on main. | Product Layer 3 remains NotEvaluated. Live deployment/client not verified; parser/operator hardening, independently pinned trust, and G3 approval remain. |
| Heuristics | Stub. | Optional; no expansion proposed before stronger evidence work. |

The four canonical verdict phrases and precedence remain unchanged. No data is not proof of authenticity. No successful check should be implied when a detector could not run.

## Fresh verification results

**Latest N3/N4:** 66 workspace tests passed/one optional model ignored; reduced-feature core 43 passed; formatter/strict lint passed. Twelve real-WASM vectors plus explicit declaration and recovery checks pass. Raw/gzip WASM: **6,968,175 / 2,341,807 bytes**. Node checks and both Chromium/Firefox lifecycle/offline-inspector suites pass, including keyboard download matching the receipt preview and unchanged production-anchor rejection of the test signer. No new model calibration, full fuzz/advisory run, or vendor-positive claim is made.

**Latest N1/N2 candidate:** workspace tests **60 passed, one ignored**, optional model known-answer test **one passed**, reduced-feature suite **37 passed**, formatter and strict lint passed. Fresh WASM passes eleven vectors plus oversized-input/recovery and sampled hostile SVG checks; raw/gzip sizes are **6,947,729 / 2,334,154 bytes**. Node bounds/background tests and Chromium/Firefox content-script tests pass. The browser harness uses real DOM and WASM but substitutes extension messaging; the background harness separately substitutes browser APIs. Exact commands, hashes, and limits are in the review packet.

The table below preserves the **earlier planning snapshot**, not current failures:

| Check | Result on 2026-09-16 |
|---|---|
| Workspace tests | 55 reported passed; the bzh known-answer test skipped inference because no model environment variable was set. |
| Core without default features | 34 passed. |
| WASM target compilation | Passed. |
| Fresh compiled-WASM smoke | All eleven vectors passed; built into ignored `target/plan-review-wasm`, without replacing the extension's engine. |
| Node scan helper tests | Passed; additional copied-storage concurrency probe reproduced one lost cache entry. |
| Format check | Failed in the existing `provenance-log` sources. |
| Strict workspace lint | Failed on `unused_mut` and `manual_is_multiple_of` in `provenance-log`; rerun after repair to uncover any further warnings. |

N0 branch result: formatter and strict workspace lint now pass. The workspace suite reports **54 passed, one explicitly ignored model test**; the reduced-feature core suite passes 34 tests. All eleven fixtures pass fresh standalone WASM smoke. A manual `--ignored` model run without the required file exits with a clear failure. The original planning snapshot above remains as dated evidence of what N0 repaired.

Fresh release-profile WASM, without the separate wasm-opt pass: **6,946,939 raw bytes / 2,333,698 gzip bytes**, within the existing 7,000,000 / 2,500,000 budgets. The new ExecPlan records artifact/dependency hashes and commands. This is a local build check, not a packaged release or comprehensive security audit.

Not refreshed for the candidate: installed-extension permission dialogs, full fuzzing, advisory audit, upstream certificate contents, store/npm publication state, full calibration, or genuine positive vendor specimens. Earlier evidence remains historical, not a substitute for these checks on a release candidate.

## Recommended next sequence

1. **N0 — Included in green remote CI.** Obtain the maintainer's dependency-manifest sign-off before merge.
2. **N1–N2 — Pushed; CI passed.** Review the bounded C2PA execution surface and complete installed-extension permission-dialog smoke before release.
3. **N3 — Candidate ready locally.** Reconciled W3-M and useful C4 documentation delivered; seek fresh human review.
4. **N4 — Candidate ready locally.** Inspector, unsigned receipts, and two-file comparison delivered; remote CI/release smoke remain.
5. **N5–N6 — Safe registry pilot and stronger measurement/release evidence.** Reuse the prototype, keep deployment/client gates explicit, and release smaller completed slices without waiting for vendor access.

These are proposals with acceptance criteria in the new plan, not approval to implement every item. No new hosted service, general AI classifier, upload connector, or blockchain feature is proposed.

## Release and workflow cautions

Source version fields still say `0.2.0`; July's historical records report store submissions and npm publication. Their current external status was not checked. Local main's C1 code must not be described as a newly published release.

The bundled anchors' header says fetched 2026-07-27. Since the scheduled refresher ignores date-only differences, that alone does not establish an outdated certificate set or failed refresh. Production anchors were not changed during this review.

Use a named `codex/` branch from the agreed main tip in `D:\sandbox\projects\provenance-lens`. Preserve the separate W3 worktree. Keep commits, local merges, pushes, cryptography approvals, deployment, and publication as distinct actions. Human approval recorded for C1 does not approve W3-M, a registry verifier, or a later SDK upgrade.
