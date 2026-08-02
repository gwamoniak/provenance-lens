#!/usr/bin/env python3
"""Generate the Layer-2 calibration corpus (roadmap plan, W2).

Builds the labeled, held-out corpus that docs/CALIBRATION.md's TPR/FPR
tables are measured on. Three sets from the same 100 base photographs
(DIV2K validation HR — real photos, guaranteed watermark-free), each
center-cropped to 768x768:

  clean/    the crops, unmodified
  sd_dwt/   the crops with the SDXL 48-bit payload embedded by the VERBATIM
            reference embedder (ShieldMnt invisible-watermark EmbedMaxDct,
            'dwtDct' — same vendored code as scripts/gen_watermark_vectors.py)
  bzh/      the crops passed through IMATAG's bzh-watermarked SDXL VAE
            (imatag/stable-signature-bzh-sdxl-vae-medium, MIT): encode ->
            decode imprints the bzh watermark the detector was trained on.
            NOTE the honest asymmetry: real bzh deployments watermark SDXL
            *generations*; VAE-roundtripped photographs are a proxy.

Each set member is then run through the transformation battery (the file
in <set>/<transform>/):

  orig          the image as produced
  jpeg90/70/50  JPEG re-encode at that quality
  resize75/50   bicubic resize to 75% / 50% of each side
  crop80        central 80% crop (both sides)
  screenshot    bicubic resize to 110% then JPEG q85 — a cheap simulation
                of render-and-recapture

Measurement is Rust's job (the exact production probes):
  cargo run --release -p provenance-core --features stable-signature \
    --example calibrate -- <corpus-root> --model <bzh.onnx>

Requires: numpy, opencv-python(-headless), PyWavelets, torch (CPU),
diffusers. The corpus lives OUTSIDE the repo (bare-machine rule).

Usage:  py scripts/gen_calibration_corpus.py <div2k_valid_hr_dir> <out_root>
"""

import os
import sys

import cv2
import numpy as np
import pywt


# --- VERBATIM from ShieldMnt/invisible-watermark imwatermark/maxDct.py ---
# (EmbedMaxDct, encode path — same vendored copy as gen_watermark_vectors.py)
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
# --- end verbatim ---

SDXL_BITS = [int(bit) for bit in bin(0b101100111110110010010000011110111011000110011110)[2:]]
SIDE = 768


def center_crop(img, side):
    h, w = img.shape[:2]
    if h < side or w < side:
        return None
    y, x = (h - side) // 2, (w - side) // 2
    return img[y:y+side, x:x+side]


def transforms(bgr):
    """Yield (name, bgr_image_or_jpeg_bytes, extension)."""
    yield "orig", bgr, ".png"
    for q in (90, 70, 50):
        ok, buf = cv2.imencode(".jpg", bgr, [cv2.IMWRITE_JPEG_QUALITY, q])
        assert ok
        yield f"jpeg{q}", buf, ".jpg"
    for pct in (75, 50):
        side = bgr.shape[0] * pct // 100
        yield f"resize{pct}", cv2.resize(bgr, (side, side), interpolation=cv2.INTER_CUBIC), ".png"
    c = bgr.shape[0] * 80 // 100
    yield "crop80", center_crop(bgr, c), ".png"
    up = cv2.resize(bgr, (int(bgr.shape[1] * 1.10), int(bgr.shape[0] * 1.10)),
                    interpolation=cv2.INTER_CUBIC)
    ok, buf = cv2.imencode(".jpg", up, [cv2.IMWRITE_JPEG_QUALITY, 85])
    assert ok
    yield "screenshot", buf, ".jpg"


def write_variant(root, set_name, tname, stem, payload, ext):
    d = os.path.join(root, set_name, tname)
    os.makedirs(d, exist_ok=True)
    path = os.path.join(d, stem + ext)
    if isinstance(payload, np.ndarray) and payload.ndim == 1:  # encoded jpeg bytes
        payload.tofile(path)
    else:
        assert cv2.imwrite(path, payload), path
    return path


def main():
    if len(sys.argv) != 3:
        sys.exit("usage: gen_calibration_corpus.py <div2k_valid_hr_dir> <out_root>")
    src_dir, out_root = sys.argv[1], sys.argv[2]

    names = sorted(f for f in os.listdir(src_dir) if f.lower().endswith(".png"))
    bases = []
    for f in names:
        img = cv2.imread(os.path.join(src_dir, f))
        if img is None:
            continue
        crop = center_crop(img, SIDE)
        if crop is not None:
            bases.append((os.path.splitext(f)[0], crop))
        if len(bases) == 100:
            break
    print(f"bases: {len(bases)} usable {SIDE}x{SIDE} crops from {len(names)} files", flush=True)

    # clean + sd_dwt sets (fast, pure cv2/numpy)
    embedder = EmbedMaxDct(SDXL_BITS, wmLen=len(SDXL_BITS))
    for stem, crop in bases:
        for tname, payload, ext in transforms(crop):
            write_variant(out_root, "clean", tname, stem, payload, ext)
        marked = embedder.encode(crop.copy())
        for tname, payload, ext in transforms(marked):
            write_variant(out_root, "sd_dwt", tname, stem, payload, ext)
    print("clean + sd_dwt sets written", flush=True)

    # bzh set: encode->decode through IMATAG's watermarked SDXL VAE
    import torch
    from diffusers import AutoencoderKL
    vae = AutoencoderKL.from_pretrained("imatag/stable-signature-bzh-sdxl-vae-medium")
    vae.eval()
    for i, (stem, crop) in enumerate(bases):
        rgb = cv2.cvtColor(crop, cv2.COLOR_BGR2RGB).astype(np.float32) / 255.0
        x = torch.from_numpy(rgb).permute(2, 0, 1).unsqueeze(0) * 2.0 - 1.0
        with torch.no_grad():
            latents = vae.encode(x).latent_dist.mode()
            out = vae.decode(latents).sample
        out = ((out[0].permute(1, 2, 0).numpy() + 1.0) / 2.0 * 255.0).clip(0, 255).astype(np.uint8)
        marked = cv2.cvtColor(out, cv2.COLOR_RGB2BGR)
        for tname, payload, ext in transforms(marked):
            write_variant(out_root, "bzh", tname, stem, payload, ext)
        if (i + 1) % 10 == 0:
            print(f"bzh VAE roundtrip: {i + 1}/{len(bases)}", flush=True)
    print("bzh set written", flush=True)


if __name__ == "__main__":
    main()
