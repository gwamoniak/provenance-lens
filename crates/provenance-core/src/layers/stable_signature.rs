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
//! and the caller hands its bytes to [`StableSignatureBzh::from_onnx_bytes`]
//! (CLI: `lens verify --watermark-model <file>`). No model, no detector —
//! the bare-machine build and tests never need one.

use sha2::{Digest, Sha256};
use tract_onnx::prelude::*;

use crate::layers::watermark::{DecodedImage, DetectorResult, WatermarkDetector, WatermarkHit};

/// Input side length the model was exported with (their preprocessor size).
const SIDE: usize = 512;

type OnnxPlan = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

pub const MODEL_SHA256: &str = "4f7d80027d5d7ecf5c176fabc3d86c8ccd791cc5c830d23a96114425cd1f3e57";
pub const MAX_MODEL_BYTES: usize = 64 * 1024 * 1024;

#[derive(Default)]
pub struct StableSignatureBzh {
    model: Option<OnnxPlan>,
}

impl StableSignatureBzh {
    /// Load the exported ONNX model. Errors are strings so the CLI can print
    /// them and exit 2 — a broken model file must fail loudly at startup,
    /// never silently downgrade to "no watermark".
    pub fn from_onnx_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_MODEL_BYTES || format!("{:x}", Sha256::digest(bytes)) != MODEL_SHA256 {
            return Err(format!(
                "unrecognized model; requires calibrated IMATAG bzh ONNX SHA-256 {MODEL_SHA256}"
            ));
        }
        let model = tract_onnx::onnx()
            .model_for_read(&mut std::io::Cursor::new(bytes))
            .map_err(|err| format!("cannot parse ONNX model: {err}"))?;
        if model.input_outlets().map_err(|e| e.to_string())?.len() != 1
            || model.output_outlets().map_err(|e| e.to_string())?.len() != 1
        {
            return Err("model must have exactly one input and one output".into());
        }
        let model = model
            .with_input_fact(0, f32::fact([1, 3, SIDE, SIDE]).into())
            .map_err(|err| format!("model rejects [1,3,512,512] input: {err}"))?
            .into_optimized()
            .map_err(|err| format!("model optimization failed: {err}"))?;
        for (fact, expected) in [
            (
                model.input_fact(0).map_err(|e| e.to_string())?,
                vec![1, 3, SIDE, SIDE],
            ),
            (model.output_fact(0).map_err(|e| e.to_string())?, vec![1, 2]),
        ] {
            if fact.datum_type != f32::datum_type()
                || fact.shape.as_concrete() != Some(expected.as_slice())
            {
                return Err("model requires f32 input [1,3,512,512] and output [1,2]".into());
            }
        }
        let model = model
            .into_runnable()
            .map_err(|err| format!("model is not runnable: {err}"))?;
        Ok(StableSignatureBzh { model: Some(model) })
    }

    /// The vendor pipeline up to the decision: preprocess, run, return
    /// `logits[0]`. Retain errors as unavailable checks, never negatives.
    fn logit0(&self, image: &DecodedImage) -> Result<f32, String> {
        let model = self.model.as_ref().ok_or("model file not provided")?;
        image.validate()?;
        if image.width * image.height < 256 * 256 {
            return Err("unsupported dimensions: requires at least 256×256 pixels of area".into());
        }
        let rgb =
            image::RgbImage::from_raw(image.width as u32, image.height as u32, image.rgb.clone())
                .ok_or("invalid RGB buffer")?;
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
        let outputs = model
            .run(tvec!(Tensor::from(input).into()))
            .map_err(|err| format!("model runtime failed: {err}"))?;
        output_logit(&outputs)
    }
}

fn output_logit(outputs: &[TValue]) -> Result<f32, String> {
    if outputs.len() != 1 {
        return Err("model returned an unexpected output count".into());
    }
    let output = outputs.first().ok_or("model returned no output")?;
    if output.shape() != [1, 2] {
        return Err("model output shape is not [1,2]".into());
    }
    let values = output
        .to_array_view::<f32>()
        .map_err(|err| format!("model output is not f32: {err}"))?;
    if values.iter().any(|v| !v.is_finite()) {
        return Err("model returned non-finite scores".into());
    }
    values
        .iter()
        .next()
        .copied()
        .ok_or_else(|| "model returned empty scores".into())
}

impl WatermarkDetector for StableSignatureBzh {
    fn vendor(&self) -> &'static str {
        "imatag-stable-signature-bzh"
    }

    fn probe(&self, image: &DecodedImage) -> DetectorResult {
        let logit0 = match self.logit0(image) {
            Ok(value) => value,
            Err(reason) => return DetectorResult::Unavailable { reason },
        };
        if logit0 < 0.0 {
            DetectorResult::Detected(WatermarkHit {
                source: "the IMATAG Stable Signature bzh watermark classifier \
                         (ResNet-18, vendor decision rule; covers IMATAG's own \
                         bzh-watermarked images only)"
                    .to_string(),
            })
        } else {
            DetectorResult::NotDetected
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrong_models_missing_outputs_and_nonfinite_scores_are_errors() {
        assert!(StableSignatureBzh::from_onnx_bytes(b"unrelated model").is_err());
        assert!(output_logit(&[]).is_err());
        for values in [
            [f32::NAN, 0.0],
            [0.0, f32::INFINITY],
            [f32::NEG_INFINITY, 0.0],
        ] {
            let tensor = Tensor::from(tract_ndarray::arr2(&[values]));
            assert!(output_logit(&[tensor.into()]).is_err());
        }
        assert!(
            output_logit(&[Tensor::from(tract_ndarray::Array2::<f32>::zeros((1, 0))).into()])
                .is_err()
        );
        assert_eq!(
            output_logit(&[Tensor::from(tract_ndarray::arr2(&[[-1.0f32, 1.0]])).into()]).unwrap(),
            -1.0
        );
        assert!(
            matches!(StableSignatureBzh::default().probe(&DecodedImage { rgb: vec![], width: 0, height: 0 }),
            DetectorResult::Unavailable { reason } if reason.contains("not provided"))
        );
    }
}
