# Provenance Lens — browser extension (MV3)

The Manifest V3 extension (functional as of Milestone 3). Right-click any image → "Verify provenance with Provenance Lens": the service worker fetches the image bytes (the only network request the extension ever makes on the user's behalf), runs the bundled WASM engine with the trust anchors from `trust/anchors.pem` (a data file — empty placeholder until M4 settles trust-list distribution), stores the report, reflects the verdict on the action badge (VER/IND/INC/TAM), and opens the popup where the report renders with the engine's verbatim phrases. Errors (engine missing, fetch failed, HTTP error) are shown as errors, never as a verdict tier.

Load unpacked — Chrome: chrome://extensions → Developer mode → Load unpacked → this directory. Firefox (≥128; the floor comes from `optional_host_permissions`, which the U7 page-scan opt-in requires): about:debugging → This Firefox → Load Temporary Add-on → select `manifest.json`. One codebase serves both (U5): the manifest declares both background keys (Chrome runs `background.js` as a module service worker, Firefox as an event page), and both scripts go through the `api` shim (`browser` where defined, else `chrome`) because Firefox's `chrome.*` is callback-style. Full manual smoke script in `PROVENANCE_LENS_EXECPLAN.md` (M3): build the engine, copy the test CA into `trust/anchors.pem`, `node scripts/serve_testpage.mjs`, verify the corpus images at http://localhost:8917, restore the placeholder.

Page-scan plumbing (U7a, maintainer-approved design in `PROVENANCE_LENS_UPGRADES_EXECPLAN.md`): `content/scan.js` is registered per granted origin only (`scripting.registerContentScripts`, synced to `permissions.getAll()` — the browser's permission store is the single source of truth; nothing is granted at install). The background answers `{ type: "pl-verify", url }` with a report entry, capped at 2 concurrent verifications with a 200-entry session cache (`lib/scan_support.js`, Node-tested by `scripts/scan_support_test.mjs`). The content script is inert until U7b lands the scanning UI.

Build the engine into `pkg/` (gitignored, never committed) — full steps and wasm-opt flags in `.claude/skills/wasm-packaging/SKILL.md`:

    wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg
    wasm-opt extension/pkg/provenance_wasm_bg.wasm -Os --enable-bulk-memory ... (see skill)
    node scripts/wasm_smoke.mjs

The engine exports `verify_bytes(bytes, mediaType?, trustAnchorsPem?)` → JSON report; the extension passes its bundled trust-anchor PEM as the third argument (M3).

Rules for this directory (enforced in review):

- Verdict wording comes verbatim from `provenance-core/src/verdict.rs` / the `verdict-language` skill — no paraphrasing in UI strings.
- Permissions stay minimal: `contextMenus` + `activeTab` + `storage` + `scripting`, no INSTALL-TIME host permissions, no remote code, no analytics. Host access is exclusively `optional_host_permissions`, granted per site at runtime through an explicit user gesture (U7 Decision Log). Every permission addition needs an ExecPlan Decision Log entry.
- The extension never renders a verdict it did not compute; a missing engine says so.
