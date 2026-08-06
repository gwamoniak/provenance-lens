# Provenance Lens W3: add honest SynthID context without pretending to detect it

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document is maintained in accordance with `../solid-broccoli/PLANS.md`. It is the implementation handoff for Sol. Sol must be able to restart from this file alone, must update the living sections at every stopping point, and must not weaken a gate merely to make the milestone appear complete.

## Purpose / Big Picture

W3 exists because users reasonably ask whether an image carries Google's SynthID watermark, while Provenance Lens currently has no runnable SynthID decoder. The useful result we can implement today is narrower and honest: when a **Verified** C2PA credential itself records that an invisible watermark was applied, the CLI, JSON report, and extension popup will show that signed declaration and any algorithm identifier carried by the credential. They will also say that the pixels were not independently checked for that watermark. This descriptive context never creates an `Indicated` finding and never changes the verdict.

Actual SynthID detection remains a separate gated result. As of 2026-08-06, Google offers interactive SynthID checks in Gemini and an early-tester portal, but no documented public programmatic SynthID detector. Google's separately documented AI Content Detection API is a private-preview probabilistic classifier over pixel artifacts, noise, and spectral anomalies; it is not documented as a SynthID watermark detector. Sol must not wire that classifier into Layer 2 or label its output SynthID.

After the implementable part of this plan, a human can verify the behavior with a committed signed test image. `lens verify` will remain Verified, its watermark layer will report no signal for the clean pixels, and its credential-claims block will say that the verified credential records an invisible watermark and that the watermark was not independently detected. This deliberate contrast is the acceptance proof.

## Progress

- [x] (2026-08-06) W3 successor plan authored for Sol from the repository's current code, planning rules, watermark and wording rules, and current official Google and C2PA documentation.
- [x] (2026-08-06) Current vendor surface classified: no local SynthID image decoder; no documented public programmatic SynthID-verification endpoint; Google's AI Content Detection API is private preview and generic/probabilistic, so it does not open the Layer-2 gate.
- [ ] W3-M0 — Maintainer approves the manifest-context mechanism and its exact user-facing sentence before production code begins, satisfying the roadmap's “each mechanism needs its own approval” gate.
- [ ] W3-M1 — Add a self-verifying C2PA vector that declares watermarking while its pixels contain no supported local watermark, then add the credential-summary extraction and negative tests.
- [ ] W3-M2 — Render the declaration in CLI, JSON, and popup; update the manual and dated research note; run native, WASM, and JavaScript validation.
- [ ] W3-M3 — Prepare the cryptography-review packet and obtain the maintainer's dated sign-off before merge because `crates/provenance-core/src/layers/c2pa.rs` is touched.
- [ ] W3-D — Implement actual SynthID detection only after the vendor-detector gate in this plan opens. This item is intentionally blocked today and is not satisfied by W3-M1 through W3-M3.

## Surprises & Discoveries

- Observation: Google's AI Content Detection API does not expose a documented SynthID presence result. Its official documentation says it uploads JPEG, PNG, or WebP bytes and produces a probabilistic estimate from pixel-level artifacts, noise patterns, and spectral anomalies. It is in Private Preview and explicitly admits false positives and false negatives.
  Evidence: Google Cloud, “Detect AI-generated images,” last updated 2026-07-21, `https://docs.cloud.google.com/gemini-enterprise-agent-platform/models/ai-content-detection`.
  Consequence: this service is a possible future Layer-4 heuristic, not a W3 watermark detector. It is outside this plan.
- Observation: Google's official SynthID page supports verification through Gemini and an early-tester upload portal, but documents no stable REST contract for SynthID verification.
  Evidence: Google DeepMind, “SynthID,” checked 2026-08-06, `https://deepmind.google/models/synthid/`.
  Consequence: do not scrape the portal, automate Gemini, reverse engineer a private endpoint, or ship guessed request/response types.
- Observation: the current roadmap's online-connector idea requires uploading image bytes, while the repository's standing watermark skill permits network detectors to send perceptual hashes only. A real vendor detector cannot be integrated by silently choosing one rule over the other.
  Evidence: `.claude/skills/watermark-detection/SKILL.md` Privacy section and `PROVENANCE_LENS_ROADMAP_EXECPLAN.md` W3.
  Consequence: full-image upload requires a maintainer-approved privacy exception, revised store/manual disclosures, per-use consent, and a new Decision Log entry before code.
