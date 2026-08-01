#!/usr/bin/env python3
"""Generate the Layer-2 watermark known-answer vectors (roadmap plan, W1).

The embedding class below is copied VERBATIM (encode path, unused SVD/decode
methods dropped) from the reference implementation the Stable Diffusion
pipelines actually call: ShieldMnt/invisible-watermark,
imwatermark/maxDct.py, class EmbedMaxDct — the library's method name is
'dwtDct', though its embed/decode path applies no DCT (Haar DWT + max-abs
coefficient quantization on 4x4 LL blocks of the U chrominance channel;
scales=[0,36,36] with a range(2) channel loop means only channel 1 is used).
Generating with the reference code, not a reimplementation, is the point:
the Rust decoder is then tested against ground truth it did not produce.

Payloads (both pinned from their primary sources, 2026-07-31):
  - SDXL (diffusers src/diffusers/pipelines/stable_diffusion_xl/watermark.py):
      WATERMARK_MESSAGE = 0b101100111110110010010000011110111011000110011110
      set_watermark("bits", [int(b) for b in bin(WATERMARK_MESSAGE)[2:]])  # 48 bits
  - SD v1 (CompVis/stable-diffusion scripts/txt2img.py):
      wm = "StableDiffusionV1"; set_watermark('bytes', wm.encode('utf-8'))
      -> np.unpackbits, MSB-first per byte, 136 bits

Deterministic: the base image is a fixed trigonometric pattern and PNG is
lossless, so rerunning regenerates equivalent vectors (byte-identity may vary
with OpenCV's PNG encoder version; the *decoded pixels* are what the tests
consume). Requires: numpy, PyWavelets, opencv-python(-headless).

Run from the repo root:  py scripts/gen_watermark_vectors.py
"""

import os

import cv2
import numpy as np
import pywt


