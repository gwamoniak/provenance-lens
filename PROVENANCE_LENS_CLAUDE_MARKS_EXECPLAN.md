# Provenance Lens: verify Claude content marks without guessing

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds. It is maintained according to the ExecPlan discipline in `../solid-broccoli/PLANS.md` (on this machine, `D:\sandbox\projects\solid-broccoli\PLANS.md`).

This plan was written on 2026-08-11 from Anthropic's official explanation of Claude content marking, the European Commission's Article 50 materials, the current Provenance Lens source tree, and the capabilities of the repository's pinned `c2pa` crate. It deliberately separates work that can be proved today from work that is blocked on detector documentation. Nothing in this plan is legal advice.

## Purpose / Big Picture

After this plan is implemented, Provenance Lens can examine supported files exported by Claude through the same generic C2PA verification path it uses for any other producer, including Claude SVG output once that format is admitted by the project. A valid, trusted Content Credential can produce the existing `Verified` verdict and show exactly what the credential declares. The product will say that the verified signer or claim generator processed the file; it will not turn that fact into an unsupported claim that Claude originated every idea, word, or pixel.

Claude's embedded text watermark is a different signal. Anthropic says supported Claude models embed an imperceptible mark in text and is working toward third-party detection, but its public article does not yet provide a detector, wire protocol, SDK, normalization rules, support thresholds, or error-rate data. Therefore this plan does not invent a text detector. It defines the evidence semantics, privacy constraints, specimen requirements, and opening gate so implementation can begin promptly when Anthropic publishes a detector contract that we can actually exercise and calibrate.

The observable outcome is intentionally asymmetric. For supported signed image and vector files, a user can obtain cryptographic proof today after the specimen and trust gates pass. For text, a user continues to receive an honest `NotEvaluated` explanation until the detector gate opens. A positive future text-watermark result can reach only `Indicated`, never `Verified`; a negative or unavailable result can never be described as proof that the text is human-authored.

## Progress

- [x] (2026-08-11) Read Anthropic's official “How Claude marks AI-generated content” article and recorded its exact product claims and limitations in this plan.
- [x] (2026-08-11) Checked the European Commission's Article 50 timing and the role of the Code of Practice on marking and detection of AI-generated content.
- [x] (2026-08-11) Audited the current core, CLI, WASM wrapper, extension, tests, trust bundle, manual, and pinned `c2pa` 0.89.3 source against Claude's file and text marking paths.
- [x] (2026-08-11) Authored this implementation plan on branch `codex/claude-content-marks-plan`; no product behavior has changed.
- [x] (2026-08-11) Maintainer opened the C0 gate with “go with C0”. Branch `codex/claude-c0-specimens` was created directly from clean `main`, then the approved plan commit was carried onto it; no temporary worktree was used.
- [x] (2026-08-11) C0 inventory contract created at `crates/provenance-core/tests/vectors/claude/README.md` and `manifest.tsv`. Genuine specimen collection remains in progress; the available Claude browser session requires maintainer sign-in, Claude Code is organization-policy blocked, and no replacement specimen was fabricated.
- [x] (2026-08-11) Three maintainer-supplied flower SVG candidates preserved byte-identically and catalogued. The shipped CLI returned C2PA `not_evaluated` because it does not admit SVG; the pinned reader with explicit `image/svg+xml` returned `no_signal` for all three. They are useful negative compatibility specimens but contain no manifest, so C0's positive-file and stripped-control acceptance gate remains open. Surface/model/acquisition facts are pending maintainer confirmation. Validation: all recorded hashes match; targeted clippy clean; all 40 `provenance-core` tests green including two new compatibility tests.
- [x] (2026-08-11) Regenerated flower batch preserved with its `files2.zip` acquisition envelope. Claude explicitly added ordinary `<metadata>AI-generated image</metadata>` and visible `AI-generated` text. Hashes of all three archive entries match the separately supplied files; explicit-MIME C2PA inspection still returns `no_signal` and no credentials. A regression test pins that editable AI-generated labels never become C2PA proof.
- [ ] C0 — Collect, hash, and inspect genuine, non-sensitive Claude specimens from supported surfaces. The maintainer gate is open; completion still requires at least one genuine file plus its stripped control and a recorded trust outcome.
- [ ] C1 — Prove Claude file provenance end to end, add SVG as an explicitly supported C2PA input, and publish accurate product wording after cryptography review.
- [ ] C2 — Open only after Anthropic publishes a usable third-party text-watermark detector contract; implement a CLI-first detector behind an explicit feature or connector and publish calibration before verdict influence.
- [ ] C3 — After C2 is accepted, add selected-text browser verification only if its detector and privacy model fit the extension; automatic page-text scanning remains out of scope.
- [ ] C4 — Complete cross-surface documentation, compatibility fixtures, release checks, and the final capability matrix.

## Surprises & Discoveries

