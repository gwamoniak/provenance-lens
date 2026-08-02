# Development status — Provenance Lens

**TL;DR (2026-07-18, end of day): 0.2.0 is OUT THE DOOR — submitted to the Chrome Web Store and Firefox AMO (both in review) and published to npm. The wedge shipped, the hardening pass merged with sign-off, the upgrades plan U1–U7 fully executed and browser-smoked, the trust list verified current at submission. 30/30 tests green, and the repo is public. Remaining: store review outcomes, the upstream c2pa report, and the gated M5–M7.**

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
| 2 — Watermark | **Real for two schemes, calibration published (W1+W2, 2026-08-01)**: `WatermarkDetector` trait; the SD invisible-watermark decoder (exact-payload dwtDct, default-on `watermark-dwt` feature, native only) and IMATAG's Stable Signature bzh classifier (opt-in `stable-signature` feature via pure-Rust tract; model runtime-supplied with `lens verify --watermark-model <ONNX>`). Ceiling `Indication`. Measured behavior in `docs/CALIBRATION.md`: 0 false positives observed anywhere; DWT TPR 45% pristine / 0% after any transform (the scheme's fragility, measured); bzh TPR 30–85% across the battery. | Further schemes need a public spec/weights + measured FPR (revised M5 gate); SynthID impossible without vendor access (W3). |
| 3 — Registry | Honest stub; PDQ/pHash + transparency-log design in skill. | GATED: a deployed log endpoint. |
| 4 — Heuristics | Honest stub. | OPTIONAL; may never ship. |

Verdict model (Tampered > Verified > Indicated > Inconclusive; only Layer 1 proves; "no data ≠ authentic"): **implemented, green, and wording-audited by CI** — `tests/wording_sync.rs` fails the suite if README, popup legend, or the verdict-language skill drift from the canonical phrases in `verdict.rs`.

## Release state (0.2.0 — submitted to both stores and published to npm, 2026-07-18)

- Extension: `dist/provenance-lens-0.2.0.zip` — 2,301,127 bytes, sha256 `e666b30c…a0817b12` (rebuild: `sh scripts/package_extension.sh`; refuses to package a test CA). Chrome + Firefox ≥ 128; **submitted to the Chrome Web Store and AMO, both in store review**. Trust list verified current upstream at packaging time (2026-07-18, 28 certs, unchanged since 2026-07-10).
- npm: `dist/provenance-lens-verify-wasm-0.2.0.tgz` — 2,441,384 bytes, sha256 `867447b9…7c7e8889` (rebuild: `sh scripts/package_npm.sh`). **Published.**
- Release notes: CHANGELOG.md `0.2.0`. Versions in lockstep (root `Cargo.toml`, `extension/manifest.json`, both packaged artifacts).

Previous (0.1.0):

- Artifact: `dist/provenance-lens-0.1.0.zip` — 2,299,035 bytes, sha256 `9687a5d25553125ee5a94b73d080bc5dccaaaa3fe177c5ccf33d21bbe07980e2`.
- Trust anchors: official C2PA conformance list, 28 certificates, sha256 `b1f399a7…7cdb46c1`, provenance header in `extension/trust/anchors.pem`; refresh via `sh scripts/update_trust_list.sh` — **the diff is trust-model data; maintainer reviews every change.**
- Collateral: `CHANGELOG.md` (0.1.0), `docs/STORE_LISTING.md` (listing copy, permission justifications, privacy policy, pre-submission checklist).
- Remote: https://github.com/gwamoniak/provenance-lens (**public** as of 2026-07-18, `origin`; `main` in sync).
- Test evidence: 20/20 tests, clippy `-D warnings` clean; corpus verified natively, through the WASM wrapper, through the compiled artifact in Node, and by the maintainer's browser smoke.

## Blockers

None. Everything left is a maintainer decision, not a blocker.

## Next actions

1. ~~Release + submissions~~ — **done 2026-07-18**: 0.2.0 submitted to the Chrome Web Store and Firefox AMO (maintainer; both awaiting store review) and published to npm (`@provenance-lens/verify-wasm`). Watch for reviewer questions — permission justifications and reviewer notes are in `docs/STORE_LISTING.md`.
2. ~~File the prepared upstream report~~ — **not needed (checked 2026-07-27)**: the c2pa maintainers already fixed our exact SaltHash-box underflow on `main` (unreleased); every release through 0.90.3 still has it. No open issue; filing would duplicate resolved work. **The bump is now automated**: `.github/workflows/c2pa-bump-check.yml` (weekly) watches crates.io, verifies the SaltHash fix is actually in the release tag, and opens a review PR bumping `crates/provenance-core/Cargo.toml` — never merges (crypto sign-off surface), same pattern as the trust-list refresh. When that PR lands, the maintainer signs off, then flips the CI fuzz job to enforcing and decides on the panic guard. Panic guard protects us meanwhile. (A cloud-routine alternative was attempted first but requires connecting the GitHub account in claude.ai; the in-repo workflow needs no external auth.)
3. ~~Make the repo public~~ — **done 2026-07-18**: https://github.com/gwamoniak/provenance-lens is public.
4. **Roadmap plan (2026-07-31, maintainer-approved)**: `PROVENANCE_LENS_ROADMAP_EXECPLAN.md`, authored from an external technical review of 0.2.0 — **R1, R2, W1, and W2 all done** (docs honesty + EU AI Act positioning; trust-list staleness defenses with crypto sign-off; the SD invisible-watermark detector, FPR gate passed 0/2,861; the IMATAG bzh classifier via tract + the calibration corpus, with measured TPR/FPR published in `docs/CALIBRATION.md` — zero false positives observed anywhere). Remaining: W3 SynthID stays vendor-gated (`lens-research` watches Google's Content Detection API); W4 = M6 registry design work on the maintainer's word.
5. **Gated milestones** (wedge plan): M5 watermark (gate revised 2026-07-31 — satisfied by the open SD detector; execution = roadmap W1–W2) / M6 registry / M7 heuristics; `lens-research` tracks availability.

## Risks being tracked

- Trust-list staleness: new conformance roots appear over time; a stale bundle turns fresh legitimate credentials into "Tampered / unverifiable". Mitigation implemented on `post-wedge-hardening`: monthly refresh workflow opening a review PR (active once the branch merges and GitHub Actions is enabled on the repo).
- Store review friction (permission justifications, privacy policy) — mitigated by the minimal-permission design and the prepared listing; still an unknown until first submission.
- Cross-origin images without `Access-Control-Allow-Origin` can't be fetched under the no-host-permissions design; the extension says so honestly. If it bites users at scale, the recorded alternative is an explicit opt-in "grant site access" flow (Decision Log).
- Layer 2/3 gates may stay closed for a long time — acceptable by design; docs must keep saying "not evaluated" plainly.
- Single-maintainer bus factor: the crypto sign-off role and store account are one person.