# --- VERBATIM from ShieldMnt/invisible-watermark imwatermark/maxDct.py ---
# (EmbedMaxDct, encode path; decode/SVD methods omitted as unused here)
class EmbedMaxDct(object):
    def __init__(self, watermarks=[], wmLen=8, scales=[0, 36, 36], block=4):
        self._watermarks = watermarks
        self._wmLen = wmLen
        self._scales = scales
        self._block = block

    def encode(self, bgr):
        (row, col, channels) = bgr.shape

        yuv = cv2.cvtColor(bgr, cv2.COLOR_BGR2YUV)

        for channel in range(2):
            if self._scales[channel] <= 0:
                continue

            ca1, (h1, v1, d1) = pywt.dwt2(yuv[:row//4*4, :col//4*4, channel], 'haar')
            self.encode_frame(ca1, self._scales[channel])

            yuv[:row//4*4, :col//4*4, channel] = pywt.idwt2((ca1, (v1, h1, d1)), 'haar')

        bgr_encoded = cv2.cvtColor(yuv, cv2.COLOR_YUV2BGR)
        return bgr_encoded

    def diffuse_dct_matrix(self, block, wmBit, scale):
        pos = np.argmax(abs(block.flatten()[1:])) + 1
        i, j = pos // self._block, pos % self._block
        val = block[i][j]
        if val >= 0.0:
            block[i][j] = (val//scale + 0.25 + 0.5 * wmBit) * scale
        else:
            val = abs(val)
            block[i][j] = -1.0 * (val//scale + 0.25 + 0.5 * wmBit) * scale
        return block

    def encode_frame(self, frame, scale):
        (row, col) = frame.shape
        num = 0
        for i in range(row//self._block):
            for j in range(col//self._block):
                block = frame[i*self._block: i*self._block + self._block,
                              j*self._block: j*self._block + self._block]
                wmBit = self._watermarks[(num % self._wmLen)]

                diffusedBlock = self.diffuse_dct_matrix(block, wmBit, scale)
                frame[i*self._block: i*self._block + self._block,
                      j*self._block: j*self._block + self._block] = diffusedBlock

                num = num+1

    def decode(self, bgr):
        (row, col, channels) = bgr.shape

        yuv = cv2.cvtColor(bgr, cv2.COLOR_BGR2YUV)

        scores = [[] for i in range(self._wmLen)]
        for channel in range(2):
            if self._scales[channel] <= 0:
                continue

            ca1, (h1, v1, d1) = pywt.dwt2(yuv[:row//4*4, :col//4*4, channel], 'haar')

            scores = self.decode_frame(ca1, self._scales[channel], scores)

        avgScores = list(map(lambda l: np.array(l).mean(), scores))

        bits = (np.array(avgScores) * 255 > 127)
        return bits

    def decode_frame(self, frame, scale, scores):
        (row, col) = frame.shape
        num = 0

        for i in range(row//self._block):
            for j in range(col//self._block):
                block = frame[i*self._block: i*self._block + self._block,
                              j*self._block: j*self._block + self._block]

                score = self.infer_dct_matrix(block, scale)
                wmBit = num % self._wmLen
                scores[wmBit].append(score)
                num = num + 1

        return scores

    def infer_dct_matrix(self, block, scale):
        pos = np.argmax(abs(block.flatten()[1:])) + 1
        i, j = pos // self._block, pos % self._block

        val = block[i][j]
        if val < 0:
            val = abs(val)

        if (val % scale) > 0.5 * scale:
            return 1
        else:
            return 0
# --- end verbatim ---


def base_image(seed):
    """768x768 deterministic photo-like BGR image (textured, mid-range).

    Smooth synthetic gradients are PATHOLOGICAL for this scheme: embedding a
    0-bit shrinks the block's max LL coefficient by up to scale*0.75, and on
    low-contrast blocks the decode-time argmax then lands on an unmodified
    neighbour — the reference decoder itself fails on such a base. Blurred
    seeded noise gives every 4x4 block a clear max coefficient, like a real
    photograph; 768x768 gives each payload bit enough block votes to survive
    the reference encode chain's own uint8 truncations. numpy's legacy
    RandomState is stability-guaranteed, so this stays reproducible across
    numpy versions. Seeds are chosen PER VECTOR (scanned 2026-07-31) so every
    bit-slot vote clears the decision threshold with margin >= 0.03 — a
    borderline slot would make the cross-implementation known-answer test
    flaky; the self-check below enforces the margin on every regeneration.
    """
    rng = np.random.RandomState(seed)
    noise = rng.randint(0, 256, (768, 768, 3)).astype(np.float64)
    textured = cv2.GaussianBlur(noise, (0, 0), sigmaX=2.5)
    # Rescale to mid-range so watermarking never clips against 0/255.
    lo, hi = textured.min(), textured.max()
    scaled = (textured - lo) / (hi - lo) * 160.0 + 48.0
    return np.clip(scaled, 0, 255).astype(np.uint8)


def slot_means(bgr, wmLen):
    """Per-bit-slot vote means via the verbatim decode path (for the margin check)."""
    embed = EmbedMaxDct(wmLen=wmLen)
    scores = embed.decode_frame(
        pywt.dwt2(cv2.cvtColor(bgr, cv2.COLOR_BGR2YUV)[
            :bgr.shape[0]//4*4, :bgr.shape[1]//4*4, 1], 'haar')[0],
        36, [[] for _ in range(wmLen)])
    return [float(np.mean(s)) for s in scores]


SDXL_BITS = [int(bit) for bit in bin(0b101100111110110010010000011110111011000110011110)[2:]]
SDV1_BITS = list(np.unpackbits(np.frombuffer(b"StableDiffusionV1", dtype=np.uint8)))
assert len(SDXL_BITS) == 48 and len(SDV1_BITS) == 136

out_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..",
                       "crates", "provenance-core", "tests", "vectors", "watermark")
os.makedirs(out_dir, exist_ok=True)

THRESHOLD = 127.0 / 255.0
MIN_MARGIN = 0.03

for name, bits, seed in (("wm_sdxl.png", SDXL_BITS, 2), ("wm_sdv1.png", SDV1_BITS, 10)):
    encoded = EmbedMaxDct(bits, wmLen=len(bits)).encode(base_image(seed))
    path = os.path.join(out_dir, name)
    assert cv2.imwrite(path, encoded), f"imwrite failed for {path}"
    # A corpus that refuses to lie (house rule): the written file must decode
    # to its payload through the verbatim reference decode path, from disk —
    # and every bit-slot vote must clear the threshold with real margin, or
    # the cross-implementation known-answer test would be flaky.
    readback = cv2.imread(path)
    decoded = list(EmbedMaxDct(wmLen=len(bits)).decode(readback).astype(int))
    assert decoded == bits, f"{name}: reference decode does not recover the payload"
    margin = min(abs(m - THRESHOLD) for m in slot_means(readback, len(bits)))
    assert margin >= MIN_MARGIN, f"{name}: worst slot margin {margin:.3f} < {MIN_MARGIN}"
    print(f"wrote {path} ({len(bits)}-bit payload, reference-decode verified, "
          f"margin {margin:.3f})")

# A clean twin (same construction, sdxl's seed), so the negative test uses
# statistically identical content.
clean_path = os.path.join(out_dir, "clean_base.png")
assert cv2.imwrite(clean_path, base_image(2))
clean = cv2.imread(clean_path)
for bits in (SDXL_BITS, SDV1_BITS):
    assert list(EmbedMaxDct(wmLen=len(bits)).decode(clean).astype(int)) != bits, \
        "clean twin must not decode to a payload"
print(f"wrote {clean_path} (no watermark, negative-checked)")