- Observation: C2PA already has a standards-based way for a signed manifest to record watermarking. Current specifications use `c2pa.watermarked.bound` and `c2pa.watermarked.unbound` actions; the older `c2pa.watermarked` action remains relevant for legacy manifests. A bound watermark is described by a `c2pa.soft-binding` assertion with an algorithm identifier.
  Evidence: C2PA Implementation Guidance and Content Credentials 2.3, checked 2026-08-06; the locally pinned `c2pa` 0.89.x source publicly exposes `assertions::SoftBinding` and `Manifest::assertions()`.
  Consequence: a verified manifest can provide useful signed context without issuer-name guessing and without claiming pixel detection.
- Observation: at plan-authoring time, the working tree already contains unrelated registry work: root `Cargo.toml` is modified and `crates/provenance-log/` is untracked.
  Evidence: `git status --short` on 2026-08-06.
  Consequence: these changes belong to the user. Sol must not overwrite, stage, reformat, move, or include them in a W3 commit. Before creating or switching branches, re-check status and have the maintainer finish or relocate that work if necessary.

## Decision Log

- Decision: Split W3 into W3-M, manifest-declared watermark context, and W3-D, actual vendor detection.
  Rationale: W3-M is implementable from signed, standardized evidence already parsed by the project. W3-D has no public detector contract today. Calling both “detection” would erase the most important distinction.
  Date/Author: 2026-08-06 / plan author.
- Decision: W3-M reads only explicit watermark actions in a **Verified active C2PA manifest**. It never infers SynthID from certificate issuer text, claim-generator text, company names, file metadata, or the fact that a product is publicly said to use SynthID.
  Rationale: issuer and product-name matching is brittle and would turn marketing knowledge into an asset-level claim. A signed action is attributable evidence; a string heuristic is not.
  Date/Author: 2026-08-06 / plan author.
- Decision: W3-M does not create a Layer-2 finding and does not alter verdict combination. It extends `CredentialSummary`, which is already present only when the combined verdict is Verified.
  Rationale: the credential says watermarking occurred, but Provenance Lens has not decoded the pixels. The result is descriptive metadata about a proof, not an independent watermark indication.
  Date/Author: 2026-08-06 / plan author.
- Decision: The fixed sentence proposed for approval is “the verified credential records an invisible watermark; the watermark was not independently detected”. Algorithm identifiers are displayed separately and verbatim. If none is present, the UI says “scheme not named in the credential”.
  Rationale: the sentence identifies the source of the claim and its limit in one breath. Displaying the algorithm verbatim avoids maintaining a speculative SynthID alias table.
  Date/Author: 2026-08-06 / plan author; maintainer approval pending at W3-M0.
- Decision: Recognize exactly `c2pa.watermarked.bound`, `c2pa.watermarked.unbound`, and legacy `c2pa.watermarked`. A soft-binding assertion alone is insufficient because it may describe a fingerprint rather than a watermark.
  Rationale: the action states that watermarking occurred. The accompanying soft-binding algorithm supplies scheme detail but does not independently classify the binding as a watermark.
  Date/Author: 2026-08-06 / plan author.
- Decision: Do not add a Rust dependency for W3-M. Use `c2pa::assertions::{Actions, SoftBinding}`, `Manifest::assertions()`, and the existing hand-rolled JSON renderer.
  Rationale: the pinned C2PA crate already exposes the required typed assertions, and the project's dependency discipline rejects convenience dependencies.
  Date/Author: 2026-08-06 / plan author.
- Decision: Google's AI Content Detection API is explicitly rejected as a W3-D implementation unless Google later documents a distinct, machine-readable SynthID-watermark result in its response.
  Rationale: generic probabilistic AI classification belongs, if anywhere, in gated Layer 4. Relabeling it as watermark detection would be a category error and an honesty violation.
  Date/Author: 2026-08-06 / plan author.
