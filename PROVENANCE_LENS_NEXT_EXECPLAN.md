# Provenance Lens: reliable evidence first, then useful new capabilities


This ExecPlan is a living document, authored 2026-09-16. Maintain `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` at every stopping point, in accordance with `../solid-broccoli/PLANS.md`. The checkout is `D:\sandbox\projects\provenance-lens`.

This document began as a proposed implementation sequence following the user's code-review request. The user subsequently authorized N0, N1/N2, and then N3/N4 implementation plus the N1/N2 push. Candidates and validation are recorded below; merge, publication, and outstanding human trust-boundary review are separate. Historical approvals in `PROVENANCE_LENS_EXECPLAN.md` remain authoritative. The Claude, SynthID, and registry plans retain their own evidence gates.

## Purpose / Big Picture


Make Provenance Lens a tool people can rely on to explain evidence about a specific file, not a universal AI detector. A user should be able to inspect a local export, understand which checks actually ran, distinguish a signed declaration from a detected watermark, compare an original with a re-saved copy, and save a reproducible report. A browser badge must refer to the image it is currently beside, rather than an earlier image at the same location or URL.

The strongest next investment is not another detector. It is repairing the release baseline and evidence-reporting gaps, recovering the already-written W3-M feature, and making the working verifier easier to use. The registry can then add evidence for some metadata-stripped/re-encoded images. Claude text and actual SynthID detection remain conditional on a usable vendor contract and validated specimens. These gates do not block generic product improvements or documentation of current limitations.

## Progress

- [x] (2026-09-18) Maintainer explicitly approved the existing N0/N1/N3–N4/N5 trust-related candidate at `2953dc4` for local-only integration once CI passes. Authoritative sign-off is in `PROVENANCE_LENS_EXECPLAN.md`; no main push, release, new certificates, or G3 client approval.
- [x] (2026-09-18) Candidate CI `35364243397` passed; approved local main fast-forward completed at `3ad82a0`. Post-merge `cargo test --workspace --locked --offline` passed: 69 tests, one explicitly ignored optional model test. No push, release or production trust change.


- [x] (2026-09-16) Reviewed the latest Claude plan, the main/upgrades/roadmap/registry/SynthID records, and current implementation paths across core, CLI, WASM, extension, packaging, and log prototype.
- [x] (2026-09-16) Verified local `main` at `5f1dcb6`, including approved C1 implementation `8a0e689`. Read-only remote check confirmed remote `main` at `4445872`: local main is seven commits ahead, not published there.
- [x] (2026-09-16) Located existing W3-M work at `codex/w3-synthid-context`, tip `30e52c8`, in the separate `provenance-lens-w3` worktree. Its review packet still requires maintainer sign-off; nothing was merged or edited there.
- [x] (2026-09-16) Re-ran workspace tests, reduced-feature core tests, WASM target check, fresh real-WASM build/smoke, JavaScript helper tests, format check, and strict lint check. Results and limitations appear below.
- [x] (2026-09-16) Reproduced a lost-update bug in the extension session cache using storage semantics that copy values, as real extension storage does.
- [x] (2026-09-16) Created this documentation branch, `codex/development-roadmap-refresh`, directly from clean local main in the real project folder; reconciled the status dashboard and plan entry points.
- [x] (2026-09-16) N0 implementation prepared on `codex/n0-reproducible-baseline` from local main. Formatter and strict workspace lint pass; the original 91,029-byte lockfile is now tracked with the same SHA-256 as the planning review, c2pa remains 0.90.13 with an exact requirement, Rust 1.97.1 and wasm-pack 0.15.0 are pinned, and CI now covers reduced-feature Rust, Node helpers, and a real standalone WASM build. Default model tests report one ignored test; the opt-in run fails if its model is absent.
- [x] (2026-09-16) N0 clean-checkout validation: a local clone of the committed branch retains the original Cargo.lock SHA-256 and passes `cargo test --workspace --locked --offline` (54 passed, one explicitly ignored). The first clone exposed Windows newline conversion; `Cargo.lock -text` in `.gitattributes` fixes its bytes across checkouts.
- [x] (2026-09-18) N0 exact C2PA dependency-pin scope approved for local merge; prerequisite checks previously passed with N1/N2.
- [x] (2026-09-16) N1 implemented on `codex/n1-n2-evidence-reliability`, created from main and carrying N0: explicit detector outcomes, missing-model reporting, exact calibrated model identity, model shape/score checks, bounded native/WASM inputs and native decoding, streamed fetch limits/deadlines, and a bounded cancellable queue. Updated every probe caller and measurement denominator. The recovered pinned model passes the vendor known-answer pair.
- [x] (2026-09-16) N2 implemented on that branch: source/document generations, native accessible buttons, fresh explicit re-verification, removal of persistent URL caching, retrieval-time/fetched-file context, explicit scan choices separate from grants, serialized consent writes, sender/document checks, and immediate stop/revocation cancellation. Node regressions and actual Chromium/Firefox DOM lifecycle tests pass.
- [x] (2026-09-17) Pushed exact N1/N2 commit `976aa35` to `origin/codex/n1-n2-evidence-reliability` at the user's request. GitHub CI run `35261458076` completed successfully for that commit, including N0 prerequisites. Main was not pushed.
- [ ] N1/N2 release acceptance remaining: installed-extension permission-dialog smoke. Maintainer approval of the bounded C2PA execution entry was recorded 2026-09-18; packet: `docs/reviews/N1_N2_REVIEW.md`.
- [x] (2026-09-17) N3 implementation reconciled from W3-M on `codex/n3-n4-local-evidence`, created from local main and fast-forwarded to N1/N2 before edits. Twelve vectors regenerated with one test CA and matching fuzz seeds; strict assertion labels, descriptive soft-binding wording, CLI/JSON/popup rendering, and native/real-WASM declaration assertions pass. `docs/CLAUDE_MARKS.md` completes useful C4 documentation, not conditional text/vendor acceptance.
- [x] (2026-09-17) N4 implemented all three requested increments: local inspector, previewed unsigned receipts with exact byte/configuration identities, and two-file comparison. Shared loader/rendering, native controls, no permissions/dependencies added. Offline Chromium/Firefox tests cover production and isolated-test trust, flowers, receipt downloads, keyboard, unsafe text, bounds, drop, clear-during-read, and missing-engine retry.
- [ ] N3/N4 release acceptance remaining: installed-extension smoke. Human cryptography approval was recorded 2026-09-18, and N3/N4 remote CI passed. Packet: `docs/reviews/N3_N4_REVIEW.md`.
- [x] (2026-09-18) N3/N4 commit `77ff1e7` pushed by user request; main unchanged. N5/N6 started on `codex/n5-n6-registry-release`, created from main and carrying prerequisites.
- [x] (2026-09-18) N5 prototype hardening and temporary local operator pilot implemented; parser bounds, fallible encoding, immutable history/index snapshots, atomic latest publication, independently supplied operator-key option, malformed/recovery tests, and registry fuzz target. Packet: `docs/reviews/N5_REGISTRY_REVIEW.md`.
- [ ] N5 remaining: candidate Ubuntu fuzz execution, explicit local-first G3 sequencing approval, calibrated quality controls and product client acceptance; deployment remains maintainer-owned. Existing prototype trust-related changes were approved for local merge 2026-09-18.
- [x] (2026-09-18) N6 preparation implemented: corrected calibration claims with exact confidence bounds/source-group audit checks, aligned unreleased 0.3.0 versions, added full dual-license texts and one isolated cross-platform packager. Development extension/npm archives pass extracted-engine smoke; package checks enforce raw/gzip budgets, exact reviewed anchor content, required files and source/artifact identity. Packet: `docs/reviews/N6_RELEASE_REVIEW.md`.
- [x] (2026-09-18) N5 `56fb265` and N6 `7003a9b` committed locally; clean-source development archives and hashes recorded in the N6 packet. Workspace 69/one ignored, reduced core 43, explicit model pair, strict lint, Node and browser checks pass. Registry fuzz target compile-check passes; runtime fuzzing remains pending.
- [ ] N6 remaining: 3,000 reviewed independent unseen negatives and genuine bzh positives; candidate Ubuntu audit/fuzz/CI, human trust reviews and installed-extension smoke. Upstream certificates differ (30 versus 28 bundled); review separately before release. No release or publication authorized/performed.
- [ ] Vendor track — Reopen Claude C0-positive/C2/C3 or W3-D only when the specific evidence/privacy gates are satisfied.