- Observation: Anthropic uses two complementary mechanisms, not one universal “Claude detector.” Supported text carries a model-level embedded watermark, while supported exported files can carry signed C2PA provenance metadata. A copied passage may retain some text signal after its file wrapper is gone; a resaved image may retain pixels while losing C2PA metadata.
  Evidence: Anthropic's official article, updated 2026-08-11, describes embedded text watermarks separately from signed Content Credentials for supported file types such as SVG, PNG, and JPEG.
  Consequence: file proof and text indication must be represented and tested as separate evidence paths. A single `claude_detected: true` field would collapse incompatible meanings and is forbidden.

- Observation: Detection documentation is not public yet. Anthropic says it is working to support third-party detection and will publish technical information, but the article contains no algorithm, SDK, endpoint, key material, minimum passage length, language matrix, confidence semantics, or false-positive rate.
  Evidence: the article's “How can I detect Claude's markings?” and “Are there limitations?” sections.
  Consequence: C2 is genuinely closed. A generic AI-writing classifier, authorship model, or vendor-name heuristic does not satisfy the gate.

- Observation: A detected Claude mark is evidence of processing, not necessarily original authorship. Claude might only have proofread, translated, summarized, converted, or lightly transformed material, and the marked output may later have been edited, excerpted, or mixed with other content.
  Evidence: Anthropic explicitly documents these cases and cautions against treating the mark as proof of authorship.
  Consequence: all positive wording uses “processed by Claude” or “the credential declares,” never an unconditional “written by Claude,” “made by Claude,” or “AI-generated by Claude.”

- Observation: Absence is especially weak. Older or unsupported models, short passages, paraphrasing, translation, mixing, unsupported platforms, metadata-stripping conversions, and screenshots can all remove or evade the relevant signal. Some cloud-partner surfaces support text marking while signed file provenance may not be available.
  Evidence: Anthropic's limitation and platform-availability notes.
  Consequence: no signal remains `Inconclusive`; unsupported or too-short text is `NotEvaluated`, not `NoSignal`. The UI must show which detector actually ran and which content form it evaluated.

- Observation: The current project already has most of the honest file-provenance semantics needed for Claude. `CredentialSummary` is emitted only beside an already-`Verified` verdict and describes issuer, claim generator, signing time, and C2PA `digitalSourceType`; generative source types already render as “the credential declares …” rather than an endorsement.
  Evidence: `crates/provenance-core/src/pipeline.rs`, `crates/provenance-core/src/layers/c2pa.rs`, and `crates/provenance-core/src/json.rs`.
  Consequence: C1 should remain vendor-neutral. It should expose genuine Claude credentials through the existing summary, not add issuer-string matching or a special “Claude proof” verdict.

- Observation: Provenance Lens currently hints or sniffs only JPEG, PNG, WebP, GIF, and AVIF. The CLI does not recognize `.svg`, and `docs/MANUAL.md` documents the same five formats. However, the pinned `c2pa` 0.89.3 dependency contains an SVG reader/builder and upstream `c2pa-rs` documents `image/svg+xml` support.
  Evidence: `sniff_media_type` in `crates/provenance-core/src/layers/c2pa.rs`, `guess_media_type` in `crates/provenance-cli/src/main.rs`, `docs/MANUAL.md`, and the locally pinned crate's `src/asset_handlers/svg_io.rs`.
  Consequence: SVG is the first concrete format gap. C1 adds explicit SVG MIME/extension support and a signed SVG vector. It does not pretend arbitrary XML can be safely identified from unlabelled bytes.

- Observation: The current pinned C2PA SDK does not provide a primary plain-text asset handler. Mentions of `text/plain` in its tree concern assertions or ingredients; its own test utilities classify a `.txt` primary asset as unsupported. Newer C2PA specifications discuss unstructured text, but that is not a capability of the dependency we ship.
  Evidence: the local `c2pa-0.89.3` source and current C2PA specification material.
  Consequence: Claude text-watermark detection does not belong in the C2PA reader and must not be presented as cryptographic proof.

- Observation: A genuine Claude C2PA file may or may not chain to the official C2PA conformance anchors currently bundled by the project. The support article does not identify the signer certificate chain, and cloud-partner support varies.
  Evidence: Anthropic's article plus the opaque certificate bundle at `extension/trust/anchors.pem`.
  Consequence: C0 must inspect real specimens before any support claim. A trust-anchor change is never inferred from an issuer name or harvested from an arbitrary sample; it requires an authoritative root source and the standing human cryptography sign-off.

- Observation: W3-M, the separate plan for verified C2PA watermark declarations, is not a prerequisite for Claude file verification and is not merged into this branch at plan-authoring time. Anthropic promises signed provenance metadata but does not promise that every file includes a `c2pa.watermarked` action.
  Evidence: `PROVENANCE_LENS_SYNTHID_EXECPLAN.md` and branch history for `codex/w3-synthid-context`.
  Consequence: C1 uses the generic verified credential summary. If W3-M later lands and a genuine Claude manifest contains an explicit watermark declaration, that context may appear too, but this plan never assumes it.