- Decision: Any change to `crates/provenance-core/src/layers/c2pa.rs` follows the standing human cryptography-review rule even though W3-M is descriptive and does not change validation state mapping.
  Rationale: the repository rule covers the entire signature-validation file. Review must verify that untrusted, invalid, and unsigned manifests can never acquire the new summary.
  Date/Author: 2026-08-06 / plan author.

## Outcomes & Retrospective

No implementation outcome yet. At W3-M1, record whether the pinned C2PA crate parsed all three action forms and multiple soft-binding assertions as expected. At W3-M3, compare the observed CLI/JSON/popup behavior against the purpose above and state plainly that actual SynthID pixel detection remains gated unless W3-D has separately opened.

## Context and Orientation

Provenance Lens is a Rust provenance verifier with four ordered evidence layers. Layer 1 validates C2PA Content Credentials. Layer 2 runs local invisible-watermark detectors. Layer 3 is a transparency-log registry, and Layer 4 is optional heuristics. Findings combine as Tampered, then Verified, then Indicated, then Inconclusive. Only Layer 1 may produce cryptographic `Proof`; watermark detectors cap at `Indication`. “No signal” never means authentic.

The main types are in `crates/provenance-core/src/pipeline.rs`. `Report` contains the verdict, per-layer findings, and `credentials: Option<CredentialSummary>`. The pipeline constructs this summary only when the combined verdict is Verified. Preserve that invariant.

`crates/provenance-core/src/layers/c2pa.rs` validates a manifest and implements `C2paLayer::credential_summary`. It currently extracts issuer, claim generator, signing time, and the first action's digital source type. The method re-parses the already-validated asset only on the Verified path and is contained by `catch_unwind`. This file is a human-sign-off surface.

`crates/provenance-core/src/layers/watermark.rs` owns local pixel detectors through `WatermarkDetector`. Do not add W3-M there: a manifest declaration is not a pixel probe. Do not add a fake SynthID detector whose `probe` method guesses from ordinary image features.

`crates/provenance-core/src/json.rs` is the single JSON renderer used by the CLI and WASM wrapper. `crates/provenance-cli/src/main.rs` renders human output. `extension/popup/popup.js` renders the same JSON using `textContent`, never `innerHTML`. A change to the core JSON automatically reaches the WASM boundary without changing `verify_bytes`.

Tests relevant to this work are `crates/provenance-core/tests/c2pa_layer.rs`, `crates/provenance-core/tests/vectors.rs`, `crates/provenance-core/tests/wording_sync.rs`, and `crates/provenance-wasm/tests/parity.rs`. `crates/provenance-core/examples/gen_vectors.rs` creates signed fixtures using ephemeral keys and commits only the public test CA, never private keys. `scripts/wasm_smoke.mjs` runs the compiled engine over the committed corpus.

A C2PA action is a signed statement describing something done to the asset. A soft binding is a non-cryptographic identifier embedded in or computed from the media, such as a watermark value or perceptual fingerprint, which may help recover a detached manifest. A soft binding is not a hard cryptographic binding and is not proof that a detector independently recovered the identifier from the queried pixels.

The official surface snapshot that controls this plan is dated 2026-08-06:

- Google DeepMind says Gemini and the SynthID Detector portal can inspect uploaded media for SynthID, with the portal still presented as an early-tester surface.
- Google Cloud's AI Content Detection API is Private Preview, accepts uploaded JPEG/PNG/WebP image payloads, and reports probabilistic AI-generation signals from generic pixel analysis. Its documentation does not promise a SynthID-specific result.
- C2PA 2.3 describes bound and unbound watermark actions and soft-binding algorithm identifiers. The project's pinned C2PA library exposes the data needed for W3-M.

## Plan of Work

### W3-M0 — Approve the honest scope

The maintainer reviews the Decision Log above and either approves or revises the fixed sentence and the action-label policy. Record the approval verbatim and dated in this plan before code. Approval of W3-M does not approve image upload, a Google Cloud dependency, or W3-D.

Re-run `git status --short`. At authoring time unrelated G2 registry edits are present. Do not create a branch or touch files until those changes can be kept out of the W3 diff. Once safe, use a dedicated branch such as `codex/w3-synthid-context`.

### W3-M1 — Prove the distinction with a signed vector, then extract it

