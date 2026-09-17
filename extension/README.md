# Provenance Lens — browser extension (MV3)

Right-click an image → "Verify provenance with Provenance Lens": the background fetches the file, runs the bundled WASM engine with production anchors from `trust/anchors.pem`, and displays the report and retrieval time. This checks the fetched bytes, not the page's displayed pixels. **Verify again** fetches anew. Image bytes are never uploaded; errors are not rendered as verdict tiers. Fetches have a 15-second deadline and a streamed 25 MiB limit.

Load unpacked — Chrome: chrome://extensions → Developer mode → Load unpacked → this directory. Firefox (≥128; the floor comes from `optional_host_permissions`): about:debugging → This Firefox → Load Temporary Add-on → select `manifest.json`. One codebase serves both: Chrome runs `background.js` as a module service worker, Firefox as an event page. Both use the `api` shim (`browser` where defined, else `chrome`). Build the engine first; use the isolated test procedure below for fixture verification without changing production anchors.

N2 page scanning requires both an explicit site choice in `storage.local.scanEnabledSites` and current browser host permission. CDN fetch permission alone never enables page scanning. Changes apply to open tabs without reload; disabling/revoking scanning clears pills, invalidates generations, and aborts queued/active page jobs. The manifest permissions have not grown. The old URL report cache is removed. At most two fetches run and 50 jobs wait; only pending work for the same document/source generation is shared. Messages carry a document token and image generation and are checked against the extension sender, top-level frame, current tab URL/document, and current consent. Source/srcset/sizes/load/viewport changes invalidate old pills. Pills are real keyboard-accessible buttons.

Run `node scripts/scan_support_test.mjs` and `node scripts/background_test.mjs` for bounds, abort, consent, sender, and copied-storage regressions. `node scripts/scan_browser_test.mjs <wasm-pkg>` exercises the actual content script in Chromium and Firefox with a controlled message transport and real WASM verification. The manual lifecycle page is `http://localhost:8917/lifecycle` after `node scripts/serve_testpage.mjs`. Use test anchors only in an isolated test copy; keep the production bundle unchanged. The human trust-boundary review remains in `docs/reviews/N1_N2_REVIEW.md`.

The popup's **Inspect or compare local files** link opens `inspect/inspect.html`. It uses native file selection/drop, the shared `lib/engine.js` loader, Web Crypto digests, and native Blob downloads; no new permissions or network service. It compares at most two files and previews unsigned receipts. Test with `node scripts/inspect_test.mjs` and, using the existing Playwright setup, `node scripts/inspect_browser_test.mjs <wasm-pkg>`. The latter exercises the real inspector/WASM offline in Chromium and Firefox; see `docs/reviews/N3_N4_REVIEW.md` and `docs/CLAUDE_MARKS.md` for evidence and limits.

Build the engine into `pkg/` (gitignored, never committed) — full steps and wasm-opt flags in `.claude/skills/wasm-packaging/SKILL.md`:

    wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg
    wasm-opt extension/pkg/provenance_wasm_bg.wasm -Os --enable-bulk-memory ... (see skill)
    node scripts/wasm_smoke.mjs

The engine exports `verify_bytes(bytes, mediaType?, trustAnchorsPem?)` → JSON report; the extension passes its bundled trust-anchor PEM as the third argument (M3).

The worker passes the fetched response's MIME type to the engine, so C2PA-bearing SVG images are supported when the server identifies them as `image/svg+xml`; JPEG, PNG, WebP, GIF, and AVIF can also be recognized from their binary containers. Signed Claude exports follow this same generic path. If a Claude export has no C2PA manifest, the result is Inconclusive—not an authorship judgment—and the extension does not claim to detect Claude's separate text watermark.

Rules for this directory (enforced in review):

- Verdict wording comes verbatim from `provenance-core/src/verdict.rs` / the `verdict-language` skill — no paraphrasing in UI strings.
- Permissions stay minimal: `contextMenus` + `activeTab` + `storage` + `scripting`, no INSTALL-TIME host permissions, no remote code, no analytics. Host access is exclusively `optional_host_permissions`, granted per site at runtime through an explicit user gesture (U7 Decision Log). Every permission addition needs an ExecPlan Decision Log entry.
- The extension never renders a verdict it did not compute; a missing engine says so.
