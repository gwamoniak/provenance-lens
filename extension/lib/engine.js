// One bundled verifier and anchor loader for fetched and local files.
export const sha256 = async (bytes) => Array.from(new Uint8Array(
  await crypto.subtle.digest("SHA-256", bytes)), (b) => b.toString(16).padStart(2, "0")).join("");

let pending;
export function loadVerifier() {
  if (!pending) pending = (async () => {
    const localBytes = async (path) => {
      const response = await fetch(new URL(path, import.meta.url));
      if (!response.ok) throw new Error(`Bundled ${path} could not be loaded; not examined`);
      return new Uint8Array(await response.arrayBuffer());
    };
    const [wasm, anchorBytes, manifestBytes, mod] = await Promise.all([
      localBytes("../pkg/provenance_wasm_bg.wasm"), localBytes("../trust/anchors.pem"),
      localBytes("../manifest.json"), import("../pkg/provenance_wasm.js"),
    ]);
    await mod.default({ module_or_path: wasm });
    const pem = new TextDecoder().decode(anchorBytes);
    if (!pem.includes("BEGIN CERTIFICATE")) throw new Error("Bundled trust anchors are missing; not examined");
    return {
      verify: (bytes, mime) => JSON.parse(mod.verify_bytes(bytes, mime, pem)),
      identity: { package_version: JSON.parse(new TextDecoder().decode(manifestBytes)).version,
        wasm_sha256: await sha256(wasm), profile: "standalone-wasm", c2pa_version: "0.90.13" },
      anchors: { sha256: await sha256(anchorBytes), fetched_date: pem.match(/^# Fetched: (\d{4}-\d{2}-\d{2})/m)?.[1] ?? null },
      checks: { c2pa: "enabled", pixel_watermarks: "not_compiled", model_sha256: null,
        registry: "not_implemented", heuristics: "not_implemented", claude_text: "not_implemented" },
    };
  })().catch((error) => { pending = null; throw error; });
  return pending;
}