## Context and Orientation


The core lives in `crates/provenance-core`. `pipeline.rs` combines four layer findings; `json.rs` renders the shared report; `verdict.rs` owns the four canonical phrases. Only the C2PA layer may return `Proof`. A watermark or registry match is at most `Indication`. `TamperEvidence` outranks `Proof`, which outranks `Indication`; otherwise the verdict is `Inconclusive`. `NoSignal` means a check ran without a usable result; `NotEvaluated` means it could not run. Neither means human-authored or authentic.

`layers/c2pa.rs` validates Content Credentials using c2pa 0.90.13 in the reviewed local resolution, with default SDK features disabled and Rust-native cryptography enabled. It does not retrieve remote manifests. Current policy maps valid-but-untrusted signatures to the Tampered tier, with a detail explaining the missing trusted anchor. That is a deliberate existing policy, not proof of a changed image, and this plan does not silently revise it. Credential summaries appear only with a final Verified verdict. SVG requires explicit `image/svg+xml`; the CLI derives this from `.svg`. Do not sniff arbitrary XML.

`layers/watermark.rs` decodes pixels for native detectors. The default core includes the SD DWT-DCT payload detector. The native CLI also enables compilation of the IMATAG bzh model support by default, but actually uses that detector only with `--watermark-model`. The standalone WASM package disables core default features and has no pixel watermark detector. A workspace-native parity test is not proof of identical compiled feature sets: Cargo can combine features across workspace packages. Test the actual standalone WASM artifact separately.

`extension/background.js::examineUrl` fetches bytes, initializes WASM, supplies the anchor bundle, and stores reports. `content/scan.js` watches page images and displays badges. `lib/scan_support.js` contains the queue/cache/presentation helpers. `popup/popup.js` presents findings and permission controls. The extension uses plain JavaScript, no UI framework, no remote code or telemetry. Existing grants cover both image-host access and page-scanning registration; N2 separates their meaning explicitly.

`crates/provenance-log` already contains signed entry encoding, Merkle-tree functions, checkpoint notes, and the `plog` operator CLI. A Merkle tree is a hash tree that lets a verifier check that an entry belongs to a particular signed snapshot. Inclusion alone does not prove append-only history, truthful registration, or similarity of the queried asset. `layers/registry.rs` remains a stub; the log prototype is not an integrated Layer 3. G0/G1 were approved/completed, but no live deployment or G3 client was verified in this review.

## Review findings and what they change


### 1. The project is tested, but the release baseline is not green — high priority


`cargo test --workspace --locked --offline` passed 55 tests on the current local resolution. However, `cargo fmt --all -- --check` fails in both `provenance-log` source files. Strict workspace clippy fails at `crates/provenance-log/src/lib.rs:133` (`unused_mut`) and `:338` (`manual_is_multiple_of`). Further warnings may appear after these are fixed; do not claim they are the exhaustive lint backlog. The first CI format step prevents later checks from running when this state is pushed.

`.gitignore` excludes `Cargo.lock`, `rust-toolchain.toml` names floating `stable`, and `crates/provenance-core/Cargo.toml` uses the compatible-version requirement `"0.90.13"`, not an exact pin. A fresh resolution can therefore differ from the signature-validation code the maintainer approved. This is a release-reproducibility gap, not evidence that an unreviewed version ran in this review. Local `cargo tree` confirmed 0.90.13.

`.github/workflows/ci.yml` runs native tests and a WASM type-check on pushes, but not the Node helper tests or actual compiled-WASM smoke. Audit and fuzz run only on the scheduled job; audit is advisory. The model-backed known-answer test returns successfully without running inference when its environment variable is absent. This happened in this review. Green is not the same as exercised model behavior.

### 2. A detector failure can become “ran, no signal” — high priority


`WatermarkDetector::probe` returns only `Option<WatermarkHit>`. In `layers/stable_signature.rs:63`, model runtime failure is converted through `.ok()?` into `None`; too-small inputs also return `None`. In `layers/watermark.rs:104`, no returned hit becomes `LayerFinding::NoSignal`. The implementation cannot distinguish negative evidence from a detector that could not run, despite the project rule and original skill contract requiring that distinction. This is established by tracing the code; no failing ONNX fixture was exercised here.

The model loader accepts a caller-supplied ONNX file without proving it is the calibrated bzh artifact; the finding still names that vendor classifier. `outputs[0]` and the first scalar are consumed without an explicit output-count/shape/finiteness contract. Treat model identity and runtime validation as an evidence boundary, not just a convenience flag.

### 3. Page badges can outlive the image they described — high priority


`extension/content/scan.js:48` samples `currentSrc` once, unobserves the image, and later refuses to replace a pill if `hosts.has(img)` is true (`:82`). The mutation observer watches added/removed nodes, not `src`/`srcset` attributes (`:148`). Reusing an `<img>` on a single-page app, changing a responsive source, or resolving an old request after a new image loads can leave the old badge beside different content. This is a source-traced correctness finding; current Chrome/Firefox interaction was not exercised during this review.

`background.js::verifyCached` also treats the URL alone as session-long identity. Bytes at a URL can change, and the same URL can have context-dependent responses. Even a refetch is evidence about fetched bytes, not a cryptographic binding to the browser's rendered pixels. The UI must say what was fetched and when; it must never promise that a URL-cache result proves the current display.

### 4. The scan cache loses concurrent writes — reproduced


`extension/lib/scan_support.js:96` reads the entire cache, changes one key, and writes the entire object back. Two asynchronous calls in one background context can interleave; being the only writer context does not serialize them. With a storage fake that returns copied objects, concurrent puts of `a` and `b` left only `b`. The existing test fake returns shared object references and tests puts sequentially, hiding this failure. The symptom is lost cache entries/redundant work, not invented cryptographic proof. Repair the shared helper, not each caller.

### 5. Work/consent limits need to match the user interface — high priority


`background.js:83-97` performs fetch plus `arrayBuffer()` without a project-level byte limit or fetch deadline. `makeLimiter(2)` caps active jobs but not waiting jobs, total page work, or synchronous WASM execution time. `content/scan.js` retains per-URL image sets without a corresponding complete cleanup path. Native file reads and pixel decoding also lack an explicit product resource budget. This review did not measure an exhaustion attack; the missing bounds are visible in the code.