- Observation: C0 could not collect a genuine specimen anonymously or through the installed Claude Code client. The available Claude web surface stopped at account sign-in, no authenticated Chrome or Edge browser was connected, and Claude Code 2.1.220 reported that organization policy disables subscription access and requires an API key. `c2patool` is also not installed on this machine.
  Evidence: browser state at `https://claude.ai/login`; browser availability checks on 2026-08-11; a benign, tool-disabled, non-persistent Claude Code request returning the organization-policy message before any model response; and `Get-Command c2patool` returning no command. Authentication output was reduced to non-identifying status fields; no credentials or API keys were read.
  Consequence: the inventory schema and safe collection procedure were completed while the maintainer signs in or supplies raw exports. C0 remains open; public samples, fake vendor strings, and locally signed lookalikes are not substituted for genuine Claude bytes. Independent `c2patool` inspection remains optional and requires separate maintainer approval to install.

- Observation: The first real Claude SVG candidates are structurally valid but unsigned. With no MIME hint, Provenance Lens reports that C2PA was not evaluated; with `image/svg+xml`, the pinned C2PA reader evaluates them and reports no manifest store.
  Evidence: the three hashes and command transcripts in `crates/provenance-core/tests/vectors/claude/README.md`, pinned by `tests/claude_svg_compat.rs`.
  Consequence: C1's SVG admission work remains justified, because it changes “did not look” to “looked, found no manifest.” It will not, by itself, verify these Claude outputs. Availability truly varies by surface or export path, exactly as Anthropic's article warns; C0 still needs a positive signed export before any product support claim.

- Observation: Asking Claude to label its SVG output produced a visible caption and a normal `<metadata>` element, not signed provenance. The regenerated archive and files are byte-distinct but still have no C2PA manifest or credentials.
  Evidence: `files2.zip` and the three `*-labeled.svg` hashes, exact two-line diffs, explicit-MIME inspection, and `editable_ai_generated_labels_do_not_become_c2pa_proof`.
  Consequence: plain producer self-declarations are useful context but cannot affect the proof verdict. Provenance Lens must not map arbitrary SVG metadata text to `Indication` or `Verified`; only a defined, independently evaluated evidence source may do so.

## Decision Log

- Decision: Treat signed file provenance and embedded text watermarking as separate evidence sources with separate gates.
  Rationale: one is a cryptographically validated manifest attached to a file; the other is an imperceptible statistical/model-level text signal whose detector contract is not public. Combining them would erase the distinction between `Proof` and `Indication`.
  Date/Author: 2026-08-11 / planning session.

- Decision: Keep Claude file handling vendor-neutral. Do not infer Claude involvement by matching `issuer`, `claim_generator`, certificate subject, filename, metadata strings, or model names.
  Rationale: strings are forgeable and C2PA already gives us the correct trust and integrity mechanism. If a trusted manifest names Claude or an Anthropic tool, the UI may display that signed claim verbatim through `CredentialSummary`; it must not create a stronger conclusion.
  Date/Author: 2026-08-11 / planning session.

- Decision: A future positive Claude text-watermark result has an `Indication` ceiling. It means that a supported detector found a Claude mark consistent with Claude processing the passage; it does not prove original authorship or exclude later editing.
  Rationale: this follows Anthropic's own limitation language and the project's rule that only Layer 1 C2PA may return `Proof`.
  Date/Author: 2026-08-11 / planning session.

- Decision: A future negative text result becomes `NoSignal` only when the official detector actually ran on content within its documented model, language, length, and normalization envelope. Unsupported content, an absent detector, an unavailable endpoint, or an ambiguous response is `NotEvaluated`.
  Rationale: “did not look” must remain distinct from “looked and found no mark,” and neither state establishes human authorship.
  Date/Author: 2026-08-11 / planning session.

- Decision: Do not add a speculative text-detector trait, dependency, network permission, endpoint, or user-facing capability flag during C0/C1.
  Rationale: the missing Anthropic contract determines the correct byte canonicalization, response semantics, licensing, and local-versus-remote architecture. Premature code would fossilize guesses at exactly the boundary that needs vendor facts.
  Date/Author: 2026-08-11 / planning session.

- Decision: Support SVG first through an explicit `.svg` extension or `image/svg+xml` caller hint. Do not broaden `sniff_media_type` to classify arbitrary XML unless a bounded, independently reviewed rule is added with adversarial tests.
  Rationale: SVG has no simple binary magic like PNG or JPEG. CLI filenames and browser response MIME types cover the normal Claude export path without making an unsafe content guess.
  Date/Author: 2026-08-11 / planning session.

- Decision: Trust changes remain a human cryptography-review surface. A real Claude sample can demonstrate a chain but cannot authorize its own root.
  Rationale: accepting a root bundled inside the artifact under test would make verification circular. The existing sign-off rule in `PROVENANCE_LENS_EXECPLAN.md` continues to bind C1.
  Date/Author: 2026-08-11 / planning session.

