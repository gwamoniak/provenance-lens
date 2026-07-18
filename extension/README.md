# Provenance Lens — browser extension (MV3)

The Manifest V3 extension (functional as of Milestone 3). Right-click any image → "Verify provenance with Provenance Lens": the service worker fetches the image bytes (the only network request the extension ever makes on the user's behalf), runs the bundled WASM engine with the trust anchors from `trust/anchors.pem` (a data file — empty placeholder until M4 settles trust-list distribution), stores the report, reflects the verdict on the action badge (VER/IND/INC/TAM), and opens the popup where the report renders with the engine's verbatim phrases. Errors (engine missing, fetch failed, HTTP error) are shown as errors, never as a verdict tier.

Load unpacked — Chrome: chrome://extensions → Developer mode → Load unpacked → this directory. Firefox (≥121): about:debugging → This Firefox → Load Temporary Add-on → select `manifest.json`. One codebase serves both (U5): the manifest declares both background keys (Chrome runs `background.js` as a module service worker, Firefox as an event page), and both scripts go through the `api` shim (`browser` where defined, else `chrome`) because Firefox's `chrome.*` is callback-style. In Firefox 121–126 the popup does not auto-open after verification (`action.openPopup()` is 127+); the badge + a click on the action icon is the flow. Full manual smoke script in `PROVENANCE_LENS_EXECPLAN.md` (M3): build the engine, copy the test CA into `trust/anchors.pem`, `node scripts/serve_testpage.mjs`, verify the corpus images at http://localhost:8917, restore the placeholder.

Build the engine into `pkg/` (gitignored, never committed) — full steps and wasm-opt flags in `.claude/skills/wasm-packaging/SKILL.md`:

    wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg
    wasm-opt extension/pkg/provenance_wasm_bg.wasm -Os --enable-bulk-memory ... (see skill)
    node scripts/wasm_smoke.mjs

The engine exports `verify_bytes(bytes, mediaType?, trustAnchorsPem?)` → JSON report; the extension passes its bundled trust-anchor PEM as the third argument (M3).

Rules for this directory (enforced in review):

- Verdict wording comes verbatim from `provenance-core/src/verdict.rs` / the `verdict-language` skill — no paraphrasing in UI strings.
- Permissions stay minimal: `contextMenus` + `activeTab`, no host permissions, no remote code, no analytics. Every permission addition needs an ExecPlan Decision Log entry.
- The extension never renders a verdict it did not compute; a missing engine says so.
