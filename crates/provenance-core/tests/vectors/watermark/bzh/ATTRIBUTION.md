# Attribution

`watermarked.png` and `not_watermarked.png` are copied UNMODIFIED from
IMATAG's model repository
<https://huggingface.co/imatag/stable-signature-bzh-detector-resnet18>
(`examples/` directory), published under the MIT license.

They are the vendor's own known-answer pair for the bzh Stable Signature
watermark classifier: `watermarked.png` carries the bzh watermark
(reference logits[0] = −11.3869 → watermarked under the vendor's
`logits[0] < 0` rule), `not_watermarked.png` does not (reference
logits[0] = +27.1844). Reference values printed by
`scripts/export_stable_signature_onnx.py` against the upstream weights
(sha256 `d11e6f1a0a339c973ac3a433c43d67d2ddb2f37f26c456061d1778ea7bf9f70e`,
fetched 2026-08-01).

Used by `tests/stable_signature.rs`, which runs only when
`PROVENANCE_LENS_BZH_ONNX` points at the exported model — no model file is
committed (bare-machine rule).