- Decision: For an online future detector, raw text upload is opt-in per use, never part of automatic page scanning, and never performed from `provenance-core`. A local detector is preferred if Anthropic publishes one under usable terms.
  Rationale: the core is sans-I/O, and text can be more sensitive than the image bytes the current privacy policy promises never to upload. A remote design requires explicit consent, endpoint disclosure, retention documentation, error handling, and store-policy review.
  Date/Author: 2026-08-11 / planning session.

- Decision: ACCEPTED — open C0 and authorize collection of benign, publishable Claude specimens under the plan's raw-byte, hashing, privacy, and trust rules.
  Rationale: the maintainer explicitly directed “go with C0” after pushing the approved plan branch. This opens specimen work only; C1 verifier changes and C2 text detection retain their own gates.
  Date/Author: 2026-08-11 / maintainer (gwamoniak).

## Outcomes & Retrospective

At plan-authoring time, no product capability was changed. The useful outcome is a bounded path that can improve Claude coverage without weakening the project's evidence model. C1 is actionable after genuine specimens and trust behavior are known. C2 is intentionally blocked on Anthropic, with objective opening conditions instead of a vague “future detector” placeholder.

Update this section after every milestone. Record the exact specimen surfaces and models that worked, the certificate-chain result, whether SVG survived browser/CLI/WASM parity, the detector version and calibration numbers if C2 opens, and any deviation from this plan. If Anthropic's forthcoming documentation contradicts an assumption here, revise the plan before changing code.

## Context and Orientation

Provenance Lens is a Rust workspace at `D:\sandbox\projects\provenance-lens`. `crates/provenance-core` is a sans-I/O engine. `crates/provenance-cli` owns local file and terminal I/O. `crates/provenance-wasm` exposes the same byte-oriented verifier to JavaScript. `extension/` is a Manifest V3 browser extension that currently verifies image URLs after an explicit context-menu action and can perform opt-in image scanning per site. The project combines four evidence layers in this order: C2PA proof, watermark indication, registry indication, and heuristics. Verdict precedence is `Tampered > Verified > Indicated > Inconclusive`. Only the C2PA layer may emit `LayerFinding::Proof`.

The term **Content Credential** means a C2PA manifest bound to an asset and signed by a credential whose chain can be evaluated. A `Verified` result in this project means the manifest validates, its asset binding is intact, and its signer chains to configured trust anchors. It does not mean every statement in the asset is factually correct. `CredentialSummary` reports what that verified manifest claims.

The term **embedded text watermark** in this plan means Anthropic's model-level, machine-readable signal incorporated into supported Claude text. It is not visible typography, file metadata, a C2PA signature, or a general classifier score. A future detector result is non-cryptographic evidence and remains an `Indication`.

Anthropic says models launched in the European Union on or after 2026-08-02 support machine-readable marking at launch, with older models updated during a transition. It says supported markings apply worldwide across the Claude Platform/API, Claude, Claude Code, Claude Cowork, and Claude Tag, while individual surfaces and features may lack some marking types. For AWS, Google Cloud, and Microsoft Foundry, supported models carry embedded text marking, while signed file provenance may not be available on every platform. These statements define a specimen matrix to test; they are not permission to label every Claude-branded output as marked.

The European Commission says Article 50 transparency obligations apply from 2026-08-02 and the Code of Practice helps providers meet marking and detection duties. Provenance Lens remains an independent verifier, not a declaration that a producer is legally compliant. Product documentation may explain that ecosystem context but must not give legal compliance advice.

Authoritative sources checked for this plan are:

* Anthropic Help Center, “How Claude marks AI-generated content,” updated 2026-08-11: `https://support.claude.com/en/articles/16266773-how-claude-marks-ai-generated-content`.
* European Commission, “Code of Practice on marking and labelling of AI-generated content”: `https://digital-strategy.ec.europa.eu/en/policies/code-practice-ai-generated-content`.
* Content Authenticity Initiative, `c2pa-rs` supported formats: `https://github.com/contentauth/c2pa-rs/blob/main/docs/supported-formats.md`.

The following capability matrix is the contract this plan implements:

| Claude output presented to Provenance Lens | Signal available in principle | Project result after this plan |
| --- | --- | --- |
| Original PNG or JPEG with a trusted, valid C2PA manifest | Cryptographic file provenance | Existing C2PA path returns `Verified`; signed claims are shown descriptively |
| Original SVG with a trusted, valid C2PA manifest | Cryptographic file provenance | C1 adds explicit SVG handling, then returns `Verified` through the same C2PA path |
| Signed file whose manifest is broken or asset binding changed | C2PA tamper evidence | `Tampered`, as today |
| Signed file whose signature is valid but signer is not trusted | Cryptographic validity without configured trust | Current policy remains `Tampered`, with the valid-signature/untrusted-signer detail; never silently add a root |
| Resaved, converted, or screenshotted file with metadata removed | Usually no C2PA manifest | `Inconclusive`; absence is not authenticity |
| Supported text long enough for a future official detector | Embedded text watermark | C2 may return at most `Indicated` after the gate and calibration pass |
| Too-short, unsupported, or unevaluated text | Detector cannot make a supported observation | `NotEvaluated`, contributing to `Inconclusive` |
| Text on which the official detector ran and found no mark | No supported mark detected | `NoSignal`, still `Inconclusive`; never “human-written” |
| PDF, office document, archive, audio, or video exported by Claude | Depends on actual surface and file support | Out of scope until a genuine specimen and parser support prove that exact format |

