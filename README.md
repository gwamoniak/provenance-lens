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
              2. Watermark         open-spec decoders (SD dwtDct, …)   → can indicate
              3. Registry lookup   transparency-log hash lookup        → can indicate   [gated]
              4. Heuristics        statistical signals                 → can indicate   [optional]
                                                          ──▶ Verified / Indicated / Inconclusive / Tampered

The watermark layer decodes supported, calibrated schemes only — including the Stable Diffusion invisible watermark (exact payload match; CLI and native builds, not in the extension engine). This project has no implemented SynthID decoder. Vendor access was last researched in August 2026; that dated research is not a claim about today's external availability. A signed watermark declaration is descriptive context, not an independent pixel detection, and a generic classifier cannot substitute for a vendor watermark decoder.

Everything is one Rust workspace: `provenance-core` (the pipeline, sans-IO), `provenance-cli` (the `lens` binary), `provenance-wasm` (the WASM wrapper the extension and the npm package embed). Blockchain appears only as optional anchoring for the registry's transparency log — never as a dependency. Privacy first: image bytes never leave your device; verification is local, and future registry lookups send perceptual hashes only, with consent.

Layer 1 accepts C2PA-bearing JPEG, PNG, WebP, GIF, AVIF, and SVG assets. Binary formats can be recognized from their container bytes; SVG requires a `.svg` filename or an explicit `image/svg+xml` MIME type because the verifier deliberately does not classify arbitrary XML as an image. Claude file exports are not special-cased: if an export carries a valid, trusted Content Credential it follows the same Verified path as any other producer. Availability varies by Claude model, surface, and export path, and a missing credential remains Inconclusive.

## What you can do today

- **CLI**: `lens verify [--json] [--trust-anchors <PEM>] <FILE>...` — one honest verdict per file, machine-readable with `--json`, exit codes scripts can branch on (0 verified / 10 indicated / 20 inconclusive / 30 tampered / 2 error). Verified reports also say what the credential *claims*: claim generator, signing time, declared digitalSourceType (with a plain note when it declares AI-generated content). Assets carrying the Stable Diffusion invisible watermark (exact payload decode) report Indicated; `--watermark-model <ONNX>` additionally probes with IMATAG's Stable Signature bzh classifier (their MIT model, covers their own bzh-watermarked images only). Measured detector behavior lives in `docs/CALIBRATION.md`.
- **Browser extension** (Chrome, and Firefox ≥ 128, one codebase): right-click any image → "Verify provenance with Provenance Lens" → verdict on the badge and in the popup. Optionally, per site and only after you grant it through the browser's own consent prompt, it scans the images on a page and marks each visible one with a small text pill — including an honest "not examined" marker when it cannot read an image's bytes.
- **JavaScript library**: the engine packs as `@provenance-lens/verify-wasm` (`sh scripts/package_npm.sh`); same API, same wording, browser and Node.

The user manual — CLI reference, extension walkthrough including page scanning, JSON shape, troubleshooting — is **[docs/MANUAL.md](docs/MANUAL.md)**.

N1/N2 development changes: incomplete watermark checks now remain `not_evaluated`; the optional IMATAG model must match the calibrated export's digest. Inputs are limited to 25 MiB, native pixel analysis to 40 million pixels, and image fetches to 15 seconds. Page scanning has a separate explicit site choice, removes badges when sources change or scanning stops, and offers keyboard-accessible **Verify again** buttons. Results name the retrieval time and apply to the fetched file, not the displayed pixels. There is no persistent URL verdict cache.

## Status

N3/N4 development candidate: Verified credentials can expose signed watermark declarations, explicitly distinguished from detection. Open **Inspect or compare local files** in the extension popup to examine one or two files offline and preview/download unsigned JSON evidence receipts. No upload, preview execution, extra permissions, or receipt-import trust is introduced. See the [Claude capability matrix](docs/CLAUDE_MARKS.md), [manual](docs/MANUAL.md), and [review evidence](docs/reviews/N3_N4_REVIEW.md). The candidate corpus has twelve coherent JPEG/PNG/SVG vectors; main and published packages have not been updated.

