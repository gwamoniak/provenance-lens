// Independent SHA-256/Ed25519 verifier for the local pilot; never a product client.
import assert from "node:assert/strict";
import { createHash, createPublicKey, verify } from "node:crypto";
import { mkdtempSync, mkdirSync, readFileSync, copyFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { execFileSync } from "node:child_process";
const dir = mkdtempSync(join(tmpdir(), "lens-registry-"));
const exe = resolve(process.argv[2] || `target/debug/plog${process.platform === "win32" ? ".exe" : ""}`);
const run = (...args) => execFileSync(exe, args, { encoding: "utf8" });
const sha = (...b) => createHash("sha256").update(Buffer.concat(b)).digest();
const key = (file) => createPublicKey({ key: Buffer.concat([Buffer.from("302a300506032b6570032100", "hex"), Buffer.from(readFileSync(file, "utf8").trim(), "hex")]), format: "der", type: "spki" });
try {
  mkdirSync(join(dir,"registrants"));
  run("keygen",join(dir,"private"),join(dir,"log-key.pub"));
  copyFileSync(join(dir,"log-key.pub"),join(dir,"registrants/test.pub"));
  const pub = key(join(dir,"log-key.pub"));
  const leaves = [];
  for (let n = 0; n < 3; n++) {
    run("append",dir,"--key",join(dir,"private"),"--registrant","test","--source-type","http://example/ai","--image",resolve("crates/provenance-core/tests/vectors/watermark/clean_base.png"));
    const bytes = readFileSync(join(dir,`entries/${String(n).padStart(8,"0")}.bin`));
    assert.equal(bytes.subarray(0,4).toString(),"PLE1");
    assert.ok(verify(null,bytes.subarray(0,-64),pub,bytes.subarray(-64)));
    leaves.push(sha(Buffer.from([0]),bytes));
  }
  run("index",dir); run("checkpoint",dir,"--key",join(dir,"private"));
  const note = readFileSync(join(dir,"checkpoints/latest.note"),"utf8");
  const lines = note.split("\n");
  const expected = sha(Buffer.from([1]),sha(Buffer.from([1]),leaves[0],leaves[1]),leaves[2]);
  assert.equal(lines[1],"3"); assert.equal(lines[2],expected.toString("hex"));
  assert.ok(verify(null,Buffer.from(lines.slice(0,4).join("\n")+"\n"),pub,Buffer.from(lines[5].split(" ")[2],"base64")));
  assert.match(run("verify",dir,"--log-key",join(dir,"log-key.pub")),/3 published entries/);
  console.log("Independent Node crypto: three-entry unbalanced tree and entry/checkpoint signatures pass (test keys only).");
} finally { rmSync(dir,{recursive:true}); }
