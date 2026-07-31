# Provenance Lens — user manual

This manual covers the `lens` command-line tool, the browser extension (including page scanning), the JSON output, and the npm package. Wording note: the four verdict phrases quoted here are the product's contract and appear character-identical in every surface; a CI test enforces it.

## What this tool is, and is not

Provenance Lens validates **C2PA Content Credentials** — cryptographically signed provenance manifests that some cameras, editors, and AI generators embed in media files. When credentials are present and valid, that is provable. When they are absent, **nothing is provable in either direction**, and the tool says so. It does not analyze pixels, does not score "AI likelihood", and never calls anything authentic.

Two related honesty notes. First, invisible watermarks such as Google's SynthID cannot be checked by this or any third-party tool: there is no public spec, decoder, or API, so only the vendor's own infrastructure can verify them — which is why the watermark layer reports itself as not evaluated. Second, ecosystem context: the EU AI Act's Article 50 transparency obligations (in force from 2 August 2026) require *generators* to mark AI content in machine-readable form; nothing is required of verifiers or their users. Provenance Lens sits on the independent verification side of that ecosystem, checking such marks locally where a runnable, spec-known verifier exists — today that means C2PA.

## The four verdicts

- **Verified**: this asset carries a valid, cryptographically signed provenance chain.
  The manifest's signature is cryptographically valid AND its certificate chains to one of your configured trust anchors. This vouches for the provenance chain — who signed it and that the bytes are unmodified since — not for the content being true or human-made. A Verified report also states what the credential *claims* (see "Credential claims" below).
- **Indicated**: signals suggest AI involvement, but no cryptographic proof chain is present.
  Reserved for the gated non-cryptographic layers (watermark, registry). With those layers gated, you will not see this verdict today.
- **Inconclusive**: no provenance data was found. This does NOT mean the asset is authentic.
  The common case for genuine and generated images alike — most images on the web carry no credentials, often because platforms strip them on upload.
- **Tampered**: provenance data is present but fails validation. Treat this asset with suspicion.
  Includes: content modified after signing (hash mismatch), broken manifests, expired signing credentials — and also cryptographically valid signatures whose certificate does not chain to any of your trust anchors (unverifiable provenance is treated conservatively, not charitably).

## The CLI: `lens`

Build it with `cargo build -p provenance-cli --release` (binary at `target/release/lens`), or run in place with `cargo run -p provenance-cli --`.

    lens verify [--json] [--trust-anchors <PEM>] <FILE>...
    lens tiers

`lens tiers` prints the four verdicts with their exact phrases.

`lens verify` examines each file and prints a report per file. Flags may appear in any order:

- `--trust-anchors <PEM>` — a PEM bundle of root certificates that signature chains may validate against. **Without it, no chain can validate as trusted**, so even validly signed assets report Tampered (unverifiable provenance). The ecosystem's canonical anchor set is the official C2PA conformance trust list — the copy the extension ships is at `extension/trust/anchors.pem`, refreshed via `sh scripts/update_trust_list.sh`.
- `--json` — one JSON object per line per file (shape below) instead of the human report.

Exit code is the highest per-file code — the worst result wins:

    0 verified    10 indicated    20 inconclusive    30 tampered    2 usage-or-IO error