All granted origins are registered for scanning (`background.js:139-152`), including a host granted only through “Allow access to <host> and verify.” Permission removal changes future script registration but does not explicitly stop existing scripts/queued jobs; the message listener ignores sender context. The existing reload note is a limitation, not immediate revocation. In addition, pills are clickable spans without keyboard behavior, and the consent text says “Every image” although only qualifying visible images are examined. These should be corrected before broadening scanning.

### 6. W3-M is recoverable work, not a new implementation — medium priority


The separate W3 branch contains implementation `bc2cc42` and review-evidence correction `30e52c8`. Its packet, `docs/reviews/W3_M_CRYPTO_REVIEW.md` on that branch, still says maintainer sign-off pending. C1 approval is not W3-M approval. The main-branch SynthID plan's unchecked implementation items therefore describe main, not the full local project history.

W3-M adds descriptive signed watermark declarations only; it does not detect SynthID. It also regenerated the old JPEG/PNG fixtures and their test CA. C1 subsequently added SVG fixtures under the current CA. Do not copy an old CA or overwrite the current vector manifest while integrating W3: reconcile the generator, then regenerate one internally consistent corpus. Expect at least twelve top-level vectors after adding the declaring vector to the current eleven.

The branch's helper collects soft-binding algorithms whenever a recognized watermark action exists. Before adopting it, specifically test a manifest containing both a fingerprint binding and a watermark action, assertion-label lookalikes, malformed assertions, and unrelated actions. Do not imply an algorithm is the watermark's scheme unless the signed evidence establishes that association. Where association cannot be established, describe it as a signed soft-binding algorithm rather than guessing. This is a targeted review question, not a claim that the previous cryptographic validator was bypassed.

### 7. The registry prototype needs a safety pass before deployment/client use — gated priority


`provenance-log/src/lib.rs::hex_decode` slices UTF-8 strings at byte offsets, so malformed non-ASCII key text can panic rather than return an error. `Entry::decode` accepts empty strings that `signed_body` rejects, while public `Entry::encode` assumes validity with `expect`. `verify_inclusion` calls `next_power_of_two` on caller-supplied sizes without an overflow guard. These are code-derived edge cases to pin with tests before exposing remote inputs, not a claim of a currently reachable browser vulnerability.

`plog keygen` uses overwriting file writes for private/public keys. `append` chooses a filename and then uses an overwriting write; contiguous-name validation exists, but simultaneous operators can race. Checkpoint history files can also be overwritten at the same tree size. Future operator commands must refuse accidental replacement and define interruption/concurrency behavior.

`plog verify` loads operator and registrant public keys from the directory it verifies. That is useful operator tooling but is not independent authentication of an untrusted remote directory. The future client must pin the operator trust source outside the downloaded data. The existing `--previous-note` prefix check is useful and must be retained; do not mistakenly design a second consistency implementation because a stateless invocation omitted that option. Registrant IDs also become path components and need a constrained identifier grammar.

### 8. Calibration supports narrow observations, not “certainty” — medium priority


`docs/CALIBRATION.md` records valuable transform results: DWT detection 45/100 pristine and 0/100 after the tested transforms; bzh 30–85/100 across variants; PDQ loses crop robustness. Preserve those negative results. But the text also says “certainty in the positive direction” and “we measured better” than a vendor's 1/1000 false-positive rate. Zero observed false positives does not establish either claim.

Even if 2,861 samples were independent, zero hits give a one-sided 95% binomial upper bound of approximately 0.10465%, using `1 - 0.05^(1/n)`. For 2,800 it is approximately 0.10693%. At least 2,995 independent negatives with zero hits would bring that bound below 0.1%. This is a proposed stronger reporting standard, not a retroactive change to the previously approved empirical gate. The current DIV2K HR/LR sets contain related source images, so their nominal counts cannot simply be treated as independent, nor can transformed siblings repair that problem. Also, fixed-threshold hit tables are not a full ROC curve (a comparison over multiple thresholds).

### 9. Trust freshness, packaging, and batch automation have smaller but real gaps


The bundled anchors record a 2026-07-27 fetch. The refresh workflow intentionally discards date-only changes, so that date alone does not prove the upstream list is stale or the refresher failed. No fresh upstream certificate comparison was performed here. Track last successful check separately from certificate-content identity, and never silently trust a fetched replacement.

The extension packager warns, rather than fails, for an unexpected trust bundle. The npm packager can proceed without optimization; neither script enforces the stated byte budgets. The fresh wasm-pack build reported missing license files. Build scripts do not record enough information to tie published artifacts to a reviewed dependency resolution. Release checks should inspect archive contents and trust-bundle identity, not just trust a successful command exit.

`lens verify --json` emits no JSON record for a file-read failure, and batch exit status is the numerical maximum of the per-file codes (`main.rs:176`), so an error code of 2 can be hidden by a verdict code of 10/20/30. This follows the documented contract; changing it needs an explicit compatibility decision. Add observable per-input outcomes first, without silently reordering existing exit codes.

## Plan of Work


### N0 — A dependable baseline before feature work


Scope: restore the standard development checks and make the reviewed build reproducible. Fix formatting and actual warnings in `crates/provenance-log`, then rerun full clippy until all affected targets pass; do not suppress warnings globally. Review and commit the existing root lockfile with no dependency updates, remove its ignore rule, and exact-pin the already reviewed C2PA version if the maintainer accepts that dependency policy. Record the complete resolved dependency diff, including a clean-clone check. Pin the tested Rust toolchain and build-tool versions, or explicitly document any remaining floating tools; do not combine this milestone with an automated SDK upgrade.

In `.github/workflows/ci.yml`, add the existing Node helper test, standalone no-default-feature core tests, and a real standalone WASM build plus `scripts/wasm_smoke.mjs`. Keep feature-heavy model compilation covered. Make the model-test skip visible in the summary; a separately opt-in model job must fail if its pinned model is expected but missing. Keep default tests free of accounts, remote inference, and runtime model downloads. A literally offline fresh clone still requires pre-fetched/vendored compiler and crate dependencies; document that prerequisite instead of promising an impossible empty-cache build.

Acceptance: on a fresh checkout with the recorded toolchain and dependency cache, format/clippy/workspace tests, reduced-feature tests, Node tests, target check, and real-WASM smoke all pass. CI runs these checks on the candidate branch. The build records the source commit and lock hash. Zero dependency version changes are hidden in a lint-cleanup commit. Scope estimate: small, approximately 1–2 focused implementation sessions, excluding external review.

### N1 — Honest detector outcomes and bounded examination


Scope: repair the shared detector contract in `layers/watermark.rs`, then update every implementation/caller found by searching for `WatermarkDetector`, `.probe(`, calibration examples, and tests. Introduce the smallest explicit result: detected hit, completed negative, or unavailable with a reason. Keep the existing single watermark layer. If any supported detector hits, return Indication naming it; if none hits and any configured detector could not run, return NotEvaluated naming incomplete checks; return NoSignal only when all configured checks completed negatively. An absent optional model is a capability limitation, not evidence that it was checked.