## Plan of Work

### C0 — Freeze the evidence and specimen contract

The maintainer first accepts this plan with a dated Decision Log entry and authorizes collection of test outputs. Generate a small, non-sensitive specimen set using only supported Claude models and surfaces. Do not manufacture a file that merely says “Anthropic” in an assertion, and do not convert or resave the original before the first hash is recorded.

Create `crates/provenance-core/tests/vectors/claude/README.md` and `manifest.tsv`. For each raw specimen record the surface, exact model identifier, generation date, region/account context when relevant, output type, original filename, HTTP content type if available, SHA-256, whether the file was directly downloaded or copied, and whether it may be committed. Use a benign prompt whose output can be published. Target at least one directly downloaded PNG or JPEG and one SVG if the surface offers it. Text specimens remain external during C0 because no detector can consume them yet; record their SHA-256 and generation facts without claiming a detectable result.

For every file specimen, inspect it with the current `lens verify --json`, the pinned library, and—when available—`c2patool --info` and `c2patool --certs`. Record the active-manifest labels, claim generator, issuer, `digitalSourceType`, signing time, validation status codes, complete certificate-chain fingerprints, and whether the current `extension/trust/anchors.pem` establishes trust. Preserve the original bytes. Make two derived negative controls only after hashing the original: one metadata-stripped resave and one byte/content edit intended to break an embedded manifest. Derived files must be named as controls and never confused with raw Claude output.

If the genuine specimen chains to the shipped official anchors, C1 may test the ordinary `Verified` path. If it is cryptographically valid but untrusted, stop the support claim at that fact. Locate an authoritative Anthropic or C2PA trust-root publication; do not take the root from the specimen. Any proposed anchor addition receives the standard trust provenance header, certificate-policy tests, and a human cryptography review. If no authoritative trust source exists, the plan records Claude files as currently untrusted and does not weaken validation to make the demo green.

C0 acceptance is observable when `manifest.tsv` contains hashes for at least one genuine Claude file and its stripped control, the inspection record states exactly which C2PA assertions and certificate chain were present, and the maintainer has made a dated go/no-go decision for C1. Specimens that cannot legally or safely be committed live outside the repository behind `PROVENANCE_LENS_CLAUDE_FIXTURES`; repository tests use recorded hashes and skip the external compatibility check with an explicit reason. At least one vendor-neutral signed SVG produced by our ephemeral test signer will still be committed in C1 so the bare-machine format tests are deterministic.

### C1 — Verify Claude-signed files through generic C2PA

Add `image/svg+xml` for `.svg` in `guess_media_type` in `crates/provenance-cli/src/main.rs`, with unit tests for lower- and upper-case filenames. Teach the core C2PA layer to accept an explicit `image/svg+xml` media hint. Do not claim hint-free SVG sniffing unless implementation adds a bounded rule with adversarial XML/HTML tests; unnamed SVG bytes without a hint may honestly remain `NotEvaluated`. Update stale comments that currently say the CLI and byte sniffer recognize exactly the same set.

Extend `crates/provenance-core/examples/gen_vectors.rs` to produce a small SVG, sign it with the existing `EphemeralSigner`, write only the public test CA, and self-verify the result. Add it to the normal vector manifest and parity suite. Tests must prove: a trusted signed SVG is `Verified`; the same SVG without its test anchor is never `Verified`; an unsigned SVG is `Inconclusive`; a post-signing SVG content edit is `Tampered`; and hostile or malformed SVG bytes do not panic. The image watermark layer may report `NotEvaluated` for SVG because its pixel decoders are raster-oriented; that does not prevent the stronger C2PA `Proof` from producing the combined `Verified` verdict.

Run the genuine Claude specimen compatibility test separately from the generated conformance vector. The generated vector proves our SVG plumbing, not Anthropic interoperability. For each permitted genuine PNG, JPEG, and SVG, assert only facts observed in C0: expected validation state, exact certificate fingerprints or trust outcome, and stable assertion labels. Avoid pinning volatile timestamps or model versions unless the specimen record makes that intentional.

Do not add a `claude` boolean or special verdict. Use `CredentialSummary` for issuer, claim generator, signing time, and `digitalSourceType`. If inspection reveals a useful signed field that the generic summary omits, propose a vendor-neutral `CredentialSummary` field named after the C2PA concept, add it only after comparing at least one non-Anthropic vector, and keep it descriptive. Wording must distinguish three statements: the chain is verified; the credential identifies a processor/claim generator; and the credential may declare an AI-related source type. Those statements do not establish authorship.

Update `docs/MANUAL.md`, `README.md`, `crates/provenance-wasm/README.md`, `extension/README.md`, and `docs/STORE_LISTING.md` only after the tests prove the behavior. Name SVG in the supported-format list. Add a short Claude note that signed file provenance is checked generically, availability varies by model/surface, and a missing mark is inconclusive. Do not advertise embedded text detection during C1.

