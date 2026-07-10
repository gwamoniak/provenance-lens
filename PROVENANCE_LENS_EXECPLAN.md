# Provenance Lens: ship an honest AI-content provenance verifier — Layer-1 CLI + browser extension first

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document is maintained in accordance with `PLANS.md`, which is checked into the sibling repository `../../cpp/solid-broccoli/PLANS.md` (this sandbox keeps the ExecPlan rules in one place; the discipline applies to every sibling project). If that file is not in your context, read it in full before revising this plan.

## Purpose / Big Picture

After this plan's wedge milestones (M0–M4), a user can right-click any image in their browser — or run `lens verify photo.jpg` in a terminal — and get one of four honest verdicts about its provenance: **Verified** (a valid, cryptographically signed C2PA provenance chain), **Indicated** (non-cryptographic signals of AI involvement), **Inconclusive** (no provenance data found — which explicitly does NOT mean authentic), or **Tampered** (provenance data present but failing validation). Nothing like this exists today with honest wording: existing "AI detectors" guess from pixels and overclaim in both directions.

The strategic bet (the "wedge"): shipping a small tool that is *provably correct about Layer 1 only* is more valuable than a big tool that guesses. In particular, the Tampered tier surfaces — at scale, in users' faces — that major platforms strip Content Credentials on upload. That visibility is the pressure mechanism for platforms to stop stripping them. Watermark detection, the registry, and heuristics come later, each behind an explicit gate, and none of them may dilute the honesty rules.

## Progress

Granular state; every stopping point must be recorded here, splitting partially-done items into done/remaining.

- [x] (2026-07-10) M0 scaffold: Cargo workspace (`provenance-core`, `provenance-cli`, `provenance-wasm`), four stub layers returning honest `NotEvaluated`, verdict tiers with approved phrases, combination rule with unit tests, std-only CLI (`lens verify`, `lens tiers`), wasm-bindgen wrapper with hand-rolled JSON, MV3 extension skeleton, 8 project agents, 2 skills, CLAUDE.md, README, git repo initialized.
- [x] (2026-07-10) M0 reconciliation with the full proposal document (`ai-content-verifier-proposal.md`, found pre-existing in the target directory at commit time — see Surprises): authored the remaining 6 skills (watermark-detection, rust-quality, wasm-packaging, webextension-mv3, security-checklist, provenance-registry), switched Layer 3 to perceptual hashing (PDQ/pHash) per the proposal, recorded the `WatermarkDetector` trait plan, adopted the proposal's privacy rules, and logged the naming/UX deviations below.
- [ ] M0 remaining: install the Rust toolchain on this machine (`rustup`, stable, `wasm32-unknown-unknown` target — see Concrete Steps), then run `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` and fix anything the scaffold got wrong (it was written without a compiler present; expect small breakage, not design breakage).
- [ ] M1: integrate the `c2pa` crate in `layers/c2pa.rs`; collect test vectors into `tests/vectors/`; CLI acceptance transcript recorded below; human cryptography-reviewer sign-off recorded in the Decision Log before merge.
- [ ] M2: wasm-pack build green; native/WASM parity test over the full vector set; artifact size measured and budgeted.
- [ ] M3: extension verifies images end-to-end via the bundled engine; honest failure states; manual smoke script written down.
- [ ] M4: ship the wedge — package the extension, audit every user-facing string against the `verdict-language` skill, write the store listing, record the final human sign-off.
- [ ] Post-wedge (each gated, see Milestones): M5 watermark layer, M6 registry layer, M7 heuristics layer.

## Surprises & Discoveries

- Observation: The development machine has no Rust toolchain and no GitHub CLI (2026-07-10; `cargo`, `rustc`, `gh` all absent from PATH, `~/.cargo` absent). The M0 scaffold was therefore authored compiler-blind.
  Evidence: `(eval):1: command not found: cargo` for all probed tools.
  Consequence: the first action of any implementer is the toolchain install in Concrete Steps, then treating the first `cargo test` run as a review of the scaffold.