In `layers/stable_signature.rs`, validate expected model input/output shape, safely access output values, reject non-finite scores, and retain runtime errors. Pin/document the known model's digest in the export/inference workflow. An unknown caller-supplied model must not inherit the calibrated IMATAG claim: refuse it in the production detector path or require a clearly experimental, non-verdict path. Choose refusal first; do not build a generic model registry. Store no private model files in git.

Define conservative initial operational limits and record them as product choices: 25 MiB input bytes, 40 million decoded pixels for native image analysis, a 15-second network deadline, and at most 50 pending page jobs with two active fetches. Measure ordinary sample behavior before finalizing; these are proposed safety defaults, not measured guarantees. Reject excess input explicitly as not examined. Check actual streamed bytes, not only `Content-Length`; bound a CLI read even when file metadata is misleading. Use existing decoder limits and checked dimensions. The shared sans-IO verification entry must also reject oversized direct WASM input. Fetch cancellation does not interrupt synchronous WASM; test parser execution separately and add a dedicated killable worker only if measured execution or recovery requires it.

Acceptance: mocked detector runtime error, unsupported dimensions, empty output, NaN, and wrong model produce no false negative claim and no panic. Mixed negative/unavailable detectors remain NotEvaluated; a genuine hit remains Indicated. A missing-length oversized response is stopped while streaming, a stalled response times out, and the next normal image still works. Native and direct-WASM oversized inputs fail honestly. Scope estimate: medium, 2–3 sessions; C2PA execution-boundary changes require the existing cryptography review before merge.

### N2 — Browser badges that stay attached to the right evidence


Scope: update `content/scan.js`, `background.js`, `lib/scan_support.js`, and popup controls together. Track each image's current request generation and resolved source. Clear an old badge before requesting a changed source, and discard responses whose source/generation no longer matches. Reobserve changes from `src`, `srcset`, `sizes`, load events, and viewport-dependent `currentSrc`; clean up detached image and URL records. Do not poll the whole DOM indefinitely. Use a real keyboard-accessible button for the pill.

For the first correct version, prefer deleting the session-persistent URL report cache over building a complex invalidation service. Retain bounded in-flight deduplication for the same document/source generation; fetch anew on an explicit “Verify again.” If session caching remains, it needs serialized writes, copied-value test doubles, explicit expiry/context/engine-and-anchor identity, and a bypass. A URL is not a byte identity. Reports must include retrieval time and distinguish fetched-file verification from verification of displayed pixels. Add a small test that fails under the current lost-update behavior even if the chosen repair is removal.

Separate “may fetch this host” from “scan pages on this site.” Store an explicit scan-enabled set, register scripts only where it intersects granted browser permissions, and do not turn an image-CDN grant into automatic scanning consent. On disable/revocation, message existing tabs to remove pills, invalidate pending generations, reject new scan requests without current authorization, and cancel queued work. A right-click verification remains a deliberate one-file action. Keep sender/document checks in the shared message boundary. No install-time broad host grant and no silent permission expansion.

Acceptance: use a test page that swaps one image from signed to unsigned, changes `srcset`, reuses the same URL with new bytes, delays an old response, removes/reinserts nodes, and revokes access during queued work. In Chrome and Firefox 128+, no old Verified badge survives an image-generation change; disable stops current scanning without requiring reload; CDN access alone does not enable page scanning; keyboard users can activate a badge. Test helper code in Node and exercise actual browser lifecycle behavior separately. Scope estimate: medium, 2–4 sessions; the revised consent semantics need maintainer agreement.

### N3 — Finish W3-M without confusing declarations and detection


Scope: start a new integration branch from the then-approved main, preserving the original W3 worktree and commits. Reuse the W3 implementation and tests after comparing them against C1; do not blindly replace whole files or carry old fixture binaries. Reconcile `examples/gen_vectors.rs`, the current c2pa 0.90.13 imports, `CredentialSummary`, `json.rs`, CLI rendering, popup rendering, and parity tests. Regenerate all affected signed vectors together under one test-only CA and verify the complete corpus. Production anchors must remain byte-identical.

Retain the exact declared-watermark note: “the verified credential records an invisible watermark; the watermark was not independently detected”. Recognize explicit bound/unbound/legacy watermark actions; a soft binding alone is insufficient. Add the mixed-binding and malformed-label tests identified above, plus JSON/DOM escaping and absence on unsigned, invalid, and untrusted reports. Extend real-WASM assertions beyond just the top-level verdict to the optional declaration and relevant findings. Get a fresh maintainer sign-off for the reconciled diff; the existing C1 approval cannot be reused.

Complete the ungated portion of Claude C4 now in `docs/CLAUDE_MARKS.md`, README/manual links, and the CLI/WASM/Chrome/Firefox capability matrix. Separate “parser accepts format,” “generic signed fixture passes,” “genuine vendor fixture passes,” and “vendor detector available.” Document JPEG/PNG/SVG evidence, missing positive Anthropic evidence, native-only pixel detectors, and unsupported/unproven other output formats. Keep C4's conditional text-integration acceptance open; do not mark all of C4 complete simply because the useful documentation exists.

Acceptance: at least twelve corpus vectors pass native and real WASM with one coherent test root. The declaration is descriptive only, never a new watermark hit. Six unsigned Claude specimens remain no-signal for C2PA. The feature works without Anthropic/Google accounts, vendor APIs, issuer-name heuristics, or new runtime dependencies. Scope estimate: small-to-medium, 1–2 sessions plus human review.

### N4 — A local evidence workbench, in three small increments


First add a local-file inspector inside the existing extension, for example `extension/inspect/inspect.html` and a small module. A file picker/drop target passes the user's chosen bytes to the same bundled engine and production anchor loader; no website, backend, upload, account, or filesystem-wide permission is needed. Reuse existing result rendering where practical rather than making a second verifier. Display filename, supported checks, the four canonical verdict phrases, and actionable explanations of unavailable checks. Do not inline-render untrusted SVG/HTML or execute it; file inspection does not require a preview.

Second add an “evidence receipt”: a locally downloadable JSON document wrapping the existing report with a format version, asset SHA-256, byte length, MIME-hint source, engine/build identity, enabled detector/model identity, anchor-bundle digest/date, and examination time. Use browser Web Crypto for the local-file digest. Keep the existing `verify_bytes` contract unless a concrete need requires changing it. Default receipts omit source URL query strings, document text, and local absolute paths; show what will be exported before saving. A receipt is an unsigned record of this tool's observations, not a Content Credential, certified audit, or legal compliance attestation. Do not import a receipt and treat it as proof; recompute when verifying it.

Third allow selecting exactly two local files and display the existing reports side by side with explicit changes: “credential present in A, absent in B,” “declared watermark context present,” or “check unavailable.” This is valuable for learning what social platforms preserve and for collecting Claude originals/exports. It cannot prove B was derived from A; even perceptual similarity would remain only an indication. Keep it two files until real users need batch workflows. If CLI receipts are added later, provide a per-input outcome for read errors and separately approve any exit-code changes.

Acceptance: disconnect the network, inspect the three labeled flower SVGs, and see unsigned C2PA results without a Claude-detection claim. Compare a generic signed JPEG with its stripped copy and observe Verified versus Inconclusive without calling the latter authentic. The exported receipt names the exact bytes/configuration and contains no implicit upload or private path. All controls work by keyboard. Scope estimate: one session each for inspector, receipt, and comparison; stop after the first useful increment if demand is weak.