First extend `crates/provenance-core/examples/gen_vectors.rs` with a special manifest definition and emit `crates/provenance-core/tests/vectors/manifest_declares_watermark.jpg`. The manifest must contain a `c2pa.watermarked.bound` action and a `c2pa.soft-binding` assertion whose `alg` is `com.example.provenance-lens.test-watermark` and whose block contains a harmless test value. The underlying clean JPEG must not be changed by a watermark embedder. Add the vector to `manifest.tsv` as Verified and keep the generator's self-verification rule.

This intentionally synthetic algorithm identifier is test-only and must never be shipped in production trust data or described as SynthID. Its purpose is to prove that Provenance Lens reports a credential declaration without converting it into a pixel detection.

In `crates/provenance-core/src/pipeline.rs`, add:

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct DeclaredWatermark {
        pub action_labels: Vec<String>,
        pub algorithms: Vec<String>,
        pub note: &'static str,
    }

and add `pub declared_watermark: Option<DeclaredWatermark>` to `CredentialSummary`. The `note` is the one maintainer-approved sentence from W3-M0. Keep action labels and algorithms in deterministic first-seen order and remove duplicates.

In `C2paLayer::credential_summary`, after obtaining the active manifest, call a private helper that:

1. Parses the active manifest's `Actions` assertion and keeps only the three accepted action labels.
2. Returns `None` when none of those actions exists, even when a `c2pa.soft-binding` assertion exists.
3. Iterates `manifest.assertions()` whose labels begin with `SoftBinding::LABEL`, decodes each with `ManifestAssertion::to_assertion::<SoftBinding>()`, and collects non-empty `alg` strings. A malformed individual assertion is ignored for summary purposes; validation remains the authority for the verdict.
4. Returns one `DeclaredWatermark` when a recognized action exists. An empty algorithm list is valid for `c2pa.watermarked.unbound` and is rendered as “scheme not named in the credential”.
5. Never examines certificate names, claim-generator strings, or image pixels.

Add focused integration tests in `crates/provenance-core/tests/c2pa_layer.rs`:

- A trusted signed manifest with a watermark action and soft-binding algorithm produces Verified plus `declared_watermark`, while Layer 2 is `NoSignal` for the clean test pixels.
- A trusted signed `c2pa.watermarked.unbound` action produces the declaration with an empty algorithm list.
- A trusted signed soft-binding assertion without a watermark action produces no declaration.
- The same declaring asset without its trust anchor is Tampered and has no credential summary.
- An unsigned asset has no credential summary.
- Duplicate actions and algorithms are de-duplicated without changing order.

Do not alter `combine`, `LayerFinding`, `WatermarkDetector`, or the four canonical verdict phrases.

### W3-M2 — Carry the signed declaration through every surface

Extend `credentials_json` in `crates/provenance-core/src/json.rs` to emit `declared_watermark` only when present. Its stable shape is:

    "declared_watermark": {
      "action_labels": ["c2pa.watermarked.bound"],
      "algorithms": ["com.example.provenance-lens.test-watermark"],
      "note": "the verified credential records an invisible watermark; the watermark was not independently detected"
    }

Keep the existing hand-written escaping function and add a small helper for arrays of JSON strings rather than adding serde. Add JSON tests proving the field is present on the declaring Verified vector and absent from ordinary, unsigned, and untrusted reports.

In `crates/provenance-cli/src/main.rs`, render the data inside the existing `credential claims:` block. Print the fixed note once, then `watermark action:` and `watermark scheme:` lines for the verbatim values. When `algorithms` is empty, print `watermark scheme: scheme not named in the credential`. Do not print “SynthID detected”.

In `extension/popup/popup.js`, append equivalent list items to the credential block using `textContent`. Treat every string as attacker-authored even though it came from a signed manifest. Never use `innerHTML`. The popup displays the engine-provided `note` verbatim rather than maintaining a second copy.

Update `docs/MANUAL.md` with a short subsection explaining the three distinct cases: locally detected watermark, verified credential declaration, and actual SynthID verification unavailable. Update `README.md` only if its status paragraph would otherwise imply W3 detection shipped. Create or update `docs/research/synthid-status.md` with the dated source snapshot and the three official links in this plan. Do not describe the generic AI Content Detection API as SynthID.

