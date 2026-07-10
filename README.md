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

Everything is one Rust workspace: `provenance-core` (the pipeline, sans-IO), `provenance-cli` (the `lens` binary), `provenance-wasm` (the WASM wrapper the extension embeds). Blockchain appears only as optional anchoring for the registry's transparency log — never as a dependency. Privacy first: image bytes never leave your device; verification is local, and future registry lookups send perceptual hashes only, with consent.

## Status

Scaffold stage (M0, 2026-07-10). The build order is deliberate: ship a **Layer-1-only** CLI + extension first — small but provable — because surfacing "Tampered / credentials stripped" at scale is the wedge that pressures platforms to stop stripping Content Credentials. Watermark and registry layers are gated behind real detectors and a real transparency log; until then they honestly report themselves as not evaluated. The full plan lives in `PROVENANCE_LENS_EXECPLAN.md`.

## Build and run

Requires Rust stable (pinned via `rust-toolchain.toml`; includes the wasm32 target) and, for the extension engine, wasm-pack.

    cargo test --workspace
    cargo run -p provenance-cli -- tiers
    cargo run -p provenance-cli -- verify photo.jpg     # exit: 0 verified / 10 indicated / 20 inconclusive / 30 tampered

Extension (skeleton today, functional from Milestone 3):

    wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg
    # chrome://extensions → Developer mode → Load unpacked → extension/

## Honesty rules

The wording above is normative (see `.claude/skills/verdict-language/SKILL.md`): Inconclusive is never styled or phrased as safety, Indicated is never an accusation, Verified vouches for the provenance chain — not for the content being human-made. Signature-validation code merges only with human cryptography-reviewer sign-off.

## License

MIT OR Apache-2.0.