### N5 — Make the registry pilot safe, then prove its usefulness locally


Scope: reuse `provenance-log`; do not build a second service/database. Begin with the parser/operator issues above: ASCII-safe hex parsing, consistent decode/encode validation, overflow-safe proof sizes, bounded note/proof/entry sizes, strict registrant IDs, refusal to overwrite keys/entries/history, and interruption-safe publication of current snapshots. Add malformed-input tests, an independent known-answer tree/signature check, and a registry fuzz target. Keep signing secrets outside the working tree; never print them in reports.

Prove a small local pilot with temporary test keys and fixtures before deployment: signed entries, deterministic index, signed checkpoint, and previous-checkpoint prefix verification. Explicitly distinguish a self-consistent directory from one authenticated by an independently pinned operator key. Refuse shrinkage, rewritten prefixes, wrong operator/registrant keys, invalid signatures, malformed dates, and inconsistent entry counts. Key rotation and first-checkpoint bootstrapping need an explicit trust decision. A test-only key must never become a production anchor.

Proposed sequencing refinement: seek approval to run the G3 contract against a local directory before a live endpoint exists, without relaxing the live-integration or cryptography gates. The existing registry plan gates G3 on G2 live; until this revision is approved, limit work to prototype tests and hardening, not the product Layer-3 client. For an approved small offline client, fetch/read the full signed snapshot and compare all entries. This avoids a network protocol and avoids depending on the exact-word nomination index: that index guarantees recall only through distance 15, not the selected threshold 31. Move to indexed lookup only when a measured size/latency limit justifies it.

Use PDQ's quality score to reject insufficiently informative images before matching, with a measured cutoff and uniform/near-uniform controls. Only qualifying entries explicitly declaring AI involvement may contribute an AI-related Indication. A registration timestamp is a signed registrant claim, not independently proven creation time. A missing entry is not evidence of authenticity. Positive matches require checked entry signature, authenticated checkpoint, distance threshold, and appropriate history checks; corrupt/unavailable/incomplete data is NotEvaluated, not a confident miss. No registry match may produce Proof.

Acceptance: a registered image's stripped/re-encoded copy matches within the documented limits; an unrelated image does not; a crop can honestly miss; malformed data and a substituted log key never create a hit. A second verifier or published independent vectors check the cryptographic constructions. Only after maintainer sign-off and a named operator/key/hosting decision should the maintainer deploy the git-backed pilot. Extension registry consent and any network lookup remain a later separately approved slice. Scope estimate: 3–5 sessions plus review/deployment; do not put it on the critical path for N4.

### N6 — Honest measurements and a release candidate, not a bigger promise


Scope: correct calibration prose first, retaining all historical measured counts. Add confidence bounds and corpus provenance/deduplication notes. Group all derived images by source so calibration and holdout sets cannot share a source; do not count HR/LR/re-encoded siblings as independent negatives. Preserve bzh's proxy limitation. Reproduce the exact exported model, preprocessing, threshold, and digest. Collect at least 3,000 independently sourced clean negatives for a proposed 0.1%-at-95%-confidence zero-hit target; if hits occur, compute the proper binomial interval and collect more evidence rather than hiding them or tuning on the holdout set. Add genuine bzh-generated positives if lawfully available. Without them, say which proxy was tested. No uncalibrated ensemble score.

Package the smallest completed slice, starting with N0–N2 if later gates are pending. Align Cargo/extension/npm versions, include license texts matching the declared dual license, and record commit, lockfile hash, toolchain, model identity if applicable, anchor digest, artifact hashes, and raw/gzip sizes. Enforce the existing WASM budgets of at most 7,000,000 raw and 2,500,000 gzip bytes or record an explicit approved budget revision. Refuse unknown/test anchors unless a clearly separate development package was requested. Verify the archive includes the content script, helper module, engine, popup/inspector as applicable, and production anchors. Run smoke against the artifact actually being packaged.

Review newer automated C2PA/trust-list branches as separate evidence packets; their existence is not approval. Retrieve current advisory/CI results and upstream trust contents before claiming release readiness. Preserve the difference between certificate-content date and last successful upstream check. Do not turn advisory audit into an enforcing policy without the maintainer's decision. Run the candidate's fuzz job on Ubuntu; a Windows runtime-launch failure is not evidence of a clean fuzz run or a missing DLL unless actually checked.

Acceptance: the documented user flows work in current Chrome and Firefox, including engine/fetch failures, worker restart, malformed SVG, unsupported text, and permission revoke. Model-backed acceptance visibly runs inference or is listed as unverified. There are no unexplained format/lint failures, hidden dependency updates, or missing package files. Prepare a release packet; pushing main, publishing npm, submitting stores, and deploying a log remain separate explicit maintainer actions. Scope estimate: 1–2 sessions for packaging/review, while expanded data collection is separately scheduled work.

### Vendor track — Keep the useful parts moving without fabricating detection


Claude C0-positive needs an original, non-sensitive Anthropic-signed specimen, byte hash, acquisition surface/model/date, permission to retain it, and an inspected trust outcome. A trusted signature may establish provenance statements but not that Claude originated every idea. Existing flower files are valuable unsigned negative controls; adding a visible label, ordinary metadata, a local signature, or converting to PNG does not create vendor evidence.

Claude C2 needs a runnable/documented third-party text-watermark contract, licensing/access, supported models/languages/lengths, normalization rules, error states, and measured calibration before any verdict influence. C3 then adds explicit user-selected text with per-use consent if a separately approved external request is necessary. Never scan page text automatically, infer authorship from writing style, or scrape private detector interfaces. Until these requirements are met, show “not evaluated” with the reason.

Actual SynthID detection, W3-D, has its own equivalent gate. Google's generic AI classifier is not a SynthID decoder. Any image/text upload needs a specifically approved privacy exception and updated disclosures; neither W3-M nor this plan grants one. The official vendor documentation in the older plans was checked in August 2026, not refreshed in this September review. Re-check primary sources when deciding whether a gate can open; do not state that no API exists today based only on that historical snapshot. No recurring monitoring task is created by this plan.

## Concrete Steps


Run from `D:\sandbox\projects\provenance-lens`. Before an approved implementation milestone, inspect status, current branch, and main/remote relationship. The current documentation branch is not permission to implement every item. Do not edit main or create a temporary worktree. Create a named `codex/` branch from the agreed main tip; if main and remote differ, report the difference rather than silently discarding local work or pushing it.

Baseline commands used for this review, with a populated local crate/tool cache:

    git status --short --branch
    git rev-parse main
    git ls-remote origin refs/heads/main
    cargo tree -p c2pa --depth 0 --locked --offline
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --locked --offline -- -D warnings
    cargo test --workspace --locked --offline
    cargo test -p provenance-core --no-default-features --locked --offline
    cargo check -p provenance-wasm --target wasm32-unknown-unknown --locked --offline
    node scripts/scan_support_test.mjs
    wasm-pack build crates/provenance-wasm --target web --out-dir ../../target/plan-review-wasm --locked --offline
    node scripts/wasm_smoke.mjs target/plan-review-wasm
    cargo test -p provenance-core --features stable-signature --test stable_signature --locked --offline -- --nocapture
    git diff --check

