# Provenance Lens upgrades: deepen the shipped wedge — machine-readable verdicts, richer credential reporting, proven format coverage, CI, and wider distribution

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document is maintained in accordance with `PLANS.md`, checked into the sibling repository `../solid-broccoli/PLANS.md` (path relative to this repo's root on the current development machine; the sandbox keeps the ExecPlan rules in one place). If that file is not in your context, read it in full before revising this plan. The wedge that this plan builds on was delivered under `PROVENANCE_LENS_EXECPLAN.md` in this repository — that plan remains the authoritative record of the wedge (M0–M4), of the honesty rules, and of the gated milestones M5–M7 (watermark, registry, heuristics), which this plan does NOT touch. Where this plan needs a rule from the wedge plan, it restates the rule here so this document stays self-contained.

## Purpose / Big Picture

The shipped wedge gives a user one honest verdict about one image at a time: `lens verify photo.jpg` in a terminal, or right-click → verify in the browser extension. After this plan, the same honest engine becomes something other software and more people can build on: scripts consume verdicts as JSON without scraping human-oriented text; verdicts on *verified* assets also say what the credentials actually claim (who generated the asset, whether AI involvement was declared, when it was signed) instead of only "a valid chain exists"; the formats the tools accept are proven by test vectors rather than assumed; every push is checked by CI instead of by whoever remembered to run the loop; and the extension reaches Firefox users while the engine reaches JavaScript developers as an npm package. The founding rule is unchanged and non-negotiable everywhere: **no provenance data ≠ authentic**, and no upgrade may add visual or verbal language of safety around the Inconclusive tier.

## Progress

- [x] (2026-07-18) Plan authored, after the post-wedge hardening pass (branch `post-wedge-hardening`: cert-policy pinning tests, fuzz target, trust-list refresh workflow — see the wedge plan's Progress for details).
- [x] (2026-07-18) U1 complete — `render_json(report, file?)` lives in `crates/provenance-core/src/json.rs`; the WASM wrapper is now logic-free (delegates, shape byte-identical — parity suite passed untouched, so no engine rebuild was needed); `lens verify` takes `[--json] [--trust-anchors <PEM>] <FILE>...` with flags in any order and exits with the highest per-file code. Suite 27/27 green (2 new json tests, 2 new CLI parser tests), clippy clean; acceptance transcript in Artifacts.
- [x] (2026-07-18) U2 complete — `Report.credentials: Option<CredentialSummary>` (Some exactly when Verified; pinned corpus-wide and by dedicated tests), extracted by the concrete Layer-1 instance the pipeline now retains; rendered in CLI human output ("credential claims:" block), in the shared JSON (`credentials` object, absent keys omitted), and in the popup (from the engine JSON, textContent only). Suite 30/30 green, clippy clean; acceptance in Artifacts. Note: the browser shows the block only after the engine is next rebuilt (wasm-pack not installed on this machine — engine-side behavior proven by the parity and JSON tests; the release packaging script rebuilds regardless).
- [ ] U3 — format coverage proven by vectors (PNG at minimum; alignment of sniffing and CLI extension-guessing).
- [ ] U4 — CI baseline: build/lint/test on every push, scheduled cargo-audit and fuzz smoke.
- [ ] U5 — Firefox port and AMO listing collateral.
- [ ] U6 — npm packaging of the WASM engine (publish prep; maintainer publishes).
- [ ] U7 — page-scan badge UX behind an explicit opt-in site-access grant (design first; separate maintainer approval for the permission change).

## Surprises & Discoveries

(None yet for this plan. The hardening pass that preceded it recorded its surprises — e.g. the c2pa verifier's hard requirement of an organizationName in the end-entity subject — in the wedge plan's Surprises & Discoveries section.)

## Decision Log

- Decision: The flat JSON report format moves INTO `provenance-core` as a small hand-rolled renderer (no serde), and both the WASM wrapper (`crates/provenance-wasm`) and the new CLI `--json` flag call it. Today the wrapper hand-rolls that JSON itself; after U1 the wrapper is truly logic-free and CLI/extension JSON parity holds by construction rather than by parity tests.
  Rationale: two hand-rolled renderers of the same shape would drift; the core is sans-IO and a pure bytes-to-string renderer respects that; serde stays out per the wedge plan's dependency discipline (every new dependency needs a Decision Log entry, and a ~40-line renderer does not justify one).
  Date/Author: 2026-07-18 / plan authoring.
- Decision: The C API for `provenance-core` (a wedge-proposal deliverable) is deliberately NOT a milestone of this plan. It is recorded here as a known candidate, to be planned only when a concrete native consumer exists.
  Rationale: no consumer exists today; a C API without a consumer is unverifiable surface area (nothing would exercise it end-to-end, violating the "demonstrably working behavior" bar).
  Date/Author: 2026-07-18 / plan authoring.
- Decision: Milestone order is U1→U7 by increasing risk to the trust surface: U1–U3 change no permissions and no trust semantics, U4 protects everything after it, U5–U6 widen distribution of unchanged behavior, and U7 — the only milestone that expands browser permissions — goes last and is split into a design gate (maintainer approves the permission model) before any implementation.
  Rationale: the wedge's credibility is the product; the order keeps every intermediate state shippable and keeps the one permission-expanding change isolated and explicitly approved.
  Date/Author: 2026-07-18 / plan authoring.
- Decision: U2 extends the pipeline's `Report` with an optional `credentials: Option<CredentialSummary>` field populated ONLY when Layer 1 returns `Proof`, rather than widening the `Proof` variant itself.
  Rationale: `LayerFinding` is the evidentiary type system (the wedge plan's Interfaces section pins it); keeping it minimal preserves the honesty typing, while a separate summary struct carries descriptive metadata that has no bearing on the verdict. A summary must never appear beside a non-Verified verdict, and a test pins that.
  Date/Author: 2026-07-18 / plan authoring.

- Decision: U2 field names finalized: `digital_source_type` (the declared digitalSourceType URI, verbatim) plus `source_type_note` (a fixed descriptive phrase) replace the sketched `ai_involvement_declared`. The note exists for exactly two IPTC generative-AI types — `trainedAlgorithmicMedia` → "the credential declares this content AI-generated", `compositeWithTrainedAlgorithmicMedia` → "the credential declares AI-generated elements composited into this content" — and is defined once in `layers/c2pa.rs::source_type_note`; every surface (CLI, JSON, popup) prints it as data from that single definition, so no `wording_sync.rs` growth is needed: there is no second copy to drift. All other source types show only the verbatim URI, uninterpreted.
  Rationale: verbatim-URI + separate note keeps the reported fact (what the credential says) cleanly apart from the vocabulary gloss (what IPTC defines that value to mean), and single-sourcing the phrase is what the wording audit exists to approximate.
  Date/Author: 2026-07-18 / U2 implementation.
- Decision: U2 extraction lives in `C2paLayer::credential_summary`, called by `Pipeline::examine` only when the combined verdict is Verified; the pipeline retains its concrete `C2paLayer` (`with_layers` pipelines have none and never carry a summary). This re-parses the asset on the Verified path — accepted and marked in code — because the alternative (widening the `Layer` trait's return type) would touch all four layers for one layer's metadata. The extraction runs under the same panic guard as validation, and any extraction failure means "no summary", never a verdict change.
  Rationale: smallest change that keeps the evidentiary `LayerFinding` type untouched; milliseconds of re-parse on the rarest (Verified) path is the right price for that.
  Date/Author: 2026-07-18 / U2 implementation.

## Outcomes & Retrospective

(To be written at milestone completions.)

## Context and Orientation

Everything below assumes only this repository's working tree. **C2PA** ("Content Credentials") is an open standard that embeds a cryptographically signed provenance manifest inside a media file — who made it, with what tool, whether generative AI was involved — bound to the exact bytes by a content hash and validated against a trust list of root certificates. The project verifies such manifests and reports one of four verdict tiers with pinned wording: Verified, Indicated, Inconclusive ("no provenance data was found. This does NOT mean the asset is authentic."), Tampered. The canonical phrases live in `crates/provenance-core/src/verdict.rs` and are wording-audited by `crates/provenance-core/tests/wording_sync.rs`, which fails the suite if `README.md`, `extension/popup/popup.html`, or `.claude/skills/verdict-language/SKILL.md` drift from them. Any user-visible string this plan adds must follow `.claude/skills/verdict-language/SKILL.md` (understatement; never imply safety from absence of data).

The layout: `crates/provenance-core` is the sans-IO library — `pipeline.rs` holds `Asset` (bytes + optional MIME hint), the `Layer` trait, `LayerFinding` (NotEvaluated/NoSignal/Proof/Indication/TamperEvidence), `Report { verdict, findings }`, and the precedence rule Tampered > Verified > Indicated > Inconclusive; `layers/c2pa.rs` is the only real layer (the other three are honest gated stubs). `crates/provenance-cli` is the std-only `lens` binary (exit codes 0/10/20/30/2 are a contract). `crates/provenance-wasm` exports `verify_bytes(bytes, media_type?, trust_anchors_pem?) -> String` (flat JSON) via wasm-bindgen; `wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg` feeds the MV3 extension in `extension/` (plain JS: `background.js` service worker, `popup/`, trust anchors as a data file in `extension/trust/anchors.pem`). Known-answer vectors live in `crates/provenance-core/tests/vectors/` (currently five JPEGs + `manifest.tsv` + `test_ca.pem`), regenerated by the self-verifying `cargo run -p provenance-core --example gen_vectors`. The build loop every milestone must keep green, from the repo root:

    cargo fmt --all
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

Standing rules inherited from the wedge plan (restated so this plan is self-contained): every new dependency requires a Decision Log entry; image bytes never leave the device; signature-validation code (Layer 1, trust-list handling) merges only with the maintainer's (gwamoniak) recorded sign-off; the extension's permissions are `contextMenus` + `activeTab` + `storage` and may not grow except through U7's explicit design gate; a fresh clone with no network must build and pass all tests; the gated milestones M5–M7 (watermark, registry, heuristics) stay in the wedge plan and are NOT opened by anything here.

## Plan of Work

The milestones are independent enough to land one at a time, each leaving the tree shippable. U1 adds a `render_json(&Report) -> String` function to `provenance-core` (new module `crates/provenance-core/src/json.rs`), switches `crates/provenance-wasm/src/lib.rs` to call it, and teaches `crates/provenance-cli/src/main.rs` a `--json` flag and multi-file invocation. U2 adds the `CredentialSummary` extraction to `layers/c2pa.rs` (reading the already-validated manifest's claim generator, signing time, and declared digitalSourceType) plus rendering in CLI text, CLI JSON, and the popup. U3 extends `examples/gen_vectors.rs` to also emit a PNG set and aligns the byte-sniffing in `layers/c2pa.rs` with the CLI's extension-guessing. U4 adds `.github/workflows/ci.yml` (and extends the fuzz/audit cadence). U5 adds Firefox compatibility to `extension/` and AMO collateral to `docs/`. U6 adds an npm packaging path for the engine. U7 first produces a written permission/consent design in this plan, then (after maintainer approval) the content-script badge implementation.

## Milestones

**U1 — Machine-readable CLI (small; no trust-surface change).** Scope: `lens verify --json [--trust-anchors <PEM>] <FILE>...` prints one JSON object per file (the same flat shape `verify_bytes` returns: verdict id, approved phrase, per-layer findings, plus the file path), newline-delimited when multiple files are given; exit code is the worst verdict across files (Tampered worst, then the existing 0/10/20/30 ladder — "worst" = highest exit code). The renderer lives in core (Decision Log) and the WASM wrapper delegates to it; the existing wasm parity tests then pin the shared shape. Multi-file also works without `--json` (repeat the current human report per file). Acceptance: `lens verify --json <vector>` output parses as JSON (prove with `node -e` or `python -m json.tool` in the transcript); the wasm parity suite still passes; `lens verify --json valid_signed.jpg manifest_corrupted.jpg` (with anchors) exits 30.

**U2 — Say what the credentials claim (medium; wording-sensitive).** Scope: when — and only when — Layer 1 yields `Proof`, the report additionally carries a `CredentialSummary`: issuer (already surfaced), claim generator name/version, signing time if present, and whether the manifest declares AI involvement (the `digitalSourceType` on the active manifest's `c2pa.created`/`c2pa.actions` assertion, e.g. `trainedAlgorithmicMedia`). Render it in the CLI (indented under the c2pa line), in the JSON, and in the popup — worded per the verdict-language skill: descriptive statements of what the credential *claims* ("credential declares AI-generated content"), never endorsements. A test pins that `credentials` is `None` for every non-Verified vector, and the wording-sync audit grows to cover any new fixed phrases. Acceptance: `lens verify --trust-anchors test_ca.pem valid_signed.jpg` shows the summary (the test vectors declare `trainedAlgorithmicMedia`, so the AI-involvement line must appear); `stripped.jpg` shows no summary and remains word-for-word Inconclusive.

**U3 — Format coverage proven, not assumed (small).** Scope: today all five vectors are JPEG; PNG/WebP/GIF flow through the same code paths unproven. Extend `examples/gen_vectors.rs` to emit a PNG trio (signed / stripped / corrupted — stripping for PNG means removing the `caBX` chunk rather than JPEG's APP11 segments) with self-verification, and add them to `manifest.tsv`, the corpus test, and the wasm smoke. Align `sniff_media_type` in `layers/c2pa.rs` and `guess_media_type` in the CLI so both recognize the same set (the CLI currently guesses AVIF but the sniffer does not; either add AVIF sniffing — the `ftyp` box — or drop the CLI's AVIF guess and record which). Acceptance: corpus grows to eight vectors, all green natively and through the WASM artifact (`node scripts/wasm_smoke.mjs`).

**U4 — CI baseline (small; multiplies the value of everything else).** Scope: `.github/workflows/ci.yml` running the exact loop (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`) on push and pull request on ubuntu-latest; a scheduled weekly job running `cargo audit` (advisory only at first — failing the build on advisories is a separate maintainer decision) and a 5-minute fuzz smoke (`cargo +nightly fuzz run manifest_parsing fuzz/corpus fuzz/corpus_seed -- -max_total_time=300`; default sanitizer on Linux — the Windows `--sanitizer none` workaround recorded in the wedge plan is a local-machine matter). No third-party actions beyond `actions/checkout` and the official rust-toolchain setup, consistent with the trust-list workflow's minimal-supply-chain rule. Acceptance: a pushed branch shows the workflow green on GitHub; a deliberately misformatted file on a scratch branch shows it red.

**U5 — Firefox port (medium; distribution).** Scope: make `extension/` load and pass the smoke flow in Firefox (MV3 is supported; expected deltas: `browser` vs `chrome` namespaces — prefer a tiny feature-detect shim over a polyfill dependency — `background.service_worker` vs `scripts` in the manifest, and `chrome.action.openPopup()` availability, falling back to the badge as the code already does). Add AMO listing collateral to `docs/STORE_LISTING.md` (same honest copy; AMO-specific fields). The manual smoke script from the wedge plan (serve `scripts/serve_testpage.mjs`, load unpacked, right-click each vector) is the acceptance bar, run in Firefox; the maintainer performs the AMO submission, same as Chrome. Acceptance: all five (post-U3: eight) vectors produce their expected verdicts in Firefox, failure states included.

**U6 — npm package for the engine (small-medium; distribution).** Scope: a `wasm-pack build --target bundler` (or `web`, decided and recorded when implemented) packaging path producing an npm-publishable package (working name `@provenance-lens/verify-wasm`; the name is a maintainer decision at publish time) with a README whose wording passes the verdict-language rules, the same trust-anchor-parameter API, and a version pinned to the workspace version. Publish prep only — `npm publish` is the maintainer's action, like every release. Acceptance: `npm pack` produces a tarball; a scratch Node project can `npm install` the tarball and reproduce the wasm smoke results against the vector corpus.

**U7 — Page-scan badges behind explicit opt-in (large; the only permission change — DESIGN-GATED).** The wedge proposal's flagship UX: a content script that finds `<img>` elements and overlays a verdict badge per image. This requires host permissions the wedge deliberately avoided. The milestone is split: (a) a design revision to THIS plan specifying the permission model (Firefox/Chrome `optional_host_permissions` requested per-site through an explicit user gesture — "verify images on this site"), the consent copy, the performance strategy (lazy verification of visible images only, byte fetches through the same worker path), and badge visuals that carry no safety language for Inconclusive (gray, neutral, never a checkmark-shaped absence); then (b) implementation only after the maintainer approves the design in this Decision Log, mirroring the human-sign-off discipline. Acceptance for (b), when it happens: on the smoke page after granting site access, every vector image shows its correct badge; on a site without the grant, the extension behaves exactly as today.

## Concrete Steps

Per milestone, the loop is unchanged (repo root): `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; for anything touching the WASM boundary additionally `wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg`, the wasm-opt step from `.claude/skills/wasm-packaging/SKILL.md`, and `node scripts/wasm_smoke.mjs`; for anything touching the extension the manual smoke from the wedge plan's M3 section. On the current development machine (Windows) wasm-pack is not yet installed: `cargo install wasm-pack` once, before U1's parity work needs it. Commit at every stopping point with the milestone name in the message; update this plan's Progress in the same commit.

## Validation and Acceptance

This plan is done when a novice can, from a fresh clone: run the loop and see green including CI parity locally; script `lens verify --json` and branch on exit codes across multiple files; verify a signed vector and read what its credential claims (and see no summary on an unverified one); watch the same eight-vector corpus pass natively, in Node through the compiled artifact, and in both Chrome and Firefox through the extension; and install the engine from an npm tarball in a scratch project. U7 additionally requires the maintainer-approved design entry in the Decision Log before any code exists.

## Idempotence and Recovery

All steps are re-runnable: cargo/wasm-pack/npm-pack commands are idempotent, vector regeneration self-verifies before writing, and no milestone has a destructive step. If a milestone stalls, split its Progress entry into done/remaining and commit — this plan plus the tree must always be enough to resume.

## Artifacts and Notes

U1 acceptance (2026-07-18; `V=crates/provenance-core/tests/vectors`; JSON parsed back with node to prove machine-readability):

    $ lens verify --json --trust-anchors $V/test_ca.pem \
          $V/valid_signed.jpg $V/manifest_corrupted.jpg $V/plain.jpg
    {"file":"…/valid_signed.jpg","verdict":"verified","phrase":…,"findings":[…4 layers…]}
    {"file":"…/manifest_corrupted.jpg","verdict":"tampered",…}
    {"file":"…/plain.jpg","verdict":"inconclusive",…}
    → exit 30 (worst of 0 / 30 / 20)
    node JSON.parse per line: valid_signed.jpg -> verified | manifest_corrupted.jpg -> tampered | plain.jpg -> inconclusive

    $ lens verify --trust-anchors $V/test_ca.pem $V/valid_signed.jpg   # human mode unchanged
    → verdict: Verified …, exit 0

U2 acceptance (2026-07-18):

    $ lens verify --trust-anchors $V/test_ca.pem $V/valid_signed.jpg
      verdict: Verified: …
      [c2pa] valid provenance chain, issuer: Self-signed ephemeral certificate …
      credential claims:
        claim generator: provenance-lens test vectors/0.1.0
        declared source type: http://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicMedia
        note: the credential declares this content AI-generated
      → exit 0
    $ lens verify --trust-anchors $V/test_ca.pem $V/stripped.jpg
      verdict: Inconclusive: no provenance data was found. This does NOT mean the asset is authentic.
      (no credential block)                                  → exit 20
    $ lens verify --json … valid_signed.jpg | node JSON.parse
      credentials: { issuer, claim_generator, digital_source_type, source_type_note }
      (signing_time absent — the test vectors carry no timestamp; absent keys are omitted)

## Interfaces and Dependencies

New in U1, in `crates/provenance-core/src/json.rs`:

    pub fn render_json(report: &Report) -> String   // flat JSON: {"verdict": id, "phrase": ..., "findings": [{"layer": ..., "kind": ..., ...}]}

`crates/provenance-wasm` keeps its exported signature unchanged and delegates to `render_json`. Landed in U2, in `crates/provenance-core/src/pipeline.rs` (final shape; see the Decision Log for the rename from the sketched `ai_involvement_declared`):

    pub struct CredentialSummary {
        pub issuer: String,
        pub claim_generator: Option<String>,        // "name/version"
        pub signing_time: Option<String>,           // as reported by the validator
        pub digital_source_type: Option<String>,    // the declared digitalSourceType URI, verbatim
        pub source_type_note: Option<&'static str>, // fixed phrase for the generative-AI types
    }
    // Report gains: pub credentials: Option<CredentialSummary> — Some exactly when the verdict is Verified.

No new Rust dependencies are anticipated for U1–U4; U5–U7 add none to the Rust workspace (extension/JS only). Any deviation requires a Decision Log entry here.

---

Revision note (2026-07-18): initial authoring, immediately after the post-wedge hardening pass, from a reflection over the wedge plan's remaining non-gated candidates (proposal deliverables not yet built, recorded UX deferrals, infrastructure gaps). Reason: the wedge is shipped and signed-off work needs a successor plan that widens reach without touching the gated layers or the honesty rules.
