---
name: webextension-mv3
description: Manifest V3 constraints, CSP rules, and store-review requirements for the extension/ directory. Load before extension work (Milestone 3/4) or any manifest change.
---

# WebExtension (Manifest V3)

## The constraints that shape the design

- **Service worker, not background page**: `background.js` is killed and restarted constantly. No in-memory state that matters; re-create the context menu in `onInstalled`, re-derive everything else per event. WASM must be (re)instantiated cheaply — load lazily on first verify, not at worker start.
- **CSP forbids remote code**: everything ships in the package — the WASM bundle (`extension/pkg/`), all JS, all CSS. `wasm-unsafe-eval` may be needed in `content_security_policy.extension_pages` for WebAssembly compilation; verify against current Chrome behavior at M3 time and record it.
- **Permissions are the review surface**: `contextMenus` + `activeTab` today. `activeTab` grants temporary access only after a user gesture — which is exactly the interaction model we want. Adding host permissions or `<all_urls>` (needed for the proposal's automatic badge UI) is a store-review and user-trust event: ExecPlan Decision Log first.

## Flagship UX (from the proposal) vs the wedge

The proposal's end state is a content script that scans `<img>`/`<video>` and overlays badge UI (green Verified / yellow Indicated / gray Inconclusive / red Tampered). The wedge (M3) ships the context-menu flow because it works under `activeTab` alone. The badge UI is post-wedge and gated on the permissions decision above — don't build it early.

## Privacy rules (from the proposal's trust model; non-negotiable)

- Images are never uploaded anywhere. Manifest parsing and hashing happen locally in WASM.
- The only network request the wedge extension makes is fetching the image the user explicitly asked to verify (its `srcUrl`).
- Post-wedge Layers 2–3 may send perceptual hashes only, never bytes, and only with explicit user consent.
- No analytics, no telemetry, no error reporting that leaves the device.

## Store review readiness (M4)

Chrome Web Store requires a privacy policy, permission justifications, and increasingly rejects vague descriptions — the honest verdict language is an asset here; the listing obeys the verdict-language skill. Firefox (post-wedge) reviews source: keep the build reproducible (`wasm-pack` command documented in extension/README.md). Keep a CHANGELOG entry per submitted version.

## Testing

Manual smoke script lives in the ExecPlan (M3). Cover: verify on a signed image, a stripped image, a corrupted image; engine-missing state; fetch-failure state (image behind auth); service-worker restart mid-session (chrome://serviceworker-internals → stop, then verify again).