The temporary WASM output is under ignored `target/`, not the extension's installed engine; it does not overwrite a loaded development package. This review ran a release-profile build without the separate wasm-opt step. For release, run the existing packaging build/optimization/smoke process after N0/N6 make its policy checks enforceable.

N1 regression tests belong alongside current watermark tests and model-free test doubles. N2 should extend `scripts/scan_support_test.mjs` and `scripts/serve_testpage.mjs` rather than introducing a test framework. N3 uses the existing vector generator and parity suites; generator invocation is deliberately not part of this read-only review because it rotates test fixtures. For N4, load the unpacked extension after building its engine and inspect files with the network disconnected. For N5, operator exercises must use a new explicitly named temporary test directory, never the real pilot or production key paths.

To reproduce the N2 cache issue with current code, run this JavaScript with Node in module mode from the repository root. The fake storage deliberately clones values:

    import { makeSessionCache } from "./extension/lib/scan_support.js";
    const data = {};
    const store = {
      async get(k) { return structuredClone({ [k]: data[k] || {} }); },
      async set(v) { Object.assign(data, structuredClone(v)); }
    };
    const cache = makeSessionCache(store, "cache", 200);
    await Promise.all([cache.put("a", 1), cache.put("b", 2)]);
    console.log(data.cache); // observed: { b: 2 }; required if retained: both keys

At each milestone, capture actual output, update this plan and `development_status.md`, inspect the complete diff, stage explicit intended paths, and commit at the milestone boundary. Never stage unrelated work with an indiscriminate add. Human-gated work stays on its branch until the required dated approval exists.

## Validation and Acceptance


The recommended first delivery is N0, not an SDK bump: green format/lint/tests plus a reproducible actual browser-engine test. The second delivery is honest detector outcomes and correct browser evidence attachment. W3-M is the next recovered feature; the local inspector/receipt/compare flow is the next new user capability. Registry work follows its own gate and should not block those releases. No milestone may claim positive Claude/SynthID interoperability on the strength of synthetic fixtures.

The four canonical verdict phrases and combination precedence remain unchanged. Errors are visibly errors or unavailable checks, never a new authenticity signal. Every feature has a negative/control case: unsigned versus signed files, missing versus present model, failed versus completed detection, old versus current image generation, revoked versus granted scan consent, authenticated versus substituted log keys. Existing native and actual WASM tests must remain green, and feature-specific regressions must fail on the old implementation and pass on the new one.

## Idempotence and Recovery


Read-only Git checks and the validation commands can be repeated. Builds update ignored output directories; they must not replace a loaded engine or a production package as a side effect of a review. Keep the original W3 branch/worktree intact while preparing its reconciliation. If integration conflicts occur, stop and inspect them; do not reset either branch or restore whole historical files over C1.

Fixture generation rotates the test CA and signed vectors together. Run it only on the feature branch after preserving the current committed corpus, review every affected fixture, and keep production anchors untouched. Roll back a bad feature by a targeted reviewed revert, not by discarding unrelated work. If a trusted dependency change fails validation, retain the last reviewed resolution and leave the candidate on its branch.

Registry operator experiments use newly created, explicitly named test directories and ephemeral test keys. Never retry key generation over an existing key file or delete a pilot directory to restart. An interrupted append/publication must leave prior entries and checkpoints verifiable; stage new outputs and publish them only after validation. Remove experimental network/client behavior by leaving it disabled, not by weakening signature or error checks. No implementation checkpoint automatically authorizes a merge, push, or release.

## Surprises & Discoveries

- Observation (N6, 2026-09-18): verifying the extracted npm archive exposed wasm-pack's omission of `provenance_wasm_bg.wasm.d.ts`; the file list now explicitly includes it. The release-profile engine already fits hard raw/gzip budgets without a separate optimizer. Upstream trust contents now differ (30 certificates versus bundled 28); the read-only check did not alter trust.

- Observation (N5, 2026-09-18): replacing the mutable index before the latest note can expose mixed snapshots. Immutable size-specific indexes plus an atomic latest-note commit point preserve the prior readable prefix across interrupted publication. The operator regression exercises interruption and retry.

- Observation (N3, 2026-09-17): the SDK's generic assertion lookup uses prefix matching; N3 explicitly accepts only known action labels and the exact soft-binding label. Existing W3 wording could imply a fingerprint algorithm identifies a watermark, so it now says signed soft-binding algorithm, not necessarily the watermark. Tiny fixtures are correctly NotEvaluated for pixels under N1.
- Observation (N3, 2026-09-17): Windows checkout conversion changed the generator's unsigned SVG input to CRLF. The generator now normalizes only that project-owned source fixture before signing, then preserves signed bytes and regenerates every vector together. User specimens and production anchors remain byte-identical.
- Observation (N4, 2026-09-17): the offline flow requires no extension API or new permission. Native browser file/digest/download primitives plus the existing verifier suffice. Actual production-anchor tests reject the synthetic test CA; an isolated resource response enables the positive comparison without changing production files.

- Observation (N1/N2, 2026-09-16): the calibrated ONNX export was in `D:\sandbox\corpus\bzh`, outside the repository. Its adjacent PyTorch weights match the earlier recorded hash, and the export at SHA-256 `4f7d80027d5d7ecf5c176fabc3d86c8ccd791cc5c830d23a96114425cd1f3e57` passes the vendor pair. Pinning that artifact closes the arbitrary-model claim gap without downloading or committing a model.
- Observation (N2, 2026-09-16): Chromium can retain a cached larger srcset candidate when sizes shrinks. The correct badge follows actual currentSrc, not the test author's expected candidate. The browser test now uses distinct candidate URLs. Both Chromium 151.0.7922.34 and Firefox 153.0 pass source/srcset/sizes/viewport changes, delayed replies, same-URL new bytes, detach/reinsert, keyboard activation, and disable/resume. A separate regression prevents registration reconciliation from overwriting newer consent writes.


- Observation: C1 is already in local main, although the latest plan/dashboard say “approved for merge.” Evidence: main `5f1dcb6` contains `8a0e689`; remote main still advertises `4445872`. Consequence: distinguish local completion from publication.
- Observation: W3-M has two existing commits outside main. Evidence: `git log main..codex/w3-synthid-context` shows `bc2cc42` and `30e52c8`. Consequence: integrate/review existing work and reconcile test roots instead of starting again.
- Observation: asynchronous cache writes race even with one writer context. Evidence: two concurrent puts yielded one persisted key. Consequence: remove unnecessary persistence first, or serialize the single shared implementation and test copied storage values.
- Observation: the fresh browser artifact already fits the documented size budget without a separate wasm-opt pass. Evidence: 6,946,939 bytes raw and 2,333,698 gzip. Consequence: preserve the budget and measure future additions rather than assuming pixel detectors will fit.
- Observation: the bzh known-answer test prints “skipping” but is counted as passed. Consequence: report model execution separately from test-process success.
- Observation (N0, 2026-09-16): the original lockfile's SHA-256 stayed `30ae9b4dbf53f1da9fc0a659076f735c40147ae76bd3593ca229707f73d752ff` after exact-pinning c2pa and running the checks; `cargo tree` still selects 0.90.13. The only new source of a vendor-model result is the explicitly ignored test when run with a supplied model. Consequence: N0 makes the existing resolution reproducible without slipping in an SDK update or a vacuous green model test.

