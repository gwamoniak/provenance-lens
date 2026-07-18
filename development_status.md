# Development status — Provenance Lens

**TL;DR (2026-07-10, end of day): the Layer-1 wedge is DONE and PUSHED — M0–M4 all closed in a single day. `lens` (CLI) and the browser extension verify real C2PA Content Credentials against the official conformance trust list; the installable zip is built; 20/20 tests green; everything is on the private GitHub remote. Remaining: maintainer submits to the store; hardening backlog; gated M5–M7.**

Snapshot dashboard, last updated **2026-07-10 (post-M4, wedge shipped)**. This file is a *derived view* for humans skimming the repo: the authoritative record is the `Progress` / `Decision Log` / `Surprises & Discoveries` sections of `PROVENANCE_LENS_EXECPLAN.md`. Update this dashboard at milestone boundaries; if it ever disagrees with the ExecPlan, the ExecPlan wins.

## Where we are, in one paragraph

The proposal's "weeks 1–6" wedge exists, works, and is proven. Layer 1 does real C2PA validation (c2pa 0.89.2, no OpenSSL, no HTTP clients compiled in — sans-IO enforced at the linker), signed off by the maintainer per the human-review rule and merged through the first full exercise of that process. The WASM engine builds reproducibly and passes the vector corpus running as a real artifact in Node. The extension passed the maintainer's browser smoke: right-click an image → Verified / Inconclusive / Tampered on the badge and in the popup, with errors shown as errors. The official C2PA conformance trust list (28 roots, provenance-stamped) ships as an updatable data file. What compressed six weeks into a day: self-verifying test infrastructure (a corpus that refuses to lie, a smoke that runs the real artifact) and an ecosystem more ready than the proposal assumed.

## Proposal deliverables → current state

| Proposal deliverable | Here | State |
|---|---|---|
| `verify-core` (Rust crate, native + WASM + C API) | `crates/provenance-core` | **Done for the wedge**: Layer 1 real, three layers honestly gated; 4-layer pipeline, verdict model with pinned wording. C API: post-wedge, not started. |
| `verify-wasm` (JS/TS bindings, npm) | `crates/provenance-wasm` | **Done for the wedge**: `verify_bytes(bytes, mediaType?, trustAnchorsPem?)` → JSON; parity + true-artifact smoke green; 6.41 MB raw / 2.15 MB gz (budget 7 / 2.5). npm package: post-wedge. |
| `provenance-lens` (WebExtension MV3) | `extension/` | **Working end-to-end** (maintainer-smoked in real Chrome): context menu → fetch → engine → badge + popup; honest failure states; permissions only `contextMenus` + `activeTab` + `storage`. |
| `verify-registry` (transparency-log service) | — | Not started; GATED (M6). Design knowledge in the `provenance-registry` skill. |
| CLI (`verify image.png`) | `crates/provenance-cli` → `lens` | **Working**: `lens verify [--trust-anchors <PEM>] <file>`, verdict exit codes 0/10/20/30. |

## Pipeline layers

| Layer | State | Gate |
|---|---|---|
| 1 — C2PA proof | **Done** (M1, maintainer crypto sign-off recorded): Trusted→Proof(issuer) / unanchored-or-invalid→TamperEvidence / absent→NoSignal / unparseable→NotEvaluated. | None. |
| 2 — Watermark | Honest stub; `WatermarkDetector` trait specified. | GATED: a runnable, licensed vendor detector (`lens-research` watches). |
| 3 — Registry | Honest stub; PDQ/pHash + transparency-log design in skill. | GATED: a deployed log endpoint. |
| 4 — Heuristics | Honest stub. | OPTIONAL; may never ship. |

Verdict model (Tampered > Verified > Indicated > Inconclusive; only Layer 1 proves; "no data ≠ authentic"): **implemented, green, and wording-audited by CI** — `tests/wording_sync.rs` fails the suite if README, popup legend, or the verdict-language skill drift from the canonical phrases in `verdict.rs`.

## Release state (0.1.0)

- Artifact: `dist/provenance-lens-0.1.0.zip` — 2,299,035 bytes, sha256 `9687a5d25553125ee5a94b73d080bc5dccaaaa3fe177c5ccf33d21bbe07980e2` (rebuild: `sh scripts/package_extension.sh`; refuses to package a test CA).
- Trust anchors: official C2PA conformance list, 28 certificates, sha256 `b1f399a7…7cdb46c1`, provenance header in `extension/trust/anchors.pem`; refresh via `sh scripts/update_trust_list.sh` — **the diff is trust-model data; maintainer reviews every change.**
- Collateral: `CHANGELOG.md` (0.1.0), `docs/STORE_LISTING.md` (listing copy, permission justifications, privacy policy, pre-submission checklist).
- Remote: https://github.com/gwamoniak/provenance-lens (private, `origin`; `main` in sync, M1 review branch pushed). `gh` authenticated as gwamoniak (HTTPS; SSH to GitHub does not work on this machine).
- Test evidence: 20/20 tests, clippy `-D warnings` clean; corpus verified natively, through the WASM wrapper, through the compiled artifact in Node, and by the maintainer's browser smoke.

## Blockers

None. Everything left is a maintainer decision, not a blocker.

## Next actions

1. **Maintainer: submit to the Chrome Web Store** — `docs/STORE_LISTING.md` has everything the form asks; the zip is in `dist/`.
2. **Maintainer, when ready**: make the repo public (`gh repo edit gwamoniak/provenance-lens --visibility public`).
3. ~~Review branch `post-wedge-hardening`~~ — **signed off and merged 2026-07-18** ("approved, merge it"); the hardening backlog (cert-policy pinning tests, fuzz target, panic guard, trust-list refresh workflow) is on `main`.
4. **Maintainer: file the prepared upstream report** — the fuzz target's first run found an unchecked-subtraction panic in the c2pa crate's JUMBF parser (still in 0.90.0); ready-to-file text is in the ExecPlan's Artifacts and Notes.
5. **Next wave when ready**: `PROVENANCE_LENS_UPGRADES_EXECPLAN.md` (JSON CLI output, credential summaries, proven format coverage, CI, Firefox, npm; page-scan badges design-gated).
4. **Gated milestones**: open M5 (watermark) / M6 (registry) only when their gates open; `lens-research` tracks detector and registry availability.

## Risks being tracked

- Trust-list staleness: new conformance roots appear over time; a stale bundle turns fresh legitimate credentials into "Tampered / unverifiable". Mitigation implemented on `post-wedge-hardening`: monthly refresh workflow opening a review PR (active once the branch merges and GitHub Actions is enabled on the repo).
- Store review friction (permission justifications, privacy policy) — mitigated by the minimal-permission design and the prepared listing; still an unknown until first submission.
- Cross-origin images without `Access-Control-Allow-Origin` can't be fetched under the no-host-permissions design; the extension says so honestly. If it bites users at scale, the recorded alternative is an explicit opt-in "grant site access" flow (Decision Log).
- Layer 2/3 gates may stay closed for a long time — acceptable by design; docs must keep saying "not evaluated" plainly.
- Single-maintainer bus factor: the crypto sign-off role and store account are one person.
