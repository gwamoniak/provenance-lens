---
name: security-checklist
description: The adversarial review checklist — parser fuzzing, cert-chain edge cases, downgrade attacks, badge/UI injection, supply-chain hygiene. Load when reviewing any diff that touches trust decisions, parsing, or dependencies (lens-security-reviewer loads this always).
---

# Security checklist

The adversary's goal is to make the tool say something stronger or weaker than the evidence: a forged Verified, a suppressed Tampered, or an Inconclusive that reads as "clean". Review against that goal.

## Signature validation (Layer 1)

- Chain validated to a trust anchor, not merely parsed? Self-signed and unknown-CA manifests must land in `TamperEvidence`, never `Proof`.
- Hard binding (content hash) checked against the actual bytes, including exclusion-range handling?
- Algorithm agility: does the code accept deprecated/weak signature algorithms the spec still encodes? Reject-list them explicitly.
- Time handling: expired certs, timestamps from the future — decide and test the policy, don't inherit a library default silently.
- **Downgrade attacks**: can an attacker strip the manifest to demote Tampered to Inconclusive? Yes — by design that's undetectable for Layer 1 alone; never claim otherwise in wording. But a *partially* stripped or truncated store must be Tampered, not NoSignal — test the truncation cases explicitly.
- Trust-list poisoning: how does the trust list get in, who can update it, is it pinned/signed? Any "fetch trust list over HTTP at runtime" is a finding.

## Parsers

- Fuzz target exists for every parser of untrusted bytes (cargo fuzz; seeds from tests/vectors/). New parser without one = finding.
- Length fields: checked arithmetic, capped allocations, no attacker-sized `Vec::with_capacity`.
- Deep-recursion structures (nested JUMBF boxes, ingredient chains): explicit depth limit.

## Extension surface

- Anything derived from page content or manifest content rendered via `innerHTML` = finding (XSS in badge/report rendering — manifests carry attacker-authored strings like titles and issuer names). `textContent` only.
- Permission creep, remote-code paths, requests not initiated by explicit user action.
- Message passing: does the popup/content script trust `sender` without checking?

## Supply chain

- Dependencies pinned; new ones justified in the Decision Log with a transitive-tree look (`cargo tree`).
- `cargo audit` clean (wire into CI when CI exists).
- Release artifacts built reproducibly from a tagged commit; post-wedge: signed releases.

## The human sign-off rule

Signature-validation diffs merge only with a human cryptography reviewer's dated sign-off in the ExecPlan Decision Log. The reviewer agent prepares the packet (diff summary, changed validation logic, covering vectors, findings) and states plainly that sign-off is pending. No agent may approve these diffs.
