# Brave background loading fix — 2026-09-20

Scope: `codex/brave-worker-loading`, branched from local main `6cb516d`.
Local development fix only; no push, merge, release or permission expansion.

## Cause and correction

The installed MV3 worker rejected `import()` in `background.js` before an image
was examined or a result saved. The shared engine loader also used a transitive
dynamic import. Existing Node policy tests substituted both imports, and browser
DOM tests substituted the extension transport, so neither exercised this rule.

Both dependencies and the generated WASM JavaScript now use static imports.
WASM initialization and trust-bundle reads remain lazy and retryable. Removed
the redundant engine/service promise wrappers. Firefox uses `background.html`
with a module script instead of its classic `background.scripts` entry. The
packager requires that HTML file in the extracted archive. No Rust, SDK, lock,
verdict, trust-anchor, permission, model or network-policy changes.

## Verification

- New `scripts/extension_worker_test.mjs` fails on the previous extracted build
  with the user's exact `import() is disallowed on ServiceWorkerGlobalScope`
  error. It passes on the fix in Chromium 151.0.7922.34 and the installed Brave
  executable (Chromium 153.0.8010.53), using disposable profiles, the actual
  toolbar popup, module graph, runtime transport, production anchors and WASM.
- Tests check an unsigned image, rejection of the synthetic signer under
  production anchors, scan permission denial/stop, and fresh verification
  after the worker is explicitly stopped. Added this test to ordinary CI;
  the updated remote workflow has not been run or pushed.
- Node scan-helper, background-policy, inspector and package-gate tests pass.
- Extracted package: all 12 WASM vectors, oversized-input recovery and hostile
  SVG recovery pass. The engine and production-anchor hashes are unchanged.
- Existing Chromium/Firefox inspector and image-lifecycle suites pass.
  These retain their documented transport/page substitutions.
- Firefox accepted the temporary installation without manifest warnings and
  reported its background running. The separate installed-popup diagnostic
  timed out navigating to the extension page; this is not full Firefox
  installed-extension acceptance evidence.

Still manual: native permission-dialog approval, enabled scanning and the native
right-click menu in the user's profile, plus full Firefox installed-extension
acceptance. The Brave regression exercises the real popup's verification
message, not native menu selection. No detection-accuracy or Claude claim is
established by these checks. Development artifacts record a dirty source tree
until the fix is committed; archive hashes identify the actual supplied bytes.
