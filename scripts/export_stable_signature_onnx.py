#!/usr/bin/env python3
"""Export IMATAG's bzh Stable Signature detector to ONNX (roadmap plan, W2).

IMATAG publishes the detector (MIT license) as a Hugging Face transformers
model — `imatag/stable-signature-bzh-detector-resnet18`, a ResNet-18
`ResNetForImageClassification` in `pytorch_model.bin` — with NO ONNX
artifact. Provenance Lens runs it through pure-Rust `tract-onnx`, so this
one-time script produces the ONNX file the CLI consumes via
`lens verify --watermark-model <file>`. The exported artifact and the
weights stay OUT of the repo; the script prints the sha256 so the export
is auditable (torch versions make exports non-byte-identical).

It also prints the model's logits for the two example images IMATAG
commits in the repo (`examples/watermarked.png`, `examples/
not_watermarked.png`) — the reference values the Rust known-answer test
checks against (decision rule, from the vendor's own demo:
watermarked <=> logits[0] < 0).

Usage (needs: pip install torch --index-url .../whl/cpu, transformers, onnx):

    py scripts/export_stable_signature_onnx.py <LOCAL_MODEL_SRC> [OUT.onnx]

Supply the local snapshot directory containing the reviewed weights. Both
the upstream weights and the calibrated ONNX export are digest-checked.
"""

import hashlib
import os
import sys

import torch
from transformers import AutoModelForImageClassification, BlipImageProcessor
from PIL import Image

SRC = sys.argv[1] if len(sys.argv) > 1 else "imatag/stable-signature-bzh-detector-resnet18"
OUT = sys.argv[2] if len(sys.argv) > 2 else "stable-signature-bzh-resnet18.onnx"
EXPECTED_WEIGHTS = "d11e6f1a0a339c973ac3a433c43d67d2ddb2f37f26c456061d1778ea7bf9f70e"
EXPECTED_ONNX = "4f7d80027d5d7ecf5c176fabc3d86c8ccd791cc5c830d23a96114425cd1f3e57"
# Use an explicit local snapshot; a moving hub revision cannot inherit calibration.
weights = os.path.join(SRC, "pytorch_model.bin")
if not os.path.isfile(weights):
    raise SystemExit("Supply a local snapshot containing the reviewed pytorch_model.bin")
with open(weights, "rb") as handle:
    if hashlib.file_digest(handle, "sha256").hexdigest() != EXPECTED_WEIGHTS:
        raise SystemExit("Unrecognized weights: not the calibrated IMATAG bzh model")

model = AutoModelForImageClassification.from_pretrained(SRC)
model.eval()


class LogitsOnly(torch.nn.Module):
    """ONNX needs a tensor-returning forward, not an HF output object."""

    def __init__(self, inner):
        super().__init__()
        self.inner = inner

    def forward(self, pixel_values):
        return self.inner(pixel_values=pixel_values).logits


wrapper = LogitsOnly(model)
dummy = torch.zeros(1, 3, 512, 512)
try:
    torch.onnx.export(wrapper, (dummy,), OUT,
                      input_names=["pixel_values"], output_names=["logits"],
                      opset_version=17, dynamo=False)
except TypeError:  # older/newer torch without the dynamo kwarg
    torch.onnx.export(wrapper, (dummy,), OUT,
                      input_names=["pixel_values"], output_names=["logits"],
                      opset_version=17)

sha = hashlib.sha256(open(OUT, "rb").read()).hexdigest()
print(f"exported {OUT}  ({os.path.getsize(OUT)} bytes)")
print(f"sha256   {sha}")
if sha != EXPECTED_ONNX:
    raise SystemExit("Export differs from the calibrated ONNX. Production loading refuses it; review and recalibrate before updating the pinned digest.")

# BlipImageProcessor EXPLICITLY, as the vendor's demo does: transformers 5's
# AutoImageProcessor maps this repo to a fast torchvision processor that
# mangles the preprocessing (observed: watermarked example -> +7.8 instead
# of -11.4). The slow Blip path matches manual bicubic-512/rescale/normalize
# preprocessing to 1e-6 — which is exactly what the Rust detector implements.
processor = BlipImageProcessor.from_pretrained(SRC)
for name in ("examples/watermarked.png", "examples/not_watermarked.png"):
    path = os.path.join(SRC, name) if os.path.isdir(SRC) else None
    if path is None or not os.path.exists(path):
        print(f"reference logits: {name} not found next to MODEL_SRC, skipping")
        continue
    img = Image.open(path).convert("RGB")
    with torch.no_grad():
        logits = wrapper(processor(img, return_tensors="pt")["pixel_values"])
    l0 = float(logits[0, 0])
    verdict = "watermarked" if l0 < 0 else "not watermarked"
    print(f"reference logits {name}: {[round(float(v), 4) for v in logits[0]]}"
          f"  -> logits[0]={l0:.4f} -> {verdict}")
