// Renders the last verification result from chrome.storage.session.
// Rules: textContent ONLY (manifest fields are attacker-authored strings),
// verdict phrases come verbatim from the engine's JSON, errors are never
// styled as a tier, and Inconclusive never gets the visual language of
// safety (neutral gray, same as the legend).

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

chrome.storage.session.get("lastResult").then(({ lastResult }) => render(lastResult));
chrome.storage.onChanged.addListener((changes, area) => {
  if (area === "session" && changes.lastResult) render(changes.lastResult.newValue);
});

// Engine presence is reported honestly; this never claims a capability.
fetch(chrome.runtime.getURL("pkg/provenance_wasm_bg.wasm"), { method: "HEAD" })
  .then(() => {
    document.getElementById("engine-status").textContent =
      "Engine bundled. Right-click any image → Verify provenance.";
  })
  .catch(() => {
    /* keep the honest default message from popup.html */
  });
