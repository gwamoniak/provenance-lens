# Provenance Lens: ship an honest AI-content provenance verifier — Layer-1 CLI + browser extension first

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document is maintained in accordance with `PLANS.md`, which is checked into the sibling repository `../solid-broccoli/PLANS.md` (this sandbox keeps the ExecPlan rules in one place; the discipline applies to every sibling project; path updated 2026-07-18 for the new machine's flat `D:\sandbox\projects\` layout). If that file is not in your context, read it in full before revising this plan.

## Purpose / Big Picture

After this plan's wedge milestones (M0–M4), a user can right-click any image in their browser — or run `lens verify photo.jpg` in a terminal — and get one of four honest verdicts about its provenance: **Verified** (a valid, cryptographically signed C2PA provenance chain), **Indicated** (non-cryptographic signals of AI involvement), **Inconclusive** (no provenance data found — which explicitly does NOT mean authentic), or **Tampered** (provenance data present but failing validation). Nothing like this exists today with honest wording: existing "AI detectors" guess from pixels and overclaim in both directions.

The strategic bet (the "wedge"): shipping a small tool that is *provably correct about Layer 1 only* is more valuable than a big tool that guesses. In particular, the Tampered tier surfaces — at scale, in users' faces — that major platforms strip Content Credentials on upload. That visibility is the pressure mechanism for platforms to stop stripping them. Watermark detection, the registry, and heuristics come later, each behind an explicit gate, and none of them may dilute the honesty rules.

## Progress

Granular state; every stopping point must be recorded here, splitting partially-done items into done/remaining.

- [x] (2026-07-10) M0 scaffold: Cargo workspace (`provenance-core`, `provenance-cli`, `provenance-wasm`), four stub layers returning honest `NotEvaluated`, verdict tiers with approved phrases, combination rule with unit tests, std-only CLI (`lens verify`, `lens tiers`), wasm-bindgen wrapper with hand-rolled JSON, MV3 extension skeleton, 8 project agents, 2 skills, CLAUDE.md, README, git repo initialized.
- [x] (2026-07-10) M0 reconciliation with the full proposal document (`ai-content-verifier-proposal.md`, found pre-existing in the target directory at commit time — see Surprises): authored the remaining 6 skills (watermark-detection, rust-quality, wasm-packaging, webextension-mv3, security-checklist, provenance-registry), switched Layer 3 to perceptual hashing (PDQ/pHash) per the proposal, recorded the `WatermarkDetector` trait plan, adopted the proposal's privacy rules, and logged the naming/UX deviations below.
- [x] (2026-07-10) Reflection pass on the origin proposal: `development_status.md` created (derived snapshot dashboard; this plan stays authoritative), status preamble and inline `> Status:` notes added to `ai-content-verifier-proposal.md`, and two gaps filled — `lens-release` agent (M4/shipping had no owner) and `test-vectors` skill (c2patool corpus generation for M1).
- [x] (2026-07-10) Human cryptography reviewer named: the maintainer (gwamoniak) — see Decision Log.
- [x] (2026-07-10) **M0 complete.** Toolchain installed (rustup, stable via `rust-toolchain.toml`, wasm32 target, wasm-pack 0.15.0 via Homebrew). First compile of the compiler-blind scaffold surfaced exactly two things: an indented doc-comment JSON example that rustdoc parsed as a failing doctest (fixed with a `text` fence) and rustfmt struct-literal layout diffs (applied). After that: clippy clean at `-D warnings`, 9/9 tests green (7 core + 2 wasm), `cargo check --target wasm32-unknown-unknown` clean, and the CLI acceptance transcript recorded in Artifacts and Notes (`tiers` prints the four approved phrases; `verify README.md` → Inconclusive, all four layers not-evaluated, exit 20).
- [x] (2026-07-10) M1 implemented on branch `m1-c2pa-validation`: real C2PA validation in `layers/c2pa.rs` (c2pa 0.89.2, default features off + `rust_native_crypto`), `Pipeline::with_trust_anchors`, CLI `--trust-anchors` flag, self-verifying vector generator (`cargo run -p provenance-core --example gen_vectors`) with a committed 5-vector corpus + `manifest.tsv` + `test_ca.pem` in `crates/provenance-core/tests/vectors/`, 6 integration tests (trusted→Verified with issuer, unanchored-valid→Tampered, unsigned→Inconclusive, content-edit→Tampered, wrong-anchor→never-Verified, hostile-bytes robustness) + corpus test + sniffing tests. 18/18 tests green, clippy `-D warnings` clean, acceptance transcript in Artifacts. Bonus: the c2pa-backed core already type-checks on wasm32 (M2's main risk retired).
- [x] (2026-07-10) **M1 merge gate satisfied and M1 complete**: maintainer (gwamoniak) reviewed the packet and signed off ("approved, merge it"); sign-off recorded in the Decision Log; branch `m1-c2pa-validation` merged to `main`.
- [x] (2026-07-10) **M2 complete.** `verify_bytes` extended with a `trust_anchors_pem` parameter (the extension bundles its trust list and passes it in). Parity suite (`crates/provenance-wasm/tests/parity.rs`): every corpus vector, with and without anchors, produces identical verdict + phrase + per-layer findings through the wrapper and the native pipeline — 19/19 tests green. wasm-pack build is reproducible (bundled wasm-opt disabled — it predates bulk-memory ops; explicit system-binaryen step documented in the wasm-packaging skill). Beyond the plan: `scripts/wasm_smoke.mjs` executes the ACTUAL compiled artifact in Node and all 5 vectors match their recorded verdicts, Verified included. Size measured: 6.41 MB raw / ~2.15 MB gzipped; budget set at ≤ 7 MB raw / ≤ 2.5 MB gzipped.
- [x] (2026-07-10) M3 implemented: module service worker wires context menu → fetch image bytes → `verify_bytes(bytes, contentTypeHint, anchorsPem)` → report in `chrome.storage.session` → action badge (VER/IND/INC/TAM in tier colors, ERR black) → `chrome.action.openPopup()` with badge fallback; popup renders the engine's verbatim phrases (textContent only) plus per-layer findings and an explicit trust-anchor status line; errors render as errors, never as tiers. Trust anchors ship as a data file (`extension/trust/anchors.pem`, deliberately empty placeholder until M4). Smoke harness: `scripts/serve_testpage.mjs` serves the corpus with CORS headers. Automated evidence green: page renders in a browser, and the worker's exact data path (HTTP fetch → content-type hint → engine with anchors) passes all 5 vectors in Node.
- [x] (2026-07-10) **M3 closed**: maintainer ran the browser smoke script and reported it passed ("smoke passed") — the full flow (context menu → badge → popup, five vectors, failure states) works in a real Chrome with the unpacked extension.
- [x] (2026-07-10) **M4 complete — the wedge is shippable.** Production trust list: `extension/trust/anchors.pem` now carries the official C2PA conformance trust list (28 certificates, provenance header with source/date/sha256; refresh via `sh scripts/update_trust_list.sh`), proven to load (the ephemeral test vector rightly fails to chain against it). Wording audit turned into CI: `tests/wording_sync.rs` fails the suite if README, popup legend, or the verdict-language skill drift from the canonical phrases. Release collateral: `CHANGELOG.md` (0.1.0), `docs/STORE_LISTING.md` (listing copy per verdict-language, permission justifications, privacy policy, pre-submission checklist), `scripts/package_extension.sh` (engine build → smoke → refuses to package a test CA → zip). Artifact: `dist/provenance-lens-0.1.0.zip`, 2,299,035 bytes, sha256 `9687a5d2…07980e2` — 20/20 tests green. **Store submission itself is the maintainer's action** (lens-release rule: agents prepare, never publish).
- [x] (2026-07-18) New development machine validated: the repo now lives at `D:\sandbox\projects\provenance-lens` on Windows 11 (previously macOS). `rust-toolchain.toml` auto-synced stable 1.97.1 + wasm32 target on first `cargo clippy`; full loop green (clippy `-D warnings` clean, 20/20 tests) with zero changes — the bare-machine rule held across an OS change. Not yet installed here: wasm-pack (needed only to rebuild the extension engine; `cargo install wasm-pack` when M-level work needs it).
- [x] (2026-07-18) Post-wedge hardening backlog implemented on branch `post-wedge-hardening` (all three M1-review follow-ups): (1) cert-policy pinning tests (`crates/provenance-core/tests/cert_policy.rs`, 5 tests — see Decision Log for the dev-dependency and design); (2) cargo-fuzz target `manifest_parsing` (`fuzz/`, workspace-excluded, nightly-only; seeds in `fuzz/corpus_seed/`); (3) trust-list refresh cadence (`.github/workflows/trust-list-refresh.yml`, monthly, opens a review PR — never lands the diff itself). Suite now 25/25 green, clippy clean. **Merge gate satisfied 2026-07-18**: maintainer (gwamoniak) signed off ("approved, merge it" — recorded in the Decision Log) and the branch was merged to `main`. Still with the maintainer: filing the prepared upstream c2pa report, and the first automated trust-list PR review when the workflow fires.
- [x] (2026-07-18) First fuzz run executed on this machine (60 s, ASAN, ~5.5k execs): found a reachable panic in the c2pa crate's JUMBF parser (see Surprises); panic containment added to `C2paLayer::examine` (see Decision Log); upstream report prepared in Artifacts — **filing it on contentauth/c2pa-rs is the maintainer's action**, as is deciding on a c2pa version bump when a fix ships.
- [x] (2026-07-18) Successor plan authored for the non-gated upgrade wave: `PROVENANCE_LENS_UPGRADES_EXECPLAN.md` (U1 machine-readable CLI, U2 credential summary, U3 proven format coverage, U4 CI baseline, U5 Firefox, U6 npm package, U7 design-gated page-scan badges). M5–M7 remain gated here, untouched.
- [ ] Post-wedge (each gated, see Milestones): M5 watermark layer, M6 registry layer, M7 heuristics layer.

## Surprises & Discoveries

- Observation: The development machine has no Rust toolchain and no GitHub CLI (2026-07-10; `cargo`, `rustc`, `gh` all absent from PATH, `~/.cargo` absent). The M0 scaffold was therefore authored compiler-blind.
  Evidence: `(eval):1: command not found: cargo` for all probed tools.
  Consequence: the first action of any implementer is the toolchain install in Concrete Steps, then treating the first `cargo test` run as a review of the scaffold.
- Observation: The full proposal document `ai-content-verifier-proposal.md` was already sitting in the target directory before scaffolding began (the maintainer had saved the pinned conversation there), and was discovered only when `git add -A` swept it into the M0 commit. It contains detail beyond the summary the scaffold was built from: the 8-skill list, the `WatermarkDetector` trait, PDQ/pHash for Layer 3, the content-script badge UI as flagship UX, the images-never-leave-the-device privacy rule, and the weeks 1–10 build order.
  Evidence: `git commit` output listed `create mode 100644 ai-content-verifier-proposal.md` among the committed files.
  Consequence: same-day reconciliation pass (see Progress); the proposal file stays in the repo as the origin document. Lesson: list the target directory before scaffolding into it.
- Observation: The `c2pa` crate (0.89.2) publicly exports `EphemeralSigner`, which mints a fresh CA + end-entity chain per call — so the vector corpus needs NO committed private keys and NO c2patool install: the generator signs with an ephemeral chain, persists only the public CA root (`test_ca.pem`), and self-verifies every vector through the real pipeline before writing it. Also: with default features off + `rust_native_crypto`, the whole stack (no OpenSSL, no HTTP clients) already type-checks on `wasm32-unknown-unknown`.
  Evidence: `cargo run -p provenance-core --example gen_vectors` → "wrote 5 vectors"; `cargo check -p provenance-wasm --target wasm32-unknown-unknown` → Finished.
  Consequence: the test-vectors skill was rewritten around the generator; M2's main known risk (c2pa on wasm32) is largely retired; c2patool remains only as an optional cross-check.
- Observation: The compiler-blind scaffold survived first contact with the toolchain almost intact — the only compile error was rustdoc treating a 4-space-indented JSON example in a `///` comment as a Rust doctest; the fix is fencing non-Rust doc examples with ```` ```text ````.
  Evidence: `error: expected one of ... found ':' --> crates/provenance-wasm/src/lib.rs:16:12` from `cargo test`; after the fence + `cargo fmt`, `test result: ok. 7 passed` (core) and `ok. 2 passed` (wasm), clippy clean at `-D warnings`.
  Consequence: house rule for this repo — indented code blocks in doc comments are forbidden; always use fenced blocks with an explicit language (`text` for non-Rust).

- Observation: The fuzz target's FIRST one-minute run found a reachable panic in the c2pa crate's JUMBF box parser: `attempt to subtract with overflow` at `c2pa-0.89.3/src/jumbf/boxes.rs:2148` — `bytes_left -= header.size;` is unchecked six lines after the neighboring subtraction was made `checked_sub`, so a crafted salt-box header whose declared size exceeds the remaining bytes underflows. Verified still present, same line, in the published c2pa 0.90.0 source. Two mitigating facts: with overflow checks off (our shipped release builds) the subtraction wraps and the very next `bytes_left != HEADER_SIZE` comparison fails into an error return, so the shipped CLI and extension do NOT crash on this input — only debug/test builds panic; and libFuzzer on Windows failed to write the crash artifact (a Rust panic fast-aborts past the crash handler; `fuzz/artifacts/` stayed empty), so no standalone regression input was captured.
  Evidence: fuzzer log after ~5,500 execs: `thread '<unnamed>' panicked at ...c2pa-0.89.3\src\jumbf\boxes.rs:2148:17: attempt to subtract with overflow`; `grep -n "bytes_left -= header.size" c2pa-0.90.0/src/jumbf/boxes.rs` → line 2148.
  Consequence: the fuzz target proved its worth in its first minute and stays the regression net (it will re-find this until an upstream fix lands, at which point a dependency bump closes it). Panic containment was added to the layer (see Decision Log). An upstream report is prepared in Artifacts and Notes — filing it on contentauth/c2pa-rs is the maintainer's action.
- Observation: The c2pa verifier (0.89.x) hard-requires an `organizationName` attribute in the end-entity certificate's subject: `Verifier::verify_signature` extracts it after the cryptographic check and returns an error when absent, which the store layer logs as `claimSignature.mismatch` — indistinguishable from a genuinely bad signature. A CN-only subject therefore fails validation even though the signature is cryptographically sound.
  Evidence: cert-policy control test failed with `claimSignature.mismatch` on a CN-only chain; `verifier.rs` `iter_organization().ok_or(MissingSigningCertificateChain)`; adding an O= RDN fixed it with no other change.
  Consequence: the in-test X.509 builder always sets CN and O (documented in `tests/cert_policy.rs`); worth remembering if we ever diagnose real-world assets whose credentials fail with `claimSignature.mismatch` — the cause may be a malformed subject, not a broken signature. The control test earned its keep: without it, the negative tests would have passed vacuously on a structurally broken builder.

## Decision Log

- Decision: Project lives at `sandbox/rust/provenance-lens`, its own git repository, not inside an existing repo.
  Rationale: the sandbox is organized by language (`cpp/`, `llm/`); this is the first Rust project. Independent history, independent GitHub remote later.
  Date/Author: 2026-07-10 / scaffold session.
- Decision: `provenance-core` stays dependency-free until M1 adds exactly one dependency, the `c2pa` crate. The CLI is std-only (no clap); the WASM wrapper hand-rolls its flat JSON (no serde).
  Rationale: authored compiler-blind, so every dependency is an unverifiable risk; and the wedge thesis rewards a small, auditable core. Each future dependency is a named milestone decision, not a convenience.
  Date/Author: 2026-07-10 / scaffold session.
- Decision: Combination precedence is Tampered > Verified > Indicated > Inconclusive; in particular tamper evidence anywhere outranks a valid proof elsewhere in the same asset.
  Rationale: conservative reading wins; an asset that is part-valid part-broken is exactly what credential transplant attacks look like.
  Date/Author: 2026-07-10 / from the original proposal.
- Decision: Only Layer 1 (C2PA) may return `LayerFinding::Proof`; heuristics may return at most `Indication`; gated/unimplemented layers return `NotEvaluated { reason }`, never `NoSignal`.
  Rationale: the verdict tiers are honest only if the type system mirrors evidentiary strength; "didn't look" must never masquerade as "looked, found nothing".
  Date/Author: 2026-07-10 / from the original proposal.
- Decision: Blockchain appears only as optional anchoring of the transparency-log registry's checkpoints, default off; no verdict path may require a chain lookup.
  Rationale: from the original proposal — the registry's verifiability comes from the Merkle transparency log itself; anchoring adds external witnesses for operators who want them, nothing more.
  Date/Author: 2026-07-10 / from the original proposal.
- Decision: Signature-validation code (Layer 1 validation path, trust-list handling, future inclusion-proof verification) merges only with human cryptography-reviewer sign-off, recorded as a dated entry in this log. The `lens-security-reviewer` agent prepares the packet but cannot approve.
  Rationale: from the original proposal — the one place agents don't get final say.
  Date/Author: 2026-07-10 / from the original proposal.
- Decision: CLI exit codes are 0 verified / 10 indicated / 20 inconclusive / 30 tampered / 2 usage-or-IO-error.
  Rationale: scripts need to branch on verdicts; spacing by 10 leaves room for sub-codes if ever needed.
  Date/Author: 2026-07-10 / scaffold session.
- Decision: Crate names are `provenance-*` and everything lives in this single repository, deviating from the proposal's working names (`verify-core`, `verify-wasm`, separate `provenance-lens` and `verify-registry` repos). The CLI binary is `lens`, not `verify`.
  Rationale: the proposal itself labels those "working names"; one repo keeps the wedge reviewable as a unit and matches the sandbox's one-repo-per-project layout; `verify` is too generic a binary name to put on a PATH. Splitting out `verify-registry` (and an npm `verify-wasm` package) remains open for post-wedge when they exist.
  Date/Author: 2026-07-10 / reconciliation pass.
- Decision: The wedge extension (M3) uses a context-menu verify flow under `contextMenus` + `activeTab`; the proposal's flagship UX — a content script that scans `<img>`/`<video>` and overlays verdict badges — is post-wedge, gated on the host-permissions decision it requires.
  Rationale: `activeTab` needs a user gesture and no host permissions, which keeps the store-review and privacy surface minimal for the first ship; automatic page-wide scanning needs `<all_urls>`-class permissions and its own performance/consent design (see the webextension-mv3 skill).
  Date/Author: 2026-07-10 / reconciliation pass.
- Decision: Layer 3 lookup uses perceptual hashing (PDQ preferred, pHash acceptable), computed locally; only hashes ever leave the device, with user consent — adopted from the proposal along with the general privacy rule that image bytes are never uploaded by any layer.
  Rationale: cryptographic hashes break on the platform re-encodes that are exactly the case Layer 3 exists for; the privacy rule is the proposal's trust model and part of the product's honesty brand.
  Date/Author: 2026-07-10 / from the proposal, reconciliation pass.
- Decision: Added a ninth agent, `lens-release` (M4 shipping and every later release: versioning, CHANGELOG, reproducible builds, store checklists; prepares but never publishes without the maintainer's explicit go), and a ninth skill, `test-vectors` (c2patool corpus generation, the trust-anchor policy for test certificates, the vector manifest format).
  Rationale: gap analysis against the proposal — store submission sat inside its Extension Agent and community inside Docs, leaving shipping mechanics unowned; and the QA corpus was named but no skill carried the know-how to generate it. Both gaps sit directly on the wedge's critical path (M1 and M4).
  Date/Author: 2026-07-10 / reflection pass.
- Decision: `development_status.md` exists as a derived, human-readable snapshot (deliverable mapping, blockers, risks), updated at milestone boundaries. This plan's living sections remain the single source of truth; on disagreement the plan wins.
  Rationale: the maintainer wants a skimmable current-state view without reading the full plan; making its derived status explicit prevents a second-source-of-truth drift.
  Date/Author: 2026-07-10 / maintainer request (goal directive), reflection pass.
- Decision: The human cryptography reviewer for signature-validation sign-offs is the maintainer, gwamoniak. M1 (and any later trust-decision code) merges only with their dated sign-off entry in this log; `lens-security-reviewer` prepares each packet.
  Rationale: maintainer's explicit choice when asked (2026-07-10); matches the proposal's "maintainer/architect reviews security-critical merges", with the option to recruit a dedicated community reviewer later if load grows.
  Date/Author: 2026-07-10 / maintainer (gwamoniak).
- Decision: M1 dependency pinned: `c2pa = { version = "0.89.2", default-features = false, features = ["rust_native_crypto"] }` (+ dev-only `base64` for PEM encoding). Default features were rejected because they pull OpenSSL and four HTTP client stacks; with no HTTP feature compiled in, remote-manifest fetching is impossible by construction, which is the sans-IO rule enforced at the linker rather than by a runtime flag.
  Rationale: bare-machine builds (no Homebrew OpenSSL), wasm32 compatibility, and a smaller audit surface.
  Date/Author: 2026-07-10 / M1 implementation.
- Decision: Layer 1 finding mapping — `Trusted` → `Proof`; `Valid` (cryptographically sound but no chain to a configured anchor, e.g. self-signed) → `TamperEvidence` per the standing conservative rule; `Invalid` → `TamperEvidence` with the validator's status codes; no manifest → `NoSignal`; parse errors → `NotEvaluated`, never `TamperEvidence`. The last point is deliberate: a corrupt plain image has no provenance to have tampered with, and an attacker who could truncate a manifest into a parse error could equally strip it entirely — both roads honestly end at Inconclusive, so mapping parse errors to Tampered would only create false alarms on innocently mangled files.
  Rationale: TamperEvidence must come from validation results over an actually-located manifest, not from failures to parse.
  Date/Author: 2026-07-10 / M1 implementation (mapping mirrored in the c2pa-spec skill and the layer's module docs).
- Decision: Test vectors are generated by a self-verifying generator (`examples/gen_vectors.rs`) using `EphemeralSigner` — fresh keys per run, never persisted; only the public CA root is committed alongside the corpus. The generator asserts every vector's expected verdict through the real pipeline before writing `manifest.tsv`, so a lying corpus cannot be committed. This replaces the original c2patool-based plan.
  Rationale: no committed private keys (nothing for a secret scanner to flag, nothing to rotate), no external tool install, and machine-checked corpus truth.
  Date/Author: 2026-07-10 / M1 implementation.
- Decision: M2 — the WASM boundary is `verify_bytes(bytes, media_type?, trust_anchors_pem?)`; the caller (extension) supplies the trust-anchor PEM bundle rather than the artifact embedding one.
  Rationale: keeps the engine policy-free and the trust list updatable by shipping a data file with the extension, not by rebuilding the WASM; also what the parity/acceptance tests need to exercise Verified through the boundary.
  Date/Author: 2026-07-10 / M2 implementation.
- Decision: M2 — wasm-pack's bundled wasm-opt is disabled (`wasm-opt = false` in provenance-wasm's package metadata); optimization is an explicit post-build step using system binaryen (`brew install binaryen`) with bulk-memory/sign-ext/reference-types feature flags (exact command in the wasm-packaging skill), followed by `node scripts/wasm_smoke.mjs` as the true-artifact acceptance check. Size budget set from evidence: ≤ 7 MB raw / ≤ 2.5 MB gzipped (measured 6.41 MB / 2.15 MB).
  Rationale: the bundled binaryen fails on bulk-memory ops that Rust emits by default; an explicit, flag-pinned step is reproducible and the smoke script proves the optimized artifact still verifies the corpus.
  Date/Author: 2026-07-10 / M2 implementation.
- Decision: M3 — the verdict renders in the popup (opened via `chrome.action.openPopup()` after verification, with the action badge as fallback), NOT as an injected page panel; results persist in `chrome.storage.session`. Permissions grew by exactly one: `storage`. `activeTab` is kept because it grants temporary host access to the page's origin on the context-menu gesture, letting the worker fetch same-origin images CORS-free; cross-origin images without `Access-Control-Allow-Origin` fail with an honest error (the alternative — standing host permissions — is rejected; revisit only as an explicit user-facing "grant site access" choice post-wedge).
  Rationale: a popup keeps the extension out of the page's DOM entirely (no injected-panel XSS surface, no page-style conflicts) and keeps the permission story minimal for store review.
  Date/Author: 2026-07-10 / M3 implementation.
- Decision: M3 — trust anchors ship as a data file (`extension/trust/anchors.pem`), read at verification time and passed to `verify_bytes`; the committed file is a deliberately EMPTY commented placeholder until M4 settles production trust-list distribution. An empty file behaves as "no anchors": nothing verifies as trusted, which is the honest default.
  Rationale: the trust list must be updatable without rebuilding the engine, and shipping the ephemeral test CA in a real package would be a trust-model bug.
  Date/Author: 2026-07-10 / M3 implementation.
- Decision: M4 — the extension ships the official C2PA conformance trust list (`https://github.com/c2pa-org/conformance-public`, `trust-list/C2PA-TRUST-LIST.pem`) as its default anchors, committed with a provenance header (source URL, fetch date, certificate count, sha256) and refreshed only via `scripts/update_trust_list.sh` so every change is auditable in git history. The packaging script refuses to ship a file with no certificates and warns if the content doesn't look like the official list.
  Rationale: an extension that can never say Verified is not a wedge; the conformance list is the ecosystem's canonical anchor set, and shipping it as data keeps updates a one-line diff rather than a rebuild. This is trust-model data — the maintainer reviews the diff whenever it changes.
  Date/Author: 2026-07-10 / M4 implementation.
- Decision: **Cryptography sign-off, M1** — the maintainer (gwamoniak) reviewed the M1 packet (Layer 1 validation code, finding mapping, vector corpus, disclosed gaps: no cargo-fuzz target yet, cert-expiry/algorithm policy inherited from the c2pa crate, depth limits delegated, cargo-audit not run, production trust-list distribution open) and approved the merge of `m1-c2pa-validation` to `main`.
  Rationale: per the standing human-sign-off rule; the disclosed gaps are accepted as recorded follow-ups, not blockers.
  Date/Author: 2026-07-10 / maintainer (gwamoniak): "approved, merge it".

- Decision: Cert-policy pinning tests (`tests/cert_policy.rs`) build their own X.509 chains with an in-test builder (adapted from the c2pa crate's private ephemeral-cert module) rather than relying on `EphemeralSigner`, which offers no control over validity dates or signature algorithms. Dev-dependencies added to provenance-core for this: `rasn`, `rasn-pkix`, `ed25519-dalek`, `pkcs8`, `sha1`, `chrono` — every one already in the dependency tree via c2pa itself at the same versions, so no new supply-chain surface. Test keys are deterministic seeds, never trusted, never persisted. The expiry pin is threefold: end-to-end (a cert minted to expire seconds after signing is signed while valid, then validated after expiry → Tampered, never Verified — costs ~6 s of suite time, accepted), the sign-time gate (signing with an expired cert must fail), and the direct profile check; the algorithm pin asserts the profile check rejects `sha1WithRSAEncryption`. A control test proves the builder's chains reach Verified so the negative tests cannot pass vacuously.
  Rationale: the M1 review accepted cert policy as inherited from the c2pa crate; these pins turn a silent policy weakening in a future c2pa upgrade into a test failure. Constructing a *weak-algorithm signed asset* end-to-end is impossible through the crate's public API (the SigningAlg enum offers only strong algorithms and the sign-time gate rejects nonconforming certs) — the profile-check pin is the honest reachable maximum, recorded here rather than faked.
  Date/Author: 2026-07-18 / post-wedge-hardening branch (merge pends maintainer sign-off).
- Decision: The fuzz crate lives in `fuzz/`, excluded from the workspace, so the stable `cargo test --workspace` loop stays bare-machine clean; fuzzing needs nightly + cargo-fuzz (`cargo +nightly fuzz run manifest_parsing fuzz/corpus fuzz/corpus_seed`). The target runs the full pipeline with NO trust anchors over attacker-controlled bytes; invariants: never panic, and never mint a `Proof` (Trusted is unreachable without anchors, so any Proof is a bug by construction). The committed vector corpus seeds the fuzzer. Proof-minting *with* anchors stays pinned by the unit suite instead — a fuzzer seeded with signed vectors can legitimately reconstruct validly-signed bytes, so "no Proof" would be a false invariant there.
  Rationale: the backlog item's goal is parser robustness on hostile bytes; the no-anchor configuration gives the strongest assertable invariant without false alarms.
  Date/Author: 2026-07-18 / post-wedge-hardening branch.
- Decision: On Windows the fuzz target runs with the default ASAN sanitizer plus MSVC's ASAN runtime DLL directory on PATH (`VC\Tools\MSVC\<ver>\bin\Hostx64\x64`, for `clang_rt.asan_dynamic-x86_64.dll`); `--sanitizer none` is rejected as a workaround because libFuzzer's coverage instrumentation fails to link on MSVC without a sanitizer (unresolved `__stop___sancov_pcs` — the `__start_/__stop_` section symbols are an ELF feature the MSVC linker only gets via the sanitizer runtime). Both failure modes are documented in the fuzz target's module docs; on Linux/macOS (e.g. CI runners) the defaults just work.
  Rationale: recorded so the next person doesn't rediscover either failure mode; ASAN also genuinely adds value over `none` (the c2pa tree contains unsafe code).
  Date/Author: 2026-07-18 / post-wedge-hardening branch, first fuzz run on the Windows machine.
- Decision: Panic containment in `C2paLayer::examine` — the c2pa parsing and state-mapping now run inside `std::panic::catch_unwind`; a panic degrades to `NotEvaluated { reason: "manifest parsing panicked..." }`, never `TamperEvidence` and never a crash of the caller. This is the same honesty rule already applied to parse errors (a panic is a parse failure with worse manners), motivated by the fuzz finding above and by the fact that other panic sites in the c2pa tree would fire in release builds too (unwraps, indexing). On wasm32 the default panic strategy is abort, so the guard compiles but cannot catch there — acceptable because the extension already renders engine failures as errors, never as verdicts. This change touches the signature-validation file and rides the same branch sign-off.
  Rationale: the layer's contract says parsing must never panic on hostile bytes; containment enforces the contract against upstream bugs we don't control, at the cost of ten lines.
  Date/Author: 2026-07-18 / post-wedge-hardening branch.
- Decision: Trust-list refresh cadence is a monthly scheduled GitHub Actions workflow (`.github/workflows/trust-list-refresh.yml`, also manually dispatchable) that runs `scripts/update_trust_list.sh` and — only when certificates actually changed (the `Fetched:` provenance line is ignored when diffing) — opens a pull request. It never pushes to `main`: anchors.pem is trust-model data and the maintainer reviews every diff, same as a manual refresh. No third-party actions are used (checkout + `gh` with the built-in token only).
  Rationale: closes the tracked staleness risk (a stale bundle turns fresh legitimate credentials into "Tampered / unverifiable") while preserving the maintainer-reviews-every-trust-change rule; avoiding third-party actions keeps the supply chain of a trust-critical file minimal.
  Date/Author: 2026-07-18 / post-wedge-hardening branch.

- Decision: **Cryptography sign-off, post-wedge hardening** — the maintainer (gwamoniak) reviewed the `post-wedge-hardening` branch (cert-policy pinning tests with their dev-dependency additions, the fuzz target and its first-run finding, the panic guard in `layers/c2pa.rs`, the trust-list refresh workflow) and approved the merge to `main`.
  Rationale: per the standing human-sign-off rule for signature-validation surface; second exercise of the process after M1.
  Date/Author: 2026-07-18 / maintainer (gwamoniak): "approved, merge it".

## Outcomes & Retrospective

- (2026-07-18) **Post-wedge hardening pass complete (pending sign-off).** Outcome: the three follow-ups the M1 review deferred are real — cert policy is pinned against silent weakening by a c2pa upgrade (expiry three ways, algorithm allowlist, with a vacuousness-proofing control), a fuzz target exists and found a genuine upstream parser panic in its first sixty seconds, and the trust list refreshes itself monthly through a maintainer-reviewed PR instead of relying on memory. The finding forced a product improvement the backlog hadn't asked for: a panic guard so upstream parser bugs degrade to an honest NotEvaluated instead of a crash. Two lessons: the control test in a pinning suite is not optional (it caught the verifier's undocumented organizationName requirement, which would otherwise have let every negative test pass vacuously), and a fuzz target that costs one file and one dependency paid for itself before its first minute was up. What remains is exactly one thing agents cannot do: the maintainer's review — of the branch, of the prepared upstream report, and eventually of the first automated trust-list PR.

- (2026-07-10) **M4 closed — the wedge shipped in one day, not six weeks.** Outcome: an installable extension zip that verifies real Content Credentials against the official C2PA trust list, a CLI that does the same, a wording audit that runs as CI, and a release process (checklist, packaging script, changelog) that the lens-release agent can repeat. Against the original purpose: everything the proposal called "Weeks 1–6" exists and is proven — what compressed the timeline was simulator-grade test infrastructure (self-verifying corpus, true-artifact smoke) making every step checkable the moment it was written, plus an ecosystem (c2pa crate, EphemeralSigner, conformance trust list) that was more ready than the proposal assumed. What remains for the maintainer: run the store submission (docs/STORE_LISTING.md has everything), and decide when to open the gated milestones. The honest-verdict rules survived contact with implementation unchanged — no exception was ever needed.
- (2026-07-10) **M2 closed — the engine runs where the extension needs it.** Outcome: a reproducible three-step artifact build (wasm-pack → system wasm-opt → corpus smoke through the real .wasm in Node), parity pinned by tests at two levels (wrapper logic natively, compiled artifact in a JS engine), and an evidence-based size budget (6.41 MB raw / 2.15 MB gzipped, budget 7 / 2.5). The milestone's one predicted risk — c2pa on wasm32 — never materialized, because M1's no-default-features decision had already removed everything wasm-hostile. Surprise: the only real friction was tooling (wasm-pack's stale bundled binaryen), not the crypto stack. M3 (extension end-to-end) now has everything it needs: a working ES module, a verified artifact, and a trust-anchor parameter across the boundary.
- (2026-07-10) **M1 closed — the wedge's spine is real.** Outcome: `lens verify` now delivers on the product's core promise for Layer 1 — cryptographic proof chains verify to Verified with the issuer named, stripped credentials read Inconclusive (never "authentic"), broken ones read Tampered with the validator's own status codes. Signed off by the maintainer per the human-review rule, first exercise of that process. What made it fast: the sans-IO/no-default-features decision (no OpenSSL, no HTTP) built cleanly everywhere including wasm32, and the c2pa crate's `EphemeralSigner` collapsed the whole test-certificate problem into "no committed keys at all". Remaining from the review: cargo-fuzz target, cert-policy pinning tests, production trust-list distribution (M4).
- (2026-07-10) **M0 closed, same day it was opened.** Outcome: a green, conventions-complete foundation — workspace, verdict model with pinned wording, pipeline spine, CLI with verdict exit codes, WASM wrapper checking on wasm32, extension skeleton, nine agents, nine skills, this plan, and a status dashboard; 9/9 tests, clippy `-D warnings` clean. Against the purpose: nothing user-visible verifies anything yet (by design — that is M1), but the honesty machinery the product stands on is implemented and test-pinned. Lesson: writing the scaffold compiler-blind cost almost nothing (one doctest fence, one fmt pass) because it stayed std-only and conservative — the zero-dependency decision earned its keep on day one.

## Context and Orientation

You are in a fresh Cargo workspace; assume no prior knowledge. The domain in one paragraph: **C2PA** ("Content Credentials") is an open standard that embeds a cryptographically signed provenance manifest inside a media file — who made it, with what tool, whether generative AI was involved — bound to the exact bytes by a content hash, signed with an X.509 certificate validated against a trust list. A deeper primer lives in `.claude/skills/c2pa-spec/SKILL.md`. Separately, some AI generators embed **invisible watermarks** (statistical patterns in pixels, e.g. Google's SynthID), and a **transparency log** is an append-only, Merkle-tree-backed public record (the Certificate Transparency model) in which generators could register hashes of what they produce.

The layout:

- `crates/provenance-core/` — the library. `src/verdict.rs` holds the four-tier `Verdict` enum and the approved phrases (normative wording rules in `.claude/skills/verdict-language/SKILL.md`). `src/pipeline.rs` holds `Asset` (bytes + optional MIME hint), the `Layer` trait, `LayerFinding` (NotEvaluated / NoSignal / Proof / Indication / TamperEvidence), the `Pipeline` runner, and `combine()` — the precedence rule. `src/layers/` holds the four layers in pipeline order: `c2pa.rs`, `watermark.rs`, `registry.rs`, `heuristics.rs`; all currently honest stubs returning `NotEvaluated`. The core is sans-IO: it never opens files or sockets; callers pass bytes in, and anything network-shaped (the future registry lookup) receives an injected transport trait.
- `crates/provenance-cli/` — the `lens` binary (std-only): `lens verify <file>` prints a report and exits with the verdict's code; `lens tiers` prints the four tiers.
- `crates/provenance-wasm/` — a thin wasm-bindgen wrapper exporting `verify_bytes(bytes, media_type) -> String` (flat JSON: verdict id, approved phrase, per-layer findings).
- `extension/` — a Manifest V3 browser extension skeleton (plain JS, no framework): a context-menu entry on images, a popup listing the tiers, and honest "engine not bundled" messaging until M3 wires in the wasm-pack output at `extension/pkg/` (gitignored build product).
- `.claude/agents/` — nine scoped agents (`lens-rust-core`, `lens-wasm`, `lens-extension`, `lens-security-reviewer`, `lens-registry`, `lens-qa`, `lens-research`, `lens-docs`, `lens-release`); `.claude/skills/` — nine skill packs (`c2pa-spec`, `watermark-detection`, `rust-quality`, `wasm-packaging`, `webextension-mv3`, `verdict-language`, `security-checklist`, `provenance-registry`, `test-vectors`).
- `development_status.md` — derived snapshot dashboard for humans; this plan stays authoritative.
- `ai-content-verifier-proposal.md` — the origin document (the maintainer's saved proposal from the pinned conversation); this plan operationalizes it, and where they differ the Decision Log entry explains why.

## Plan of Work

Work proceeds milestone by milestone, in order; the story of each is under Milestones below. In brief: M0 finishes the toolchain bring-up and validates the compiler-blind scaffold. M1 replaces the `C2paLayer` stub in `crates/provenance-core/src/layers/c2pa.rs` with real validation via the `c2pa` crate, maps its validation states onto `Proof`/`TamperEvidence`/`NoSignal` exactly as specified in the c2pa-spec skill, and proves it against known-answer vectors in `tests/vectors/`. M2 makes `wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg` produce a working artifact and adds a parity test asserting identical verdicts native vs WASM over every vector. M3 wires the extension: background worker fetches the right-clicked image's bytes, calls the engine, and renders the report with verbatim approved phrases. M4 packages and audits. M5–M7 are gated expansions of the remaining layers.

## Concrete Steps

Toolchain bring-up (M0, once per machine; the project pins stable + the wasm target via `rust-toolchain.toml`):

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    cargo install wasm-pack

Build and test (repo root; this is the loop every implementer runs before declaring anything done):

    cargo fmt --all
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

Expected at the end of M0 (counts will grow):

    running 7 tests ... test result: ok. 7 passed  (core: 2 verdict + 5 pipeline; wasm: 2)

Try the CLI on any file:

    cargo run -p provenance-cli -- verify some-photo.jpg
    # some-photo.jpg
    #   verdict: Inconclusive: no provenance data was found. This does NOT mean the asset is authentic.
    #   [c2pa] not evaluated — C2PA validation via the c2pa crate lands in Milestone 1
    #   ... (echo $? → 20)

Build the WASM engine and load the extension (M2/M3):

    wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg
    # then chrome://extensions → Developer mode → Load unpacked → extension/

## Milestones

**M0 — Scaffold and toolchain (no gate).** Scope: everything described in Context and Orientation exists and compiles; the test suite is green on this machine. The scaffold was written 2026-07-10 without a compiler present, so the milestone closes only when the loop above runs clean — treat the first compile as a code review of the scaffold. Acceptance: `cargo test --workspace` green; `lens tiers` prints the four approved phrases; `lens verify <any file>` reports Inconclusive with all four layers listed as not-evaluated and exit code 20.

**M1 — Layer 1 for real: C2PA validation (no gate; ~2 weeks of the 6-week wedge).** Scope: add the `c2pa` crate (pin the version; record it here) to `provenance-core`, implement manifest reading and validation in `C2paLayer::examine` per the mapping in `.claude/skills/c2pa-spec/SKILL.md` — full pass → `Proof { issuer }`; absent manifest → `NoSignal`; present-but-invalid (bad signature, untrusted chain, hash mismatch, truncated store) → `TamperEvidence { detail }`. Parsing must never panic on hostile bytes. Collect vectors into `tests/vectors/` (C2PA public test files plus self-generated ones via c2patool: a signed image, the same image re-saved to strip credentials, a byte-corrupted manifest) with a manifest file listing each vector's origin and expected verdict; tests assert every one. Acceptance: `lens verify` on a validly signed vector prints Verified and exits 0; on the corrupted vector prints Tampered and exits 30; on a plain camera JPEG prints Inconclusive and exits 20. Merge gate: `lens-security-reviewer` review done AND a human cryptography reviewer's sign-off recorded in the Decision Log.

**M2 — WASM parity (no gate; ~1 week).** Scope: make the workspace build for `wasm32-unknown-unknown` (the `c2pa` crate has WASM support; whatever feature flags this needs, record them in the Decision Log — this is the milestone's main known risk), produce the artifact with wasm-pack, and add a parity test: every vector in `tests/vectors/` produces the identical verdict and findings through `verify_bytes` as through the native pipeline. Measure the gzipped artifact size and set the budget here once known (placeholder target: ≤ 4 MB gzipped; revise with evidence). Acceptance: wasm-pack build succeeds; parity suite green; size recorded in Artifacts and Notes.

**M3 — Extension end-to-end (no gate; ~2 weeks).** Scope: background worker fetches the bytes of the right-clicked image (the only network request the extension ever makes on the user's behalf — image bytes never leave the device), calls the engine, renders the report in the popup or an injected panel — approved phrases verbatim, badge colors per the proposal (green Verified / yellow Indicated / gray Inconclusive / red Tampered), and no visual language of safety anywhere near Inconclusive. The proposal's automatic page-scanning badge UI is post-wedge (Decision Log). Honest failure states for engine-missing, fetch-failed, unsupported-format. Acceptance: on a page with a Content-Credentials image, the flow shows Verified; on a stripped copy, Inconclusive; on a corrupted-manifest copy, Tampered.

The manual smoke script (run from the repo root; ~5 minutes):

    1. Build the engine (three steps in .claude/skills/wasm-packaging/SKILL.md):
       wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg
       wasm-opt extension/pkg/provenance_wasm_bg.wasm -Os --enable-bulk-memory --enable-bulk-memory-opt \
           --enable-sign-ext --enable-mutable-globals --enable-nontrapping-float-to-int \
           --enable-reference-types -o extension/pkg/provenance_wasm_bg.wasm
       node scripts/wasm_smoke.mjs                      # must print "all vectors match"
    2. cp crates/provenance-core/tests/vectors/test_ca.pem extension/trust/anchors.pem   # smoke only
    3. node scripts/serve_testpage.mjs                  # http://localhost:8917
    4. chrome://extensions → Developer mode → Load unpacked → extension/
    5. Open http://localhost:8917; right-click each image → "Verify provenance with Provenance Lens".
       Expect: valid_signed → Verified (green badge VER), stripped → Inconclusive (gray INC),
       manifest_corrupted → Tampered (red TAM), plain → Inconclusive, content_tampered → Tampered.
       The popup shows the verbatim phrase, per-layer findings, and "Trust anchors: loaded".
    6. Failure states: stop the server and verify again → plain error, no tier, badge ERR.
       Delete extension/pkg/, reload the extension, verify → "engine is not bundled" error.
    7. git checkout extension/trust/anchors.pem         # restore the empty placeholder

**M4 — Ship the wedge (no gate; ~1 week).** Scope: packaging for the Chrome Web Store (and the store listing text, which obeys the verdict-language skill — understatement is the brand), a final audit of every user-facing string in all four wording locations, README claims verified against actual behavior, the human sign-off recorded. Acceptance: an installable zip; a dated Outcomes & Retrospective entry; the wedge is shippable.

**M5 — Watermark layer (GATED: a runnable, licensed vendor detector — e.g. a published SynthID detector — on disk or under an API license).** Statistical detection; ceiling `Indication`. Introduces the `WatermarkDetector` trait (vendor name + probe; the layer holds a pluggable detector list, per the watermark-detection skill) so vendors plug in without core changes; every detector's false-positive rate is measured on the clean corpus before it may contribute to verdicts. Do not start, and do not fake with a lookalike classifier, until the gate is satisfied; `lens-research` tracks when it opens.

**M6 — Registry layer (GATED: a deployed transparency-log endpoint, or an explicit plan revision scoping standing one up).** Design work (log schema, inclusion proofs, lookup privacy) may proceed in the plan; implementation waits for the gate. Anchoring stays optional per the Decision Log.

**M7 — Heuristics layer (OPTIONAL; may never ship).** Only if a heuristic with a published, reproducible false-positive rate exists; ceiling `Indication`; must be trivially removable.

## Validation and Acceptance

The wedge is done when a novice can: clone the repo, run the toolchain bring-up and the build loop and see green; run `lens tiers` and read the four approved phrases; run `lens verify` against the three M1 acceptance vectors and observe Verified/exit 0, Tampered/exit 30, Inconclusive/exit 20 respectively; build the WASM engine, load the extension unpacked, and reproduce the same three verdicts through the right-click flow. Tests pin the combination precedence and the approved phrases, so wording drift fails the suite (`verdict::tests::inconclusive_wording_never_implies_authenticity`).

## Idempotence and Recovery

Every step is re-runnable: cargo commands are idempotent, wasm-pack overwrites `extension/pkg/` (gitignored), rustup re-runs harmlessly. There is no database, no migration, no destructive step anywhere in the wedge. If the toolchain half-installs, `rustup toolchain list` then `rustup toolchain install stable` recovers. If a milestone stalls mid-way, update Progress with a done/remaining split and commit — the plan plus the tree must always be enough to resume.

## Artifacts and Notes

M0 scaffold transcript (2026-07-10): toolchain probe on this machine —

    (eval):1: command not found: cargo
    (eval):1: command not found: gh

M0 acceptance transcript (2026-07-10, after toolchain install):

    $ cargo test --workspace          # 7 passed (core) + 2 passed (wasm), 0 failed
    $ cargo run -q -p provenance-cli -- tiers
    verified      Verified: this asset carries a valid, cryptographically signed provenance chain.
    indicated     Indicated: signals suggest AI involvement, but no cryptographic proof chain is present.
    inconclusive  Inconclusive: no provenance data was found. This does NOT mean the asset is authentic.
    tampered      Tampered: provenance data is present but fails validation. Treat this asset with suspicion.
    $ cargo run -q -p provenance-cli -- verify README.md
    README.md
      verdict: Inconclusive: no provenance data was found. This does NOT mean the asset is authentic.
      [c2pa] not evaluated — C2PA validation via the c2pa crate lands in Milestone 1
      [watermark] not evaluated — gated: no vendor watermark detector integrated yet
      [registry] not evaluated — gated: no transparency-log registry deployed yet
      [heuristics] not evaluated — optional layer, not implemented
    $ echo $?
    20

M1 acceptance transcript (2026-07-10, branch `m1-c2pa-validation`; `V=crates/provenance-core/tests/vectors`):

    $ lens verify --trust-anchors $V/test_ca.pem $V/valid_signed.jpg
      verdict: Verified: this asset carries a valid, cryptographically signed provenance chain.
      [c2pa] valid provenance chain, issuer: Self-signed ephemeral certificate (Content Authenticity SDK) -- LOCAL USE ONLY
      → exit 0
    $ lens verify --trust-anchors $V/test_ca.pem $V/manifest_corrupted.jpg
      verdict: Tampered: provenance data is present but fails validation. Treat this asset with suspicion.
      [c2pa] tamper evidence — manifest validation failed: assertion.hashedURI.mismatch
      → exit 30
    $ lens verify $V/plain.jpg
      verdict: Inconclusive: no provenance data was found. This does NOT mean the asset is authentic.
      [c2pa] ran, no signal
      → exit 20

Prepared upstream report (2026-07-18, for the maintainer to file on `contentauth/c2pa-rs` — title "Unchecked subtraction in JUMBF box parser panics on malformed salt box (debug builds)"):

    In src/jumbf/boxes.rs, CAIUUIDAssertionBox/salt-box parsing does
    `bytes_left -= header.size;` (line 2148 in c2pa 0.89.3 and 0.90.0)
    without a check, six lines after the sibling subtraction was hardened
    to `checked_sub` returning InvalidBoxHeader. A crafted box header whose
    declared size exceeds the remaining bytes underflows: builds with
    overflow checks (debug, cargo-fuzz) panic with "attempt to subtract
    with overflow"; release builds wrap and happen to fall into the
    `bytes_left != HEADER_SIZE` error return. Found by libFuzzer within
    ~5,500 executions of a JPEG-seeded corpus driven through
    Reader::from_context(...).with_stream("image/jpeg", ...). Suggested
    fix: the same checked_sub + InvalidBoxHeader treatment as the
    surrounding code.

0.2.0 release artifacts (2026-07-18, packaged on the Windows machine after the upgrades plan closed; suite 30/30 and the eight-vector corpus green through the optimized artifact in both cases):

    dist/provenance-lens-0.2.0.zip              2,301,127 B  sha256 e666b30c746e73cb04fc3a58f9807a842f5c9a5b7a37f17a7810340ea0817b12
    dist/provenance-lens-verify-wasm-0.2.0.tgz  2,441,384 B  sha256 867447b97945f7801688e9ba93cd275f54ab9ab4841b6753cb55fa3c7c7e8889
    optimized engine: 6,456,124 B raw (budget ≤ 7 MB) — wasm-opt via the npm binaryen
    wrapper (v112; the packaging scripts probe for the newer --enable-bulk-memory-opt
    flag and verify wasm-opt actually produced output — see the upgrades plan's Surprises).
    Bundled anchors: official C2PA conformance list refreshed pre-submission
    (2026-07-18) — byte-identical to the 2026-07-10 fetch, 28 certificates,
    list sha256 b1f399a7…7cdb46c1 (upstream unchanged; only the Fetched:
    provenance line moved, and the zip was repackaged to match).
    Store submission and npm publish remain the maintainer's actions.

M4 release artifact (2026-07-10): `dist/provenance-lens-0.1.0.zip`, 2,299,035 bytes, sha256 `9687a5d25553125ee5a94b73d080bc5dccaaaa3fe177c5ccf33d21bbe07980e2`; bundled anchors: official C2PA conformance list, 28 certificates, sha256 `b1f399a7235f188a22f3db97992f1cc1417517664600335f9d105a6a7cdb46c1`.

M3 automated evidence (2026-07-10): the smoke page (scripts/serve_testpage.mjs, CORS headers verified with curl) renders all five vectors in a browser; the worker's exact data path simulated in Node (HTTP fetch → content-type hint → verify_bytes with the test CA):

    PASS valid_signed.jpg → verified          PASS stripped.jpg → inconclusive
    PASS manifest_corrupted.jpg → tampered    PASS plain.jpg → inconclusive
    PASS content_tampered.jpg → tampered

M2 artifact + true-WASM smoke (2026-07-10): raw 6,412,435 bytes, gzipped 2,256,194 bytes (~2.15 MB) after system wasm-opt; corpus through the compiled artifact in Node:

    $ node scripts/wasm_smoke.mjs
    PASS plain.jpg: inconclusive          PASS valid_signed.jpg: verified
    PASS stripped.jpg: inconclusive       PASS manifest_corrupted.jpg: tampered
    PASS content_tampered.jpg: tampered
    wasm smoke: all vectors match

(WASM artifact sizes, and extension screenshots get appended here as milestones land.)

## Interfaces and Dependencies

The stable spine of the system; changes here require a Decision Log entry.

In `crates/provenance-core/src/pipeline.rs` (exists as of M0):

    pub struct Asset<'a> { pub bytes: &'a [u8], pub media_type: Option<&'a str> }

    pub enum LayerFinding {
        NotEvaluated { reason: String },
        NoSignal,
        Proof { issuer: String },          // Layer 1 only
        Indication { source: String },
        TamperEvidence { detail: String },
    }

    pub trait Layer {
        fn name(&self) -> &'static str;
        fn examine(&self, asset: &Asset) -> LayerFinding;
    }

    pub fn combine(findings: &[(String, LayerFinding)]) -> Verdict  // Tampered > Verified > Indicated > Inconclusive

In `crates/provenance-wasm/src/lib.rs` (exists as of M0; JSON shape is the extension's contract):

    #[wasm_bindgen]
    pub fn verify_bytes(bytes: &[u8], media_type: Option<String>) -> String

Dependencies by milestone: M0 — wasm-bindgen 0.2 only. M1 — adds the `c2pa` crate (CAI Rust SDK) to provenance-core, plus dev-dependency vectors under `tests/vectors/`. M2 — possibly `getrandom`/feature flags for wasm32 (record what was actually needed). M5+ — named at gate-opening time. Nothing else without a Decision Log entry.

---

Revision note (2026-07-10): initial authoring, from the project proposal (4-layer pipeline, honest tiers, 8 agents + skills, Layer-1-first wedge) at M0-scaffold time. Reason: project bootstrap.

Revision note (2026-07-18): post-wedge hardening pass recorded (Progress, Surprises, Decision Log) — new development machine validated, the three M1-review follow-ups implemented on branch `post-wedge-hardening` pending maintainer sign-off; the `PLANS.md` sibling path corrected for the new machine's layout. Successor planning for non-gated upgrades now lives in `PROVENANCE_LENS_UPGRADES_EXECPLAN.md`; the gated milestones M5–M7 remain owned by this plan. Reason: keeping the living sections truthful at a stopping point, and separating the shipped wedge's record from the next wave of work.
