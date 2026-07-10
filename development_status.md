# Development status — Provenance Lens

**TL;DR (2026-07-10): the wedge is DONE — M0–M4 all closed. Layer 1 verifies real Content Credentials (maintainer-signed-off crypto); the extension passed the maintainer's browser smoke; the official C2PA trust list ships as data; the installable zip is built (`dist/provenance-lens-0.1.0.zip`, sha256 in the ExecPlan). 20/20 tests green. Remaining: maintainer submits to the store; post-wedge hardening backlog; gated M5–M7.**

Snapshot dashboard, last updated **2026-07-10, post-M0** (M0 closed; M1 is next). This file is a *derived view* for humans skimming the repo: the authoritative, always-current record is the `Progress` / `Decision Log` / `Surprises & Discoveries` sections of `PROVENANCE_LENS_EXECPLAN.md`. Update this dashboard at milestone boundaries; if it ever disagrees with the ExecPlan, the ExecPlan wins.

## Where we are, in one paragraph

**Milestone 0 is closed and green.** The repository is a complete, conventions-following implementation of the proposal's foundation: workspace, verdict model, pipeline spine, CLI, WASM wrapper, extension skeleton, nine agents, nine skills, and a self-contained ExecPlan — now with the toolchain installed (rustup stable + wasm32 target + wasm-pack 0.15.0), clippy clean at `-D warnings`, **9/9 tests passing**, the WASM crate type-checking on `wasm32-unknown-unknown`, and the CLI acceptance transcript recorded in the ExecPlan (`lens tiers` prints the four approved phrases; `lens verify` honestly reports Inconclusive with exit code 20). The verdict model — the four tiers, their pinned wording, the conservative combination rule — is fully implemented and test-pinned; it is the product's spine and every later layer plugs into it. Next: M1, real C2PA validation.

## Proposal deliverables → current state

| Proposal deliverable | Here | State |
|---|---|---|
| `verify-core` (Rust crate, native + WASM + C API) | `crates/provenance-core` | Verdict model + pipeline + 4 honest stub layers; **builds, 7/7 tests green**, clippy clean. Layer 1 becomes real in M1. C API: post-wedge, not started. |
| `verify-wasm` (JS/TS bindings, npm) | `crates/provenance-wasm` | `verify_bytes` → JSON; **builds natively (2/2 tests) and type-checks on wasm32**. wasm-pack artifact + parity suite land in M2. npm: post-wedge. |
| `provenance-lens` (WebExtension MV3) | `extension/` | Skeleton: loads unpacked, shows tiers, honestly reports no engine. Functional from M3. |
| `verify-registry` (transparency-log service) | — | Not started; GATED (M6). Design knowledge captured in the `provenance-registry` skill. |
| CLI (`verify image.png`) | `crates/provenance-cli` → `lens` binary | **Working**: `verify`/`tiers` verbs, verdict exit codes (acceptance transcript in the ExecPlan). Verifies nothing yet — all layers honestly not-evaluated until M1. |

## Pipeline layers

| Layer | State | Gate |
|---|---|---|
| 1 — C2PA proof | **Done (M1, signed off & merged)**: c2pa 0.89.2 validation, Trusted→Proof / unanchored-or-invalid→TamperEvidence / absent→NoSignal / unparseable→NotEvaluated; vector corpus + 6 integration tests. | None. Follow-ups recorded: cargo-fuzz target, cert-policy tests, production trust list (M4). |
| 2 — Watermark | Honest stub. `WatermarkDetector` trait specified (skill + M5). | GATED: a runnable, licensed vendor detector. |
| 3 — Registry | Honest stub. PDQ/pHash + transparency-log design captured in skill. | GATED: a deployed log endpoint. |
| 4 — Heuristics | Honest stub. | OPTIONAL; may never ship (needs published false-positive rates). |

Verdict model and combination rule (Tampered > Verified > Indicated > Inconclusive; only Layer 1 may prove; "no data ≠ authentic" pinned by tests): **implemented and green**.

## Agents and skills vs the proposal

All eight proposed agents exist (`lens-rust-core`, `lens-wasm`, `lens-extension`, `lens-security-reviewer`, `lens-registry`, `lens-qa`, `lens-research`, `lens-docs`), plus one the proposal implied but never assigned an owner: **`lens-release`** — the proposal placed store submission inside the Extension Agent and community/RFC work inside Docs & Community, leaving nobody owning M4's actual shipping mechanics (versioning, reproducible/signed builds, store checklists, the post-wedge npm package). All eight proposed skills exist, plus **`test-vectors`** — the proposal's QA corpus ("signed/stripped/tampered/clean media") needs concrete c2patool generation know-how that no proposed skill carried; it is the practical unblocker for M1.

## Blockers, in priority order

1. ~~No Rust toolchain~~ — **resolved 2026-07-10**: rustup + stable + wasm32 + wasm-pack 0.15.0 installed; suite green.
2. ~~No human cryptography reviewer identified~~ — **resolved 2026-07-10**: the maintainer (gwamoniak) signs off on signature-validation merges (Decision Log entry in the ExecPlan).
3. **No GitHub remote / no `gh` CLI.** Local repo only; publishing is `brew install gh && gh auth login && gh repo create provenance-lens --private --source . --push` from the repo root, when the maintainer chooses.

## Next actions

1. **Maintainer: submit to the Chrome Web Store** — everything needed is in `docs/STORE_LISTING.md` (copy, permission justifications, privacy policy, pre-submission checklist); the zip is `dist/provenance-lens-0.1.0.zip` (rebuild anytime: `sh scripts/package_extension.sh`).
2. **Optional next**: take items off the hardening backlog (cargo-fuzz, cert-policy tests), make the GitHub repo public when ready (https://github.com/gwamoniak/provenance-lens, currently private), or open a gated milestone when its gate opens (`lens-research` tracks detector and registry availability).

## Risks being tracked

- The `c2pa` crate's wasm32 build (feature flags, crypto backend) is M2's main known unknown — discover early, record in the Decision Log.
- c2patool's test certificates are not on the production trust list; M1 must define the test-time trust-anchor policy explicitly (see `test-vectors` skill) or Verified vectors will "fail" for the wrong reason.
- WASM artifact size vs the placeholder ≤ 4 MB gzipped budget.
- Layer 2/3 gates may stay closed for a long time — acceptable by design (honest `NotEvaluated`), but the README must never imply otherwise.
- Store review friction at M4 (permission justifications, privacy policy) — mitigated by the minimal-permissions decision.
