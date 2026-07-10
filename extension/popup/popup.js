// Milestone 3 wires this popup to real reports (WASM engine result via
// runtime messaging). Until then it only reflects whether the engine bundle
// exists, so the UI never claims a capability it does not have.

const status = document.getElementById("engine-status");

fetch(chrome.runtime.getURL("pkg/provenance_wasm_bg.wasm"), { method: "HEAD" })
  .then(() => {
    status.textContent =
      "Engine bundled. Right-click any image → Verify provenance.";
  })
  .catch(() => {
    /* keep the honest default message from popup.html */
  });
