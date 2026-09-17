import { loadVerifier } from "../lib/engine.js";
import { credentialLines, appendLines } from "../lib/report_view.js";
import { inspectFile, compareReports } from "./evidence.js";

const files = document.getElementById("files"), results = document.getElementById("results");
const status = document.getElementById("status"), comparison = document.getElementById("comparison");
let generation = 0;
const element = (tag, text) => { const node = document.createElement(tag); node.textContent = text; return node; };
function clear() {
  generation++; results.replaceChildren(); comparison.hidden = true; files.value = "";
  status.textContent = "Files and results cleared. Choose one or two files.";
}
document.getElementById("clear").addEventListener("click", clear);
async function examine(selected) {
  const current = ++generation;
  results.replaceChildren(); comparison.hidden = true;
  if (!selected.length || selected.length > 2) { status.textContent = "Choose exactly one or two files; nothing examined."; return; }
  status.textContent = "Examining selected files locally…";
  let verifier;
  try { verifier = await loadVerifier(); }
  catch (error) { if (current === generation) status.textContent = `Not examined: ${error.message}`; return; }
  if (current !== generation) return;
  const reports = [];
  for (const [index, file] of selected.entries()) {
    let receipt, error;
    try { receipt = await inspectFile(file, verifier); } catch (err) { error = err; }
    if (current !== generation) return;
    const panel = element("article", "");
    panel.append(element("h2", `${index === 0 ? "A" : "B"}: ${file.name.split(/[\\/]/).pop()}`));
    if (error) { const note = element("p", `Not examined: ${error.message}`); note.className = "error"; panel.append(note); reports.push(null); }
    else {
      const report = receipt.report;
      reports.push(report);
      const phrase = element("p", report.phrase);
      if (["verified", "indicated", "inconclusive", "tampered"].includes(report.verdict)) phrase.className = report.verdict;
      panel.append(phrase, element("p", `${receipt.asset.byte_length} bytes · SHA-256 ${receipt.asset.sha256}`));
      const findings = element("ul", "");
      appendLines(findings, report.findings.map((f) => [f.layer, `${f.status.replaceAll("_", " ")}${f.detail ? ` — ${f.detail}` : ""}`]));
      appendLines(findings, credentialLines(report));
      panel.append(findings, element("p", receipt.notice));
      const details = element("details", ""); details.open = true;
      const json = JSON.stringify(receipt, null, 2);
      details.append(element("summary", "Preview the complete receipt before saving"), element("pre", json));
      const save = element("button", "Save unsigned JSON receipt"); save.type = "button";
      save.addEventListener("click", () => {
        const url = URL.createObjectURL(new Blob([json], { type: "application/json" }));
        const link = document.createElement("a"); link.href = url;
        link.download = `provenance-lens-${receipt.asset.sha256.slice(0, 12)}.json`;
        link.click(); setTimeout(() => URL.revokeObjectURL(url), 1000);
      });
      panel.append(details, save);
    }
    results.append(panel);
  }
  if (selected.length === 2) {
    const list = document.getElementById("changes"); list.replaceChildren();
    for (const text of compareReports(...reports)) list.append(element("li", text));
    comparison.hidden = false;
  }
  status.textContent = "Local examination finished. Results describe only the selected bytes.";
}
files.addEventListener("change", () => examine(Array.from(files.files)));
const zone = document.getElementById("drop-zone");
document.addEventListener("dragover", (event) => event.preventDefault());
document.addEventListener("drop", (event) => event.preventDefault());
zone.addEventListener("drop", (event) => { event.preventDefault(); examine(Array.from(event.dataTransfer.files)); });
loadVerifier().then(() => { if (!generation) status.textContent = "Ready. Files can be inspected with the network disconnected."; })
  .catch((error) => { if (!generation) status.textContent = `Not examined: ${error.message}`; });
