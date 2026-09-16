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
let sameSigned = true;

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
<p><a href="/lifecycle">N2 image lifecycle test page</a></p>
<p>Right-click each image → “Verify provenance with Provenance Lens”. Expected verdicts below each image
(Verified requires the test CA in extension/trust/anchors.pem — see the placeholder file).</p>
${IMAGES.map((name) => `<figure><img src="/${name}" alt="${name}"><figcaption>${name}<br><strong>${EXPECTED[name]}</strong></figcaption></figure>`).join("\n")}
<figure><img src="http://127.0.0.1:${PORT}/nocors/valid_signed.jpg" alt="second-origin case"><figcaption>127.0.0.1 (no CORS)<br><strong>U7: “· · ·” until 127.0.0.1 granted, then Verified</strong></figcaption></figure>
<p>The last image is the U7 second-origin case: served from the 127.0.0.1 hostname WITHOUT a CORS
header, so the engine cannot fetch its bytes until that host is granted (page pill shows the neutral
“not examined” marker; the popup offers “Allow access to 127.0.0.1 and verify”).</p>
</body></html>`;

const lifecyclePage = `<!doctype html><html lang="en"><meta charset="utf-8">
<title>Provenance Lens image lifecycle checks</title>
<style>body{font:16px system-ui;margin:2rem}img{width:160px;height:160px}button{margin:.3rem}</style>
<h1>Image lifecycle checks</h1>
<p>Enable scanning using the extension popup. Signed fixtures require the test CA in an isolated test extension copy;
the automated scan_browser_test.mjs supplies that root without changing production anchors.</p>
<p>Change the image, try a delayed source, and remove/reinsert it. Old badges must disappear as the source changes.
Use the extension's Stop scanning control to remove badges without reloading. A CDN host grant alone must not enable scanning.</p>
<div id="images"><img id="subject" src="/valid_signed.png" alt="changing provenance specimen"></div>
<p><button id="signed">Signed</button><button id="unsigned">Unsigned</button>
<button id="responsive">Change srcset</button><button id="delay">Delayed signed source</button>
<button id="same">Reuse URL with changed bytes</button><button id="detach">Remove/reinsert</button></p>
<p>For the reused URL, activate the badge with Enter or use Verify again in the popup. The retrieval time must change.</p>
<script>
const img = document.getElementById('subject');
function use(url) { img.removeAttribute('srcset'); img.src = url; }
document.getElementById('signed').onclick = () => use('/valid_signed.png');
document.getElementById('unsigned').onclick = () => use('/stripped.png');
document.getElementById('responsive').onclick = () => { img.srcset = '/stripped.png?responsive 1x'; };
document.getElementById('delay').onclick = () => use('/delayed.png');
document.getElementById('same').onclick = async () => { await fetch('/lifecycle-flip', {method:'POST'}); use('/same.png'); };
document.getElementById('detach').onclick = () => { if(img.isConnected) img.remove(); else document.getElementById('images').append(img); };
</script></html>`;

createServer((req, res) => {
  res.setHeader("cache-control", "no-store");
  const path = req.url === "/" ? "/" : decodeURIComponent(req.url.split("?")[0]);
  if (path === "/lifecycle") {
    res.writeHead(200, { "content-type": "text/html; charset=utf-8" }); res.end(lifecyclePage); return;
  }
  if (path === "/lifecycle-flip" && req.method === "POST") {
    sameSigned = !sameSigned; res.writeHead(204); res.end(); return;
  }
  if (path === "/same.png" || path === "/delayed.png") {
    res.writeHead(200, { "content-type": "image/png", "access-control-allow-origin": "*" });
    const send = () => res.end(readFileSync(new URL(path === "/same.png" && !sameSigned ? "stripped.png" : "valid_signed.png", vectors)));
    if (path === "/delayed.png") setTimeout(send, 1500); else send();
    return;
  }
  if (path === "/") {
    res.writeHead(200, { "content-type": "text/html; charset=utf-8" });
    res.end(page);
    return;
  }
  // U7 second-origin case: /nocors/<vector> serves WITHOUT the CORS header,
  // so the extension's background fetch is blocked until the user grants
  // the requesting hostname (the page embeds it via 127.0.0.1, which is a
  // different host than localhost in match patterns).
  const noCors = path.startsWith("/nocors/");
  const name = noCors ? path.slice("/nocors/".length) : path.slice(1);
  if (!IMAGES.includes(name)) {
    res.writeHead(404, { "content-type": "text/plain" });
    res.end("not found");
    return;
  }
  const contentType = name.endsWith(".png")
    ? "image/png"
    : name.endsWith(".svg")
      ? "image/svg+xml"
      : "image/jpeg";
  const headers = { "content-type": contentType };
  if (!noCors) headers["access-control-allow-origin"] = "*";
  res.writeHead(200, headers);
  res.end(readFileSync(new URL(name, vectors)));
}).listen(PORT, () => console.log(`smoke page: http://localhost:${PORT}`));
