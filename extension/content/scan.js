// Page-scan content script (U7b). Registered by the background PER GRANTED
// ORIGIN via scripting.registerContentScripts — it never appears in the
// manifest, so its reach is exactly the user's grants at any moment.
//
// This script verifies NOTHING itself. It watches <img> elements, queues a
// URL to the background only when an image becomes visible at a meaningful
// size, and renders the pill spec the background answers with. All
// presentation (text, colors, tooltips) is computed by the background from
// extension/lib/scan_support.js, so page pills cannot drift from the action
// badge. Pills are text-only by design rule: no checkmarks, no shields —
// iconography smuggles safety semantics the wording rules exist to prevent.
//
// Message contract (background.js):
//   { type: "pl-verify", url } → { entry, pill: { kind, text, color, title } }
//   { type: "pl-show",   url } → surfaces the full report in badge + popup.
//
// Classic script (registerContentScripts js files are not modules); IIFE to
// keep the page's global scope clean. A hostile page can remove the pills —
// the extension never claims tamper-proof UI; the popup stays authoritative.

(() => {
  const api = typeof browser !== "undefined" ? browser : chrome;

  // Below this rendered size an image is treated as decoration (icons,
  // sprites, trackers) and skipped. From the approved U7 design.
  const MIN_SIZE = 64;

  // url → { pill: spec | null, imgs: Set<img> } — one verification per URL,
  // however many <img> elements repeat it.
  const byUrl = new Map();
  // img → pill host element (for repositioning and cleanup).
  const hosts = new Map();

  const io = new IntersectionObserver(onIntersect, { rootMargin: "64px" });

  function watch(img) {
    if (!hosts.has(img)) io.observe(img);
  }

  function onIntersect(entries) {
    for (const e of entries) {
      if (!e.isIntersecting) continue;
      const img = e.target;
      const rect = e.boundingClientRect;
      // Too small right now: leave it observed — it may lazy-load or grow,
      // and it re-fires on the next intersection change.
      if (rect.width < MIN_SIZE || rect.height < MIN_SIZE) continue;
      const url = img.currentSrc || img.src;
      if (!url) continue;
      io.unobserve(img);
      enqueue(url, img);
    }
  }

  function enqueue(url, img) {
    let rec = byUrl.get(url);
    if (!rec) {
      rec = { pill: null, imgs: new Set() };
      byUrl.set(url, rec);
      api.runtime
        .sendMessage({ type: "pl-verify", url })
        .then((response) => {
          rec.pill = (response && response.pill) || {
            kind: "error",
            text: "ERR",
            color: "#000000",
            title: "no response from the extension background",
          };
          for (const each of rec.imgs) placePill(each, url, rec.pill);
        })
        .catch(() => {
          // Background unreachable (e.g. extension reloading): show nothing
          // rather than a stale or invented state.
          byUrl.delete(url);
        });
    }
    rec.imgs.add(img);
    if (rec.pill) placePill(img, url, rec.pill);
  }

  function placePill(img, url, pill) {
    if (hosts.has(img) || !img.isConnected) return;
    const host = document.createElement("div");
    host.style.cssText =
      "position:absolute;z-index:2147483647;margin:0;padding:0;border:0;pointer-events:none;";
    const shadow = host.attachShadow({ mode: "closed" });
    const el = document.createElement("span");
    el.textContent = pill.text;
    el.title = pill.title;
    const common =
      "pointer-events:auto;cursor:pointer;display:inline-block;" +
      "font:700 10px/1.4 system-ui,sans-serif;padding:1px 5px;border-radius:3px;" +
      "letter-spacing:.5px;user-select:none;";
    el.style.cssText =
      pill.kind === "not-examined"
        ? common + `color:${pill.color};border:1px dashed ${pill.color};background:rgba(255,255,255,.75);`
        : common + `color:#fff;background:${pill.color};`;
    el.addEventListener("click", (event) => {
      event.preventDefault();
      event.stopPropagation();
      api.runtime.sendMessage({ type: "pl-show", url }).catch(() => {});
    });
    shadow.appendChild(el);
    document.body.appendChild(host);
    hosts.set(img, host);
    position(img, host);
    scheduleReposition();
  }

  function position(img, host) {
    const rect = img.getBoundingClientRect();
    host.style.top = `${rect.top + window.scrollY + 4}px`;
    host.style.left = `${rect.left + window.scrollX + 4}px`;
  }

  // Keep pills glued to their images across scroll/resize/layout changes;
  // drop pills whose image left the DOM. rAF-throttled.
  let repositionQueued = false;
  function scheduleReposition() {
    if (repositionQueued) return;
    repositionQueued = true;
    requestAnimationFrame(() => {
      repositionQueued = false;
      for (const [img, host] of hosts) {
        if (!img.isConnected) {
          host.remove();
          hosts.delete(img);
        } else {
          position(img, host);
        }
      }
    });
  }
  addEventListener("scroll", scheduleReposition, { passive: true, capture: true });
  addEventListener("resize", scheduleReposition, { passive: true });

  // Existing images, then SPA-inserted ones.
  document.querySelectorAll("img").forEach(watch);
  new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      for (const node of mutation.addedNodes) {
        if (node.nodeType !== Node.ELEMENT_NODE) continue;
        if (node.tagName === "IMG") watch(node);
        else if (node.querySelectorAll) node.querySelectorAll("img").forEach(watch);
      }
    }
    scheduleReposition();
  }).observe(document.documentElement, { childList: true, subtree: true });
})();