Example (against the repo's test corpus; `V=crates/provenance-core/tests/vectors`):

    $ lens verify --trust-anchors $V/test_ca.pem $V/valid_signed.jpg
    crates/provenance-core/tests/vectors/valid_signed.jpg
      verdict: Verified: this asset carries a valid, cryptographically signed provenance chain.
      [c2pa] valid provenance chain, issuer: …
      [watermark] not evaluated — gated: no vendor watermark detector integrated yet
      [registry] not evaluated — gated: no transparency-log registry deployed yet
      [heuristics] not evaluated — optional layer, not implemented
      credential claims:
        claim generator: provenance-lens test vectors/0.1.0
        declared source type: http://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicMedia
        note: the credential declares this content AI-generated

Supported formats (hinted by file extension, or sniffed from bytes when the extension is unknown): JPEG, PNG, WebP, GIF, AVIF. Anything else reports the c2pa layer as "not evaluated" with the reason — honestly distinct from "looked and found nothing".

### Credential claims

Only Verified reports carry this section, because only a validated manifest is worth quoting. It is descriptive, never an endorsement: the claim generator ("name/version" of the signing software), the signing time when the credential carries a timestamp, and the declared digitalSourceType verbatim. For the IPTC generative-AI source types a fixed plain-language note is added (e.g. "the credential declares this content AI-generated"). The credential says it; the tool relays it.

### JSON output

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

Right-click any image → **"Verify provenance with Provenance Lens"**. The extension fetches that image's bytes — the only network request it ever makes on your behalf — runs the bundled engine locally, shows the verdict on the toolbar badge (VER green / IND amber / INC gray / TAM red; ERR black for errors), and opens the popup with the full report: the verbatim phrase, per-layer findings, credential claims when Verified, and a line stating whether trust anchors are loaded. Errors are shown as errors, never dressed as a verdict.

### Page scanning (opt-in, per site)

Off by default, everywhere. To enable it for a site: open the popup on that site → **Page scanning** → "Scan images on \<host\>" → accept the browser's own consent prompt → reload the page. Every image shown on that site is then examined locally as it becomes visible; image bytes never leave your device.

What you will see on the page, as small text pills on each examined image:

- **VER / IND / INC / TAM** in the tier colors — hover for the exact phrase, click for the full report in the popup. Small icons/decoration images (under 64×64 as rendered) are skipped.
- **· · ·** (gray, dashed) — *not examined*: the image is hosted somewhere your grant does not cover (a CDN, typically), so its bytes could not be read. Honest absence, never a guess. Clicking it opens the report with the reason; the popup then offers **"Allow access to \<image-host\> and verify"**, which re-examines the image under the new grant.
- **ERR** (black) — an error other than access (e.g. the engine is missing). The report has the specifics.

Expect a lot of gray INC pills — that is the point. Most images have no credentials, frequently because platforms strip them; seeing that plainly, at scale, is what this tool exists to show.

To stop: popup → "Stop scanning \<host\>" → reload. Grants live in your browser's own permission store (also removable from the browser's extension settings); the extension keeps no list of its own, and its per-URL result cache clears when the browser closes.

### Permissions, plainly

`contextMenus` (the right-click entry), `activeTab` (fetch the image you right-clicked, from the page you are on), `storage` (session-only: last result + verdict cache), `scripting` (register the page-scan script, only for sites you granted). Host access is exclusively opt-in per site at runtime — nothing at install. No analytics, no remote code.

## The npm package

The engine is packaged as `@provenance-lens/verify-wasm` (`sh scripts/package_npm.sh` → tarball in `dist/`). API, JSON shape, and wording are identical to everything above; usage examples for Node and bundlers are in the package README (`crates/provenance-wasm/README.md`). Remember the trust-anchor rule: pass a PEM bundle, or nothing can verify as trusted.

## Building and testing from source

    cargo fmt --all
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace          # includes corpus, parity, cert-policy, wording audits

The test corpus (`crates/provenance-core/tests/vectors/`, eight vectors, JPEG + PNG) is generated by a self-verifying tool — `cargo run -p provenance-core --example gen_vectors` — that refuses to write a vector whose verdict does not match its catalogue entry. The compiled WASM artifact is smoked in Node with `node scripts/wasm_smoke.mjs`, and the browser flows have a manual smoke page: `node scripts/serve_testpage.mjs` → http://localhost:8917 (includes a deliberately access-blocked second-origin image for the page-scan flow). Fuzzing: `cargo +nightly fuzz run manifest_parsing fuzz/corpus fuzz/corpus_seed`.

## Troubleshooting

- **"verification engine is not bundled"** (extension): build the engine into `extension/pkg/` (command above) and reload the extension.
- **"could not fetch the image (network failure or cross-origin restrictions)"**: the image's host does not allow the read. In page-scan mode this shows as the · · · marker with the allow-access offer; in the right-click flow, the asset was simply not examined — the verdict is an error, not a tier.
- **A validly signed image reports Tampered with "does not chain to a configured trust anchor"**: working as designed — you have no anchors configured (CLI without `--trust-anchors`), or the signer's root is not in your list. Unverifiable provenance is reported conservatively.
- **"media type … is not supported" / "unrecognized media type"**: the container is outside the supported set; the c2pa layer reports itself not evaluated rather than guessing.
- **My own photo says Inconclusive**: expected, and not a judgment. Your camera or editor did not embed Content Credentials (or a platform stripped them). Inconclusive means exactly what it says: no provenance data was found. This does NOT mean the asset is authentic — and it does not mean it isn't.