## Decision Log

- Decision (2026-09-18, maintainer approval): record the user's "I approve this changes" as sign-off for the explained existing trust-related changes at candidate `2953dc4`, for local integration once CI passes. Scope and exclusions are recorded in the original ExecPlan. This is not permission to push main, update certificates, activate G3 or publish a release.

- Decision (N6, 2026-09-18): prepare development-only 0.3.0 artifacts carrying current work, rather than publish an unapproved slice. Reuse Node stdlib and wasm-pack for one isolated packaging flow; keep shell entry points, enforce existing size limits and a certificate-content allowlist, and record exact archive contents. No new dependency. Do not update anchors or label existing HR/LR siblings as independent; expanded calibration and release remain incomplete. Ubuntu audit retains advisory policy and becomes manually dispatchable for a later authorized candidate run.

- Decision (N5, 2026-09-18): implement the user's next-two-stages request as prototype hardening plus N6 preparation, preserving the explicit G3 ordering gate. Reuse the current decoder, tree and prefix verifier; no new dependency. Entry encoding becomes fallible. Add bounded canonical parsing and serialized, no-overwrite history publication with an atomic latest-note commit point. Human review is pending; this is not trust-policy or deployment approval.

- Decision (N3/N4, 2026-09-17): implement the user's two requested milestones on a new branch created from main and carrying unmerged N0–N2 prerequisites. Do not merge those prerequisites merely to start N3. Preserve the original W3 worktree; reuse reviewed source hunks, not its obsolete signed corpus. Rationale: main approval remains separate from implementation authorization.
- Decision (N3, 2026-09-17): keep the W3 `algorithms` JSON field for descriptive soft-binding identifiers, with explicit non-association wording in every UI; exact known assertion labels only. Regenerate twelve current-container vectors and fuzz seeds together under one test root. No SDK, anchor, detector, or dependency update. Rationale: avoid both lookalike labels and treating a coexisting fingerprint as watermark detection.
- Decision (N4, 2026-09-17): use native file inputs/drop, Web Crypto, Blob downloads, the existing WASM entry, and shared claim rendering. Share one immutable-resource loader with background verification; missing anchors fail explicitly. Receipts are unsigned format version 1, bind exact asset/WASM/anchor bytes, and contain no source URL or absolute path. Do not implement previews, receipt imports, batch workflows, CLI receipts, or a new verifier. Rationale: the requested three local increments need no framework, service, permission, or new dependency.

- Decision (N1/N2, 2026-09-16): implement the two user-requested milestones together on a new branch containing N0. Use `DetectorResult::{Detected, NotDetected, Unavailable}` and a byte-taking model loader; only the pinned calibrated ONNX may influence production verdicts. Add `sha2 =0.10.9` as an optional direct dependency already present in the lockfile; no resolved package versions change. The new detector/constructor Rust APIs are intentional source compatibility changes; JSON verdict/report shape and CLI exit precedence remain unchanged.
- Decision (N1/N2, 2026-09-16): adopt the proposed 25 MiB, 40 million pixel, 15-second, two-active/50-waiting defaults, plus existing-decoder allocation limits and a 64 MiB model ceiling. Representative fixtures fit comfortably; direct-WASM oversized and nested/truncated SVG recovery checks pass. The slowest sampled WASM examination was 24.1 ms; this is not a worst-case bound. Do not add a killable product worker without evidence requiring it. The changed pre-parser execution boundary still requires the standing human review before merge.
- Decision (N2, 2026-09-16): delete URL report persistence, keep only pending work sharing, and persist explicit scan choices separately from browser grants. Existing grants do not silently opt into scanning. The user's request to implement N2 authorizes this specified consent redesign; it is not approval to merge/publish. Playwright 1.62.1 is pinned test tooling only, with generated installs under ignored target output.


- Decision: Treat this review as analysis and planning only; no runtime fixes, dependency updates, merges, or publication. Rationale: that is the current user request. Date/author: 2026-09-16 / plan author.
- Decision: Recommend N0–N2 before expanding detection coverage. Rationale: correctness of evidence attachment and unavailable states matters more than another unsupported feature. Date/author: 2026-09-16 / proposed sequence, not maintainer implementation approval.
- Decision: Reuse W3-M and the existing registry prototype; prefer existing engine/rendering and browser APIs for new UX. Rationale: avoid duplicate implementations and speculative services. Date/author: 2026-09-16 / planning recommendation.
- Decision: Split useful C4 documentation from vendor-dependent acceptance; retain C0-positive/C2/C3/W3-D gates. Rationale: documentation and local evidence tools can improve without inventing vendor detection. Date/author: 2026-09-16 / planning recommendation.
- Decision: Any revised registry ordering, trust policy, permissions semantics, model acceptance policy, or compatibility-breaking CLI behavior requires explicit approval before implementation/merge as applicable. Rationale: planning must not erase earlier authority boundaries. Date/author: 2026-09-16 / gate clarification.
- Decision (N0): commit the previously local lockfile unchanged, change the C2PA requirement from a compatible range to exactly the reviewed 0.90.13, and pin the installed Rust/wasm-pack versions. Keep the model inference test explicitly ignored by default and offer a manually triggered, SHA-256-checked CI job for a separately supplied model. Rationale: preserve the reviewed verification resolution while making both the default and opt-in validation outcomes observable. Date/author: 2026-09-16 / implementation author; dependency-manifest review before merge is pending.

## Outcomes & Retrospective

2026-09-18 local merge: candidate CI `35364243397` passed and the approved implementation plus documentation-only approval record fast-forwarded into local main at `3ad82a0`. Separate data, deployment, trust-update and release gates remain open; no main push or publication occurred.

N6 preparation (2026-09-18): corrected measurement claims, source-group audit and archive gates are working; both development archives pass extracted WASM smoke and the extension passes Chromium/Firefox offline/lifecycle checks. N3/N4 remote CI `35362161502` passed after its authorized push. Human approvals, current-anchor review, candidate Ubuntu evidence, expanded independent data and final release remain outstanding. No claim of full N5/N6 acceptance is made.

N5 prototype (2026-09-18): hardening and local recovery/prefix checks are implemented without activating product Layer 3. N3/N4 is now pushed; main remains untouched. G3 sequencing, quality calibration, independent operator trust, human review and deployment remain separate decisions.

N3/N4 local implementation (2026-09-17): all requested increments are available on the candidate branch. Workspace 66 passed/one model ignored, reduced core 43 passed, format/strict lint, Node helpers/background/receipt checks, twelve real-WASM vectors plus declaration/hostile-input checks, and Chromium/Firefox lifecycle and offline-inspector checks pass. Raw/gzip WASM is 6,968,175/2,341,807 bytes. The review packet records identities and remaining gates. N1/N2 was pushed and its CI is green; N3/N4 remains local. No main merge/push, new vendor detection, or product release is implied.

N1/N2 local implementation (2026-09-16): unavailable detector states survive into reports, arbitrary models cannot inherit calibration, and input/queue/fetch limits reject unexamined work explicitly. Page badges follow the current source generation and current consent; report wording now identifies fetched-file scope. Local verification passes: 60 default workspace tests, 37 reduced-feature tests, the explicitly invoked model known-answer pair, strict lint/format, Node service regressions, all eleven real-WASM vectors, and Chromium/Firefox lifecycle cases. The review packet records hashes, commands, limitations, and the remaining human/remote-CI/installed-extension checks. Production C2PA validation code, anchors, and signed corpus are unchanged; the shared execution-entry guard is the trust-review surface.


