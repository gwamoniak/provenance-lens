# Claude content-mark specimens

This directory is the C0 compatibility inventory for `PROVENANCE_LENS_CLAUDE_MARKS_EXECPLAN.md`. It is not a claim that every Claude output is marked, and it is not part of the self-generated C2PA conformance corpus in the parent directory.

## Current status

C0 was approved by the maintainer on 2026-08-11 with “go with C0”. Genuine specimen collection is in progress on branch `codex/claude-c0-specimens`. The maintainer first supplied three SVG candidates: daisy, tulip, and sunflower. Claude then regenerated them with an editable metadata tag and visible AI-generated label; the second delivery arrived in `files2.zip`, which is preserved with its extracted entries. Their byte-identical corpus copies and hashes are recorded below. The exact Claude surface, model identifier, and generation time still require maintainer confirmation, so those catalogue fields remain explicitly unknown rather than inferred.

The available Claude web session required the maintainer to sign in. Claude Code 2.1.220 reported an authenticated first-party `claude.ai` account, but the organization policy disabled subscription access from Claude Code and required an API key; no key was requested or inspected. No substitute, fabricated Anthropic assertion, converted download, or unrelated public file is accepted as a genuine positive specimen.

The local machine does not currently have `c2patool` installed. The production `lens` CLI and the pinned `c2pa` Rust library remain the required inspection path. `c2patool --info` and `c2patool --certs` are optional independent cross-checks if the maintainer separately approves installing the tool.

## Directory contract

Raw files must be saved byte-for-byte under `raw/` before any other program opens or converts them. Compute SHA-256 immediately and add the result to `manifest.tsv`. Use only benign prompts and outputs that can safely be published. Do not commit account identifiers, private prompts, personal data, customer data, or secrets.

Derived files belong under `controls/` and must name their raw parent in both the filename and `manifest.tsv`. C0 calls for two controls when the source format permits them:

* a normal resave or conversion that strips the embedded manifest; and
* a post-signing content edit intended to preserve the container while breaking the C2PA hard binding.

Never edit a raw file in place. Every transformation must be scripted or recorded with the exact tool, version, command, and source hash. Controls are observations about signal loss or tamper handling, never evidence about Claude authorship.

If Anthropic's terms, the specimen's content, or repository-size constraints prevent committing bytes, keep the specimen outside the repository in the directory named by `PROVENANCE_LENS_CLAUDE_FIXTURES`. Record its SHA-256 and collection facts in `manifest.tsv`; set `commit_status` to `external`; and ensure compatibility tests skip with an explicit reason when the environment variable is absent.

## Required collection facts

Each row in `manifest.tsv` records the immutable file identity and collection context. The adjacent inspection record below expands the C2PA-specific facts that do not fit cleanly in the table.

For every raw file, record:

* Claude surface and exact model identifier shown by that surface;
* generation timestamp and whether the output was downloaded directly or copied;
* original filename and HTTP content type when visible;
* SHA-256 before any resave, screenshot, metadata inspection, or conversion;
* whether the bytes may be committed; and
* expected C2PA state based only on an actual inspection, never on the filename or vendor name.

For text, preserve the exact UTF-8 bytes externally and record the SHA-256, surface, model, timestamp, and copy method. Text rows must use `expected_c2pa=not-applicable`. Until Anthropic publishes a third-party detector contract, no row may claim that the embedded text watermark was detected or absent.

## Inspection procedure

From the repository root, first verify through the shipped product path:

    cargo run -p provenance-cli -- verify --json --trust-anchors extension/trust/anchors.pem <raw-file>

Record the verdict, C2PA finding detail, issuer, claim generator, signing time, declared `digitalSourceType`, and source-type note exactly as emitted. The result describes the verified credential's claims; it does not prove that Claude originated every idea or pixel.

When an approved `c2patool` installation is available, independently inspect without rewriting the file:

    c2patool <raw-file> --info
    c2patool <raw-file> --certs

Record the tool version, validation statuses, active-manifest label, assertion labels, certificate subjects/issuers, and SHA-256 fingerprints. A certificate embedded in the specimen cannot authorize its own trust. Any proposed trust-anchor change requires an authoritative publication source and the standing human cryptography sign-off.

## Inspection record

### Shared result for the three flower SVG candidates

The originals were supplied at `docs/flower-1-daisy.svg`, `docs/flower-2-tulip.svg`, and `docs/flower-3-sunflower.svg`. Each was hashed before parsing. The files were then moved into this directory's `raw/` corpus after byte-for-byte hash verification; the `docs/` staging copies were removed to avoid committing duplicates. All three parse as well-formed XML with root element `svg`, namespace `http://www.w3.org/2000/svg`, and `viewBox="0 0 100 100"`. A case-insensitive byte search found no `c2pa`, `manifest`, `jumbf`, `metadata`, `xmp`, `anthropic`, `claude`, `claim_generator`, or `trainedAlgorithmicMedia` string.

