---
name: watermark-detection
description: The detector-trait contract and per-vendor integration notes for Layer 2 (invisible watermarks). Load before touching crates/provenance-core/src/layers/watermark.rs or evaluating a vendor detector. The layer is GATED until a runnable, licensed detector exists.
---

# Watermark detection — Layer 2

Invisible watermarks are statistical patterns embedded in pixels at generation time (not metadata — they survive re-encodes that strip C2PA manifests). Detection is probabilistic: the ceiling for this layer is `LayerFinding::Indication`, never `Proof`.

## The trait contract (from the proposal; implement in M5)

Detectors are vendor-specific and must plug in without core changes:

    pub trait WatermarkDetector {
        fn vendor(&self) -> &'static str;                       // "synthid", "stable-signature", ...
        fn probe(&self, asset: &Asset) -> DetectorResult;       // Detected { confidence } / NotDetected / Unavailable { reason }
    }

`WatermarkLayer` owns a `Vec<Box<dyn WatermarkDetector>>`; any `Detected` above the vendor's calibrated threshold maps to `Indication { source: vendor }`. An empty detector list or all-`Unavailable` maps to `NotEvaluated`, not `NoSignal`.

## Vendors to track (verify current status via lens-research before relying on this)

- **SynthID** (Google/DeepMind) — image/audio/video/text; detector access historically gated behind Google's API. Network detector ⇒ consent + hash-only rules apply (see privacy below).
- **Stable Signature** (Meta) — latent-diffusion watermark, research code published.
- **invisible-watermark** (DWT-DCT) — the Stable Diffusion default; open source, weak against attack, still worth probing.

## Calibration rules

- Every detector ships with a measured false-positive rate on a clean corpus (lens-qa's corpus) before it may contribute to verdicts; thresholds live in code with the measurement recorded in the ExecPlan.
- Never average or ensemble detector confidences into a single score without a plan-level decision — an uncalibrated combined number is exactly the overclaiming this project exists to avoid.
- Wording: "Likely AI-generated (watermark: X)" is the approved pattern (see verdict-language); the vendor name is always shown.

## Privacy

Local detectors run on-device. Detectors that require a network call may send perceptual hashes only, never image bytes, and only with explicit user consent (per-request or a clearly labeled setting). This rule is from the proposal's trust model and is not negotiable.
