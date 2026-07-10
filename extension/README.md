# Provenance Lens — browser extension (MV3)

Skeleton for the Manifest V3 extension. It loads unpacked today (chrome://extensions → Developer mode → Load unpacked → this directory) and shows the four verdict tiers; actual verification activates in Milestone 3 once the WASM engine is bundled.

Build the engine into `pkg/` (gitignored, never committed) — full steps and wasm-opt flags in `.claude/skills/wasm-packaging/SKILL.md`:

    wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg
    wasm-opt extension/pkg/provenance_wasm_bg.wasm -Os --enable-bulk-memory ... (see skill)
    node scripts/wasm_smoke.mjs

The engine exports `verify_bytes(bytes, mediaType?, trustAnchorsPem?)` → JSON report; the extension passes its bundled trust-anchor PEM as the third argument (M3).

Rules for this directory (enforced in review):

- Verdict wording comes verbatim from `provenance-core/src/verdict.rs` / the `verdict-language` skill — no paraphrasing in UI strings.
- Permissions stay minimal: `contextMenus` + `activeTab`, no host permissions, no remote code, no analytics. Every permission addition needs an ExecPlan Decision Log entry.
- The extension never renders a verdict it did not compute; a missing engine says so.