The shipped CLI was run unchanged with the production trust bundle:

    cargo run -q -p provenance-cli -- verify --json --trust-anchors extension/trust/anchors.pem crates/provenance-core/tests/vectors/claude/raw/flower-1-daisy.svg
    cargo run -q -p provenance-cli -- verify --json --trust-anchors extension/trust/anchors.pem crates/provenance-core/tests/vectors/claude/raw/flower-2-tulip.svg
    cargo run -q -p provenance-cli -- verify --json --trust-anchors extension/trust/anchors.pem crates/provenance-core/tests/vectors/claude/raw/flower-3-sunflower.svg

During C0, before the C1 SVG change, each command returned exit 20 and `Inconclusive`. The C2PA finding was `not_evaluated` with detail `unrecognized media type; cannot locate a manifest store`, proving that the then-current CLI did not admit SVG by filename or sniffing.

The pinned library was then exercised through the developer-only `inspect_with_mime` example, passing `image/svg+xml` explicitly:

    cargo run -q -p provenance-core --example inspect_with_mime -- crates/provenance-core/tests/vectors/claude/raw/flower-1-daisy.svg image/svg+xml extension/trust/anchors.pem
    cargo run -q -p provenance-core --example inspect_with_mime -- crates/provenance-core/tests/vectors/claude/raw/flower-2-tulip.svg image/svg+xml extension/trust/anchors.pem
    cargo run -q -p provenance-core --example inspect_with_mime -- crates/provenance-core/tests/vectors/claude/raw/flower-3-sunflower.svg image/svg+xml extension/trust/anchors.pem

The pinned C2PA reader successfully evaluated the SVG container in all three cases and returned `no_signal`: no manifest store is present. The combined verdict remained `Inconclusive`. There is therefore no active-manifest label, assertion inventory, signer certificate, certificate fingerprint, or trust-chain outcome to record for these files. Adding SVG admission in C1 will change the C2PA sub-state from `not_evaluated` to `no_signal` for these exact bytes, but it cannot turn them into cryptographic proof.

Because the candidates contain no manifest, creating a “metadata-stripped” derivative would be meaningless: there is no metadata signal to strip. They are retained as genuine negative compatibility specimens. C0's positive acceptance gate remains open until a directly exported Claude file with a manifest is supplied and a stripped control can be derived from it.

Individual immutable facts:

* `raw/flower-1-daisy.svg`: 900 bytes; SHA-256 `83154a25b465f96e229554f7434679a37c16a2322b111992742f64e5105a2230`.
* `raw/flower-2-tulip.svg`: 612 bytes; SHA-256 `c1e4cd83a949dd4f1fbd2f38a8f33b61eece30818359054231b1872c7fdba5f2`.
* `raw/flower-3-sunflower.svg`: 1,154 bytes; SHA-256 `269b31a4325a351293fc650bb521bd2423d995854d52347b060b01c4712e0e3e`.

### Regenerated self-labeled batch

The maintainer supplied Claude's explanation: “Updated — each SVG now has a small gray ‘AI-generated’ label in the bottom corner plus an `<metadata>` tag.” The acquisition envelope `raw/files2.zip` is 1,533 bytes with SHA-256 `ea3cd47f403919b372b31a8aa35b34b66331fe84916550340502d601ed6ae9c1`. Its three entries match the separately supplied regenerated SVG bytes exactly.

Relative to the first batch, each regenerated file adds only these two lines:

    <metadata>AI-generated image</metadata>
    <text x="50" y="98" font-family="sans-serif" font-size="6" fill="#999999" text-anchor="middle">AI-generated</text>

The first line is ordinary, unsigned SVG metadata; the second is ordinary visible artwork. Either can be added, removed, or changed by anyone. Neither is a C2PA manifest, signature, hard binding, certificate chain, or embedded text-watermark detector result. The files remain valuable negative specimens because they pin the distinction between a producer's editable self-declaration and cryptographic provenance.

The C0 CLI again returned exit 20, `Inconclusive`, and C2PA `not_evaluated` because it did not yet admit SVG. The pinned reader with explicit `image/svg+xml` evaluated each container and returned `no_signal`; no credentials were present. Individual facts:

