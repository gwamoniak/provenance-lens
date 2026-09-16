// Registered only for explicitly enabled, currently granted sites. The page
// never computes verdicts. Each response belongs to one image generation.
(() => {
  if (globalThis.__provenanceLensScanInstalled) return;
  globalThis.__provenanceLensScanInstalled = true;
  const api = typeof browser !== "undefined" ? browser : chrome;
  const MIN_SIZE = 64;
  const images = new Map();
  let enabled = false;
  let starting = false;
  let lifecycle = 0;
  let token;
  let sequence = 0;
  const source = (img) => img.currentSrc || img.src;
  const io = new IntersectionObserver((entries) => {
    if (!enabled) return;
    for (const entry of entries) {
      const rec = images.get(entry.target);
      if (!rec) continue;
      rec.visible = entry.isIntersecting;
      if (rec.visible) examine(entry.target);
    }
  }, { rootMargin: "64px" });
  const ro = new ResizeObserver((entries) => {
    if (!enabled) return;
    for (const { target } of entries) {
      refresh(target);
      if (images.get(target)?.visible) examine(target);
    }
    scheduleLayout();
  });
  function invalidate(img) {
    const rec = images.get(img);
    if (!rec) return;
    rec.generation = ++sequence;
    rec.host?.remove();
    rec.host = null;
    rec.requested = false;
    rec.url = source(img);
  }
  function watch(img) {
    if (!enabled || images.has(img) || !img.isConnected) return;
    images.set(img, { url: source(img), generation: ++sequence, requested: false, visible: false, host: null });
    io.observe(img);
    ro.observe(img);
  }
  function forget(img) {
    invalidate(img);
    images.delete(img);
    io.unobserve(img);
    ro.unobserve(img);
  }
  function refresh(img, force = false) {
    if (!enabled) return;
    if (!img.isConnected) { forget(img); return; }
    watch(img);
    const rec = images.get(img);
    if (force || rec.url !== source(img)) {
      invalidate(img);
      io.unobserve(img);
      io.observe(img);
    }
  }
  function current(img, rec, generation, url, session) {
    return enabled && token === session && img.isConnected && images.get(img) === rec
      && rec.generation === generation && rec.url === url && source(img) === url;
  }
  function examine(img, show = false) {
    const rec = images.get(img);
    if (!rec || !enabled || !img.complete || !source(img)) return;
    const rect = img.getBoundingClientRect();
    if (rect.width < MIN_SIZE || rect.height < MIN_SIZE) {
      if (rec.requested || rec.host) invalidate(img);
      return;
    }
    if (rec.url !== source(img)) invalidate(img);
    if (rec.requested && !show) return;
    if (show) invalidate(img);
    rec.requested = true;
    const url = rec.url;
    const generation = rec.generation;
    const session = token;
    api.runtime.sendMessage({ type: show ? "pl-show" : "pl-verify", url, generation, token: session })
      .then((response) => {
        if (!current(img, rec, generation, url, session)) return;
        if (response?.stopped) { stop(); return; }
        if (response?.pill) placePill(img, rec, response.pill, response.entry);
      }).catch(() => {
        if (current(img, rec, generation, url, session)) { rec.requested = false; rec.host?.remove(); rec.host = null; }
      });
  }
  function placePill(img, rec, pill, entry) {
    rec.host?.remove();
    const host = document.createElement("div");
    host.style.cssText = "position:absolute;z-index:2147483647;margin:0;padding:0;border:0;pointer-events:none;";
    const shadow = host.attachShadow({ mode: "closed" });
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = pill.text;
    const scope = entry?.retrievedAt
      ? ` Fetched file checked at ${entry.retrievedAt}; displayed pixels were not verified.` : "";
    button.title = pill.title + scope;
    button.setAttribute("aria-label", `Verify again and show details. ${button.title}`);
    const common = "pointer-events:auto;cursor:pointer;font:700 10px/1.4 system-ui,sans-serif;padding:1px 5px;border-radius:3px;letter-spacing:.5px;";
    button.style.cssText = pill.kind === "not-examined"
      ? common + `color:${pill.color};border:1px dashed ${pill.color};background:#fff;`
      : common + `color:#fff;background:${pill.color};border:1px solid #fff;`;
    button.addEventListener("click", (event) => {
      event.preventDefault(); event.stopPropagation();
      if (images.get(img) === rec && img.isConnected) examine(img, true);
    });
    shadow.appendChild(button);
    document.body.appendChild(host);
    rec.host = host;
    position(img, host);
  }
  function position(img, host) {
    const rect = img.getBoundingClientRect();
    host.style.top = `${rect.top + window.scrollY + 4}px`;
    host.style.left = `${rect.left + window.scrollX + 4}px`;
  }
  let queued = false;
  function scheduleLayout() {
    if (!enabled || queued) return;
    queued = true;
    requestAnimationFrame(() => {
      queued = false;
      if (!enabled) return;
      for (const [img, rec] of images) {
        refresh(img);
        if (rec.host) position(img, rec.host);
      }
    });
  }
  function eachImage(node, fn) {
    if (node.nodeType !== Node.ELEMENT_NODE) return;
    if (node.tagName === "IMG") fn(node);
    node.querySelectorAll?.("img").forEach(fn);
  }
  const mutations = new MutationObserver((changes) => {
    if (!enabled) return;
    // Removal invalidates even if the node was reinserted in the same batch.
    for (const change of changes) for (const node of change.removedNodes) eachImage(node, forget);
    for (const change of changes) {
      for (const node of change.addedNodes) eachImage(node, watch);
      if (change.type === "attributes" && change.target.tagName === "IMG") refresh(change.target, true);
      const picture = change.target.closest?.("picture");
      if (picture) eachImage(picture, (img) => refresh(img, true));
    }
    scheduleLayout();
  });
  function stop() {
    enabled = false; starting = false; lifecycle++;
    io.disconnect(); ro.disconnect(); mutations.disconnect();
    for (const rec of images.values()) rec.host?.remove();
    images.clear();
  }
  async function start() {
    if (starting || enabled) return;
    starting = true;
    const version = ++lifecycle;
    token = crypto.randomUUID();
    try {
      const response = await api.runtime.sendMessage({ type: "pl-scan-start", token });
      if (version !== lifecycle) return;
      starting = false;
      if (!response?.enabled) return;
      enabled = true;
      document.querySelectorAll("img").forEach(watch);
      mutations.observe(document.documentElement, { childList: true, subtree: true, attributes: true,
        attributeFilter: ["src", "srcset", "sizes", "media", "type"] });
    } catch { if (version === lifecycle) starting = false; }
  }
  document.addEventListener("load", (event) => {
    if (enabled && event.target.tagName === "IMG") refresh(event.target, true);
  }, true);
  document.addEventListener("error", (event) => {
    if (enabled && event.target.tagName === "IMG") invalidate(event.target);
  }, true);
  addEventListener("scroll", scheduleLayout, { passive: true, capture: true });
  addEventListener("resize", scheduleLayout, { passive: true });
  addEventListener("pagehide", stop);
  addEventListener("pageshow", start);
  api.runtime.onMessage.addListener((message, sender) => {
    if (sender.id !== api.runtime.id) return;
    if (message?.type === "pl-scan-control") { if (message.enabled) start(); else stop(); }
    if (message?.type === "pl-scan-refresh" && enabled) {
      for (const img of images.keys()) if (source(img) === message.url) refresh(img, true);
    }
  });
  start();
})();