The Layer-1 wedge shipped and has since been hardened and extended: real C2PA validation against the official conformance trust list (shipped as an updatable data file, refreshed by a monthly review PR), an eleven-vector self-verifying test corpus (JPEG + PNG + SVG) exercised natively and through the WASM boundary, cert-policy pinning tests, SVG-aware fuzzing, and CI on every push. The approved C2PA 0.90.13 and generic SVG work is merged into local main. A credential that fails validation can produce Tampered; a credential stripped completely leaves no C2PA signal and is Inconclusive. Comparing a known original with a re-saved copy can show that loss. The watermark layer became real on 2026-07-31 (roadmap W1): the Stable Diffusion invisible-watermark decoder, transcribed from the open reference implementation and tested against vectors that implementation generated. Detectors without a public spec, and the registry layer, stay gated; where a layer cannot run it honestly reports itself as not evaluated. The development sequence lives in `PROVENANCE_LENS_NEXT_EXECPLAN.md`, with detailed Claude, SynthID, registry, and historical wedge decisions in their respective ExecPlans.

Ecosystem context: the EU AI Act's Article 50 transparency obligations apply from 2 August 2026 (machine-readable marking for generative systems already on the market is deferred to 2 December 2026). The Act obliges *generators* to mark AI content; it asks nothing of verifiers. Provenance Lens is the independent verification side of that ecosystem — as marked content becomes the norm, checking those marks locally is exactly what this tool does.

## Build and run

Requires Rust 1.97.1 (pinned with the wasm32 target in `rust-toolchain.toml`), the committed `Cargo.lock`, Node.js 24.18.0 for JavaScript checks, and wasm-pack 0.15.0 for the extension engine. An offline build still needs these tools and the locked crate dependencies installed locally.

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo test --workspace --locked
    cargo test -p provenance-core --no-default-features --locked
    node scripts/scan_support_test.mjs
    node scripts/background_test.mjs
    node scripts/inspect_test.mjs
    cargo check -p provenance-wasm --target wasm32-unknown-unknown --locked
    cargo run -p provenance-cli -- tiers
    cargo run -p provenance-cli -- verify photo.jpg     # exit: 0 verified / 10 indicated / 20 inconclusive / 30 tampered

Extension:

    wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg
    node scripts/wasm_smoke.mjs
    # Chrome:  chrome://extensions → Developer mode → Load unpacked → extension/
    # Firefox: about:debugging → This Firefox → Load Temporary Add-on → extension/manifest.json

The model-backed IMATAG test is visibly ignored by default because the model is not bundled. To exercise it, set `PROVENANCE_LENS_BZH_ONNX` to an exported model path, then run `cargo test -p provenance-core --features stable-signature --test stable_signature --locked -- --ignored`. That command fails if the model is missing. CI also offers a manual model job that downloads an HTTPS model only when given its URL and SHA-256.

The production loader accepts ONNX SHA-256 `4f7d80027d5d7ecf5c176fabc3d86c8ccd791cc5c830d23a96114425cd1f3e57`; other exports require review and calibration. Browser lifecycle checks run through Playwright 1.62.1 in Chromium and Firefox with `node scripts/scan_browser_test.mjs <wasm-pkg-directory>`. Setup and evidence are recorded in `docs/reviews/N1_N2_REVIEW.md`.

## Honesty rules

The wording above is normative (see `.claude/skills/verdict-language/SKILL.md`): Inconclusive is never styled or phrased as safety, Indicated is never an accusation, Verified vouches for the provenance chain — not for the content being human-made. The four phrases are pinned character-identical across the README, the popup, the npm README, and the manual by a CI test. Signature-validation code merges only with human cryptography-reviewer sign-off.

## License

MIT OR Apache-2.0.
