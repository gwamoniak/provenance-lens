# Changelog

## Unreleased — the post-wedge upgrades (U1–U7, 2026-07-18)

Version number is the maintainer's release decision. All changes below are
on `main`, CI-green, and browser-smoked where they touch the extension.

- CLI: `lens verify --json` (one JSON object per file, same shape the WASM
  engine returns), multiple files per invocation, exit code = the worst
  per-file result.
- Verified reports now say what the credential *claims*: claim generator,
  signing time, and the declared digitalSourceType with a fixed descriptive
  note for the generative-AI types — never shown beside any other verdict.
- Format coverage proven by vectors: PNG signed/stripped/corrupted join the
  corpus (eight vectors); byte sniffing covers JPEG/PNG/WebP/GIF/AVIF and
  the whole corpus is tested hint-free, native and through the artifact.
- Firefox support (≥128) from the same codebase; AMO listing collateral.
- npm publish prep: `@provenance-lens/verify-wasm` via
  `sh scripts/package_npm.sh` (web-target artifact, corpus-smoked, packed).
- Opt-in page scanning: per-site grants through the browser's own consent
  prompt (`optional_host_permissions` + `scripting`; nothing granted at
  install), text-only verdict pills on visible images, honest
  "not examined" markers for non-granted image hosts with a one-click
  allow-and-verify, popup toggle to start/stop per site.
- Hardening: cert-policy pinning tests (expiry, algorithm allowlist), a
  fuzz target that found an upstream c2pa parser panic (panic guard added;
  report prepared upstream), a monthly trust-list refresh workflow that
  opens a review PR, and CI on every push.

## 0.1.0 — 2026-07-10 (the Layer-1 wedge)

First release: honest C2PA provenance verdicts, nothing more.

- `lens` CLI: `lens verify [--trust-anchors <PEM>] <file>` prints a verdict
  report (Verified / Indicated / Inconclusive / Tampered) with per-layer
  findings; verdict-mapped exit codes (0/10/20/30, 2 for errors).
- Browser extension (MV3): right-click any image → "Verify provenance with
  Provenance Lens" → verdict on the action badge and in the popup. Image
  bytes never leave the device; the only network request is fetching the
  image the user asked about.
- Verification engine: C2PA manifest validation via the CAI Rust SDK
  (c2pa 0.89.2), compiled natively and to WASM. No OpenSSL, no HTTP clients
  compiled in. Ships the official C2PA conformance trust list as an
  updatable data file.
- Watermark, registry, and heuristics layers are honestly reported as
  "not evaluated" — they are gated on real detectors and a real
  transparency log, and this tool does not guess.