- Observation: The full proposal document `ai-content-verifier-proposal.md` was already sitting in the target directory before scaffolding began (the maintainer had saved the pinned conversation there), and was discovered only when `git add -A` swept it into the M0 commit. It contains detail beyond the summary the scaffold was built from: the 8-skill list, the `WatermarkDetector` trait, PDQ/pHash for Layer 3, the content-script badge UI as flagship UX, the images-never-leave-the-device privacy rule, and the weeks 1–10 build order.
  Evidence: `git commit` output listed `create mode 100644 ai-content-verifier-proposal.md` among the committed files.
  Consequence: same-day reconciliation pass (see Progress); the proposal file stays in the repo as the origin document. Lesson: list the target directory before scaffolding into it.

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

## Outcomes & Retrospective

(To be written at milestone boundaries. Nothing shipped yet; M0 scaffold exists as of 2026-07-10.)

## Context and Orientation

You are in a fresh Cargo workspace; assume no prior knowledge. The domain in one paragraph: **C2PA** ("Content Credentials") is an open standard that embeds a cryptographically signed provenance manifest inside a media file — who made it, with what tool, whether generative AI was involved — bound to the exact bytes by a content hash, signed with an X.509 certificate validated against a trust list. A deeper primer lives in `.claude/skills/c2pa-spec/SKILL.md`. Separately, some AI generators embed **invisible watermarks** (statistical patterns in pixels, e.g. Google's SynthID), and a **transparency log** is an append-only, Merkle-tree-backed public record (the Certificate Transparency model) in which generators could register hashes of what they produce.

The layout:

- `crates/provenance-core/` — the library. `src/verdict.rs` holds the four-tier `Verdict` enum and the approved phrases (normative wording rules in `.claude/skills/verdict-language/SKILL.md`). `src/pipeline.rs` holds `Asset` (bytes + optional MIME hint), the `Layer` trait, `LayerFinding` (NotEvaluated / NoSignal / Proof / Indication / TamperEvidence), the `Pipeline` runner, and `combine()` — the precedence rule. `src/layers/` holds the four layers in pipeline order: `c2pa.rs`, `watermark.rs`, `registry.rs`, `heuristics.rs`; all currently honest stubs returning `NotEvaluated`. The core is sans-IO: it never opens files or sockets; callers pass bytes in, and anything network-shaped (the future registry lookup) receives an injected transport trait.
- `crates/provenance-cli/` — the `lens` binary (std-only): `lens verify <file>` prints a report and exits with the verdict's code; `lens tiers` prints the four tiers.
- `crates/provenance-wasm/` — a thin wasm-bindgen wrapper exporting `verify_bytes(bytes, media_type) -> String` (flat JSON: verdict id, approved phrase, per-layer findings).
- `extension/` — a Manifest V3 browser extension skeleton (plain JS, no framework): a context-menu entry on images, a popup listing the tiers, and honest "engine not bundled" messaging until M3 wires in the wasm-pack output at `extension/pkg/` (gitignored build product).
- `.claude/agents/` — eight scoped agents (`lens-rust-core`, `lens-wasm`, `lens-extension`, `lens-security-reviewer`, `lens-registry`, `lens-qa`, `lens-research`, `lens-docs`); `.claude/skills/` — eight skill packs (`c2pa-spec`, `watermark-detection`, `rust-quality`, `wasm-packaging`, `webextension-mv3`, `verdict-language`, `security-checklist`, `provenance-registry`).
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

**M3 — Extension end-to-end (no gate; ~2 weeks).** Scope: background worker fetches the bytes of the right-clicked image (the only network request the extension ever makes on the user's behalf — image bytes never leave the device), calls the engine, renders the report in the popup or an injected panel — approved phrases verbatim, badge colors per the proposal (green Verified / yellow Indicated / gray Inconclusive / red Tampered), and no visual language of safety anywhere near Inconclusive. The proposal's automatic page-scanning badge UI is post-wedge (Decision Log). Honest failure states for engine-missing, fetch-failed, unsupported-format. Write the manual smoke script into this plan. Acceptance: on a page with a Content-Credentials image (e.g. a c2patool-signed test image served locally), the flow shows Verified; on a stripped copy of the same image, Inconclusive; on a corrupted-manifest copy, Tampered.

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

(WASM artifact sizes, M1 acceptance transcripts, and extension screenshots get appended here as milestones land.)

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
