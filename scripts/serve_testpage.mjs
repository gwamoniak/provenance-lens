// M3 smoke harness: serves a test page embedding the acceptance vectors
// (valid_signed / stripped / manifest_corrupted) from the committed corpus,
// with Access-Control-Allow-Origin: * so the extension's service worker can
// fetch the image bytes without host permissions.
//
//   node scripts/serve_testpage.mjs        → http://localhost:8917

import { createServer } from "node:http";
import { readFileSync } from "node:fs";

const vectors = new URL("../crates/provenance-core/tests/vectors/", import.meta.url);
const PORT = 8917;

// Image list and expected verdicts come straight from the corpus catalogue,
// so this page can never go stale against the vectors (U3).
const TIER = { verified: "Verified", indicated: "Indicated", inconclusive: "Inconclusive", tampered: "Tampered" };
const ROWS = readFileSync(new URL("manifest.tsv", vectors), "utf8")
  .trim().split("\n").slice(1)
  .map((line) => line.split("\t"))
  .map(([file, verdict]) => [file, TIER[verdict] ?? verdict]);
const IMAGES = ROWS.map(([file]) => file);
const EXPECTED = Object.fromEntries(ROWS);

const page = `<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><title>Provenance Lens smoke page</title>
<style>body{font-family:system-ui;margin:2rem}figure{display:inline-block;margin:1rem;text-align:center}img{width:128px;height:128px;image-rendering:pixelated;border:1px solid #ccc}figcaption{font-size:12px;margin-top:4px}</style>
</head><body>
<h1>Provenance Lens — M3 smoke page</h1>
<p>Right-click each image → “Verify provenance with Provenance Lens”. Expected verdicts below each image
(Verified requires the test CA in extension/trust/anchors.pem — see the placeholder file).</p>
${IMAGES.map((name) => `<figure><img src="/${name}" alt="${name}"><figcaption>${name}<br><strong>${EXPECTED[name]}</strong></figcaption></figure>`).join("\n")}
</body></html>`;

createServer((req, res) => {
  const path = req.url === "/" ? "/" : decodeURIComponent(req.url.split("?")[0]);
  if (path === "/") {
    res.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    res.end(page);
    return;
  }
  const name = path.slice(1);
  if (!IMAGES.includes(name)) {
    res.writeHead(404, { "content-type": "text/plain" });
    res.end("not found");
    return;
  }
  res.writeHead(200, {
    "content-type": name.endsWith(".png") ? "image/png" : "image/jpeg",
    "access-control-allow-origin": "*",
  });
  res.end(readFileSync(new URL(name, vectors)));
}).listen(PORT, () => console.log(`smoke page: http://localhost:${PORT}`));
