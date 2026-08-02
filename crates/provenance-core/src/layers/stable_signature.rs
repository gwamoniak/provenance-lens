//! The IMATAG Stable Signature "bzh" watermark classifier (roadmap plan, W2)
//! — the project's first model-based detector, and deliberately narrow:
//! it recognizes ONLY images produced by IMATAG's own bzh-watermarked
//! SDXL-VAE variants, not Stable Signature deployments in general. Its value
//! is proving the model-detector plumbing and the calibration discipline,
//! not broad coverage; the docs must never imply more.
//!
//! Everything here is pinned from IMATAG's published repo
//! (`imatag/stable-signature-bzh-detector-resnet18` on Hugging Face, MIT):
//!
//! - Model: ResNet-18 `ResNetForImageClassification`, two labels
//!   ("no watermark detected" / "watermarked"), trained so
//!   `logits[1] == -logits[0]`.
//! - Preprocessing (their `preprocessor_config.json`, BlipImageProcessor):
//!   RGB → bicubic resize to 512×512 → ×1/255 → normalize mean 0.5 /
//!   std 0.5 per channel (i.e. x → 2·x/255 − 1), NCHW f32.
//! - Decision rule (their `detect_demo_simple.py`, verbatim):
//!   watermarked ⇔ `logits[0] < 0`. Their card states ~1/1000 false
//!   positives for this rule; our calibration corpus measures it ourselves
//!   before the detector may influence verdicts (the revised M5 gate).
//!
//! The model file is NEVER bundled: IMATAG publishes PyTorch weights only,
//! so `scripts/export_stable_signature_onnx.py` exports them to ONNX once,
//! and the caller hands the file to [`StableSignatureBzh::from_onnx_path`]
//! (CLI: `lens verify --watermark-model <file>`). No model, no detector —
//! the bare-machine build and tests never need one.

use std::path::Path;

use tract_onnx::prelude::*;

use crate::layers::watermark::{DecodedImage, WatermarkDetector, WatermarkHit};

/// Input side length the model was exported with (their preprocessor size).
const SIDE: usize = 512;

type OnnxPlan = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

pub struct StableSignatureBzh {
    model: OnnxPlan,
}

impl StableSignatureBzh {
    /// Load the exported ONNX model. Errors are strings so the CLI can print
    /// them and exit 2 — a broken model file must fail loudly at startup,
    /// never silently downgrade to "no watermark".
    pub fn from_onnx_path(path: &Path) -> Result<Self, String> {
        let model = tract_onnx::onnx()
            .model_for_path(path)
            .map_err(|err| format!("cannot read ONNX model {}: {err}", path.display()))?
            .with_input_fact(0, f32::fact([1, 3, SIDE, SIDE]).into())
            .map_err(|err| format!("model rejects [1,3,512,512] input: {err}"))?
            .into_optimized()
            .map_err(|err| format!("model optimization failed: {err}"))?
            .into_runnable()
            .map_err(|err| format!("model is not runnable: {err}"))?;
        Ok(StableSignatureBzh { model })
    }

    /// The vendor pipeline up to the decision: preprocess, run, return
    /// `logits[0]`. `None` on any runtime failure — a failed probe claims
    /// nothing, it never becomes a detection.
    fn logit0(&self, image: &DecodedImage) -> Option<f32> {
        let rgb =
            image::RgbImage::from_raw(image.width as u32, image.height as u32, image.rgb.clone())?;
        let resized = if image.width == SIDE && image.height == SIDE {
            rgb
        } else {
            image::imageops::resize(
                &rgb,
                SIDE as u32,
                SIDE as u32,
                image::imageops::FilterType::CatmullRom,
            )
        };
        let input = tract_ndarray::Array4::from_shape_fn((1, 3, SIDE, SIDE), |(_, c, y, x)| {
            (resized.get_pixel(x as u32, y as u32)[c] as f32 / 255.0 - 0.5) / 0.5
        });
        let outputs = self.model.run(tvec!(Tensor::from(input).into())).ok()?;
        outputs[0]
            .to_array_view::<f32>()
            .ok()?
            .iter()
            .next()
            .copied()
    }
}

impl WatermarkDetector for StableSignatureBzh {
    fn vendor(&self) -> &'static str {
        "imatag-stable-signature-bzh"
    }

    fn probe(&self, image: &DecodedImage) -> Option<WatermarkHit> {
        // The reference decoder refuses images under 256×256 pixels of area;
        // upscaling tiny images to 512² would classify invented detail.
        if image.width * image.height < 256 * 256 {
            return None;
        }
        let logit0 = self.logit0(image)?;
        if logit0 < 0.0 {
            Some(WatermarkHit {
                source: "the IMATAG Stable Signature bzh watermark classifier \
                         (ResNet-18, vendor decision rule; covers IMATAG's own \
                         bzh-watermarked images only)"
                    .to_string(),
            })
        } else {
            None
        }
    }
}
