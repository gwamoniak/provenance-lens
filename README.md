# Provenance Lens

An honest provenance verifier for AI-era media: a Rust pipeline, a CLI, and a browser extension that tell you what can actually be known about where an image came from — and refuse to pretend to know more.

## The four verdicts

- **Verified**: this asset carries a valid, cryptographically signed provenance chain.
- **Indicated**: signals suggest AI involvement, but no cryptographic proof chain is present.
- **Inconclusive**: no provenance data was found. This does NOT mean the asset is authentic.
- **Tampered**: provenance data is present but fails validation. Treat this asset with suspicion.

The founding rule: **no data ≠ authentic.** Most genuine images on the web carry no provenance data, so "nothing found" licenses no conclusion — and this tool never renders it as one. Pixel-guessing "AI detectors" overclaim in both directions; Provenance Lens only reports what it can back.

## How it works

One asset flows through four layers, ordered by evidentiary strength; findings combine conservatively (tamper evidence outranks everything):

    bytes ──▶ 1. C2PA proof        cryptographic manifest validation   → can prove
              2. Watermark         vendor detectors (SynthID, …)       → can indicate   [gated]
              3. Registry lookup   transparency-log hash lookup        → can indicate   [gated]
              4. Heuristics        statistical signals                 → can indicate   [optional]
                                                          ──▶ Verified / Indicated / Inconclusive / Tampered

Everything is one Rust workspace: `provenance-core` (the pipeline, sans-IO), `provenance-cli` (the `lens` binary), `provenance-wasm` (the WASM wrapper the extension and the npm package embed). Blockchain appears only as optional anchoring for the registry's transparency log — never as a dependency. Privacy first: image bytes never leave your device; verification is local, and future registry lookups send perceptual hashes only, with consent.

## What you can do today

- **CLI**: `lens verify [--json] [--trust-anchors <PEM>] <FILE>...` — one honest verdict per file, machine-readable with `--json`, exit codes scripts can branch on (0 verified / 10 indicated / 20 inconclusive / 30 tampered / 2 error). Verified reports also say what the credential *claims*: claim generator, signing time, declared digitalSourceType (with a plain note when it declares AI-generated content).
- **Browser extension** (Chrome, and Firefox ≥ 128, one codebase): right-click any image → "Verify provenance with Provenance Lens" → verdict on the badge and in the popup. Optionally, per site and only after you grant it through the browser's own consent prompt, it scans the images on a page and marks each visible one with a small text pill — including an honest "not examined" marker when it cannot read an image's bytes.
- **JavaScript library**: the engine packs as `@provenance-lens/verify-wasm` (`sh scripts/package_npm.sh`); same API, same wording, browser and Node.

The user manual — CLI reference, extension walkthrough including page scanning, JSON shape, troubleshooting — is **[docs/MANUAL.md](docs/MANUAL.md)**.

## Status

The Layer-1 wedge shipped and has since been hardened and extended (2026-07-18): real C2PA validation against the official conformance trust list (shipped as an updatable data file, refreshed by a monthly review PR), an eight-vector self-verifying test corpus (JPEG + PNG) exercised natively, through the compiled WASM artifact, and in real browsers, cert-policy pinning tests, a fuzz target, and CI on every push. The build order is deliberate: a **Layer-1-only** tool first — small but provable — because surfacing "Tampered / credentials stripped" at scale is the wedge that pressures platforms to stop stripping Content Credentials. Watermark and registry layers stay gated behind real detectors and a real transparency log; until then they honestly report themselves as not evaluated. The plans live in `PROVENANCE_LENS_EXECPLAN.md` (the wedge) and `PROVENANCE_LENS_UPGRADES_EXECPLAN.md` (the executed upgrade wave).

## Build and run

Requires Rust stable (pinned via `rust-toolchain.toml`; includes the wasm32 target) and, for the extension engine, wasm-pack.

    cargo test --workspace
    cargo run -p provenance-cli -- tiers
    cargo run -p provenance-cli -- verify photo.jpg     # exit: 0 verified / 10 indicated / 20 inconclusive / 30 tampered

Extension:

    wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg
    # Chrome:  chrome://extensions → Developer mode → Load unpacked → extension/
    # Firefox: about:debugging → This Firefox → Load Temporary Add-on → extension/manifest.json

## Honesty rules

The wording above is normative (see `.claude/skills/verdict-language/SKILL.md`): Inconclusive is never styled or phrased as safety, Indicated is never an accusation, Verified vouches for the provenance chain — not for the content being human-made. The four phrases are pinned character-identical across the README, the popup, the npm README, and the manual by a CI test. Signature-validation code merges only with human cryptography-reviewer sign-off.

## License

MIT OR Apache-2.0.
