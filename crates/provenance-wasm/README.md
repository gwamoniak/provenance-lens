# @provenance-lens/verify-wasm

Honest C2PA provenance verdicts in WebAssembly. One function: bytes in, JSON report out — computed entirely locally.

Verdicts come in four tiers, and the wording is part of the contract:

- **Verified**: this asset carries a valid, cryptographically signed provenance chain.
- **Indicated**: signals suggest AI involvement, but no cryptographic proof chain is present.
- **Inconclusive**: no provenance data was found. This does NOT mean the asset is authentic.
- **Tampered**: provenance data is present but fails validation. Treat this asset with suspicion.

The founding rule: **no provenance data ≠ authentic**. Most genuine images carry no Content Credentials; for them the honest answer is Inconclusive, and that is what you will get. This package validates C2PA manifests cryptographically — it does not guess from pixels, score "AI likelihood", or call anything authentic.

## Usage

The artifact is a wasm-pack `web`-target build: an ES-module JS glue plus the `.wasm` binary. You initialize it with the wasm bytes (Node) or a URL (browser), then call `verify_bytes`.

Node:

    import init, { verify_bytes } from "@provenance-lens/verify-wasm";
    import { readFileSync } from "node:fs";

    const wasm = new URL(import.meta.resolve("@provenance-lens/verify-wasm/provenance_wasm_bg.wasm"));
    await init({ module_or_path: readFileSync(wasm) });

    const anchorsPem = readFileSync("trust-anchors.pem", "utf8"); // your trust list
    const report = JSON.parse(verify_bytes(readFileSync("photo.jpg"), undefined, anchorsPem));
    console.log(report.verdict, "—", report.phrase);

Browser / bundler:

    import init, { verify_bytes } from "@provenance-lens/verify-wasm";
    await init(); // fetches the .wasm next to the glue module

## API

    verify_bytes(bytes: Uint8Array, mediaType?: string, trustAnchorsPem?: string) -> string (JSON)

- `mediaType` is an optional MIME hint (`"image/jpeg"`, …). Omit it and the engine sniffs the container (JPEG, PNG, WebP, GIF, AVIF).
- `trustAnchorsPem` is a PEM bundle of root certificates that signature chains may validate against. **Without it, nothing can verify as trusted** — cryptographically valid but unanchored provenance reports as Tampered (unverifiable), which is the conservative reading. The [C2PA conformance trust list](https://github.com/c2pa-org/conformance-public) is the ecosystem's canonical anchor set.
- The returned JSON: `{ "verdict": "verified"|"indicated"|"inconclusive"|"tampered", "phrase": <the tier phrase verbatim>, "credentials": { … only on Verified: issuer, claim_generator, signing_time, digital_source_type, source_type_note … }, "findings": [ { "layer", "status", "detail" } … ] }`. Report what the `phrase` says; do not soften it.

## Privacy and scope

Verification is sans-IO: this module never fetches anything, and image bytes never leave the process. Layer 1 (C2PA cryptographic validation) is the only layer that runs today; watermark, registry, and heuristic layers report `not_evaluated` honestly rather than pretending to run.

Part of [Provenance Lens](https://github.com/gwamoniak/provenance-lens). MIT OR Apache-2.0.