This milestone edits `crates/provenance-core/src/layers/c2pa.rs`, so it occurs on its own implementation branch and does not merge until the maintainer records cryptography sign-off in `PROVENANCE_LENS_EXECPLAN.md`. Acceptance requires native and WASM parity for the generated signed SVG, the genuine-specimen result recorded, no trust relaxation, and exact conservative wording across CLI JSON and the extension popup.

### C2 — Add Claude text-watermark detection only after the vendor gate opens

C2 opens only when an official Anthropic source provides all information needed to produce and challenge a result: a programmatic detector or implementable algorithm; covered models, surfaces, languages, and content types; minimum and maximum input lengths; exact byte/text normalization rules; response/status semantics; versioning; documented or measurable error behavior; licensing compatible with this repository; and, if remote, endpoint authentication, retention, regional processing, rate limits, and terms that permit an independent client. Record the source URLs, version, retrieval date, and maintainer approval in this plan before writing code.

When the gate opens, first capture a detector conformance corpus in `crates/provenance-core/tests/vectors/claude-text/` or an external fixture directory if vendor terms forbid redistribution. Include marked text from at least two supported Claude surfaces and models, human-authored controls, and documented unsupported/too-short cases. Keep original UTF-8 bytes. Build a transformation battery that measures copy/paste, CRLF↔LF, leading/trailing whitespace, Unicode NFC/NFD, markdown/code formatting, light edits, excerpting, concatenation with unmarked text, paraphrase, and translation. Apply only normalizations the official contract authorizes; “helpful” normalization can destroy or fabricate the signal.

Use the existing Layer 2 evidence ceiling. The stable semantic outcome, independent of the eventual SDK shape, is:

    pub enum TextMarkOutcome {
        Detected { provider: String, detail: String },
        NotDetected,
        Unsupported { reason: String },
    }

`Detected` maps to `LayerFinding::Indication` with user wording equivalent to: “A supported Claude mark was detected. This suggests Claude processed the text; it does not establish original authorship.” `NotDetected` maps to `NoSignal` only inside the vendor's supported envelope. `Unsupported`, detector absence, transport failure, authentication failure, and ambiguous vendor responses map to `NotEvaluated` with a specific reason. Do not expose a probability as calibrated confidence unless Anthropic defines it and our calibration confirms its interpretation.

Choose architecture only after the detector is known. If Anthropic publishes a local library, model, or implementable specification, prefer a `claude-text-watermark` opt-in feature and keep inference local. The bare-machine workspace must still compile and test without downloaded weights. If Anthropic exposes only a network service, the HTTP transport belongs in the CLI or extension integration, never in `provenance-core`; the core receives a typed, precomputed detector outcome. Add no default endpoint and no silent fallback to another AI detector.

For the CLI, add an explicit text entry point such as `lens verify-text [--json] <FILE|->` rather than treating arbitrary `.txt` bytes as C2PA media. The final spelling follows the published detector's input contract. Standard input and files are read as exact bytes, UTF-8 validity is handled according to that contract, and the report clearly labels C2PA/registry/image-only layers as not applicable rather than claiming they ran. Exit codes stay 0/10/20/30/2; a text-watermark hit normally exits 10 (`Indicated`), no mark or unsupported text exits 20 (`Inconclusive`), and malformed input or required-connector failure exits 2 only when the command cannot produce an honest report.

Before a text detector can affect a verdict, publish `docs/CLAUDE_TEXT_CALIBRATION.md` with corpus provenance, detector/version, supported envelope, counts, false-positive and true-positive rates, confidence intervals where meaningful, and every transformation result. The clean corpus must be large and diverse enough to test the threshold claimed by the vendor; if no acceptable false-positive gate can be measured, the detector remains diagnostic-only and does not produce `Indication`. C2 acceptance includes deterministic positive, negative, unsupported, short-text, Unicode, and failure-path tests; CLI JSON snapshots; no network in default tests; and documentation that quotes no vendor certainty stronger than the official contract.

### C3 — Add explicit selected-text verification to the extension

C3 starts only after C2 is shipped and reviewed in the CLI. Add a separate context-menu item with `contexts: ["selection"]`; use `selectionText` only after the user's explicit gesture. Do not scan all page text, observe typing, inspect password fields, or automatically upload content. The existing image context menu and site-image scanning remain unchanged.

If detection is local and fits the WASM size/performance budget, extend the wrapper with a text-specific function rather than overloading `verify_bytes` ambiguously. If detection is remote, show a per-use confirmation naming the destination and the fact that selected text will leave the device. Request only the minimum host permission, store no raw selection in `chrome.storage`, clear transient results on popup close or session end, and update `docs/STORE_LISTING.md` and the privacy policy before packaging. A vendor login, API key, or billing requirement must be explicit setup, never bundled secret material.

