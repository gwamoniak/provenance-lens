# Development status — Provenance Lens

Snapshot dashboard, last updated **2026-07-10** (M0 scaffold stage). This file is a *derived view* for humans skimming the repo: the authoritative, always-current record is the `Progress` / `Decision Log` / `Surprises & Discoveries` sections of `PROVENANCE_LENS_EXECPLAN.md`. Update this dashboard at milestone boundaries; if it ever disagrees with the ExecPlan, the ExecPlan wins.

## Where we are, in one paragraph

The repository is a complete, conventions-following scaffold of the proposal in `ai-content-verifier-proposal.md`: workspace, verdict model, pipeline spine, CLI, WASM wrapper, extension skeleton, nine agents, nine skills, and a self-contained ExecPlan. Nothing has been compiled yet — the machine had no Rust toolchain when the scaffold was written — so the honest status of every line of Rust is "authored, never validated". The verdict model (the four tiers, their pinned wording, the conservative combination rule) is the most finished thing in the repo, which is deliberate: it is the product's spine and every later layer plugs into it.

## Proposal deliverables → current state

| Proposal deliverable | Here | State |
|---|---|---|
| `verify-core` (Rust crate, native + WASM + C API) | `crates/provenance-core` | Scaffold: verdict model + pipeline + 4 stub layers, unit tests written, **never compiled**. C API: post-wedge, not started. |
| `verify-wasm` (JS/TS bindings, npm) | `crates/provenance-wasm` | Scaffold: `verify_bytes` → JSON, tests written, never compiled. npm packaging: post-wedge, not started. |
| `provenance-lens` (WebExtension MV3) | `extension/` | Skeleton: loads unpacked, shows tiers, honestly reports no engine. Functional from M3. |
| `verify-registry` (transparency-log service) | — | Not started; GATED (M6). Design knowledge captured in the `provenance-registry` skill. |
| CLI (`verify image.png`) | `crates/provenance-cli` → `lens` binary | Scaffold: `verify`/`tiers` verbs, verdict exit codes, never compiled. |

## Pipeline layers

| Layer | State | Gate |
|---|---|---|
| 1 — C2PA proof | Honest stub (`NotEvaluated`). **M1 is next**: real validation via the `c2pa` crate. | None — needs toolchain + test vectors + human crypto sign-off at merge. |
| 2 — Watermark | Honest stub. `WatermarkDetector` trait specified (skill + M5). | GATED: a runnable, licensed vendor detector. |
| 3 — Registry | Honest stub. PDQ/pHash + transparency-log design captured in skill. | GATED: a deployed log endpoint. |
| 4 — Heuristics | Honest stub. | OPTIONAL; may never ship (needs published false-positive rates). |

Verdict model and combination rule (Tampered > Verified > Indicated > Inconclusive; only Layer 1 may prove; "no data ≠ authentic" pinned by tests): **implemented**, pending first compile.

## Agents and skills vs the proposal

All eight proposed agents exist (`lens-rust-core`, `lens-wasm`, `lens-extension`, `lens-security-reviewer`, `lens-registry`, `lens-qa`, `lens-research`, `lens-docs`), plus one the proposal implied but never assigned an owner: **`lens-release`** — the proposal placed store submission inside the Extension Agent and community/RFC work inside Docs & Community, leaving nobody owning M4's actual shipping mechanics (versioning, reproducible/signed builds, store checklists, the post-wedge npm package). All eight proposed skills exist, plus **`test-vectors`** — the proposal's QA corpus ("signed/stripped/tampered/clean media") needs concrete c2patool generation know-how that no proposed skill carried; it is the practical unblocker for M1.

## Blockers, in priority order

1. **No Rust toolchain on this machine** (`cargo`, `rustc`, `wasm-pack` absent). Everything is unvalidated until it lands; install per the ExecPlan's Concrete Steps, then the first `cargo test --workspace` doubles as the scaffold's code review. Closes M0.
2. **No human cryptography reviewer identified.** The proposal's non-negotiable rule — agents draft, a human signs off on signature-validation code — has no named human yet. M1 cannot *merge* without one. Maintainer decision required.
3. **No GitHub remote / no `gh` CLI.** Local repo only; publishing is one `gh repo create` away once `gh` is installed and authenticated.

## Next actions

1. Install the toolchain; run fmt/clippy/test; fix scaffold breakage; check off M0.
2. Name the human cryptography reviewer (maintainer).
3. Generate the M1 vector corpus per the `test-vectors` skill; then implement Layer 1 (M1).
4. WASM parity + size budget (M2), extension end-to-end (M3), ship the wedge (M4, owned by `lens-release`).

## Risks being tracked

- The `c2pa` crate's wasm32 build (feature flags, crypto backend) is M2's main known unknown — discover early, record in the Decision Log.
- c2patool's test certificates are not on the production trust list; M1 must define the test-time trust-anchor policy explicitly (see `test-vectors` skill) or Verified vectors will "fail" for the wrong reason.
- WASM artifact size vs the placeholder ≤ 4 MB gzipped budget.
- Layer 2/3 gates may stay closed for a long time — acceptable by design (honest `NotEvaluated`), but the README must never imply otherwise.
- Store review friction at M4 (permission justifications, privacy policy) — mitigated by the minimal-permissions decision.