Update this plan and `PROVENANCE_LENS_ROADMAP_EXECPLAN.md` at the milestone boundary. W3-M may be marked complete; W3-D stays open.

### W3-M3 — Review and merge gate

Prepare a packet for the human cryptography reviewer containing the diff to `c2pa.rs`, the exact invariant that credentials remain `None` unless the final verdict is Verified, test output, the declaring-vector CLI transcript, and the negative untrusted transcript. The reviewer must specifically confirm that the new parser cannot strengthen a verdict, cannot create Layer-2 `Indication`, and cannot surface data from an invalid or untrusted manifest.

Record the maintainer's dated sign-off in this plan and in the main wedge plan's Decision Log as required by the standing rule. Do not merge before that entry exists.

### W3-D — Gate for a real SynthID detector

Do not create connector code, credentials UI, HTTP dependencies, or dead feature flags until every condition below is true:

1. A vendor publishes or grants this project access to a documented programmatic endpoint whose response contains a SynthID-specific watermark result, not only a generic AI probability.
2. The contract documents supported media, request limits, authentication, error semantics, data retention, regional processing, pricing, and terms compatible with an open-source client.
3. The maintainer explicitly approves the exception allowing image bytes to leave the device. The decision names which surfaces may upload and requires per-use consent; a global pre-checked consent box is insufficient.
4. At least one positive vendor-produced fixture and one negative control can be exercised repeatedly under the granted terms. A transformation battery and held-out false-positive/false-negative measurements are possible.
5. The connector can be tested with an injected mock transport on a bare machine. No network, account, or secret is required for the default build and test suite.

When these conditions are met, revise this ExecPlan before coding. The revision must record the exact endpoint and response schema, place transport in the CLI/extension rather than the sans-IO core, define a precomputed vendor result that the pipeline can consume, specify secret handling, add consent and store-listing copy, and define calibration acceptance. If the service uploads full images, the UI must say so immediately before each request. Vendor unavailable, access denied, authentication failure, rate limit, or network failure are errors/not-evaluated states, never Inconclusive evidence and never “no watermark”.

The current Google AI Content Detection API may be evaluated only under a separate Layer-4 heuristics plan. It cannot satisfy W3-D unless its documented contract changes to expose a SynthID-specific result.

## Concrete Steps

Work from `D:\sandbox\projects\provenance-lens`. Before edits:

    git status --short

Expect a clean tree for W3 work. If the registry changes recorded in Surprises are still present, do not stage or move them and do not switch branches over them.

After W3-M0 approval and a safe branch:

    git switch -c codex/w3-synthid-context
    cargo run -p provenance-core --example gen_vectors
    cargo fmt --all
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo test -p provenance-core --no-default-features
    cargo check -p provenance-wasm --target wasm32-unknown-unknown
    wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg
    node scripts/wasm_smoke.mjs
    node --check extension/popup/popup.js

Exercise the declaring vector through the CLI using the committed test root:

    cargo run -p provenance-cli -- verify --trust-anchors crates/provenance-core/tests/vectors/test_ca.pem crates/provenance-core/tests/vectors/manifest_declares_watermark.jpg

The important portion of the output must have this meaning:

    verdict: Verified
    [watermark] ran, no signal
    credential claims:
      note: the verified credential records an invisible watermark; the watermark was not independently detected
      watermark action: c2pa.watermarked.bound
      watermark scheme: com.example.provenance-lens.test-watermark

Then omit `--trust-anchors` and expect Tampered with no `credential claims:` block. Run with `--json` and confirm `declared_watermark` is present only in the trusted result.

Before any commit, inspect only the intended diff:

    git status --short
    git diff -- PROVENANCE_LENS_SYNTHID_EXECPLAN.md PROVENANCE_LENS_ROADMAP_EXECPLAN.md crates/provenance-core crates/provenance-cli extension/popup/popup.js docs README.md

Do not use `git add -A` while unrelated registry work exists. Stage explicit W3 paths only after the reviewer packet is ready and the maintainer has approved the branch contents.

## Validation and Acceptance

