# Claude content-mark specimens

This directory is the C0 compatibility inventory for `PROVENANCE_LENS_CLAUDE_MARKS_EXECPLAN.md`. It is not a claim that every Claude output is marked, and it is not part of the self-generated C2PA conformance corpus in the parent directory.

## Current status

C0 was approved by the maintainer on 2026-08-11 with “go with C0”. Genuine specimen collection is in progress on branch `codex/claude-c0-specimens`. At the time this file was created, the available Claude web session required the maintainer to sign in. Claude Code 2.1.220 reported an authenticated first-party `claude.ai` account, but the organization policy disabled subscription access from Claude Code and required an API key; no key was requested or inspected. No substitute, fabricated Anthropic assertion, converted download, or unrelated public file is accepted as a genuine specimen.

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

No genuine specimen has been recorded yet. Replace this sentence with one subsection per immutable raw specimen after collection. Each subsection must link to its `manifest.tsv` row and include the exact command transcript or a checked-in, redaction-safe transcript file.
