# Provenance Lens — user manual

This manual covers the `lens` command-line tool, the browser extension (including page scanning), the JSON output, and the npm package. Wording note: the four verdict phrases quoted here are the product's contract and appear character-identical in every surface; a CI test enforces it.

## What this tool is, and is not

Provenance Lens validates **C2PA Content Credentials** — cryptographically signed provenance manifests that some cameras, editors, and AI generators embed in media files — and decodes **openly specified invisible watermarks** (today: the Stable Diffusion scheme). When credentials are present and valid, that is provable; when a known watermark payload decodes exactly, that is an indication. When neither is found, **nothing is provable in either direction**, and the tool says so. It never guesses: no pixel-statistics "AI likelihood" scoring, and it never calls anything authentic.

Two related honesty notes. First, this project has no implemented SynthID or Claude text-watermark decoder. Vendor-access research dated August 2026 is not a current availability guarantee; no general classifier substitutes for an actual watermark detector. Second, checking a mark is not a legal compliance attestation. Provenance Lens reports the specific cryptographic or statistical evidence its implemented checks can support.

## The four verdicts

- **Verified**: this asset carries a valid, cryptographically signed provenance chain.
  The manifest's signature is cryptographically valid AND its certificate chains to one of your configured trust anchors. This vouches for the provenance chain — who signed it and that the bytes are unmodified since — not for the content being true or human-made. A Verified report also states what the credential *claims* (see "Credential claims" below).
