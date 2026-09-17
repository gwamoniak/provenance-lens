# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**provenance-lens** is an honest AI-content provenance verifier: a four-layer Rust pipeline (C2PA cryptographic proof → watermark detection → transparency-log registry lookup → optional heuristics) compiled natively for a CLI (`lens`) and to WASM for a Manifest V3 browser extension. Verdicts come in four honest tiers — Verified, Indicated, Inconclusive, Tampered — with the founding rule baked into the types and the wording: **no provenance data ≠ authentic**. The authoritative design is **`PROVENANCE_LENS_EXECPLAN.md`** at the repo root — read it before any work; it follows the ExecPlan discipline (`PLANS.md` in `../solid-broccoli`). The current proposed implementation sequence is **`PROVENANCE_LENS_NEXT_EXECPLAN.md`**; historical feature approvals stay in their original plans. The origin document is `ai-content-verifier-proposal.md` (the maintainer's saved proposal); the plans operationalize it and their Decision Logs explain deviations.

## Current status (2026-09-16 — skimmable snapshot in `development_status.md`; the ExecPlans' Progress sections are authoritative)

**The wedge and upgrades shipped historically; approved generic C1 is now in local main at `5f1dcb6`, seven commits ahead of remote main as checked 2026-09-16.** C2PA resolves to reviewed 0.90.13, with `default-features = false` + `rust_native_crypto`, no HTTP/OpenSSL boundary change, and retained panic containment. Eleven JPEG/PNG/SVG vectors pass fresh native and real-WASM checks; SVG is explicit-hint only. The six unsigned Claude SVGs are negative controls, not positive vendor interoperability. **C1 approval does not approve W3-M or a later SDK upgrade.** W3-M implementation exists separately at `codex/w3-synthid-context` (`30e52c8`) and needs reconciliation/review; the registry has PDQ plus a log prototype but no integrated product client. N0 on `codex/n0-reproducible-baseline` repairs formatting/lint and pins the reviewed dependency/toolchain build; local checks pass, while online CI and maintainer review remain. The development plan then prioritizes detector error semantics, browser evidence lifecycle, W3-M/local evidence tools, and gated registry work. No new release or push occurred; source version remains 0.2.0. Dev machine: Windows 11, `D:\sandbox\projects\provenance-lens`; remote: https://github.com/gwamoniak/provenance-lens.

## Build

**Latest candidate (2026-09-17):** N1/N2 `976aa35` was pushed by explicit request; GitHub CI `35261458076` passed. `codex/n3-n4-local-evidence` carries those prerequisites and implements reconciled W3-M declarations, useful C4 documentation, and the local inspector/unsigned receipts/two-file comparison. Workspace 66 passed/one ignored, reduced core 43, strict lint, Node, twelve real-WASM vectors, and Chromium/Firefox offline/lifecycle tests pass. See `docs/reviews/N3_N4_REVIEW.md`. Main and the original W3 worktree remain unchanged; N3/N4 is not pushed or merged. Human review of N0/N1/N3 trust surfaces remains open. N5 and release/publication are not implicitly authorized.

    cargo fmt --all
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo run -p provenance-cli -- verify <file>      # exit: 0 verified / 10 indicated / 20 inconclusive / 30 tampered / 2 error
    wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg

**Dependencies:** Rust 1.97.1 + `wasm32-unknown-unknown`, committed `Cargo.lock`, wasm-pack 0.15.0, Node.js 24.18.0 for JavaScript checks, and the dependencies recorded in the Cargo manifests. The reviewed C2PA 0.90.13 requirement is exact-pinned on the N0 branch; any later SDK upgrade requires a separate review. Every further dependency requires an ExecPlan Decision Log entry. Default tests require no model, account, or live registry; offline builds require the tools and locked crate dependencies to be cached first. The bzh model test is explicitly ignored without a model and must be run with `--ignored` plus `PROVENANCE_LENS_BZH_ONNX` to exercise inference.

## Architecture

