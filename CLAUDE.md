# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**provenance-lens** is an honest AI-content provenance verifier: a four-layer Rust pipeline (C2PA cryptographic proof → watermark detection → transparency-log registry lookup → optional heuristics) compiled natively for a CLI (`lens`) and to WASM for a Manifest V3 browser extension. Verdicts come in four honest tiers — Verified, Indicated, Inconclusive, Tampered — with the founding rule baked into the types and the wording: **no provenance data ≠ authentic**. The authoritative design is **`PROVENANCE_LENS_EXECPLAN.md`** at the repo root — read it before any work; it is fully self-contained per the ExecPlan discipline (`PLANS.md` in `../../cpp/solid-broccoli`). The origin document is `ai-content-verifier-proposal.md` (the maintainer's saved proposal); the plan operationalizes it and its Decision Log explains any deviation.

## Current status (2026-07-10 — skimmable snapshot in `development_status.md`; the ExecPlan's Progress section is authoritative)

**M0–M2 closed** (M1 signed off by the maintainer per the human-review rule). Layer 1 is real (c2pa 0.89.2, default features off + `rust_native_crypto`; `lens verify --trust-anchors`; self-verifying vector corpus, regenerate: `cargo run -p provenance-core --example gen_vectors`). The WASM engine builds reproducibly (wasm-pack → system wasm-opt with flags from the wasm-packaging skill → `node scripts/wasm_smoke.mjs`), passes the corpus through the real artifact, and exports `verify_bytes(bytes, mediaType?, trustAnchorsPem?)`; size budget ≤ 7 MB raw / ≤ 2.5 MB gzipped (measured 6.41 / 2.15). 20/20 tests green (including the wording-sync audit in `tests/wording_sync.rs` — canonical phrases in `verdict.rs` must match README, popup legend, and the verdict-language skill). **The wedge (M0–M4) is complete and shippable**: extension verified end-to-end in a real Chrome (maintainer smoke), official C2PA conformance trust list ships in `extension/trust/anchors.pem` (refresh: `sh scripts/update_trust_list.sh`; the diff is trust-model data — maintainer reviews it), release zip via `sh scripts/package_extension.sh` → `dist/`, listing collateral in `docs/STORE_LISTING.md`, versions in lockstep between root `Cargo.toml` and `extension/manifest.json`, `CHANGELOG.md` per release. **Store submission is the maintainer's action.** Open: post-wedge hardening backlog (cargo-fuzz, cert-policy tests) and the gated milestones M5–M7. No GitHub remote yet (`gh` not installed). **Gated milestones:** M5 watermark (needs a runnable licensed vendor detector), M6 registry (needs a deployed transparency log), M7 heuristics (optional, may never ship) — do not start a gated milestone without its gate satisfied.

## Build

    cargo fmt --all
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo run -p provenance-cli -- verify <file>      # exit: 0 verified / 10 indicated / 20 inconclusive / 30 tampered / 2 error
    wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg

**Dependencies:** Rust stable + `wasm32-unknown-unknown` (pinned via `rust-toolchain.toml`), wasm-pack. Crates: wasm-bindgen only until M1 adds `c2pa`. Every further dependency requires an ExecPlan Decision Log entry. The bare-machine rule (inherited from solid-broccoli) is law: a fresh clone with no network, no models, no registry must build and pass 100% of tests.

## Architecture

| Target | Directory | Role |
|--------|-----------|------|
| `provenance-core` | `crates/provenance-core/` | Verdict tiers + approved phrases (`verdict.rs`), `Asset`/`Layer`/`LayerFinding`/`Pipeline`/`combine()` (`pipeline.rs`), the four layers (`layers/`). Sans-IO. |
| `provenance-cli` (`lens`) | `crates/provenance-cli/` | std-only CLI; owns all file I/O. |
| `provenance-wasm` | `crates/provenance-wasm/` | Thin wasm-bindgen wrapper: `verify_bytes(bytes, mime) -> JSON`. No logic. |
| extension | `extension/` | MV3, plain JS; consumes the wasm-pack output in `extension/pkg/` (gitignored). |

## Conventions (enforced; rationale in the ExecPlan Decision Log)

- **Honest verdicts:** `Inconclusive` never implies authenticity, anywhere — code, UI, docs, logs. The four approved phrases in `crates/provenance-core/src/verdict.rs` are canonical and must stay character-identical in `.claude/skills/verdict-language/SKILL.md`, `extension/popup/popup.html`, and `README.md`; tests pin them.
- **Evidentiary typing:** only Layer 1 (c2pa) may return `Proof`; heuristics at most `Indication`; a layer that didn't run returns `NotEvaluated { reason }`, never `NoSignal`. Combination precedence: Tampered > Verified > Indicated > Inconclusive.
- **Sans-IO core:** `provenance-core` never opens files/sockets; transports are injected; CLI and WASM wrappers own I/O.
- **Human sign-off rule:** signature-validation code (C2PA validation path, trust lists, inclusion proofs) merges only with a human cryptography reviewer's sign-off recorded in the ExecPlan Decision Log. Agents prepare; humans approve.
- **Extension surface:** permissions stay `contextMenus` + `activeTab`; no host permissions, no remote code, no analytics; the extension never renders a verdict it didn't compute.
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
