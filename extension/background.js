// Module service worker. Owns the context-menu entry and the verification
// flow: fetch the right-clicked image (the ONLY network request this
// extension ever makes on the user's behalf — image bytes never leave the
// device), run the bundled WASM engine, store the result for the popup, and
// reflect the verdict on the action badge.
//
// Honesty rules enforced here: errors are errors (never mapped onto a
// verdict tier), a missing engine says so, and nothing is rendered that the
// engine did not compute.

const MENU_ID = "provenance-lens-verify";

// Badge colors mirror popup.css tier colors; Inconclusive stays neutral gray.
const BADGE = {
  verified: { text: "VER", color: "#2e7d32" },
  indicated: { text: "IND", color: "#e09b00" },
  inconclusive: { text: "INC", color: "#757575" },
  tampered: { text: "TAM", color: "#c62828" },
};

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: MENU_ID,
    title: "Verify provenance with Provenance Lens",
    contexts: ["image"],
  });
});

chrome.contextMenus.onClicked.addListener((info) => {
  if (info.menuItemId !== MENU_ID || !info.srcUrl) return;
  verifyImage(info.srcUrl);
});

// The service worker is killed and restarted constantly; the engine is
// (re)initialized lazily on first use, never at worker start.
let enginePromise = null;
function engine() {
  if (!enginePromise) {
    enginePromise = (async () => {
      const mod = await import("./pkg/provenance_wasm.js");
      await mod.default({
        module_or_path: chrome.runtime.getURL("pkg/provenance_wasm_bg.wasm"),
      });
      return mod;
    })().catch((err) => {
      enginePromise = null; // allow retry after a failed init
      throw err;
    });
  }
  return enginePromise;
}

// trust/anchors.pem ships as data so the trust list updates without
// rebuilding the engine. A placeholder without certificates behaves exactly
// like "no anchors": nothing can verify as trusted.
async function loadAnchors() {
  try {
    const response = await fetch(chrome.runtime.getURL("trust/anchors.pem"));
    const pem = await response.text();
    return pem.includes("BEGIN CERTIFICATE") ? pem : undefined;
  } catch {
    return undefined;
  }
}

async function verifyImage(srcUrl) {
  const entry = { srcUrl, at: new Date().toISOString() };
  try {
    let mod;
    try {
      mod = await engine();
    } catch {
      throw new Error(
        "verification engine is not bundled — build extension/pkg/ per extension/README.md. The asset was NOT examined."
      );
    }

    let response;
    try {
      response = await fetch(srcUrl);
    } catch {
      throw new Error(
        "could not fetch the image (network failure or cross-origin restrictions). The asset was NOT examined."
      );
    }
    if (!response.ok) {
      throw new Error(
        `could not fetch the image (HTTP ${response.status}). The asset was NOT examined.`
      );
    }
    const bytes = new Uint8Array(await response.arrayBuffer());
    const mediaType =
      (response.headers.get("content-type") || "").split(";")[0].trim() || undefined;

    const anchors = await loadAnchors();
    entry.anchorsLoaded = Boolean(anchors);
    entry.report = JSON.parse(mod.verify_bytes(bytes, mediaType, anchors));
  } catch (err) {
    entry.error = String((err && err.message) || err);
  }

  await chrome.storage.session.set({ lastResult: entry });
  updateBadge(entry);
  try {
    await chrome.action.openPopup();
  } catch {
    // openPopup needs a recent user gesture and is not available everywhere;
    // the badge plus a manual click on the action icon is the fallback.
  }
}

function updateBadge(entry) {
  const tier = entry.report && BADGE[entry.report.verdict];
  if (tier) {
    chrome.action.setBadgeText({ text: tier.text });
    chrome.action.setBadgeBackgroundColor({ color: tier.color });
  } else {
    chrome.action.setBadgeText({ text: "ERR" });
    chrome.action.setBadgeBackgroundColor({ color: "#000000" });
  }
}