The planning milestone is complete: the current implementation was checked against the latest plans, reproducible issues were separated from source-derived risks, a fresh native/WASM baseline was recorded, and the next work is staged with user-visible acceptance criteria. Runtime defects identified here remain unfixed intentionally. No new detector, permission, remote upload, or trust decision was introduced. The most important change in direction is to improve evidence quality and usability while treating vendor-specific coverage as separately demonstrable work.

N0 local implementation (2026-09-16): the previously failing formatter/linter now pass; native and reduced-feature suites and the freshly built WASM vector smoke pass. The root lockfile was made a tracked build input without changing its checksum, and a new `.gitattributes` rule preserves those bytes in a fresh Windows clone. That clone passed the locked, offline workspace suite. A reviewed model is still required for the optional model-inference result. Online GitHub CI and the human dependency review remain separate acceptance steps.

## Artifacts and Notes


Review baseline: local main `5f1dcb6a9d5f5669ede78e46babb8d61930b3ddc`; Rust/Cargo 1.97.1; Node v24.18.0; resolved c2pa 0.90.13. The ignored root lockfile used here has SHA-256 `30ae9b4dbf53f1da9fc0a659076f735c40147ae76bd3593ca229707f73d752ff`. It is local evidence, not a lockfile currently available from a fresh clone.

Observed on 2026-09-16: workspace 55 tests reported passed, including one bzh test that skipped inference; reduced-feature core 34 tests passed; WASM target check passed; Node scan helper assertions passed; all eleven vectors passed through the freshly built real WASM artifact. Format and strict lint checks failed as described above. No actual browser interaction, new fuzz run, advisory audit, production trust-list refresh, release/store verification, expanded calibration run, or genuine positive vendor-specimen test was performed. This is a development review, not a comprehensive security audit.

Fresh WASM SHA-256: `44cdeef03b739bd1e20772119b2ce29fc1c99cd9a99e98538f22bb54aa2efd9f`. Output: `target/plan-review-wasm/provenance_wasm_bg.wasm`, 6,946,939 raw bytes, 2,333,698 gzip bytes. Production anchors were not modified. The original W3 worktree and all historical fixture files were left untouched.

N0 local result: Rust 1.97.1, wasm-pack 0.15.0, Node 24.18.0; workspace 54 tests passed and one model test ignored; reduced-feature core 34 tests passed; `cargo fmt --all -- --check`, strict workspace clippy, Node helper assertions, WASM target check, and all eleven vectors through fresh `target/n0-wasm` passed. That WASM artifact has SHA-256 `44cdeef03b739bd1e20772119b2ce29fc1c99cd9a99e98538f22bb54aa2efd9f`, 6,946,939 raw bytes and 2,333,698 gzip bytes. The explicit `--ignored` model command without `PROVENANCE_LENS_BZH_ONNX` exited nonzero as intended. Production anchors, proof mapping, and vector contents were not changed.

N0 clean-checkout result: local clone at commit `fce9ca2` checked out tracked `Cargo.lock` with SHA-256 `30ae9b4dbf53f1da9fc0a659076f735c40147ae76bd3593ca229707f73d752ff` and passed `cargo test --workspace --locked --offline` with 54 passing tests and one explicitly ignored model test. This used an already populated offline compiler/crate cache, not an empty machine; no remote CI has run yet.

## Interfaces and Dependencies

N3 adds optional `CredentialSummary.declared_watermark` containing `action_labels`, `algorithms` (signed soft-binding identifiers, association unspecified), and the fixed note. N4 adds `extension/inspect/` and shared `lib/engine.js`/`lib/report_view.js`; receipts wrap rather than replace the existing report. Runtime, test dependency versions, and the lockfile remain unchanged.

N1 implemented interface update: `WatermarkDetector::probe(&DecodedImage) -> DetectorResult`, where the result is Detected(WatermarkHit), NotDetected, or Unavailable { reason }. `StableSignatureBzh::from_onnx_bytes(&[u8])` replaces the file-reading constructor; the CLI owns bounded file reads. `MAX_ASSET_BYTES` is checked in `Pipeline::examine`, including direct WASM calls. N2 background entries add retrievedAt/evidenceScope; the core JSON shape is unchanged. The consent set is `storage.local.scanEnabledSites`; session storage retains only the selected last result, not a URL report cache.


Keep `verify_bytes(bytes, media_type?, trust_anchors_pem?) -> JSON`, existing verdict identifiers/phrases, and the no-network core boundary. N1 needs an explicit detector outcome type because the current `Option` cannot express failure; update existing detectors, not a parallel interface. N3 reuses `CredentialSummary.declared_watermark` from the W3 branch after review. N4 adds an outer versioned receipt around the existing report and uses native file inputs/Web Crypto/download APIs. N5 reuses current Ed25519/SHA-256/PDQ code and existing dependencies under review; it does not justify a database, hosted API, blockchain, or new framework yet.

Defer PDF/audio/video claims until each has real supported-format vectors and product demand; defer a generic AI-text classifier, cloud upload connectors, detector marketplace, account system, signed receipts, blockchain anchoring, and broad background scanning. Revisit double-parsing of Verified C2PA summaries only if profiling identifies a material cost; correctness/resource limits come first. Creative features should explain existing evidence better, not manufacture more certainty.

Revision note (2026-09-16): initial consolidated successor plan from live code, Git state, executable checks, and the historical feature plans. Corrects the local-merge/W3/prototype status distinctions and prioritizes reproducibility, honest failure states, image-bound browser results, reusable evidence tooling, and gated registry/vendor expansion.

Revision note (2026-09-16, N0): recorded the implemented reproducibility baseline, native/WASM evidence, unchanged dependency resolution, and the remaining remote-CI and human-review gates without declaring the later milestones complete.

Revision note (2026-09-16, N1/N2): recorded both authorized local implementations, model identity recovery, bounds/consent decisions, full regression evidence, and remaining pre-merge checks. Historical planning findings remain as dated evidence; N3 and later work is unchanged.

Revision note (2026-09-17, N3/N4): recorded the requested N1/N2 push and successful remote CI, reconciled W3-M with twelve current-format fixtures, delivered useful C4 documentation and all three local evidence increments, and retained human/release/vendor gates. Historical planning findings above are snapshots, not claims that implemented defects remain unfixed.

Revision note (2026-09-18, N5): recorded the authorized N3/N4 push and N5 prototype hardening, publication/recovery design and remaining G3 gates.

Revision note (2026-09-18, N6): recorded implemented calibration/packaging preparation, real-archive checks, upstream trust differences and explicit remaining data/review/release gates.

Revision note (2026-09-18, approval): recorded the maintainer's explicit sign-off for the explained existing trust-related candidate and local-only merge condition; preserved independent release, trust-update, calibration and G3 gates.

Revision note (2026-09-18, local merge): recorded successful candidate CI, the approved fast-forward into local main and 69 passing post-merge tests with one optional model test ignored. Remote main and production anchors remain unchanged.