The popup renders the same `Indicated`, `NoSignal`/`Inconclusive`, and `NotEvaluated` distinctions as the CLI and includes input scope, detector version, and limitations. C3 acceptance requires Chrome and Firefox manual smoke tests; tests that an image action never touches text and a text action never fetches an image URL; denied-consent and offline behavior; hostile-text rendering through `textContent`; session-data cleanup; and a fresh WASM size measurement.

### C4 — Compatibility, documentation, and release boundary

Create `docs/CLAUDE_MARKS.md` as the user-facing capability matrix derived from observed specimens and detector tests. It names supported formats and surfaces, states the test date and model versions, distinguishes C2PA proof from text indication, explains “processed” versus “authored,” and lists the transformations that can remove each signal. Link it from the README and manual. Do not list a cloud partner or file type merely because Anthropic mentions the platform generally; list only exact combinations that C0/C2 exercised or mark them explicitly untested.

Run a cross-surface matrix for CLI text/file modes, WASM, Chrome, and Firefox. The same bytes, trust anchors, and detector version must produce the same verdict, approved phrase, and evidence detail wherever that path is supported. Add JSON schema or snapshot tests for any new fields. Existing canonical verdict phrases remain byte-identical and `wording_sync` stays green.

C4 is complete when a novice can answer, from one report, what was evaluated, which mark type was checked, whether the result is cryptographic proof or non-cryptographic indication, what the credential/detector actually says, and why absence proves nothing. Package and store publication remain maintainer actions under the existing release rules.

## Concrete Steps

All commands run from `D:\sandbox\projects\provenance-lens`. Each implementation milestone gets its own `codex/` branch from a clean, current `main`; do not use a temporary checkout path.

For C0, create the specimen inventory and inspect without changing verification behavior:

    git switch main
    git pull --ff-only
    git switch -c codex/claude-c0-specimens
    cargo run -p provenance-cli -- verify --json --trust-anchors extension/trust/anchors.pem <raw-claude-file>
    c2patool <raw-claude-file> --info
    c2patool <raw-claude-file> --certs
    certutil -hashfile <raw-claude-file> SHA256

`c2patool` is an optional independent inspection aid, not a production dependency. On systems where it is absent, record that and use the pinned Rust reader plus certificate fingerprints. Never install or download it without the maintainer's approval.

For C1, after the specimen gate and on a fresh implementation branch, update the CLI hint, SVG vector generator, C2PA tests, parity tests, and docs. Then run:

    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo test -p provenance-wasm --test parity
    cargo check -p provenance-wasm --target wasm32-unknown-unknown
    cargo run -p provenance-cli -- verify --json --trust-anchors crates/provenance-core/tests/vectors/test_ca.pem crates/provenance-core/tests/vectors/<signed-svg>
    cargo run -p provenance-cli -- verify --json --trust-anchors extension/trust/anchors.pem <raw-claude-file>

As of 2026-08-11, the main branch has known unrelated formatting and `unused_mut` drift in `crates/provenance-log`. Record those baseline failures if still present; do not hide them or mix their repair into C1 unless the maintainer explicitly expands scope. Package or path-specific checks must still be green for every touched crate, and the full-workspace result must not regress.

For C2, do nothing until the gate entry is dated in this plan. Once open, pin the official detector/version, capture fixtures, implement CLI-first, run the transformation battery through the production detector, and publish the calibration document before enabling `Indication`. Record the exact commands here when the vendor interface makes them knowable; do not invent placeholders that look executable.

For C3, rebuild the WASM package, run the existing Node smoke script and extension tests, then perform the documented Chrome/Firefox manual flow. Record raw/gzip WASM sizes before and after. For C4, run the complete workspace checks, wording audit, release packaging checks, and the observed compatibility matrix.

At every milestone boundary, inspect:

    git status --short
    git diff --check
    git diff --stat main...HEAD

Commit only files in the milestone's stated scope. Do not push, merge, or publish without the maintainer's explicit instruction; C1 additionally requires the recorded cryptography sign-off.

## Validation and Acceptance

Plan-level acceptance is behavioral. A raw, supported Claude PNG/JPEG/SVG with a valid trusted Content Credential produces the same generic `Verified` proof as any other producer and displays only signed descriptive claims. A modified signed asset produces `Tampered`. A metadata-stripped copy produces `Inconclusive`, accompanied by the reminder that absence is not authenticity. An SVG without a MIME hint may remain honestly `NotEvaluated`; the CLI extension and browser content type provide the supported normal path.

Text acceptance is unavailable until C2 opens. After it opens, a supported marked passage tested by the pinned official detector produces `Indicated` with “processed, not necessarily authored” wording. A supported clean result and an unsupported/too-short result are visibly different. Neither produces “human-written.” False-positive and robustness measurements are published before that positive can influence the verdict.

The release is rejected if any of these occur: an issuer-name match changes a verdict; a sample-supplied root is trusted without an authoritative source; a generic AI classifier is labeled a Claude watermark detector; a remote detector uploads without per-use consent; unsupported text maps to `NoSignal`; a missing C2PA manifest maps to authenticity; or the UI says Claude authored content when the evidence only proves processing.