* `raw/flower-1-daisy-labeled.svg`: 1,059 bytes; SHA-256 `eafd1fed7428dc9d725ef1c54a0f8ccba8361786e75485442975b4aa143674a6`.
* `raw/flower-2-tulip-labeled.svg`: 771 bytes; SHA-256 `fa1d0c60522359b7afd5d1d6222fc86286a1c3a084b3025f99ca61edff6a1193`.
* `raw/flower-3-sunflower-labeled.svg`: 1,313 bytes; SHA-256 `369c0aeedb76121baaca70412e3fc3290a442b268718c62f36b389ce90bb33b7`.

### Claude self-report and interpretation

After inspection, the maintainer supplied a Claude reply saying that these SVGs contain no C2PA manifest, that Claude could not inject a genuine signing credential in that interaction, and that the visible disclosure and plain `<metadata>` value are only lightweight non-cryptographic markers. Those statements accurately describe the preserved bytes and agree with both Provenance Lens inspection paths.

Two broader parts of the reply are not accepted as authoritative product or SDK facts:

* C2PA creation and C2PA trust are separate questions. Development credentials can create a signed manifest that validates cryptographically but is reported as untrusted. A conforming production generator needs an appropriate signing certificate whose chain reaches a trust anchor accepted by the verifier. Therefore “not trusted by the production list” is not the same as “cannot be signed,” and a locally trusted test certificate must never be described as Anthropic provenance.
* SVG embedding is specified by C2PA and is supported by the project's pinned `c2pa` library for reading and building. Current upstream `c2pa-rs` also lists `svg` / `image/svg+xml` as supported. The relevant project gap is automatic SVG admission in the shipped CLI, not an absent or merely hypothetical SVG format implementation.

Authoritative references are Anthropic's [content-marking support article](https://support.claude.com/en/articles/16266773-how-claude-marks-ai-generated-content), the C2PA specification's [SVG embedding rules](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html#_embedding_manifests_into_svg), upstream `c2pa-rs` [supported formats](https://github.com/contentauth/c2pa-rs/blob/main/docs/supported-formats.md), and the CAI guidance for [test certificates](https://opensource.contentauthenticity.org/docs/signing/test-certs/) and [production signing certificates](https://opensource.contentauthenticity.org/docs/signing/get-cert/). Anthropic's article says supported output types can include SVG while availability varies by platform and feature. The Claude reply is therefore recorded as surface-specific collection context, not as a contradiction of Anthropic's platform documentation.

Converting these files to PNG and signing the result locally would test PNG ingestion, manifest construction, and trust policy. It would prove only that the local signer processed and declared the converted file; it would not prove that Anthropic signed it or that Claude authored the SVG. Such a file may be created later as a clearly labelled generic C1 conformance vector, but it cannot satisfy C0's genuine positive Claude-specimen gate.

### C1 reinspection after explicit SVG admission

On 2026-08-13, branch `codex/claude-c1-svg-support` upgraded the pinned C2PA SDK to 0.90.13 and taught the CLI to pass `image/svg+xml` for `.svg` filenames. Re-running all six raw SVG files through the production CLI now returns exit 20, `Inconclusive`, and C2PA `no_signal`, matching the earlier explicit-MIME inspection. The product now distinguishes “looked and found no manifest” from “did not recognize the container.” It does not strengthen the evidence in these files.

The separate generated conformance corpus now includes `plain.svg`, `valid_signed.svg`, and `content_tampered.svg`, signed by the repository's ephemeral test CA. With that test CA explicitly injected, the signed SVG is `Verified`, its post-signing artwork edit is `Tampered`, and the unsigned SVG is `Inconclusive`; without the test CA, the signed SVG is never `Verified`. These vectors prove generic SVG plumbing only. They do not count as Anthropic-signed specimens, and C0's genuine positive-file gate remains open.

## Validation

The catalogue was parsed as ten tab-separated columns, every `raw/` filename had exactly one row, and every recorded SHA-256 matched the working bytes. The repository's `.gitattributes` marks only this raw directory `-text`; after re-staging, every index blob ID exactly matched `git hash-object --no-filters` for its source, proving Git did not normalize line endings. `cargo fmt -p provenance-core -- --check` passed. Targeted clippy for `claude_svg_compat` and `inspect_with_mime` passed with `-D warnings`. `cargo test -p provenance-core` passed all 41 tests, including the three SVG compatibility tests and the existing vector/wording/certificate-policy suites.

`cargo clippy -p provenance-core --all-targets -- -D warnings` still fails on pre-existing `unused_mut` warnings in `examples/calibrate.rs:36` and `examples/measure_fpr.rs:37`. C0 does not edit those unrelated calibration tools. The targeted C0 clippy surfaces are clean, and the complete core test run merely prints those two existing warnings while passing.
