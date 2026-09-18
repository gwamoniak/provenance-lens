// Source-grouped corpus audit + exact one-sided 95% Clopper-Pearson upper bound.
// Input JSON: {samples:[{path,sha256,source_id,split,source,license,clean_basis}],results?:[{detector,source_id,outcome}]}.
// Human provenance review is still necessary: software cannot prove source independence.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { dirname, resolve } from "node:path";
import { pathToFileURL } from "node:url";
export function upper95(hits, n) {
  assert.ok(Number.isSafeInteger(n) && n > 0 && Number.isSafeInteger(hits) && hits >= 0 && hits <= n);
  if (hits === n) return 1;
  if (hits === 0) return -Math.expm1(Math.log(0.05) / n);
  const cdf = (p) => {
    let term = n * Math.log1p(-p), sum = term;
    for (let i = 1; i <= hits; i++) {
      term += Math.log(n-i+1)-Math.log(i)+Math.log(p)-Math.log1p(-p);
      const max = Math.max(sum,term); sum = max + Math.log(Math.exp(sum-max)+Math.exp(term-max));
    }
    return Math.exp(sum);
  };
  let lo = hits/n, hi = 1;
  for (let i=0;i<80;i++) { const mid=(lo+hi)/2; if(cdf(mid)>0.05) lo=mid; else hi=mid; }
  return (lo+hi)/2;
}
export function audit(manifest, base, read = readFileSync) {
  assert.ok(Array.isArray(manifest.samples) && manifest.samples.length > 0,"samples required");
  const sources = new Map(), hashes = new Map(), paths = new Set();
  for (const s of manifest.samples) {
    for(const field of ["path","sha256","source_id","split","source","license","clean_basis"]) assert.ok(typeof s[field]==="string" && s[field].trim(),`missing ${field}`);
    assert.ok(["calibration","holdout"].includes(s.split),"invalid split");
    assert.match(s.sha256,/^[a-f0-9]{64}$/);
    const file=resolve(base,s.path); assert.ok(!paths.has(file),"duplicate path"); paths.add(file);
    assert.equal(createHash("sha256").update(read(file)).digest("hex"),s.sha256,"bytes changed");
    assert.ok(!sources.has(s.source_id) || sources.get(s.source_id)===s.split,"source crosses calibration/holdout");
    assert.ok(!hashes.has(s.sha256) || hashes.get(s.sha256)===s.source_id,"identical bytes assigned different source IDs");
    sources.set(s.source_id,s.split); hashes.set(s.sha256,s.source_id);
  }
  const detectors = new Map();
  for(const r of manifest.results || []) {
    assert.ok(typeof r.detector==="string" && r.detector.trim(),"detector required");
    assert.equal(sources.get(r.source_id),"holdout","result must belong to held-out source");
    assert.ok(["hit","negative","unavailable"].includes(r.outcome),"invalid outcome");
    if(!detectors.has(r.detector)) detectors.set(r.detector,new Map());
    const rows=detectors.get(r.detector); assert.ok(!rows.has(r.source_id),"only one prespecified result per source and detector"); rows.set(r.source_id,r.outcome);
  }
  const holdout=[...sources.values()].filter(s=>s==="holdout").length;
  return { files:manifest.samples.length, independent_source_groups_claimed:sources.size, holdout_source_groups:holdout,
    warning:"Source identity, licensing, negative labels and independence need human review; results must come from the pinned production probe, with its run evidence retained.",
    detectors:[...detectors].map(([detector,rows])=>{
      const hits=[...rows.values()].filter(x=>x==="hit").length, completed=[...rows.values()].filter(x=>x!=="unavailable").length;
      const upper=completed ? upper95(hits,completed) : null;
      return {detector,hits,completed,unavailable:rows.size-completed,missing:holdout-rows.size,upper_one_sided_95:upper,
        numerical_target_met:completed>=3000 && rows.size===holdout && completed===holdout && upper<0.001};
    }) };
}
if(process.argv[1] && import.meta.url===pathToFileURL(resolve(process.argv[1])).href) {
  const file=resolve(process.argv[2] || "corpus-manifest.json");
  console.log(JSON.stringify(audit(JSON.parse(readFileSync(file,"utf8")),dirname(file)),null,2));
}
