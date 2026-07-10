# Changelog

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
