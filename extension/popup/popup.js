// Renders the last verification result from storage.session.
// Rules: textContent ONLY (manifest fields are attacker-authored strings),
// verdict phrases come verbatim from the engine's JSON, errors are never
// styled as a tier, and Inconclusive never gets the visual language of
// safety (neutral gray, same as the legend).

// Firefox's chrome.* is callback-style; promise-style APIs live on
// browser.*. Chrome defines only chrome.* (promise-capable in MV3).
import { credentialLines, appendLines } from "../lib/report_view.js";
const api = typeof browser !== "undefined" ? browser : chrome;

const TIER_CLASSES = ["verified", "indicated", "inconclusive", "tampered"];

function render(entry) {
  const report = document.getElementById("report");
  const error = document.getElementById("error");
  report.hidden = true;
  error.hidden = true;
  const again = document.getElementById("verify-again");
  again.hidden = !entry?.srcUrl;
  again.onclick = () => reverify(entry.srcUrl);
  if (!entry) return;

  if (entry.report) {
    document.getElementById("report-url").textContent = entry.srcUrl;
    document.getElementById("report-retrieval").textContent = entry.retrievedAt
      ? `Fetched at ${entry.retrievedAt}. This result checks the fetched file; it does not verify the displayed pixels.`
      : "This is an earlier result without a recorded retrieval time. Verify again to fetch the current file.";
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
    const lines = credentialLines(entry.report);
    appendLines(credentials, lines);
    credentials.hidden = lines.length === 0;
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
    if (entry.needsHostAccess && entry.srcUrl) {
      let host = "";
      let pattern;
      try {
        const url = new URL(entry.srcUrl);
        if (["http:", "https:"].includes(url.protocol)) {
          host = url.hostname;
          pattern = `${url.protocol}//${host}/*`;
        }
      } catch {
        /* unparseable srcUrl: no affordance */
      }
      if (host) {
        grantButton.textContent = `Allow access to ${host} and verify`;
        grantButton.onclick = async () => {
          const granted = await api.permissions
            .request({ origins: [pattern] })
            .catch(() => false);
          // Re-verify through the background; failures are not cached, so
          // this is a fresh examination under the new grant. The result
          // arrives via storage.onChanged and re-renders this popup.
          if (granted) reverify(entry.srcUrl);
        };
        grantButton.hidden = false;
      }
    }
    error.hidden = false;
  }
}

async function reverify(url) {
  const button = document.getElementById("verify-again");
  button.disabled = true;
  try {
    const entry = await api.runtime.sendMessage({ type: "pl-reverify", url });
    render(entry?.srcUrl ? entry : { srcUrl: url, error: entry?.error || "verification failed" });
  } catch (err) { render({ srcUrl: url, error: String(err.message || err) }); }
  finally { button.disabled = false; }
}

// U7c: per-site scanning controls. Shown only when the active tab is an
// http(s) page whose URL this popup may read (activeTab covers both the
// action-click and context-menu paths). Scanning requires the explicit site
// choice as well as browser host permission.
async function initScanControls() {
  try {
    const [tab] = await api.tabs.query({ active: true, currentWindow: true });
    if (!tab || !tab.url) return;
    const url = new URL(tab.url);
    if (url.protocol !== "http:" && url.protocol !== "https:") return;
    const host = url.hostname;
    const pattern = `${url.protocol}//${host}/*`;
    const status = document.getElementById("scan-status");
    const toggle = document.getElementById("scan-toggle");
    const note = document.getElementById("scan-reload-note");

    const refresh = async () => {
      const { scanEnabledSites = [] } = await api.storage.local.get("scanEnabledSites");
      const enabled = scanEnabledSites.includes(pattern) && await api.permissions.contains({ origins: [pattern] });
      if (enabled) {
        status.textContent = `Scanning enabled on ${host}.`;
        toggle.textContent = `Stop scanning ${host}`;
      } else {
        // Consent copy per the approved U7 design — understated, factual.
        status.textContent =
          "Scan visible images of at least 64×64 pixels on this site. The extension fetches image files and checks them locally; it does not verify the page's displayed pixels. Some checks may be unavailable.";
        toggle.textContent = `Scan images on ${host}`;
      }
      toggle.onclick = async () => {
        const granted = enabled || await api.permissions.request({ origins: [pattern] }).catch(() => false);
        if (!granted) return;
        toggle.disabled = true;
        try {
          const result = await api.runtime.sendMessage({ type: "pl-set-scan", site: pattern, enabled: !enabled });
          note.textContent = result?.error || "Changes apply to open pages immediately. Image-host access remains a separate permission.";
          note.hidden = false;
        } catch (err) { note.textContent = String(err.message || err); note.hidden = false; }
        finally { toggle.disabled = false; await refresh(); }
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
