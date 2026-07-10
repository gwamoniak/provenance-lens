---
name: provenance-registry
description: Perceptual hashing (pHash/PDQ), transparency-log append/verify flow, and anchoring cadence for Layer 3. Load for registry design or implementation work (GATED milestone — design may proceed, implementation needs a deployed log).
---

# Provenance registry — Layer 3

Layer 3 answers: "was this asset registered as AI-generated at creation time, even though its metadata was stripped on upload?" The proposal's answer: hash locally, look up in registries — C2PA's planned recovery services, Numbers Protocol, and/or this project's own transparency log (`verify-registry`, a self-hostable service, post-wedge deliverable).

## Perceptual hashing

Cryptographic hashes break on any re-encode, so lookup uses a perceptual hash: **PDQ** (Meta's open standard, 256-bit, Hamming-distance matching — the default choice) or pHash. Rules:

- The exact algorithm, preprocessing (resize/colorspace), and match threshold are specified in the ExecPlan before implementation and never changed silently — the threshold IS the false-positive rate.
- Measure the false-match rate on lens-qa's clean corpus before the layer may contribute verdicts; a registry hit is `Indication`, and the wording shows the match basis: "matches asset registered by X on date Y".
- A perceptual near-match is weaker evidence than an exact match — if both tiers exist, the report distinguishes them.

## Transparency log (Sigstore/Rekor model)

Append-only Merkle log: generators (or their infrastructure) append `(hash, registrant, timestamp, signature)` entries; anyone can verify an inclusion proof against a signed checkpoint (tree head), and consistency proofs guarantee history is never rewritten.

- Client verifies inclusion proofs locally — trusting the log operator's word without the proof is a finding. Inclusion-proof verification is trust-decision code: security review + the human sign-off rule apply.
- **Anchoring** (from the proposal, settled): checkpoints may optionally be anchored to a public blockchain as external witnesses, cadence ~daily. Anchoring is a checkpoint-witness mechanism, NOT a dependency — default off, and no verdict path may require a chain lookup. Layers 1–2 work fully offline regardless.

## Privacy

Lookups reveal what the user is checking. Only perceptual hashes leave the device (never bytes), only with user consent; document the leak plainly in user-facing docs; design for self-hosted registries so orgs can keep lookups internal. Batch/pad lookups if traffic analysis becomes a credible concern (post-wedge).

## Failure semantics

Registry unreachable → `NotEvaluated { reason }` (shown honestly), never silently `NoSignal`. A miss in a sparsely-populated log means almost nothing — wording must not imply "not registered ⇒ not AI".