- **Indicated**: signals suggest AI involvement, but no cryptographic proof chain is present.
  Comes from the non-cryptographic layers. One real source exists today: the Stable Diffusion invisible-watermark detector (in the CLI and native builds; the browser extension's engine does not include it yet). It claims a detection only when a known payload decodes exactly — the finding then reads e.g. "indication from the Stable Diffusion XL invisible watermark (48-bit dwtDct payload, decoded exactly)".
- **Inconclusive**: no provenance data was found. This does NOT mean the asset is authentic.
  The common case for genuine and generated images alike — most images on the web carry no credentials, often because platforms strip them on upload.
- **Tampered**: provenance data is present but fails validation. Treat this asset with suspicion.
  Includes: content modified after signing (hash mismatch), broken manifests, expired signing credentials — and also cryptographically valid signatures whose certificate does not chain to any of your trust anchors (unverifiable provenance is treated conservatively, not charitably).

## Local-file inspection, receipts, and comparison

On the N3/N4 development candidate, build the bundled engine and load/reload the unpacked extension as described in the README. Open the toolbar popup and choose **Inspect or compare local files**. Use the labeled file picker or drop one or two files onto the drop area; each file is limited to 25 MiB. No site grant is needed. The same bundled C2PA verifier and production anchor bundle run locally, including when offline. File bytes are never uploaded, and SVG/HTML is not displayed or executed as a preview.

One file shows its exact byte length, SHA-256, per-layer findings, and any verified credential claims. Two files show independent reports and explain evidence differences. A signed test JPEG verifies only with its test CA in an isolated development test; the production bundle correctly does not trust that CA. The three labeled flower SVGs contain no C2PA credentials. Neither an unsigned result nor a difference between files proves authenticity, authorship, or that B was derived from A.

Each successful examination offers an expanded **Preview the complete receipt before saving**, followed by **Save unsigned JSON receipt**. The receipt is a versioned wrapper around the report with exact asset, engine, and anchor digests; examination time; byte length; MIME-hint source; package/SDK/profile identifiers; and enabled checks/model absence. It contains the basename, not a local absolute path, and no source URL or copied document text. Signed claim text is visible in the preview—review it before sharing. The receipt is unsigned, editable, and not a Content Credential, certified audit, or legal compliance attestation. There is deliberately no receipt-import verification; select the original file to recompute.

Pixel watermark and Claude text detectors, registry lookup, and heuristics are not available in the standalone browser engine. These limitations remain NotEvaluated. Oversized/unreadable files show errors without a report receipt. **Clear files and results** clears the page state and prevents pending reads from bringing results back; previously downloaded receipts remain your files. File inputs, buttons, and receipt disclosure controls are keyboard-accessible.

## The CLI: `lens`

Build it with `cargo build -p provenance-cli --release` (binary at `target/release/lens`), or run in place with `cargo run -p provenance-cli --`.

    lens verify [--json] [--trust-anchors <PEM>] <FILE>...
    lens tiers

`lens tiers` prints the four verdicts with their exact phrases.

`lens verify` examines each file and prints a report per file. Flags may appear in any order:

- `--trust-anchors <PEM>` — a PEM bundle of root certificates that signature chains may validate against. **Without it, no chain can validate as trusted**, so even validly signed assets report Tampered (unverifiable provenance). The ecosystem's canonical anchor set is the official C2PA conformance trust list — the copy the extension ships is at `extension/trust/anchors.pem`, refreshed via `sh scripts/update_trust_list.sh`.
- `--json` — one JSON object per line per file (shape below) instead of the human report.
- `--watermark-model <ONNX>` — adds the IMATAG Stable Signature "bzh" watermark classifier to the watermark probes for this run. The model is not bundled: IMATAG publishes the weights (MIT) as a PyTorch model, and `scripts/export_stable_signature_onnx.py` exports them to the ONNX file this flag consumes. Honest coverage note: this classifier recognizes ONLY images produced by IMATAG's own bzh-watermarked SDXL-VAE variants — it is not a general Stable Signature detector. Measured behavior under common transformations: `docs/CALIBRATION.md`.

Only the calibrated ONNX export is accepted: SHA-256 `4f7d80027d5d7ecf5c176fabc3d86c8ccd791cc5c830d23a96114425cd1f3e57`. Unknown models fail at startup. An absent model, unsupported image size, or inference failure is an unavailable check, never a negative. A completed DWT hit can still yield Indicated when the optional model is absent. If no detector hits and any configured check is incomplete, the watermark finding is `not_evaluated`; only completed negative checks yield `no_signal`.

Examinations accept at most 25 MiB of file bytes and 40 million decoded pixels for native watermark analysis. Oversized input is explicitly not examined; the shared native/WASM pipeline runs no layers on an input over the byte limit. CLI reads stop after one byte beyond that limit, even if file metadata is inaccurate. Image decoding also uses a 256 MiB decoder allocation budget. These are operational limits, not guarantees about total process memory or parser execution time.

Exit code remains the highest per-file code (so a verdict code can outrank a file-read error code of 2):

    0 verified    10 indicated    20 inconclusive    30 tampered    2 usage-or-IO error

Example (against the repo's test corpus; `V=crates/provenance-core/tests/vectors`):

    $ lens verify --trust-anchors $V/test_ca.pem $V/valid_signed.jpg
    crates/provenance-core/tests/vectors/valid_signed.jpg
      verdict: Verified: this asset carries a valid, cryptographically signed provenance chain.
      [c2pa] valid provenance chain, issuer: …
      [watermark] not evaluated — unsupported dimensions or optional model unavailable (see reported reason)
      [registry] not evaluated — gated: no transparency-log registry deployed yet
      [heuristics] not evaluated — optional layer, not implemented
      credential claims:
        claim generator: provenance-lens test vectors/0.1.0
        declared source type: http://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicMedia
        note: the credential declares this content AI-generated

Supported C2PA containers are JPEG, PNG, WebP, GIF, AVIF, and SVG. The binary formats can be hinted by extension or sniffed from their container bytes. SVG is accepted from a `.svg` filename or an explicit `image/svg+xml` MIME hint; unlabelled XML is intentionally not sniffed as SVG. Anything else reports the C2PA layer as "not evaluated" with the reason — honestly distinct from "looked and found nothing".

### Credential claims

Only Verified reports carry this section, because only a validated manifest is worth quoting. It is descriptive, never an endorsement: the claim generator ("name/version" of the signing software), the signing time when the credential carries a timestamp, and the declared digitalSourceType verbatim. For the IPTC generative-AI source types a fixed plain-language note is added (e.g. "the credential declares this content AI-generated"). The credential says it; the tool relays it.

### Claude file exports

The cross-surface evidence matrix is in [CLAUDE_MARKS.md](CLAUDE_MARKS.md). Generic signed JPEG/PNG/SVG fixtures pass; a genuine Anthropic-signed positive fixture has not been demonstrated. The labeled flower SVGs remain unsigned negative controls.

Verified credential summaries may include `declared_watermark` with `action_labels`, `algorithms`, and the note “the verified credential records an invisible watermark; the watermark was not independently detected”. The algorithms are signed soft-binding identifiers, not necessarily watermark schemes: a fingerprint may coexist with a watermark action. A soft binding alone is insufficient. This context never contributes a Layer-2 hit or changes a verdict, and is absent from unsigned, invalid, and untrusted reports.

Claude file outputs use the same vendor-neutral C2PA path as every other asset. A valid credential can prove which trusted signer processed the file and what that credential declares; it does not prove Claude authored every element. Marking availability varies by model, surface, feature, and export route. A Claude file with no manifest is therefore Inconclusive, not evidence that the file is human-made or that Claude was uninvolved. Provenance Lens does not currently detect Claude's separate embedded text watermark.

### JSON output

Local browser receipts wrap this existing report rather than changing its contract. The CLI does not import/export receipts and its exit codes are unchanged.

`lens verify --json` emits, per file, the same flat shape the WASM engine returns (one renderer in `provenance-core` serves both — they cannot drift):

    {
      "file": "photo.jpg",
      "verdict": "verified" | "indicated" | "inconclusive" | "tampered",
      "phrase": "<the tier phrase, verbatim>",
      "credentials": {            // present ONLY when verdict is "verified"
        "issuer": "…",
        "claim_generator": "…",   // absent keys are omitted, never null
        "signing_time": "…",
        "digital_source_type": "…",
        "source_type_note": "…"
      },
      "findings": [
        { "layer": "c2pa", "status": "proof" | "no_signal" | "tamper_evidence" | "indication" | "not_evaluated", "detail": "…" },
        …
      ]
    }

Parse the `verdict` id and branch on the exit code; display the `phrase` verbatim — do not soften it.

## The browser extension

One codebase for Chrome and Firefox (Firefox 128 or newer). Until the store listings are live, load it unpacked: build the engine first (`wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg`), then Chrome: chrome://extensions → Developer mode → Load unpacked → `extension/`; Firefox: about:debugging → This Firefox → Load Temporary Add-on → `extension/manifest.json`.

### Verifying one image (no permissions beyond the click)

Right-click any image → **"Verify provenance with Provenance Lens"**. The extension fetches that image file, checks it locally, shows the verdict on the toolbar badge (VER green / IND amber / INC gray / TAM red; ERR black for errors), and opens the popup with the full report and retrieval time. This checks the fetched file, not the page's displayed pixels: a server can return different bytes on another request. **Verify again** fetches anew. Image bytes are never uploaded. Fetching has a 15-second deadline and a streamed 25 MiB limit, including responses without a Content-Length header. Errors are shown as errors, never dressed as a verdict.

### Page scanning (opt-in, per site)

Off by default, everywhere. To enable it for a site: open the popup on that site → **Page scanning** → "Scan images on \<host\>" → accept the browser's consent prompt. It starts on the open page without reloading. Scanning consent is stored separately from permission to fetch that host. Existing host grants alone do not opt a site into scanning. Visible images rendered at least 64×64 pixels are queued; some images cannot be examined because of format, access, or resource limits.

What you will see on the page, as small text pills on each examined image:

- **VER / IND / INC / TAM** in the tier colors — hover for the exact phrase and retrieval time; activate the button with a click, Enter, or Space to verify again and show the full report. A changed image source invalidates its old badge; delayed results for an earlier source are discarded.
- **· · ·** (gray, dashed) — *not examined*: missing image-host access, a fetch failure, an exceeded limit, or a full queue. The tooltip reports the reason. For missing access, the popup offers **"Allow access to \<image-host\> and verify"**. That grant permits fetching that host; it does not enable scanning its pages.
- **ERR** (black) — an error other than access (e.g. the engine is missing). The report has the specifics.

Expect a lot of gray INC pills — that is the point. Most images have no credentials, frequently because platforms strip them; seeing that plainly, at scale, is what this tool exists to show.

To stop: popup → "Stop scanning \<host\>". Badges disappear and active/queued page work is cancelled without a reload; the separate host-fetch grant remains. Revoking host access in browser settings also stops scanning and removes that site's scan choice. There are at most two active fetches and 50 waiting jobs. There is no persistent URL report cache; identical pending requests for the same document/source generation may share work.

### Permissions, plainly

`contextMenus` (the right-click entry), `activeTab` (fetch the image you right-clicked, from the page you are on), `storage` (session last result and a local list of explicit scanning choices), `scripting` (register the script where scanning consent and host permission both exist). Host access is exclusively opt-in at runtime — nothing at install. No analytics, no remote code.

## The npm package

The engine is packaged as `@provenance-lens/verify-wasm` (`sh scripts/package_npm.sh` → tarball in `dist/`). API, JSON shape, and wording are identical to everything above; usage examples for Node and bundlers are in the package README (`crates/provenance-wasm/README.md`). Remember the trust-anchor rule: pass a PEM bundle, or nothing can verify as trusted.

## Building and testing from source

    cargo fmt --all
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace          # includes corpus, parity, cert-policy, wording audits

The test corpus (`crates/provenance-core/tests/vectors/`, eleven vectors, JPEG + PNG + SVG) is generated by a self-verifying tool — `cargo run -p provenance-core --example gen_vectors` — that refuses to write a vector whose verdict does not match its catalogue entry and refreshes the committed fuzz seeds. Raster vectors exercise hint-free sniffing; SVG vectors use the explicit MIME path. The compiled WASM artifact is smoked in Node with `node scripts/wasm_smoke.mjs`, and the browser flows have a manual smoke page: `node scripts/serve_testpage.mjs` → http://localhost:8917 (includes a deliberately access-blocked second-origin image for the page-scan flow). Fuzzing: `cargo +nightly fuzz run manifest_parsing fuzz/corpus fuzz/corpus_seed`.

## Troubleshooting

- **"verification engine is not bundled"** (extension): build the engine into `extension/pkg/` (command above) and reload the extension.
- **"could not fetch the image (network failure or cross-origin restrictions)"**: the image's host does not allow the read. In page-scan mode this shows as the · · · marker with the allow-access offer; in the right-click flow, the asset was simply not examined — the verdict is an error, not a tier.
- **A validly signed image reports Tampered with "does not chain to a configured trust anchor"**: working as designed — you have no anchors configured (CLI without `--trust-anchors`), or the signer's root is not in your list. Unverifiable provenance is reported conservatively.
- **"media type … is not supported" / "unrecognized media type"**: the container is outside the supported set; the c2pa layer reports itself not evaluated rather than guessing.
- **My own photo says Inconclusive**: expected, and not a judgment. Your camera or editor did not embed Content Credentials (or a platform stripped them). Inconclusive means exactly what it says: no provenance data was found. This does NOT mean the asset is authentic — and it does not mean it isn't.
