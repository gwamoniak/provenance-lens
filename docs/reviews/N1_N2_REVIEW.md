# N1/N2 implementation and review evidence

Prepared 2026-09-16 on `codex/n1-n2-evidence-reliability`. The branch was created from local main `5f1dcb6`, then fast-forwarded to the N0 candidate `5677e3d` before implementation. Main and the separate W3 worktree were not modified. This is a local implementation candidate, not a release or a cryptography sign-off.

## Result and boundaries

N1 replaces the watermark `Option` contract with `Detected`, `NotDetected`, and `Unavailable { reason }` throughout the detectors, tests, and calibration tools. A hit still yields only Indication. Without a hit, any incomplete configured probe yields NotEvaluated naming the missing check; only completed negatives yield NoSignal. A native build with the bzh capability and no supplied model explicitly reports that limitation. Calibration tools exclude unavailable checks from completed-check denominators.

The bzh loader now takes bytes; file I/O belongs to the CLI/measurement harness. It rejects any model except ONNX SHA-256 `4f7d80027d5d7ecf5c176fabc3d86c8ccd791cc5c830d23a96114425cd1f3e57` before parsing. That 44,705,871-byte export was recovered at `D:\sandbox\corpus\bzh\stable-signature-bzh-resnet18.onnx`; adjacent PyTorch weights match the earlier recorded SHA-256 `d11e6f1a0a339c973ac3a433c43d67d2ddb2f37f26c456061d1778ea7bf9f70e`. The vendor's positive/negative example pair passes through the pinned production loader. Input/output count, f32 shapes, empty output, RGB buffer validity, dimensions, runtime errors, and finite scores are checked. The exporter refuses unknown weights or a different export digest. Model files remain outside git; no inference-time download exists. Different exporters require an explicit review/calibration, not a caller-supplied digest override.

Operational defaults are 25 MiB asset bytes, 40 million decoded pixels for native watermark analysis, a 256 MiB decoder allocation budget, a 64 MiB model-read ceiling, a 15-second image-fetch deadline, and two active fetches plus 50 waiting jobs. CLI file reads use a bounded reader rather than trusting file metadata. The shared pipeline refuses oversized bytes before any layer runs, producing only NotEvaluated findings and no credential summary. Direct WASM calls use this same guard (the generated glue still copies the supplied bytes into WASM memory first). Streaming checks count actual received bytes, including missing-length responses. These are operational ceilings, not total-memory or worst-case parser-time guarantees.

N2 deletes the persistent URL report cache. Pending requests may share work only within a document/source generation; completed requests are fetched again with `cache: no-store`. Content-script generations and currentSrc checks prevent an old response from attaching to a replaced image. src/srcset/sizes, picture sources, image loads, viewport changes, removal/reinsertion, and resize eligibility are observed without periodic full-DOM polling. Pills are native buttons with accessible labels and retain keyboard focus behavior.

Scanning now requires an explicit site choice plus current host permission. A CDN grant authorizes fetching its files but never opts its pages into scanning. Existing grants are not migrated into implicit scanning consent. Stop scanning removes current pills and aborts active/queued page jobs while leaving the separate file-fetch grant intact. Host revocation also removes the scan choice, preventing a later one-file grant from silently re-enabling scanning. Consent writes are serialized; registration reconciliation cannot overwrite a newer choice. The message boundary checks extension identity, popup versus content-script origin, top-level frame, current tab URL, document identity/token, and current authorization. Reports identify retrieval time and fetched-file scope; they do not certify displayed pixels. The manifest and its install-time permission set are unchanged.

## Trust and dependency review

The C2PA SDK remains exactly 0.90.13. `layers/c2pa.rs`, production anchors, and signed fixtures are unchanged from N0. The review surface is the earlier execution boundary: a file over 25 MiB now bypasses all parsers, even if it would have contained a valid credential. This is explicitly reported as not examined, never as a completed negative. The standing human cryptography sign-off remains required before merging this boundary change.

The sole Cargo.lock change is adding an existing resolved package, `sha2 0.10.9`, to provenance-core's direct dependencies under its optional native model feature. No resolved package/version changed. Current lock SHA-256: `08570dfc2372bedc69dee9da93f3e5895c7457061cf61a1b9e1e0ed64c62b639`. Production anchor-file SHA-256: `bd884642cf69151739833b0e4489332f7b30d49a0545abaafa982927dc7f739d`. The public Rust detector result and model-construction APIs change deliberately; the JSON verdict/report shape and CLI exit-code precedence do not.

