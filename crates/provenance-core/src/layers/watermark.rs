//! Layer 2 — invisible-watermark detection (SynthID, Stable Signature, and
//! whatever vendors publish detectors for). Watermarks survive some
//! re-encoding that strips C2PA metadata, so this layer catches assets whose
//! credentials were laundered away — but detection is statistical, so its
//! ceiling is `Indication`, never `Proof`.
//!
//! GATED (post-wedge milestone): requires a runnable vendor detector — a
//! published model or API we are licensed to call. Do not fake this with a
//! lookalike classifier; a wrong "watermark found" is worse than none.

use crate::pipeline::{Asset, Layer, LayerFinding};

pub struct WatermarkLayer;

impl Layer for WatermarkLayer {
    fn name(&self) -> &'static str {
        "watermark"
    }

    fn examine(&self, _asset: &Asset) -> LayerFinding {
        LayerFinding::NotEvaluated {
            reason: "gated: no vendor watermark detector integrated yet".to_string(),
        }
    }
}
