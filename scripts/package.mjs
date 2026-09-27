// Build and verify a candidate in an isolated ignored directory; never publishes.
import assert from "node:assert/strict";
import { readFileSync,writeFileSync,mkdirSync,mkdtempSync,cpSync,copyFileSync,readdirSync,existsSync } from "node:fs";
import { resolve,join,dirname,relative,sep } from "node:path";
import { execFileSync,spawnSync } from "node:child_process";
import { checkAnchors,checkSize,sha256 } from "./package_checks.mjs";
const [kind,...flags]=process.argv.slice(2), development=flags.includes("--development");
assert.ok(["extension","npm"].includes(kind) && flags.every(f=>f==="--development"),"usage: node scripts/package.mjs extension|npm [--development]");
const root=resolve("."), win=process.platform==="win32";
const run=(cmd,args=[],cwd=root)=>execFileSync(cmd,args,{cwd,encoding:"utf8",stdio:["ignore","pipe","inherit"]}).trim();
const node=(script,...args)=>console.log(run(process.execPath,[script,...args]));
const version=JSON.parse(readFileSync("extension/manifest.json","utf8")).version;
assert.equal(readFileSync("Cargo.toml","utf8").match(/^version = "([^"]+)"/m)?.[1],version,"Cargo/extension version mismatch");
const commit=run("git",["rev-parse","HEAD"]), dirty=!!run("git",["status","--porcelain"]);
assert.ok(development || !dirty,"non-development candidate requires a committed clean checkout");
const anchors=checkAnchors(readFileSync("extension/trust/anchors.pem","utf8"),development);
mkdirSync("target",{recursive:true}); mkdirSync("dist",{recursive:true});
const stage=mkdtempSync(join(root,"target","package-")), pkg=join(stage,"wasm"), bundle=kind==="extension"?join(stage,"extension"):pkg;
console.log(run("wasm-pack",["build","crates/provenance-wasm","--target","web","--out-dir",pkg,"--locked"]));
let optimization="wasm-pack release profile; no separate wasm-opt";
const opt=spawnSync("wasm-opt",["--version"],{encoding:"utf8"});
if(!opt.error && opt.status===0) {
  const help=run("wasm-opt",["--help"]), output=join(pkg,"optimized.wasm");
  run("wasm-opt",[join(pkg,"provenance_wasm_bg.wasm"),"-Os","--enable-bulk-memory",...(help.includes("bulk-memory-opt")?["--enable-bulk-memory-opt"]:[]),"--enable-sign-ext","--enable-mutable-globals","--enable-nontrapping-float-to-int","--enable-reference-types","-o",output]);
  assert.ok(existsSync(output) && readFileSync(output).length,"wasm-opt produced no output");
  copyFileSync(output,join(pkg,"provenance_wasm_bg.wasm")); optimization=opt.stdout.trim();
}
const wasm=checkSize(readFileSync(join(pkg,"provenance_wasm_bg.wasm")));
const p=JSON.parse(readFileSync(join(pkg,"package.json"),"utf8")); assert.equal(p.version,version,"npm version mismatch");
if(kind==="extension") {
  cpSync("extension",bundle,{recursive:true,filter:source=>relative(resolve("extension"),resolve(source)).split(sep)[0]!=="pkg"});
  cpSync(pkg,join(bundle,"pkg"),{recursive:true,filter:source=>!source.endsWith("optimized.wasm")});
} else {
  p.name="@provenance-lens/verify-wasm";
  p.description="Honest C2PA provenance verdicts in WebAssembly. No provenance data does NOT mean authentic.";
  p.keywords=["c2pa","content-credentials","provenance","wasm","verification"];
  p.files=[...new Set([...p.files,"provenance_wasm_bg.wasm.d.ts","LICENSE-MIT","LICENSE-APACHE","build-evidence.json"])];
  writeFileSync(join(pkg,"package.json"),JSON.stringify(p,null,2)+"\n");
}
for(const license of ["LICENSE-MIT","LICENSE-APACHE"]) copyFileSync(license,join(bundle,license));
const evidence={format_version:1,kind,version,development,source_commit:commit,source_dirty:dirty,lock_sha256:sha256(readFileSync("Cargo.lock")),
  toolchain:run("rustc",["--version"]),wasm_pack:run("wasm-pack",["--version"]),node:process.version,optimization,
  wasm,anchors:kind==="extension"?anchors:null,model:null,features:"standalone-wasm; no pixel detector; no registry client",approval:"candidate only; merge, trust freshness and release approval are separate"};
