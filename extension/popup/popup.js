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
    // U7c: when the failure was "could not fetch the bytes", the fix the
    // user can actually take is granting the image's host — offered here,
    // where permissions.request runs inside a real user click.
    const grantButton = document.getElementById("grant-image-host");
    grantButton.hidden = true;
    grantButton.onclick = null;
    if (entry.notFetched && entry.srcUrl) {
      let host = "";
      try {
        host = new URL(entry.srcUrl).hostname;
      } catch {
        /* unparseable srcUrl: no affordance */
      }
      if (host) {
        grantButton.textContent = `Allow access to ${host} and verify`;
        grantButton.onclick = async () => {
          const granted = await api.permissions
            .request({ origins: [`*://${host}/*`] })
            .catch(() => false);
          // Re-verify through the background; failures are not cached, so
          // this is a fresh examination under the new grant. The result
          // arrives via storage.onChanged and re-renders this popup.
          if (granted) api.runtime.sendMessage({ type: "pl-show", url: entry.srcUrl }).catch(() => {});
        };
        grantButton.hidden = false;
      }
    }
    error.hidden = false;
  }
}

// U7c: per-site scanning controls. Shown only when the active tab is an
// http(s) page whose URL this popup may read (activeTab covers both the
// action-click and context-menu paths). The browser's permission store is
// the single source of truth; this UI only mirrors it.
async function initScanControls() {
  try {
    const [tab] = await api.tabs.query({ active: true, currentWindow: true });
    if (!tab || !tab.url) return;
    const url = new URL(tab.url);
    if (url.protocol !== "http:" && url.protocol !== "https:") return;
    const host = url.hostname;
    const pattern = `*://${host}/*`;
    const status = document.getElementById("scan-status");
    const toggle = document.getElementById("scan-toggle");
    const note = document.getElementById("scan-reload-note");

    const refresh = async () => {
      const enabled = await api.permissions.contains({ origins: [pattern] });
      if (enabled) {
        status.textContent = `Scanning enabled on ${host}.`;
        toggle.textContent = `Stop scanning ${host}`;
      } else {
        // Consent copy per the approved U7 design — understated, factual.
        status.textContent =
          "Every image shown on this site is examined locally. Image bytes never leave your device.";
        toggle.textContent = `Scan images on ${host}`;
      }
      toggle.onclick = async () => {
        const changed = enabled
          ? await api.permissions.remove({ origins: [pattern] }).catch(() => false)
          : await api.permissions.request({ origins: [pattern] }).catch(() => false);
        if (changed) note.hidden = false;
        refresh();
      };
    };
    await refresh();
    document.getElementById("scan-site").hidden = false;
  } catch {
    /* leave the section hidden — never guess at a host */
  }
}
initScanControls();

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
