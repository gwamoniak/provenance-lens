#!/usr/bin/env python3
"""Generate PDQ known-answer expectations from the reference implementation
(registry plan, G1).

`pdqhash` (pip) compiles Meta's reference C++ from facebook/ThreatExchange —
the same source `crates/provenance-core/src/layers/pdq.rs` transcribes — so
these expectations are ground truth the Rust code did not produce. Inputs
are the repo's already-committed PNG vectors (lossless: both sides see
identical pixels; JPEG would not be decoder-stable), plus one non-square
crop generated here so the decimation path is exercised off the square case.

Writes `crates/provenance-core/tests/vectors/pdq/kats.tsv`:
    <path relative to tests/vectors> \t <64-hex-char hash> \t <quality>
Hex packing matches the Rust module: bit k in byte k/8 at position 7-k%8.

Run from the repo root:  py scripts/gen_pdq_kats.py   (needs pdqhash, Pillow)
"""

import os

import numpy as np
import pdqhash
from PIL import Image

VECTORS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..",
                       "crates", "provenance-core", "tests", "vectors")
OUT_DIR = os.path.join(VECTORS, "pdq")
os.makedirs(OUT_DIR, exist_ok=True)

# A deterministic non-square vector: top 768x400 of the committed clean base.
nonsquare_path = os.path.join(OUT_DIR, "nonsquare.png")
Image.open(os.path.join(VECTORS, "watermark", "clean_base.png")) \
    .crop((0, 0, 768, 400)).save(nonsquare_path)

FILES = [
    "watermark/clean_base.png",
    "watermark/wm_sdv1.png",
    "watermark/bzh/watermarked.png",
    "watermark/bzh/not_watermarked.png",
    "pdq/nonsquare.png",
]


def to_hex(bits):
    assert len(bits) == 256
    out = bytearray(32)
    for k, bit in enumerate(bits):
        if bit:
            out[k // 8] |= 1 << (7 - (k % 8))
    return out.hex()


rows = []
for rel in FILES:
    arr = np.array(Image.open(os.path.join(VECTORS, rel)).convert("RGB"))
    bits, quality = pdqhash.compute(arr)
    rows.append(f"{rel}\t{to_hex(list(bits))}\t{int(quality)}")
    print(rows[-1])

with open(os.path.join(OUT_DIR, "kats.tsv"), "w", newline="\n") as fh:
    fh.write("file\thash_hex\tquality\n")
    fh.write("\n".join(rows) + "\n")
print(f"wrote {os.path.join(OUT_DIR, 'kats.tsv')}")