writeFileSync(join(bundle,"build-evidence.json"),JSON.stringify(evidence,null,2)+"\n");
const required=kind==="extension"?["manifest.json","background.js","background.html","content/scan.js","lib/scan_support.js","lib/engine.js","lib/report_view.js","popup/popup.html","popup/popup.js","popup/popup.css","inspect/inspect.html","inspect/inspect.js","inspect/inspect.css","inspect/evidence.js","trust/anchors.pem","pkg/provenance_wasm.js","pkg/provenance_wasm_bg.wasm"]:["package.json","provenance_wasm.js","provenance_wasm_bg.wasm","provenance_wasm.d.ts","provenance_wasm_bg.wasm.d.ts"];
for(const file of [...required,"LICENSE-MIT","LICENSE-APACHE","build-evidence.json"]) assert.ok(existsSync(join(bundle,file)),`missing ${file}`);
const out=mkdtempSync(join(root,"dist",`${development?"development-":"candidate-"}${kind}-${version}-${commit.slice(0,8)}-`));
let archive, extracted=join(stage,"extracted"); mkdirSync(extracted);
if(kind==="extension") {
  archive=join(out,`provenance-lens-${version}.zip`);
  if(win) { const tar=join(process.env.SystemRoot,"System32","tar.exe"); run(tar,["-a","-cf",archive,"."],bundle); run(tar,["-xf",archive,"-C",extracted]); }
  else { run("zip",["-qr",archive,"."],bundle); run("unzip",["-q",archive,"-d",extracted]); }
} else {
  const args=["pack","--ignore-scripts","--json","--pack-destination",out];
  const packed=win?run(process.execPath,[join(dirname(process.execPath),"node_modules/npm/bin/npm-cli.js"),...args],pkg):run("npm",args,pkg);
  archive=join(out,JSON.parse(packed)[0].filename);
  run(win?join(process.env.SystemRoot,"System32","tar.exe"):"tar",["-xf",archive,"-C",extracted]); extracted=join(extracted,"package");
}
const hashes=dir=>Object.fromEntries(readdirSync(dir,{recursive:true,withFileTypes:true}).filter(e=>e.isFile()).map(e=>{
  const path=join(e.parentPath,e.name);return [relative(dir,path).split(sep).join("/"),sha256(readFileSync(path))];
}).sort(([a],[b])=>a.localeCompare(b)));
const contents=hashes(extracted);
for(const file of [...required,"LICENSE-MIT","LICENSE-APACHE","build-evidence.json"]) assert.equal(contents[file],sha256(readFileSync(join(bundle,file))),`archive changed or omitted ${file}`);
if(kind==="extension") assert.deepEqual(contents,hashes(bundle),"extension archive differs from staged source");
else for(const file of Object.keys(contents)) assert.equal(contents[file],sha256(readFileSync(join(bundle,file))),`npm archive changed ${file}`);
node("scripts/wasm_smoke.mjs",kind==="extension"?join(extracted,"pkg"):extracted);
writeFileSync(join(out,"artifact-evidence.json"),JSON.stringify({...evidence,artifact:{file:relative(out,archive),bytes:readFileSync(archive).length,sha256:sha256(readFileSync(archive))},contents},null,2)+"\n");
console.log(`Prepared ${archive}\nExtracted smoke directory: ${extracted}\nNo publication performed.`);