W3-M is accepted when all of the following observable statements are true:

- The committed declaring vector is Verified only with its test trust anchor, and the generator refuses to catalogue it under a different verdict.
- The same clean pixels produce no local watermark signal, while the credential block separately reports the signed watermark declaration and explicitly says it was not independently detected.
- A soft binding without a watermark action produces no watermark declaration.
- Untrusted, invalid, tampered, and unsigned assets never carry `CredentialSummary` or `declared_watermark`.
- CLI human output, core JSON, WASM parity, Node artifact smoke, and extension popup agree on the declaration.
- No canonical verdict phrase changes, no new dependency is added, and the core remains sans-I/O.
- `cargo fmt --all -- --check`, clippy with `-D warnings`, the full workspace tests, no-default-feature core tests, wasm32 check, WASM smoke, and JavaScript syntax check are green.
- A human cryptography reviewer has signed off in the Decision Log before merge.

W3-D is accepted only by a later revision with a real vendor contract and calibration. W3-M completion must never be reported as “SynthID detection shipped.”

## Idempotence and Recovery

The vector generator is the only supported way to regenerate committed signed fixtures; it uses ephemeral keys and overwrites only its named vector outputs. Re-running it is safe after reviewing the manifest catalogue diff. Never commit private keys or cloud credentials.

All W3-M schema additions are additive. If extraction fails on a manifest, omit `declared_watermark`; do not change the verdict and do not fall back to issuer guessing. If popup rendering fails, the JSON remains the source of truth and the change must not merge until the popup is fixed.

Rollback is deletion of the additive summary field, renderer branches, vector, and docs while leaving existing verdict and validation behavior untouched. Do not roll back by mapping a parsing failure to `NoSignal` or by suppressing C2PA validation errors.

W3-D must be feature-off and account-free by default when eventually designed. Revoking vendor access must leave local C2PA and open watermark detectors fully functional.

## Artifacts and Notes

Primary sources captured for the 2026-08-06 gate decision:

- Google DeepMind SynthID overview and detector availability: `https://deepmind.google/models/synthid/`.
- Google announcement separating SynthID verification in consumer products from the AI Content Detection API: `https://blog.google/innovation-and-ai/products/identifying-ai-generated-media-online/`.
- Google Cloud AI Content Detection API documentation: `https://docs.cloud.google.com/gemini-enterprise-agent-platform/models/ai-content-detection`.
- C2PA watermark/soft-binding guidance: `https://spec.c2pa.org/specifications/specifications/2.3/guidance/Guidance.html`.
- C2PA soft-binding algorithm list: `https://github.com/c2pa-org/softbinding-algorithm-list`.

Record concise acceptance transcripts here during implementation. Do not paste tokens, project IDs, full HTTP payloads, private-preview response bodies restricted by vendor terms, or image bytes.

## Interfaces and Dependencies

W3-M adds no dependency. The public report-side types are:

    pub struct DeclaredWatermark {
        pub action_labels: Vec<String>,
        pub algorithms: Vec<String>,
        pub note: &'static str,
    }

    pub struct CredentialSummary {
        // existing fields unchanged
        pub declared_watermark: Option<DeclaredWatermark>,
    }

The JSON addition is optional and backward-compatible:

    credentials.declared_watermark.action_labels: string[]
    credentials.declared_watermark.algorithms: string[]
    credentials.declared_watermark.note: string

No change is made to `WatermarkDetector`, `Layer`, `LayerFinding`, `Pipeline::configured`, `verify_bytes`, CLI exit codes, or verdict precedence.

No W3-D transport interface is specified yet because there is no qualifying vendor contract. Inventing one now would make this plan less executable, not more. The future revision must preserve the sans-I/O core by injecting a completed vendor response; network ownership, authentication, retry, and consent stay in the caller surface.

---

Revision note (2026-08-06): initial standalone W3 implementation plan for Sol. It replaces the roadmap's single gated paragraph with an executable manifest-context milestone and an explicit detector gate. The split is necessary because the newly documented Google AI Content Detection API is generic probabilistic analysis, not a documented SynthID watermark detector, while C2PA already offers a signed and standards-based declaration path that can be implemented without overclaiming.
