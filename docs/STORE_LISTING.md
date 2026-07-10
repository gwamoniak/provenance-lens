# Chrome Web Store listing — Provenance Lens 0.1.0

Copy below obeys `.claude/skills/verdict-language/SKILL.md`: understatement is the brand; no "detect AI fakes" claims, ever. The maintainer submits; nothing here is published by agents.

## Name

Provenance Lens

## Summary (132 chars max)

Honest provenance verdicts for images. Verifies C2PA Content Credentials locally. No provenance data does NOT mean authentic.

## Description

Provenance Lens tells you what can actually be known about where an image came from — and refuses to pretend to know more.

Right-click any image and choose "Verify provenance with Provenance Lens". You get one of four verdicts:

- Verified: this asset carries a valid, cryptographically signed provenance chain.
- Indicated: signals suggest AI involvement, but no cryptographic proof chain is present.
- Inconclusive: no provenance data was found. This does NOT mean the asset is authentic.
- Tampered: provenance data is present but fails validation. Treat this asset with suspicion.

What it does: validates C2PA Content Credentials (the open provenance standard) directly on your device, against the official C2PA conformance trust list. What it does not do: guess from pixels, score "AI likelihood", or call anything authentic. Most genuine images on the web carry no provenance data — for them the honest answer is Inconclusive, and that is what you will see.

Verification runs locally in WebAssembly. Your images never leave your device. The only network request the extension ever makes is fetching the image you explicitly asked it to verify. No analytics, no accounts, no remote code.

Open source (MIT OR Apache-2.0).

## Permission justifications (store review form)

- `contextMenus` — the single entry point: the right-click "Verify provenance" item on images.
- `activeTab` — lets the extension fetch the image you right-clicked from the page you are on, without any standing host permissions.
- `storage` (session) — holds the last verification result so the popup can display it; cleared when the browser closes.
- No host permissions. No remote code. WASM is bundled in the package (`wasm-unsafe-eval` CSP is required to instantiate it).

## Privacy policy (single purpose)

Provenance Lens verifies image provenance locally. It does not collect, transmit, store (beyond the current browser session), or sell any user data. Image bytes are fetched only on explicit user action and are processed entirely on-device. No telemetry of any kind.

## Pre-submission checklist (lens-release runs this; maintainer presses publish)

1. `cargo test --workspace` green (includes the wording-sync audit) and `node scripts/wasm_smoke.mjs` passing.
2. Versions in lockstep: root `Cargo.toml` `[workspace.package]` and `extension/manifest.json`.
3. `CHANGELOG.md` entry for the version.
4. `extension/trust/anchors.pem` is the official C2PA list with a current provenance header (`sh scripts/update_trust_list.sh`), NOT a test CA.
5. `sh scripts/package_extension.sh` → zip builds from a clean tree; record sha256 and size in the ExecPlan's Artifacts.
6. Manual browser smoke (ExecPlan M3 script) passed on the packaged build.
7. Any release touching signature-validation code: maintainer cryptography sign-off recorded in the Decision Log first.
