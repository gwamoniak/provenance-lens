import { MAX_ASSET_BYTES } from "../lib/scan_support.js";
import { sha256 } from "../lib/engine.js";

export function fileHint(file) {
  const types = { jpg: "image/jpeg", jpeg: "image/jpeg", png: "image/png", svg: "image/svg+xml",
    webp: "image/webp", gif: "image/gif", avif: "image/avif" };
  const extension = file.name.split(".").pop().toLowerCase();
  if (types[extension]) return { value: types[extension], source: "filename-extension" };
  return { value: file.type || null, source: file.type ? "file-type" : "none" };
}

export async function inspectFile(file, verifier) {
  if (file.size > MAX_ASSET_BYTES) throw new Error("File exceeds the 25 MiB examination limit; not examined.");
  const bytes = new Uint8Array(await file.slice(0, MAX_ASSET_BYTES + 1).arrayBuffer());
  if (bytes.length > MAX_ASSET_BYTES) throw new Error("File exceeds the 25 MiB examination limit; not examined.");
  const hint = fileHint(file);
  const report = verifier.verify(bytes, hint.value ?? undefined);
  return {
    format: "provenance-lens-evidence-receipt", format_version: 1,
    notice: "Unsigned local observation, not a Content Credential, certified audit, or legal compliance attestation. Recompute from the file; this receipt is not proof.",
    examined_at: new Date().toISOString(), scope: "selected-local-file",
    asset: { name: file.name.split(/[\\/]/).pop(), byte_length: bytes.length,
      sha256: await sha256(bytes), mime_hint: hint },
    engine: verifier.identity, enabled_checks: verifier.checks, trust_anchors: verifier.anchors,
    report,
  };
}

export function compareReports(a, b) {
  if (!a || !b) return ["Comparison unavailable: both files must be examined."];
  const status = (r) => r.findings.find((f) => f.layer === "c2pa")?.status;
  const description = (r) => ({ proof: "credential present and verified", no_signal: "no credential found",
    tamper_evidence: "provenance present but fails validation", not_evaluated: "C2PA check unavailable" })[status(r)] || "C2PA check unavailable";
  const lines = [`A: ${description(a)}. B: ${description(b)}.`];
  if (status(a) === "proof" && status(b) === "no_signal") lines.push("Verified credential present in A, absent in B.");
  if (status(b) === "proof" && status(a) === "no_signal") lines.push("Verified credential present in B, absent in A.");
  for (const [label, r] of [["A", a], ["B", b]]) {
    if (r.verdict === "verified" && r.credentials?.declared_watermark) lines.push(`${label}: declared watermark context present; not an independent detection.`);
    const missing = r.findings.filter((f) => f.status === "not_evaluated").map((f) => f.layer);
    if (missing.length) lines.push(`${label}: checks unavailable: ${missing.join(", ")}.`);
  }
  lines.push("These are two independent examinations. This does not prove that B was derived from A. Missing evidence does not mean authentic.");
  return lines;
}
