# Provenance Lens upgrades: deepen the shipped wedge — machine-readable verdicts, richer credential reporting, proven format coverage, CI, and wider distribution

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

This document is maintained in accordance with `PLANS.md`, checked into the sibling repository `../solid-broccoli/PLANS.md` (path relative to this repo's root on the current development machine; the sandbox keeps the ExecPlan rules in one place). If that file is not in your context, read it in full before revising this plan. The wedge that this plan builds on was delivered under `PROVENANCE_LENS_EXECPLAN.md` in this repository — that plan remains the authoritative record of the wedge (M0–M4), of the honesty rules, and of the gated milestones M5–M7 (watermark, registry, heuristics), which this plan does NOT touch. Where this plan needs a rule from the wedge plan, it restates the rule here so this document stays self-contained.

## Purpose / Big Picture

The shipped wedge gives a user one honest verdict about one image at a time: `lens verify photo.jpg` in a terminal, or right-click → verify in the browser extension. After this plan, the same honest engine becomes something other software and more people can build on: scripts consume verdicts as JSON without scraping human-oriented text; verdicts on *verified* assets also say what the credentials actually claim (who generated the asset, whether AI involvement was declared, when it was signed) instead of only "a valid chain exists"; the formats the tools accept are proven by test vectors rather than assumed; every push is checked by CI instead of by whoever remembered to run the loop; and the extension reaches Firefox users while the engine reaches JavaScript developers as an npm package. The founding rule is unchanged and non-negotiable everywhere: **no provenance data ≠ authentic**, and no upgrade may add visual or verbal language of safety around the Inconclusive tier.

## Progress

- [x] (2026-07-18) Plan authored, after the post-wedge hardening pass (branch `post-wedge-hardening`: cert-policy pinning tests, fuzz target, trust-list refresh workflow — see the wedge plan's Progress for details).
- [x] (2026-07-18) U1 complete — `render_json(report, file?)` lives in `crates/provenance-core/src/json.rs`; the WASM wrapper is now logic-free (delegates, shape byte-identical — parity suite passed untouched, so no engine rebuild was needed); `lens verify` takes `[--json] [--trust-anchors <PEM>] <FILE>...` with flags in any order and exits with the highest per-file code. Suite 27/27 green (2 new json tests, 2 new CLI parser tests), clippy clean; acceptance transcript in Artifacts.
- [x] (2026-07-18) U2 complete — `Report.credentials: Option<CredentialSummary>` (Some exactly when Verified; pinned corpus-wide and by dedicated tests), extracted by the concrete Layer-1 instance the pipeline now retains; rendered in CLI human output ("credential claims:" block), in the shared JSON (`credentials` object, absent keys omitted), and in the popup (from the engine JSON, textContent only). Suite 30/30 green, clippy clean; acceptance in Artifacts. Note: the browser shows the block only after the engine is next rebuilt (wasm-pack not installed on this machine — engine-side behavior proven by the parity and JSON tests; the release packaging script rebuilds regardless).
- [x] (2026-07-18) U3 complete — corpus grown to eight vectors (PNG trio: signed / caBX-stripped / caBX-corrupted-with-CRC-refresh, all self-verified by the generator); corpus test, parity suite, and wasm smoke now run HINT-FREE so the whole corpus doubles as the sniffing test; `sniff_media_type` gained AVIF (`ftyp` brand) and the CLI gained `.gif`, aligning both sides on jpeg/png/webp/gif/avif; smoke page and its content types are derived from `manifest.tsv` so they cannot go stale. True-artifact smoke green through a fresh wasm-pack build on this machine (wasm-pack installed; 8/8 PASS). Unoptimized artifact 7,031,523 B raw / 2,204,797 B gz — the release script's wasm-opt step (binaryen, not on this machine) brings raw below the ≤7 MB budget; gz is already within ≤2.5 MB. Suite 30/30 green, clippy clean.
- [x] (2026-07-18) U4 complete — `.github/workflows/ci.yml`: on every push/PR, the exact local loop (fmt check, clippy `-D warnings`, full test suite) plus a wasm32 type-check of the engine crate; weekly scheduled job runs cargo-audit and a 5-minute fuzz smoke, both ADVISORY for now (`continue-on-error`) — audit because enforcing on advisories is a maintainer decision, fuzz because it re-finds the known upstream c2pa JUMBF panic and would sit permanently red until the upstream fix ships in a version bump (flip to enforcing then). Only external action: `actions/checkout`; the runner's rustup honors `rust-toolchain.toml`. Acceptance proven live: the `main` push ran green on GitHub, and a scratch branch with a deliberate fmt violation ran red (failing at the first gate), then was deleted.
- [x] U5 — Firefox port and AMO listing collateral. **Complete: the maintainer ran the Firefox browser smoke and reported it passed ("firefox smoke passed", 2026-07-18) — the acceptance bar (all vectors + failure states in a real Firefox) is met.** Implemented (2026-07-18): dual background keys in the manifest (Chrome runs `background.js` as a module service worker, Firefox ≥121 as an event page — the file already had no static imports and registers listeners synchronously, so one script serves both), `api` namespace shim in `background.js` and `popup.js` (`browser` where defined, else `chrome` — Firefox's `chrome.*` is callback-style and would break the popup's `.then` chain), `browser_specific_settings.gecko` (id `provenance-lens@gwamoniak.github.io`, `strict_min_version` 121.0), AMO section in `docs/STORE_LISTING.md` (ID pinning, version floor, reviewer notes), Firefox loading instructions in `extension/README.md`; both scripts pass `node --check`, manifest valid, suite 30/30, Chrome-side CI green. **Remaining: the acceptance bar — the manual browser smoke in a real Firefox (about:debugging → Load Temporary Add-on; all eight vectors + failure states) — is the maintainer's step, mirroring M3's Chrome smoke. On Firefox 121–126 the popup does not auto-open (openPopup is 127+); badge + click is the expected flow.**
- [x] (2026-07-18) U6 complete — `sh scripts/package_npm.sh`: wasm-pack `web`-target build into `dist/npm-pkg/`, best-effort wasm-opt (warns and proceeds when binaryen is absent), package.json patched to the publishable name `@provenance-lens/verify-wasm` (maintainer may rename BEFORE first publish, never after), corpus smoke through the exact packed artifact (`scripts/wasm_smoke.mjs` now takes a pkg-dir argument, defaulting to the extension engine), then `npm pack` → `dist/provenance-lens-verify-wasm-0.1.0.tgz` (2.4 MB packed / 7.0 MB unpacked, 5 files). The crate README ships as the npm README (honest wording; its four tier phrases are now covered by the wording-sync audit as a fourth location). Acceptance proven: a scratch Node project installed the tarball by name and reproduced all eight corpus verdicts through the README's documented usage. **`npm publish` is the maintainer's action** (the scoped name also needs the npm org to exist first).
- [x] U7 — page-scan badge UX behind an explicit opt-in site-access grant. **Complete: the maintainer ran the U7 browser smoke (Chrome + Firefox) and reported it passed ("U7 smoke passed", 2026-07-18) — grant flow, lazy pills on all eight vectors, the blocked-host marker → allow-and-verify path, and the revoke-returns-to-wedge regression all verified in real browsers.** Design approved by the maintainer 2026-07-18 (Decision Log). **U7a complete (2026-07-18)**: manifest gains `scripting` + `optional_host_permissions: ["*://*/*"]` (nothing granted at install) and the Firefox floor rises to 128 (`optional_host_permissions` lands there — Decision Log); the background registers/updates/unregisters `content/scan.js` per granted origin, synced to `permissions.getAll()` on `permissions.onAdded`/`onRemoved`/`onInstalled`/`onStartup`; the `pl-verify` message service answers content scripts with report entries through a 2-slot FIFO concurrency limiter and a 200-entry capped session cache (`extension/lib/scan_support.js`, dynamically imported so the background stays static-import-free; logic proven by `node scripts/scan_support_test.mjs` — cap, FIFO order, slot release on failure, eviction, refresh-on-reput all asserted). Collateral (extension README, store listing permission justifications, CLAUDE.md surface rule) updated. **U7b complete (2026-07-18)**: `content/scan.js` is live — IntersectionObserver (64 px minimum rendered size, images left observed until they qualify) + MutationObserver for SPA-inserted images, one verification per URL however many `<img>` repeat it, pills rendered from the background-supplied spec in closed shadow roots anchored to `document.body` (rAF-throttled repositioning on scroll/resize; pills whose image leaves the DOM are dropped), pill click → `pl-show` → full report in badge + popup. Presentation is single-sourced: `TIER_BADGE`/`pillSpec`/`actionBadge` live in `lib/scan_support.js`, the background computes the pill spec in its `pl-verify` response, and the Node test asserts every entry shape including the unknown-verdict fallthrough to ERR (never an invented tier). Two design refinements recorded in the Decision Log (grant affordance moved to the popup; spec-from-background). **U7c implemented (2026-07-18)**: the popup gains a "Page scanning" section for the active tab's host — "Scan images on <host>" fires `permissions.request` inside a real user click (the browser renders the consent prompt), the enabled state offers "Stop scanning <host>" via `permissions.remove`, state is read fresh from `permissions.contains` every render, and an honest "Reload the page to apply the change" note appears after either change (registration applies to future page loads). The error view gains the per-host affordance from the U7b Decision Log: a not-fetched entry offers "Allow access to <image-host> and verify", which re-examines under the new grant (the background no longer caches failed entries — a cached failure would have replayed forever). The smoke page embeds a ninth image served from the `127.0.0.1` hostname WITHOUT a CORS header (`/nocors/` route) — a genuinely blocked image host, verified with curl (no ACAO header on that route, `*` on the others). **Remaining — the U7 acceptance: the maintainer's browser smoke below.**

The U7 browser smoke (run after the M3 smoke steps — engine built, test CA in `extension/trust/anchors.pem`, `node scripts/serve_testpage.mjs`, extension loaded in Chrome and Firefox ≥128):

    1. Regression first: with NO grants, the context-menu flow on the eight corpus images
       behaves exactly as before (this is the "no grants = today's extension" guarantee).
    2. Open the popup on http://localhost:8917 → "Page scanning" shows
       "Scan images on localhost" → click → accept the browser prompt → reload the page.
    3. Scroll the page: each of the eight corpus images gains its pill lazily as it becomes
       visible (VER / INC / TAM per the captions; text pills, tier colors, no icons).
       Hover a pill → the verbatim approved phrase. Click a pill → badge + popup show the
       full report (credential summary on the Verified ones).
    4. The ninth image (served from 127.0.0.1, no CORS) shows the gray dashed "· · ·"
       marker. Click it → popup shows the honest fetch error and the button
       "Allow access to 127.0.0.1 and verify" → click, accept → the report re-renders as
       Verified; after a reload the image gets a VER pill like the rest.
    5. Popup → "Stop scanning localhost" → reload → no pills, and the context-menu flow
       still works (revocation returns to the wedge behavior).
    6. Firefox repeat: steps 2–5 in Firefox ≥128 (the grant prompt is Firefox's own).

## Surprises & Discoveries

- Observation: The npm `wasm-opt` wrapper (which downloads a binaryen binary — v112 on this machine) EXITS 0 when the underlying wasm-opt fails, printing the error but writing no output file. During the 0.2.0 release packaging this made `package_npm.sh` "succeed" while silently packing the unoptimized artifact: the hardcoded `--enable-bulk-memory-opt` flag (a newer-binaryen flag) was rejected, the wrapper swallowed the failure, `set -eu` saw exit 0, and the input file — never overwritten — went into the tarball at 7.0 MB instead of 6.5 MB.
  Evidence: `wasm-opt … --enable-bulk-memory-opt -o out.wasm` → stderr "Unknown option '--enable-bulk-memory-opt'", `echo $?` → 0, no `out.wasm` created; tarball unpacked size 7.0 MB (the exact unoptimized size) on the first packaging run.
  Consequence: both packaging scripts now (a) probe `wasm-opt --help` for the flag instead of pinning it, and (b) write to a temp output and refuse to continue unless that file exists non-empty — an exit code from this wrapper is not evidence of anything. The corpus smoke would not have caught this (an unoptimized artifact verifies fine); only the size did.

## Decision Log

- Decision: The flat JSON report format moves INTO `provenance-core` as a small hand-rolled renderer (no serde), and both the WASM wrapper (`crates/provenance-wasm`) and the new CLI `--json` flag call it. Today the wrapper hand-rolls that JSON itself; after U1 the wrapper is truly logic-free and CLI/extension JSON parity holds by construction rather than by parity tests.
  Rationale: two hand-rolled renderers of the same shape would drift; the core is sans-IO and a pure bytes-to-string renderer respects that; serde stays out per the wedge plan's dependency discipline (every new dependency needs a Decision Log entry, and a ~40-line renderer does not justify one).
  Date/Author: 2026-07-18 / plan authoring.
- Decision: The C API for `provenance-core` (a wedge-proposal deliverable) is deliberately NOT a milestone of this plan. It is recorded here as a known candidate, to be planned only when a concrete native consumer exists.
  Rationale: no consumer exists today; a C API without a consumer is unverifiable surface area (nothing would exercise it end-to-end, violating the "demonstrably working behavior" bar).
  Date/Author: 2026-07-18 / plan authoring.
- Decision: Milestone order is U1→U7 by increasing risk to the trust surface: U1–U3 change no permissions and no trust semantics, U4 protects everything after it, U5–U6 widen distribution of unchanged behavior, and U7 — the only milestone that expands browser permissions — goes last and is split into a design gate (maintainer approves the permission model) before any implementation.
  Rationale: the wedge's credibility is the product; the order keeps every intermediate state shippable and keeps the one permission-expanding change isolated and explicitly approved.
  Date/Author: 2026-07-18 / plan authoring.
- Decision: U2 extends the pipeline's `Report` with an optional `credentials: Option<CredentialSummary>` field populated ONLY when Layer 1 returns `Proof`, rather than widening the `Proof` variant itself.
  Rationale: `LayerFinding` is the evidentiary type system (the wedge plan's Interfaces section pins it); keeping it minimal preserves the honesty typing, while a separate summary struct carries descriptive metadata that has no bearing on the verdict. A summary must never appear beside a non-Verified verdict, and a test pins that.
  Date/Author: 2026-07-18 / plan authoring.

- Decision: U2 field names finalized: `digital_source_type` (the declared digitalSourceType URI, verbatim) plus `source_type_note` (a fixed descriptive phrase) replace the sketched `ai_involvement_declared`. The note exists for exactly two IPTC generative-AI types — `trainedAlgorithmicMedia` → "the credential declares this content AI-generated", `compositeWithTrainedAlgorithmicMedia` → "the credential declares AI-generated elements composited into this content" — and is defined once in `layers/c2pa.rs::source_type_note`; every surface (CLI, JSON, popup) prints it as data from that single definition, so no `wording_sync.rs` growth is needed: there is no second copy to drift. All other source types show only the verbatim URI, uninterpreted.
  Rationale: verbatim-URI + separate note keeps the reported fact (what the credential says) cleanly apart from the vocabulary gloss (what IPTC defines that value to mean), and single-sourcing the phrase is what the wording audit exists to approximate.
  Date/Author: 2026-07-18 / U2 implementation.
- Decision: U2 extraction lives in `C2paLayer::credential_summary`, called by `Pipeline::examine` only when the combined verdict is Verified; the pipeline retains its concrete `C2paLayer` (`with_layers` pipelines have none and never carry a summary). This re-parses the asset on the Verified path — accepted and marked in code — because the alternative (widening the `Layer` trait's return type) would touch all four layers for one layer's metadata. The extraction runs under the same panic guard as validation, and any extraction failure means "no summary", never a verdict change.
  Rationale: smallest change that keeps the evidentiary `LayerFinding` type untouched; milliseconds of re-parse on the rarest (Verified) path is the right price for that.
  Date/Author: 2026-07-18 / U2 implementation.

- Decision: U3 alignment resolved as "add, don't drop": the byte sniffer gained AVIF (`ftyp` box, brands `avif`/`avis`) rather than the CLI losing its `.avif` guess, and the CLI gained `.gif` to match the sniffer — both sides now recognize exactly jpeg/png/webp/gif/avif. The corpus tests (native, parity, wasm smoke) pass NO media-type hint anymore, so every vector must reach its verdict from bytes alone; the PNG "corrupted" vector refreshes the caBX chunk CRC after the byte flip so the container stays structurally valid and the Tampered verdict comes from manifest validation, not a parse failure — same honesty line as the JPEG corpus.
  Rationale: the c2pa crate supports AVIF via its BMFF handler, so widening the sniffer is real capability, not a claim; hint-free testing is strictly stronger than hinted (the hinted path is a subset); and a corrupted vector that fails at the container level would test the wrong thing.
  Date/Author: 2026-07-18 / U3 implementation.

- Decision: U4's scheduled jobs (cargo-audit, weekly fuzz smoke) start ADVISORY (`continue-on-error: true`); the push/PR loop is the only hard gate. Two named triggers flip them to enforcing: cargo-audit when the maintainer decides advisories should fail the build, and the fuzz smoke when the upstream c2pa JUMBF-panic fix ships and the dependency is bumped (until then a red weekly job would be permanent noise that trains people to ignore CI). The workflow also adds a wasm32 type-check of `provenance-wasm` to the hard gate — a widening of the plan's sketched "exact loop", because the extension engine surface would otherwise only break at the next manual wasm-pack build.
  Rationale: a gate that is red for reasons nobody can act on stops being a gate; advisory visibility now, enforcement tied to concrete unblock events.
  Date/Author: 2026-07-18 / U4 implementation.

- Decision: U5 cross-browser mechanics — one background file under BOTH manifest keys (`service_worker` for Chrome, `scripts` for Firefox's MV3 event pages; MDN's documented pattern), a two-line feature-detect shim (`const api = typeof browser !== "undefined" ? browser : chrome`) instead of the webextension-polyfill dependency, and a hard Firefox floor of 121 (below it, Firefox refuses to start the event page when a `service_worker` key is also present, which would mean no background at all). The gecko add-on ID is pinned as `provenance-lens@gwamoniak.github.io` — the maintainer may change it any time BEFORE the first AMO submission, never after (AMO IDs are permanent).
  Rationale: the polyfill is a dependency for what two lines do (Firefox's `chrome.*` is callback-style, so bare `chrome.*` promise chains break; everything else is API-compatible); the dual-key manifest keeps a single codebase; the version floor turns a silent no-background failure mode into an install-time refusal.
  Date/Author: 2026-07-18 / U5 implementation.

- Decision: U6 ships the wasm-pack `web` target as the one npm artifact flavor, not `bundler` or `nodejs`. `web` is the only target that works everywhere the package is aimed: browsers (URL init), bundlers (ESM glue), and plain Node (bytes init — proven by the smoke and the acceptance test); it is also exactly what the extension consumes, so one artifact shape is exercised by every consumer. The generated package.json is patched (name, description, keywords) by the packaging script rather than committed, keeping wasm-pack the single source of the package layout and the version in automatic lockstep with the workspace.
  Rationale: one flavor everyone runs beats three flavors nobody fully tests; the Node-usability of the `web` target removes the only argument for a second build.
  Date/Author: 2026-07-18 / U6 implementation.

- Decision: **U7 design approved** — the maintainer approved the U7 design as drafted (permission model with `optional_host_permissions: ["*://*/*"]` + `scripting`, per-site browser-rendered consent, grant-scoped content-script registration, lazy visible-only scanning, text-only shadow-root pills, honest not-examined markers, U7a–c split). Implementation may proceed.
  Rationale: per the design gate; the permissions delta is the one the maintainer signed off.
  Date/Author: 2026-07-18 / maintainer (gwamoniak): "approved, start U7a".
- Decision: U7a raises the Firefox floor from 121 to 128 (`strict_min_version: "128.0"`): `optional_host_permissions` — the load-bearing key of the approved permission model — exists only from Firefox 128 (Chrome 102, per MDN browser-compat-data). Below 128 the grant flow could not work at all; declaring the origins under plain `host_permissions` instead is rejected because the same manifest would then prompt at INSTALL time in Chrome, violating the opt-in model. Side effect: the Firefox 121–126 "popup does not auto-open" caveat (openPopup is 127+) is moot at the new floor and is removed from the collateral.
  Rationale: one manifest, one permission story, in both browsers; an install-time refusal on old Firefox beats a silently broken grant button.
  Date/Author: 2026-07-18 / U7a implementation.

- Decision: two U7b refinements to the approved design, neither changing the permission surface. (1) The "click a not-examined marker to grant that image's host" affordance moves to the popup (U7c): `permissions.request` is unavailable to content scripts, and a user gesture does not reliably survive `sendMessage` into the background, so the honest in-page behavior is that the marker click surfaces the error report (which names the blocked host) and the popup offers the grant. (2) Pill presentation (text, tier color, tooltip) is computed by the BACKGROUND from a single table in `lib/scan_support.js` and delivered in the `pl-verify` response, instead of the content script keeping its own copy — the action badge and the page pills render from one definition, the same single-sourcing rule the wording audit enforces for phrases, and the mapping became Node-testable pure logic (including the unknown-verdict → ERR fallthrough).
  Rationale: (1) is a platform constraint, resolved in favor of never faking a grant prompt; (2) turns a color-drift risk into a tested invariant for the cost of one response field.
  Date/Author: 2026-07-18 / U7b implementation.

- Decision: U7c mechanics. (1) The verify cache stores ONLY successful examinations: failures are transient by nature (the very next user action may be granting the blocked host), and the allow-and-verify flow depends on a retry actually retrying. (2) After a grant or revoke the popup shows "Reload the page to apply the change" instead of injecting into open tabs — script registration applies to future loads, and injecting retroactively would add a code path the smoke can't distinguish from registration working. (3) The smoke page's second-origin case uses the `127.0.0.1` hostname (a different host than `localhost` in match patterns, which ignore ports) served through a `/nocors/` route with no ACAO header — without both, the background's CORS-following fetch would succeed and the blocked-host path would be undemonstrable.
  Rationale: each choice keeps the demonstrated behavior identical to the mechanism that produces it — no cached lies, no invisible injection, no accidentally-fetchable "blocked" host.
  Date/Author: 2026-07-18 / U7c implementation.

## Outcomes & Retrospective

- (2026-07-18) **Plan complete — U1 through U7 all closed in one day, the last two browser-smoked by the maintainer.** Against the Purpose: every promise holds. Scripts consume verdicts as JSON with contract exit codes; Verified reports state what the credential claims (and only then); format coverage is proven by an eight-vector JPEG+PNG corpus tested hint-free at every level down to the compiled artifact; CI gates every push; the extension runs in Firefox from the same codebase; the engine is one `npm publish` from JavaScript developers; and the flagship page-scan UX ships behind per-site consent the browser itself renders, with honest not-examined markers where access is missing. The honesty rules survived unchanged — the closest calls (pill iconography, blocked-image handling, cached failures) were each resolved by asking "what would this claim that we didn't compute?". What worked: the single-source pattern (one JSON renderer, one tier-presentation table, one phrase set audited by CI) repeatedly turned drift risks into tested invariants; the U7 design gate cost one plan revision and caught two real platform constraints (chrome.* callbacks, content-script permissions) before any code; and acceptance-by-construction (self-verifying corpus, curl-verified no-CORS route, scratch-project npm install) kept every milestone provable from a machine with no browser. What remains is all external and all the maintainer's: store submissions (Chrome + AMO), `npm publish`, the upstream c2pa report, and the wedge plan's still-gated M5–M7.

## Context and Orientation

Everything below assumes only this repository's working tree. **C2PA** ("Content Credentials") is an open standard that embeds a cryptographically signed provenance manifest inside a media file — who made it, with what tool, whether generative AI was involved — bound to the exact bytes by a content hash and validated against a trust list of root certificates. The project verifies such manifests and reports one of four verdict tiers with pinned wording: Verified, Indicated, Inconclusive ("no provenance data was found. This does NOT mean the asset is authentic."), Tampered. The canonical phrases live in `crates/provenance-core/src/verdict.rs` and are wording-audited by `crates/provenance-core/tests/wording_sync.rs`, which fails the suite if `README.md`, `extension/popup/popup.html`, or `.claude/skills/verdict-language/SKILL.md` drift from them. Any user-visible string this plan adds must follow `.claude/skills/verdict-language/SKILL.md` (understatement; never imply safety from absence of data).

The layout: `crates/provenance-core` is the sans-IO library — `pipeline.rs` holds `Asset` (bytes + optional MIME hint), the `Layer` trait, `LayerFinding` (NotEvaluated/NoSignal/Proof/Indication/TamperEvidence), `Report { verdict, findings }`, and the precedence rule Tampered > Verified > Indicated > Inconclusive; `layers/c2pa.rs` is the only real layer (the other three are honest gated stubs). `crates/provenance-cli` is the std-only `lens` binary (exit codes 0/10/20/30/2 are a contract). `crates/provenance-wasm` exports `verify_bytes(bytes, media_type?, trust_anchors_pem?) -> String` (flat JSON) via wasm-bindgen; `wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg` feeds the MV3 extension in `extension/` (plain JS: `background.js` service worker, `popup/`, trust anchors as a data file in `extension/trust/anchors.pem`). Known-answer vectors live in `crates/provenance-core/tests/vectors/` (currently five JPEGs + `manifest.tsv` + `test_ca.pem`), regenerated by the self-verifying `cargo run -p provenance-core --example gen_vectors`. The build loop every milestone must keep green, from the repo root:

    cargo fmt --all
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

Standing rules inherited from the wedge plan (restated so this plan is self-contained): every new dependency requires a Decision Log entry; image bytes never leave the device; signature-validation code (Layer 1, trust-list handling) merges only with the maintainer's (gwamoniak) recorded sign-off; the extension's permissions are `contextMenus` + `activeTab` + `storage` and may not grow except through U7's explicit design gate; a fresh clone with no network must build and pass all tests; the gated milestones M5–M7 (watermark, registry, heuristics) stay in the wedge plan and are NOT opened by anything here.

## Plan of Work

The milestones are independent enough to land one at a time, each leaving the tree shippable. U1 adds a `render_json(&Report) -> String` function to `provenance-core` (new module `crates/provenance-core/src/json.rs`), switches `crates/provenance-wasm/src/lib.rs` to call it, and teaches `crates/provenance-cli/src/main.rs` a `--json` flag and multi-file invocation. U2 adds the `CredentialSummary` extraction to `layers/c2pa.rs` (reading the already-validated manifest's claim generator, signing time, and declared digitalSourceType) plus rendering in CLI text, CLI JSON, and the popup. U3 extends `examples/gen_vectors.rs` to also emit a PNG set and aligns the byte-sniffing in `layers/c2pa.rs` with the CLI's extension-guessing. U4 adds `.github/workflows/ci.yml` (and extends the fuzz/audit cadence). U5 adds Firefox compatibility to `extension/` and AMO collateral to `docs/`. U6 adds an npm packaging path for the engine. U7 first produces a written permission/consent design in this plan, then (after maintainer approval) the content-script badge implementation.

## Milestones

**U1 — Machine-readable CLI (small; no trust-surface change).** Scope: `lens verify --json [--trust-anchors <PEM>] <FILE>...` prints one JSON object per file (the same flat shape `verify_bytes` returns: verdict id, approved phrase, per-layer findings, plus the file path), newline-delimited when multiple files are given; exit code is the worst verdict across files (Tampered worst, then the existing 0/10/20/30 ladder — "worst" = highest exit code). The renderer lives in core (Decision Log) and the WASM wrapper delegates to it; the existing wasm parity tests then pin the shared shape. Multi-file also works without `--json` (repeat the current human report per file). Acceptance: `lens verify --json <vector>` output parses as JSON (prove with `node -e` or `python -m json.tool` in the transcript); the wasm parity suite still passes; `lens verify --json valid_signed.jpg manifest_corrupted.jpg` (with anchors) exits 30.

**U2 — Say what the credentials claim (medium; wording-sensitive).** Scope: when — and only when — Layer 1 yields `Proof`, the report additionally carries a `CredentialSummary`: issuer (already surfaced), claim generator name/version, signing time if present, and whether the manifest declares AI involvement (the `digitalSourceType` on the active manifest's `c2pa.created`/`c2pa.actions` assertion, e.g. `trainedAlgorithmicMedia`). Render it in the CLI (indented under the c2pa line), in the JSON, and in the popup — worded per the verdict-language skill: descriptive statements of what the credential *claims* ("credential declares AI-generated content"), never endorsements. A test pins that `credentials` is `None` for every non-Verified vector, and the wording-sync audit grows to cover any new fixed phrases. Acceptance: `lens verify --trust-anchors test_ca.pem valid_signed.jpg` shows the summary (the test vectors declare `trainedAlgorithmicMedia`, so the AI-involvement line must appear); `stripped.jpg` shows no summary and remains word-for-word Inconclusive.

**U3 — Format coverage proven, not assumed (small).** Scope: today all five vectors are JPEG; PNG/WebP/GIF flow through the same code paths unproven. Extend `examples/gen_vectors.rs` to emit a PNG trio (signed / stripped / corrupted — stripping for PNG means removing the `caBX` chunk rather than JPEG's APP11 segments) with self-verification, and add them to `manifest.tsv`, the corpus test, and the wasm smoke. Align `sniff_media_type` in `layers/c2pa.rs` and `guess_media_type` in the CLI so both recognize the same set (the CLI currently guesses AVIF but the sniffer does not; either add AVIF sniffing — the `ftyp` box — or drop the CLI's AVIF guess and record which). Acceptance: corpus grows to eight vectors, all green natively and through the WASM artifact (`node scripts/wasm_smoke.mjs`).

**U4 — CI baseline (small; multiplies the value of everything else).** Scope: `.github/workflows/ci.yml` running the exact loop (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`) on push and pull request on ubuntu-latest; a scheduled weekly job running `cargo audit` (advisory only at first — failing the build on advisories is a separate maintainer decision) and a 5-minute fuzz smoke (`cargo +nightly fuzz run manifest_parsing fuzz/corpus fuzz/corpus_seed -- -max_total_time=300`; default sanitizer on Linux — the Windows `--sanitizer none` workaround recorded in the wedge plan is a local-machine matter). No third-party actions beyond `actions/checkout` and the official rust-toolchain setup, consistent with the trust-list workflow's minimal-supply-chain rule. Acceptance: a pushed branch shows the workflow green on GitHub; a deliberately misformatted file on a scratch branch shows it red.

**U5 — Firefox port (medium; distribution).** Scope: make `extension/` load and pass the smoke flow in Firefox (MV3 is supported; expected deltas: `browser` vs `chrome` namespaces — prefer a tiny feature-detect shim over a polyfill dependency — `background.service_worker` vs `scripts` in the manifest, and `chrome.action.openPopup()` availability, falling back to the badge as the code already does). Add AMO listing collateral to `docs/STORE_LISTING.md` (same honest copy; AMO-specific fields). The manual smoke script from the wedge plan (serve `scripts/serve_testpage.mjs`, load unpacked, right-click each vector) is the acceptance bar, run in Firefox; the maintainer performs the AMO submission, same as Chrome. Acceptance: all five (post-U3: eight) vectors produce their expected verdicts in Firefox, failure states included.

**U6 — npm package for the engine (small-medium; distribution).** Scope: a `wasm-pack build --target bundler` (or `web`, decided and recorded when implemented) packaging path producing an npm-publishable package (working name `@provenance-lens/verify-wasm`; the name is a maintainer decision at publish time) with a README whose wording passes the verdict-language rules, the same trust-anchor-parameter API, and a version pinned to the workspace version. Publish prep only — `npm publish` is the maintainer's action, like every release. Acceptance: `npm pack` produces a tarball; a scratch Node project can `npm install` the tarball and reproduce the wasm smoke results against the vector corpus.

**U7 — Page-scan badges behind explicit opt-in (large; the only permission change — DESIGN-GATED).** The wedge proposal's flagship UX: verdict badges on the images of a page, per site, only where the user explicitly asked for it. The design below (drafted 2026-07-18) is a PROPOSAL; implementation starts only after the maintainer approves it with a dated Decision Log entry, mirroring the human-sign-off discipline. This requires the host permissions the wedge deliberately avoided, which is exactly why the design is the gate.

*Permission model.* The manifest gains `optional_host_permissions: ["*://*/*"]` and the `scripting` permission; NOTHING is granted at install (optional host permissions produce no install-time warning, and Firefox's MV3 model already treats host access as user-controlled). Access is granted per site, through an explicit gesture: the popup shows, for the current tab, a button "Scan images on <host>", whose click calls `permissions.request({ origins: ["*://<host>/*"] })` — the browser renders its own consent UI on top. The granted-site list is never duplicated into extension storage: `permissions.getAll()` is the single source of truth, and the same popup surface offers "Stop scanning <host>" via `permissions.remove`. Consent copy, verbatim and understated: "Scan images on <host> — every image shown on this site is examined locally. Image bytes never leave your device." The existing context-menu flow is untouched and remains the whole product for users who never grant anything.

*Injection.* On grant, the background registers a content script for that origin via `scripting.registerContentScripts` (persistent across sessions, scoped to the granted origin pattern); on revoke it unregisters. No static `content_scripts` entry ever appears in the manifest, so the extension's reach is exactly the set of user grants at any moment.

*Scanning strategy.* The content script does no verification itself. It watches `<img>` elements with an IntersectionObserver (plus a MutationObserver for SPA-inserted images) and queues an image only when it becomes visible and its rendered size is at least 64×64 px (skips icons, sprites, trackers). Queued URLs go to the background, which fetches the bytes, runs the engine, and answers with the report — at most 2 verifications in flight, the rest queued, and a per-URL session cache so repeated images cost one verification. Images on origins the grant does not cover (CDN-hosted images are the norm) are NOT silently skipped: they get a neutral "not examined" marker whose tooltip names the image's host, and clicking it offers `permissions.request` for that host — no automatic permission requests, ever. `data:`/`blob:` images verify normally (their bytes are already local).

*Badge visuals — the no-safety-language rules.* Badges are small corner pills rendered inside a closed shadow root (page CSS cannot restyle them; a hostile page CAN remove them — the extension never claims tamper-proof UI, and the popup remains the authoritative view). Pills carry TEXT, not icons: VER / IND / INC / TAM in the exact tier colors already used by the action badge and popup (green / amber / neutral gray / red), plus black ERR for errors and a gray dashed "· · ·" for not-examined. No checkmarks, no shields, no padlocks anywhere — iconography smuggles safety semantics that the wording rules exist to prevent. The tooltip is the verbatim approved phrase; clicking a pill runs the existing single-image flow (badge + popup with the full report, credential summary included). Inconclusive pills render on every examined credential-less image — that visibility, at scale, is the wedge thesis' pressure mechanism, and the per-site opt-in is what makes the noise consented-to.

*Privacy.* Nothing new leaves the device. The background keeps no persistent record of sites or URLs; the verdict cache lives in `storage.session` (cleared when the browser closes) and the grant list lives in the browser's own permission store. The store-listing privacy policy gains one sentence stating exactly that.

*Implementation split (after approval).* U7a: background plumbing — grant/revoke management, content-script (un)registration, the fetch→verify→cache service with the concurrency cap. U7b: the content script — observers, queueing, shadow-DOM pills. U7c: popup site-toggle, store-listing updates (permission justifications for `scripting` + optional host access), and a smoke-page extension: a second page on the test server embedding all eight vectors plus one image served from a second port (the "different origin" case). Each sub-milestone lands green and shippable.

*Acceptance (for the implementation, when it happens).* With no grants: behavior is byte-for-byte today's (regression: the M3 smoke passes unchanged). After granting the smoke-page origin: every vector image shows its correct pill lazily as it scrolls into view, the second-origin image shows the not-examined marker until its host is granted too, and revoking the grant removes the content script and all pills on next load. A page with 100 images stays responsive: pills appear only for viewed images and never more than 2 verifications run concurrently.

## Concrete Steps

Per milestone, the loop is unchanged (repo root): `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; for anything touching the WASM boundary additionally `wasm-pack build crates/provenance-wasm --target web --out-dir ../../extension/pkg`, the wasm-opt step from `.claude/skills/wasm-packaging/SKILL.md`, and `node scripts/wasm_smoke.mjs`; for anything touching the extension the manual smoke from the wedge plan's M3 section. On the current development machine (Windows) wasm-pack is not yet installed: `cargo install wasm-pack` once, before U1's parity work needs it. Commit at every stopping point with the milestone name in the message; update this plan's Progress in the same commit.

## Validation and Acceptance

This plan is done when a novice can, from a fresh clone: run the loop and see green including CI parity locally; script `lens verify --json` and branch on exit codes across multiple files; verify a signed vector and read what its credential claims (and see no summary on an unverified one); watch the same eight-vector corpus pass natively, in Node through the compiled artifact, and in both Chrome and Firefox through the extension; and install the engine from an npm tarball in a scratch project. U7 additionally requires the maintainer-approved design entry in the Decision Log before any code exists.

## Idempotence and Recovery

All steps are re-runnable: cargo/wasm-pack/npm-pack commands are idempotent, vector regeneration self-verifies before writing, and no milestone has a destructive step. If a milestone stalls, split its Progress entry into done/remaining and commit — this plan plus the tree must always be enough to resume.

## Artifacts and Notes

U1 acceptance (2026-07-18; `V=crates/provenance-core/tests/vectors`; JSON parsed back with node to prove machine-readability):

    $ lens verify --json --trust-anchors $V/test_ca.pem \
          $V/valid_signed.jpg $V/manifest_corrupted.jpg $V/plain.jpg
    {"file":"…/valid_signed.jpg","verdict":"verified","phrase":…,"findings":[…4 layers…]}
    {"file":"…/manifest_corrupted.jpg","verdict":"tampered",…}
    {"file":"…/plain.jpg","verdict":"inconclusive",…}
    → exit 30 (worst of 0 / 30 / 20)
    node JSON.parse per line: valid_signed.jpg -> verified | manifest_corrupted.jpg -> tampered | plain.jpg -> inconclusive

    $ lens verify --trust-anchors $V/test_ca.pem $V/valid_signed.jpg   # human mode unchanged
    → verdict: Verified …, exit 0

U2 acceptance (2026-07-18):

    $ lens verify --trust-anchors $V/test_ca.pem $V/valid_signed.jpg
      verdict: Verified: …
      [c2pa] valid provenance chain, issuer: Self-signed ephemeral certificate …
      credential claims:
        claim generator: provenance-lens test vectors/0.1.0
        declared source type: http://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicMedia
        note: the credential declares this content AI-generated
      → exit 0
    $ lens verify --trust-anchors $V/test_ca.pem $V/stripped.jpg
      verdict: Inconclusive: no provenance data was found. This does NOT mean the asset is authentic.
      (no credential block)                                  → exit 20
    $ lens verify --json … valid_signed.jpg | node JSON.parse
      credentials: { issuer, claim_generator, digital_source_type, source_type_note }
      (signing_time absent — the test vectors carry no timestamp; absent keys are omitted)

U3 acceptance (2026-07-18; fresh wasm-pack build on this machine, corpus through the compiled artifact in Node, no content-type hints anywhere):

    $ cargo run -q -p provenance-core --example gen_vectors
    wrote 8 vectors + test_ca.pem + manifest.tsv to …/tests/vectors
    $ node scripts/wasm_smoke.mjs
    PASS plain.jpg: inconclusive            PASS valid_signed.jpg: verified
    PASS stripped.jpg: inconclusive         PASS manifest_corrupted.jpg: tampered
    PASS content_tampered.jpg: tampered     PASS valid_signed.png: verified
    PASS stripped.png: inconclusive         PASS manifest_corrupted.png: tampered
    wasm smoke: all vectors match

    Unoptimized artifact: 7,031,523 B raw / 2,204,797 B gzipped (wasm-opt runs in the
    release packaging step; gz already within the ≤2.5 MB budget).

U4 acceptance (2026-07-18; run conclusions polled from the GitHub Actions API):

    main        fce8832  ci  completed  success   (fmt + clippy + tests + wasm32 check)
    ci-redcheck 986d40b  ci  completed  failure   (deliberate fmt violation; failed at the
                                                   first gate; scratch branch deleted after)

U6 acceptance (2026-07-18):

    $ sh scripts/package_npm.sh
    warning: wasm-opt (binaryen) not found - packing the unoptimized artifact
    wasm smoke: all vectors match        (through dist/npm-pkg, the packed artifact)
    provenance-lens-verify-wasm-0.1.0.tgz  (2.4 MB packed / 7.0 MB unpacked, 5 files)

    scratch project: npm install <tarball> → import by name per the README →
    PASS ×8 (all corpus vectors, JPEG + PNG, hint-free) → "npm-install acceptance: all vectors match"

## Interfaces and Dependencies

New in U1, in `crates/provenance-core/src/json.rs`:

    pub fn render_json(report: &Report) -> String   // flat JSON: {"verdict": id, "phrase": ..., "findings": [{"layer": ..., "kind": ..., ...}]}

`crates/provenance-wasm` keeps its exported signature unchanged and delegates to `render_json`. Landed in U2, in `crates/provenance-core/src/pipeline.rs` (final shape; see the Decision Log for the rename from the sketched `ai_involvement_declared`):

    pub struct CredentialSummary {
        pub issuer: String,
        pub claim_generator: Option<String>,        // "name/version"
        pub signing_time: Option<String>,           // as reported by the validator
        pub digital_source_type: Option<String>,    // the declared digitalSourceType URI, verbatim
        pub source_type_note: Option<&'static str>, // fixed phrase for the generative-AI types
    }
    // Report gains: pub credentials: Option<CredentialSummary> — Some exactly when the verdict is Verified.

No new Rust dependencies are anticipated for U1–U4; U5–U7 add none to the Rust workspace (extension/JS only). Any deviation requires a Decision Log entry here.

---

Revision note (2026-07-18): initial authoring, immediately after the post-wedge hardening pass, from a reflection over the wedge plan's remaining non-gated candidates (proposal deliverables not yet built, recorded UX deferrals, infrastructure gaps). Reason: the wedge is shipped and signed-off work needs a successor plan that widens reach without touching the gated layers or the honesty rules.

Revision note (2026-07-18, later): U7 milestone section replaced by the full design proposal (permission model, injection, scanning strategy, badge rules, privacy, U7a–c split, acceptance) with a pending-approval slot in the Decision Log. Reason: U1–U6 are complete; U7's design gate is the plan's only remaining work, and the design must be written down before the maintainer can approve or amend it.
