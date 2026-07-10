// Service worker: owns the context-menu entry and, from Milestone 3 on, the
// call into the WASM engine (extension/pkg/, produced by wasm-pack — never
// committed, see .gitignore). Permissions stay minimal: contextMenus +
// activeTab, no host permissions, no remote code.

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: "provenance-lens-verify",
    title: "Verify provenance with Provenance Lens",
    contexts: ["image"],
  });
});

chrome.contextMenus.onClicked.addListener((info) => {
  if (info.menuItemId !== "provenance-lens-verify") return;

  // Milestone 3: fetch info.srcUrl bytes, call verify_bytes() from the WASM
  // engine, and render the report. Until the engine is bundled we say so
  // honestly instead of pretending to verify.
  console.warn(
    "Provenance Lens: verification engine not bundled yet (build it with " +
      "wasm-pack, see extension/README.md). Asset was NOT examined:",
    info.srcUrl
  );
});
