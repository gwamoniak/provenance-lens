---
name: c2pa-spec
description: Working knowledge of the C2PA content-provenance standard for implementing and reviewing Layer 1 — manifest structure, claims, assertions, hard bindings, signature validation, trust lists, and how each maps to this project's verdict tiers. Load before touching crates/provenance-core/src/layers/c2pa.rs or reviewing its diffs.
---

# C2PA for Provenance Lens

C2PA (Coalition for Content Provenance and Authenticity) is the open standard for cryptographically signed media provenance — "Content Credentials". Canonical spec: https://c2pa.org/specifications/ (verify the current version before citing details; this summary was written against v2.x in mid-2026).

## The object model, bottom up

- **Manifest store** — a JUMBF (JPEG universal metadata box format) container embedded in the asset (JPEG APP11 segments, PNG `caBX` chunk, similar per format). Holds one or more manifests: the **active manifest** (most recent) plus **ingredient** manifests from prior editing steps, forming the provenance chain.
- **Claim** — the heart of a manifest: a CBOR structure listing the assertions it covers (as hashed references) and identifying the claim generator (the software that produced it).
- **Assertions** — the individual statements: `c2pa.hash.data` (the hard binding), `c2pa.actions` (created/edited/how — including whether generative AI was used, e.g. digitalSourceType `trainedAlgorithmicMedia`), `c2pa.ingredient` (links to parent manifests), thumbnails, EXIF, etc.
- **Claim signature** — a COSE (CBOR Object Signing and Encryption) signature over the claim, made with an X.509 certificate. Trust comes from validating the cert chain against a trust list (the C2PA conformance program publishes one; validators may configure their own).
- **Hard binding** — the content hash assertion tying the manifest to these exact bytes (exclusion ranges cover the manifest's own bytes). This is what makes a manifest non-transplantable: paste it into another image and the hash check fails.

## Validation, in order (what the `c2pa` crate's Reader does)

1. Locate and parse the manifest store (malformed → tamper-relevant, distinguish from absent).
2. Validate the claim signature (COSE) and the certificate chain against the trust list.
3. Verify each hashed assertion reference against the assertions present.
4. Verify the hard binding against the asset bytes.
5. Recurse into ingredients for chain validation.

## Mapping to this project's types (normative)

- Steps 1–5 all pass, chain trusted → `LayerFinding::Proof { issuer }` (issuer from the signing cert). → Verdict `Verified`.
- No manifest store found at all → `NoSignal`. Most web images have none; this is the common case and is NOT evidence of anything. → contributes to `Inconclusive`.
- Manifest present but signature invalid, cert untrusted/self-signed, hash mismatch, truncated store → `TamperEvidence { detail }`. → Verdict `Tampered`. This tier is the wedge: platforms that strip or mangle credentials produce it at scale.
- A **valid** manifest whose assertions declare generative-AI provenance is still `Proof` — the verdict says the provenance is verified, and the report's detail says what the provenance asserts. Do not conflate "verified" with "not AI".

## Implementation notes for this repo

- Use the `c2pa` crate (CAI Rust SDK — the reference implementation; c2patool is its CLI, useful for generating test vectors). Pin the version in the workspace and record upgrades in the ExecPlan Decision Log; the API moved fast historically.
- Only this layer may return `Proof` (project rule, enforced in review).
- Manifest parsing consumes attacker-controlled bytes: no panics, no unbounded allocations; fuzz target required (see lens-qa).
- Signature-validation code changes require human cryptography-reviewer sign-off before merge (project rule from the proposal; lens-security-reviewer prepares the packet).
- Test vectors: the C2PA public test files repository plus self-generated vectors via c2patool, stored under `tests/vectors/` with expected verdicts.
