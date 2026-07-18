// Renders the last verification result from storage.session.
// Rules: textContent ONLY (manifest fields are attacker-authored strings),
// verdict phrases come verbatim from the engine's JSON, errors are never
// styled as a tier, and Inconclusive never gets the visual language of
// safety (neutral gray, same as the legend).

// Firefox's chrome.* is callback-style; promise-style APIs live on
// browser.*. Chrome defines only chrome.* (promise-capable in MV3).
const api = typeof browser !== "undefined" ? browser : chrome;

const TIER_CLASSES = ["verified", "indicated", "inconclusive", "tampered"];

function render(entry) {
  const report = document.getElementById("report");
  const error = document.getElementById("error");
  report.hidden = true;
  error.hidden = true;
  if (!entry) return;

  if (entry.report) {
    document.getElementById("report-url").textContent = entry.srcUrl;
    const phrase = document.getElementById("report-phrase");
    phrase.textContent = entry.report.phrase;
    phrase.className = "phrase";
    if (TIER_CLASSES.includes(entry.report.verdict)) {
      phrase.classList.add(entry.report.verdict);
    }
    const list = document.getElementById("report-findings");
    list.textContent = "";
    for (const finding of entry.report.findings || []) {
      const item = document.createElement("li");
      item.textContent = finding.detail
        ? `${finding.layer}: ${finding.status.replaceAll("_", " ")} — ${finding.detail}`
        : `${finding.layer}: ${finding.status.replaceAll("_", " ")}`;
      list.appendChild(item);
    }
    // Credential claims: present in the engine JSON only for Verified.
    // Descriptive lines of what the credential itself states — the
    // source_type_note phrase comes verbatim from the engine.
    const credentials = document.getElementById("report-credentials");
    credentials.textContent = "";
    credentials.hidden = true;
    if (entry.report.credentials) {
      const c = entry.report.credentials;
      const lines = [
        ["credential claims", ""],
        ["claim generator", c.claim_generator],
        ["signed at", c.signing_time],
        ["declared source type", c.digital_source_type],
        ["note", c.source_type_note],
      ];
      for (const [label, value] of lines) {
        if (value === undefined || value === null) continue;
        const item = document.createElement("li");
        item.textContent = value === "" ? `${label}:` : `${label}: ${value}`;
        credentials.appendChild(item);
      }
      credentials.hidden = false;
    }
    document.getElementById("report-anchors").textContent = entry.anchorsLoaded
      ? "Trust anchors: loaded from trust/anchors.pem."
      : "Trust anchors: none configured — no signature chain can validate as trusted.";
    report.hidden = false;
  } else if (entry.error) {
    document.getElementById("error-url").textContent = entry.srcUrl;
    document.getElementById("error-message").textContent = `Error: ${entry.error}`;
    error.hidden = false;
  }
}

api.storage.session.get("lastResult").then(({ lastResult }) => render(lastResult));
api.storage.onChanged.addListener((changes, area) => {
  if (area === "session" && changes.lastResult) render(changes.lastResult.newValue);
});

// Engine presence is reported honestly; this never claims a capability.
fetch(api.runtime.getURL("pkg/provenance_wasm_bg.wasm"), { method: "HEAD" })
  .then(() => {
    document.getElementById("engine-status").textContent =
      "Engine bundled. Right-click any image → Verify provenance.";
  })
  .catch(() => {
    /* keep the honest default message from popup.html */
  });
