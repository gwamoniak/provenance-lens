import assert from "node:assert/strict";
import { createHash, X509Certificate } from "node:crypto";
import { gzipSync } from "node:zlib";
export const sha256 = bytes => createHash("sha256").update(bytes).digest("hex");
// Certificate content, not a forgeable comment or a line-ending-dependent PEM hash.
export const reviewedAnchors = "af8e560cae9bd92cf123672c10b45c2946144b482d1b4c9ff921a26c727cd1e2";
export function checkAnchors(pem, development=false) {
  const certs=pem.match(/-----BEGIN CERTIFICATE-----[\s\S]*?-----END CERTIFICATE-----/g) || [];
  assert.ok(certs.length,"anchor bundle has no certificates");
  const content=sha256(certs.map(c=>new X509Certificate(c).raw.toString("hex")).sort().join("\n"));
  assert.ok(development || content===reviewedAnchors,"unknown/test anchors require a separate --development package; trust changes need review");
  return {sha256:sha256(pem),certificate_content_sha256:content,certificates:certs.length,reviewed_content:content===reviewedAnchors};
}
export function checkSize(bytes) {
  const sizes={raw:bytes.length,gzip:gzipSync(bytes).length,sha256:sha256(bytes)};
  assert.ok(sizes.raw<=7_000_000 && sizes.gzip<=2_500_000,"WASM exceeds 7000000 raw / 2500000 gzip byte budget");
  return sizes;
}
