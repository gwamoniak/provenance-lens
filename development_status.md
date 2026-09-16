# Development status — Provenance Lens

Snapshot: **2026-09-16**, based on local code and checks at main `5f1dcb6`, not a new release. The consolidated next-step plan is [PROVENANCE_LENS_NEXT_EXECPLAN.md](PROVENANCE_LENS_NEXT_EXECPLAN.md). Historical approvals and trust rules remain in [PROVENANCE_LENS_EXECPLAN.md](PROVENANCE_LENS_EXECPLAN.md); this dashboard is a derived view.

## Current position

C1's generic SVG/C2PA 0.90.13 implementation is approved and already merged into **local main**. A read-only remote check on this date confirmed remote main at `4445872`; local main is seven commits ahead. No push, merge, release, or runtime-code change was performed during the September planning review. The new plan is on `codex/development-roadmap-refresh`, created from local main in the real project folder.

The latest code has a working cryptographic verifier, native watermark detectors, a browser extension, and a registry prototype. It is not a universal AI-content detector. Current priorities are a clean reproducible build, honest detector failure reporting, and browser badges that cannot retain results for an earlier image.

N0 is now implemented locally on `codex/n0-reproducible-baseline` (from local main, carrying the reviewed plan). Its locked Rust/wasm-pack/C2PA baseline, formatter, strict lint, reduced-feature suite, and actual WASM smoke pass. The C2PA version did not change: its compatible range became an exact requirement for reviewed 0.90.13. A fresh clone passed the locked offline workspace tests and retained the pre-N0 lockfile SHA-256 after an explicit line-ending rule fixed Windows checkout conversion. The remaining checks are online CI after the branch is pushed and maintainer review before merging the dependency-manifest change.

## Capabilities and evidence boundaries

| Area | Verified current state | Remaining boundary |
|---|---|---|
| C2PA | Local main resolves c2pa 0.90.13; eleven JPEG/PNG/SVG corpus vectors pass. SVG requires an explicit MIME/filename hint. | Only trusted validated provenance becomes Proof; valid-but-untrusted remains Tampered under existing policy. Dependency/trust changes need human sign-off. |
| Claude files | Six original/labeled SVG negative controls contain no C2PA credentials. | No genuine Anthropic-signed positive specimen has been validated. Visible labels/free-form metadata are not signatures. |
| Claude text / actual SynthID | No implemented detector in this tree. | Older research snapshots are dated August; usable vendor contract, fixtures/calibration, and any privacy exception must be established before implementation. |
| Native watermark analysis | SD DWT-DCT decoder; CLI compiles IMATAG bzh support by default but uses it only with a supplied ONNX model. Historical calibration is in `docs/CALIBRATION.md`. | Runtime detector failure can currently collapse into NoSignal; model-backed inference was not exercised in this review. Neither scheme covers AI images generally. |
| WASM / extension | Fresh standalone WASM passes eleven vectors. No pixel-watermark detector is compiled into that standalone build. Context-menu verification and opt-in page scanning exist. | No new real-browser smoke this review. Source changes, URL caching, cache concurrency, consent lifecycle, and resource bounds need the N2 work. |
| W3-M signed watermark declarations | Implementation exists on `codex/w3-synthid-context` at `30e52c8`, outside main. | Needs reconciliation with C1 and its own maintainer review/sign-off. It reports a signed declaration, not pixel detection. |
| Registry | G0 approved; G1 PDQ implemented/calibrated; `provenance-log`/`plog` prototype exists on main. | Product Layer 3 remains NotEvaluated. Live deployment/client not verified; parser/operator hardening, independently pinned trust, and G3 approval remain. |
| Heuristics | Stub. | Optional; no expansion proposed before stronger evidence work. |

The four canonical verdict phrases and precedence remain unchanged. No data is not proof of authenticity. No successful check should be implied when a detector could not run.

## Fresh verification results

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

Not refreshed here: Chrome/Firefox interaction, fuzzing, advisory audit, upstream certificate contents, store/npm publication state, full calibration, or genuine positive vendor specimens. Earlier evidence remains historical, not a substitute for these checks on a release candidate.

## Recommended next sequence

1. **N0 — Candidate ready locally.** Review the branch and review packet, push it when desired so the new GitHub checks can run, and obtain the maintainer's dependency-manifest sign-off before merge.
2. **N1–N2 — Evidence reliability.** Distinguish detector failures from negatives; bound resource use; correct image/cache/permission lifecycle and accessibility.
3. **N3 — Recover W3-M and finish useful Claude documentation.** Reuse existing implementation, reconcile fixtures, seek its separate sign-off, and publish an honest capability matrix.
4. **N4 — Local evidence workbench.** File inspection, an unsigned downloadable evidence receipt, then two-file comparison using the existing verifier.
5. **N5–N6 — Safe registry pilot and stronger measurement/release evidence.** Reuse the prototype, keep deployment/client gates explicit, and release smaller completed slices without waiting for vendor access.

These are proposals with acceptance criteria in the new plan, not approval to implement every item. No new hosted service, general AI classifier, upload connector, or blockchain feature is proposed.

## Release and workflow cautions

Source version fields still say `0.2.0`; July's historical records report store submissions and npm publication. Their current external status was not checked. Local main's C1 code must not be described as a newly published release.

The bundled anchors' header says fetched 2026-07-27. Since the scheduled refresher ignores date-only differences, that alone does not establish an outdated certificate set or failed refresh. Production anchors were not changed during this review.

Use a named `codex/` branch from the agreed main tip in `D:\sandbox\projects\provenance-lens`. Preserve the separate W3 worktree. Keep commits, local merges, pushes, cryptography approvals, deployment, and publication as distinct actions. Human approval recorded for C1 does not approve W3-M, a registry verifier, or a later SDK upgrade.