All malformed-input and hostile-content tests must terminate without panic. No new default network request is allowed. Native and WASM outputs must stay semantically identical for supported file inputs. Every edit to C2PA validation or trust handling requires the standing human cryptography review.

## Idempotence and Recovery

Specimen collection is append-only: preserve raw bytes, compute hashes first, and put every derived file under a clearly named `controls/` directory. Re-running inspection must not rewrite originals. If specimen terms forbid committing bytes, remove only the working copy after verifying its recorded hash and keep the test env-gated; never fabricate a replacement.

SVG support is additive and can be disabled by reverting the MIME/extension mapping and generated vector without changing existing raster verification. Trust-anchor proposals stay isolated from format support so a rejected root does not block generic SVG parsing. If a signed SVG exposes a parser panic or upstream validation ambiguity, preserve the minimal non-sensitive reproducer, keep SVG unadvertised, and file an upstream issue only with maintainer approval.

Text detector work remains behind an opt-in feature or explicit connector. If calibration misses its gate, disable verdict influence while retaining a diagnostic harness and results document. If a remote service changes or disappears, the connector returns `NotEvaluated`; existing file verification and offline builds continue to work. No database migration or destructive operation is part of this plan.

## Artifacts and Notes

The C0 inspection record should use one line per immutable specimen in `manifest.tsv`, with a header similar to:

    path\tsha256\tsurface\tmodel\tgenerated_at\tmedia_type\torigin\tcommit_status\texpected_c2pa\tnotes

Certificate fingerprints and verbose assertion inventories belong in the adjacent README so the table remains readable. Do not commit prompts or outputs containing personal, customer, proprietary, or account-identifying information.

Suggested compatibility cases, to be converted into exact tests only after specimens exist, are direct Claude web download, Claude API file output, Claude Code textual output, and one supported cloud-partner text output. These are separate cases; success on one does not imply another. “Claude Code output” is treated as source text only if Anthropic's future detector documentation explicitly includes code and the passage meets its support envelope.

The phrase “may have been processed by Claude” is central to interpretation. Paraphrase it in the product rather than quoting the support article at length, and keep the direct source URL in `docs/CLAUDE_MARKS.md` so users can inspect the current vendor claim.

## Interfaces and Dependencies

C1 should not add a runtime dependency. It uses the existing `c2pa = 0.89.3` library and extends admitted MIME hints and test vectors. If implementing SVG reveals that a dependency bump is necessary, stop, update this plan with the exact blocker and release notes, and follow the existing c2pa bump/fuzz/cryptography-review process instead of folding the bump into format support silently.

The existing file boundary remains:

    pub struct Asset<'a> {
        pub bytes: &'a [u8],
        pub media_type: Option<&'a str>,
    }

    pub fn verify_bytes(
        bytes: &[u8],
        media_type: Option<String>,
        trust_anchors_pem: Option<String>,
    ) -> String

For C1, `media_type = Some("image/svg+xml")` is the explicit SVG path. Existing `CredentialSummary` remains the public description of a verified manifest. Any added summary field must be based on a standard C2PA assertion and remain optional and vendor-neutral.

For C2, `TextMarkOutcome` above is the semantic boundary, not approval to implement a trait now. When Anthropic publishes its interface, revise this section with the exact provider type. A local implementation may become a `TextWatermarkDetector` held by Layer 2. A remote implementation must split transport from evidence mapping: the CLI/extension performs the consented request, validates and version-tags the response, and passes a typed result to the sans-I/O core. Raw detector responses are never interpolated into HTML and never promoted to `Proof`.

Any local model or SDK is opt-in, version-pinned, license-recorded, checksum-verified, and absent from the repository unless redistribution is authorized. Any network connector requires a dependency Decision Log entry, explicit endpoint configuration, bounded timeouts and payload sizes, no retries that duplicate sensitive uploads without notice, and redaction-safe logs.

---

Revision note (2026-08-11, planning session): initial plan authored from Anthropic's official content-marking article and a source-level audit of Provenance Lens. The principal revision to the roadmap is the split between specimen-gated generic C2PA file support (C1) and vendor-documentation-gated text detection (C2), with SVG identified as the immediate proven format gap.

Revision note (2026-08-11, C0 session): recorded the maintainer's C0 approval, the branch boundary, the specimen inventory contract, and the authenticated-session blocker. C0 remains open until genuine raw Claude bytes and a stripped control are hashed and inspected.

Revision note (2026-08-11, C0 SVG inspection): catalogued three maintainer-supplied flower SVG candidates as negative compatibility specimens. The pinned library supports the container when explicitly hinted, but the files contain no C2PA manifest. Added a repeatable MIME-aware inspection helper and tests without changing production behavior.

Revision note (2026-08-11, C0 labeled SVG inspection): preserved Claude's regenerated archive and labeled SVGs. Recorded that ordinary metadata and visible disclosure remain unsigned self-declarations, extended the compatibility tests, and left the positive signed-file gate open.