| Target | Directory | Role |
|--------|-----------|------|
| `provenance-core` | `crates/provenance-core/` | Verdict tiers + approved phrases (`verdict.rs`), `Asset`/`Layer`/`LayerFinding`/`Pipeline`/`combine()` (`pipeline.rs`), the four layers (`layers/`). Sans-IO. |
| `provenance-cli` (`lens`) | `crates/provenance-cli/` | std-only CLI; owns all file I/O. |
| `provenance-wasm` | `crates/provenance-wasm/` | Thin wasm-bindgen wrapper: `verify_bytes(bytes, mime?, trust_anchors_pem?) -> JSON`. Shared core report. |
| extension | `extension/` | MV3, plain JS; consumes the wasm-pack output in `extension/pkg/` (gitignored). |

## Conventions (enforced; rationale in the ExecPlan Decision Log)

- **Honest verdicts:** `Inconclusive` never implies authenticity, anywhere — code, UI, docs, logs. The four approved phrases in `crates/provenance-core/src/verdict.rs` are canonical and must stay character-identical in `.claude/skills/verdict-language/SKILL.md`, `extension/popup/popup.html`, and `README.md`; tests pin them.
- **Evidentiary typing:** only Layer 1 (c2pa) may return `Proof`; heuristics at most `Indication`; a layer that didn't run returns `NotEvaluated { reason }`, never `NoSignal`. Combination precedence: Tampered > Verified > Indicated > Inconclusive.
- **Sans-IO core:** `provenance-core` never opens files/sockets; transports are injected; CLI and WASM wrappers own I/O.
- **Human sign-off rule:** signature-validation code (C2PA validation path, trust lists, inclusion proofs) merges only with a human cryptography reviewer's sign-off recorded in the ExecPlan Decision Log. Agents prepare; humans approve.
- **Extension surface:** permissions are `contextMenus` + `activeTab` + `storage` + `scripting`, with host access ONLY via `optional_host_permissions` granted per site at runtime by explicit user gesture (U7 Decision Log, maintainer-approved); no install-time host permissions, no remote code, no analytics; the extension never renders a verdict it didn't compute.
- **Privacy:** image bytes never leave the device — parsing and hashing happen locally; post-wedge network layers (2–3) may send perceptual hashes only, with explicit user consent.
- **Blockchain:** optional anchoring of registry checkpoints only, default off; no verdict path may require it.
- **ExecPlan discipline:** living sections updated at every stopping point; commit at milestone boundaries with the milestone name in the message.

## Testing

`cargo test --workspace`; tests live beside the code (`#[cfg(test)]`) until vector suites justify `tests/`. Philosophy: **known-answer vectors** — Layer 1 is validated against C2PA public test files and self-generated c2patool vectors (`tests/vectors/` from M1, each with a recorded expected verdict); native/WASM **parity** over the whole vector set from M2; **fuzz targets** for every parser that touches attacker-controlled bytes; wording and precedence are pinned by tests so drift fails CI.

## Agents & skills

Nine agents in `.claude/agents/`: `lens-rust-core`, `lens-wasm`, `lens-extension` (implementers), `lens-security-reviewer` (read-only; prepares the human sign-off packet), `lens-registry` (gated Layer 3), `lens-qa`, `lens-research`, `lens-docs`, `lens-release` (M4 shipping; prepares but never publishes without the maintainer's go). All defer to `PROVENANCE_LENS_EXECPLAN.md` as the single source of truth. Nine skill packs in `.claude/skills/`: `c2pa-spec` (before touching Layer 1), `verdict-language` (before writing ANY user-facing string), `rust-quality` (before writing any Rust), `security-checklist` (every trust-decision review), `test-vectors` (before adding corpus assets), `watermark-detection`, `provenance-registry`, `wasm-packaging`, `webextension-mv3`.

## Sibling projects (same sandbox, same maintainer)

- `../../cpp/solid-broccoli` — Qt6/QML tricorder instrument; origin of the house conventions reused here (ExecPlan discipline and `PLANS.md`, gated milestones, bare-machine rule, sans-IO, docs-stay-true) and of the agent-team pattern.
- `../../cpp/Coffee_Dispenser` — Qt6/QML coffee-machine HMI, same discipline.
- `../../llm/dobromir` — pre-alpha LLM/VLM project with different laws; read its `CLAUDE.md` before doing anything there.