Playwright 1.62.1 is a test-only browser driver, installed into ignored build output in CI. It does not enter the extension or Rust runtime. CI runs the Node service tests, real WASM smoke, and Chromium/Firefox lifecycle harness. Its manually triggered model job requires the same pinned model digest and a bounded HTTPS download.

## Validation

From the repository root:

    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --locked --offline -- -D warnings
    cargo test --workspace --locked --offline
    cargo test -p provenance-core --no-default-features --locked --offline
    cargo check -p provenance-wasm --target wasm32-unknown-unknown --locked --offline
    node scripts/scan_support_test.mjs
    node scripts/background_test.mjs
    wasm-pack build crates/provenance-wasm --target web --out-dir ../../target/n1-n2-wasm --locked --offline
    node scripts/wasm_smoke.mjs target/n1-n2-wasm

For the recovered optional model, set `PROVENANCE_LENS_BZH_ONNX` to its local path and run:

    cargo test -p provenance-core --features stable-signature --test stable_signature --locked --offline -- --ignored

The default workspace suite passes 60 tests with one explicitly ignored model test; that model test passes when explicitly run with the pinned artifact (4.44 seconds for its positive/negative probes). The reduced-feature suite and strict lint pass. No full calibration rerun is claimed. Representative PNG fixtures are 768×768 / 980,216 bytes and 512×512 / 364,337 or 354,916 bytes, well below the chosen input ceilings.

All eleven JPEG/PNG/SVG vectors pass through the newly built standalone WASM. Oversized direct input yields only not-evaluated findings, then a normal signed file still verifies. Ten-thousand-level nested and truncated unsigned SVG samples produce Inconclusive and do not prevent the next signed verification; the slowest sampled examination in this run was 24.1 ms. This finite sample does not establish an adversarial CPU bound, so the 15-second network timeout is not represented as an interrupt for synchronous WASM. A killable product worker was not added without measured evidence requiring it.

The unoptimized release WASM is 6,947,729 raw / 2,334,154 gzip bytes, SHA-256 `cfb5f1942cc1870b4afbe4c4b0d38877d2b46f9ffbcd815aa388744e441ec425`, within the existing 7,000,000 / 2,500,000 budgets. This is a test artifact, not a packaged release.

To run real browser DOM tests, install Playwright 1.62.1 and its browsers (or use an existing installation). In PowerShell, an isolated installation is:

    npm install --prefix target/browser-tests --no-save playwright@1.62.1
    $env:NODE_PATH = "$PWD/target/browser-tests/node_modules"
    $env:PLAYWRIGHT_BROWSERS_PATH = "$PWD/target/test-browsers"
    node target/browser-tests/node_modules/playwright/cli.js install chromium firefox
    node scripts/scan_browser_test.mjs target/n1-n2-wasm

Observed browser passes: Chromium 151.0.7922.34 and Firefox 153.0. The harness runs the actual content script, real DOM observers, keyboard events, streamed fetch helper, and real WASM with an isolated test CA. Only extension messaging is substituted; `background_test.mjs` separately exercises the actual background message code with copied-value storage and mocked permission/tab APIs. Covered: signed→unsigned replacement, srcset/sizes/viewport selection, delayed old responses, changed bytes at the same URL, detach/reinsert, disable during pending work, re-enable, and Enter activation. Chromium may retain a cached higher-resolution srcset candidate; tests use distinct candidate URLs and assert the badge matches currentSrc. Firefox required running the isolated headless process outside the restricted Windows sandbox. No production browser profile or installed extension was changed.

Manual page: `node scripts/serve_testpage.mjs`, then `http://localhost:8917/lifecycle`. Use an isolated test extension copy for test-root trust; do not replace production anchors. Live browser permission prompts and store installation were not retested. Remote GitHub CI awaits an authorized push. New full fuzzing and advisory audits were not run in this task.

## Maintainer decision

Review the bounded pipeline execution change and the concrete evidence above before recording cryptography approval in `PROVENANCE_LENS_EXECPLAN.md`. The user's request to implement N1/N2 authorizes the specified consent redesign and local implementation; it does not imply a merge, push, or release. N0's prior review packet remains part of this branch's prerequisite history.
