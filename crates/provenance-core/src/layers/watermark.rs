//! Layer 2 — invisible-watermark detection. Watermarks survive some
//! re-encoding that strips C2PA metadata, so this layer catches assets whose
//! credentials were laundered away — but detection is statistical, so its
//! ceiling is `Indication`, never `Proof`.
//!
//! The layer holds a pluggable list of [`WatermarkDetector`]s (vendor name +
//! probe over decoded pixels); new vendors plug in without core changes. Per
//! the revised M5 gate (wedge plan Decision Log, 2026-07-31), a detector
//! qualifies only if its scheme has a public specification or published
//! weights AND its false-positive rate is measured on a clean corpus before
//! it may contribute to verdicts. The first real detector is the Stable
//! Diffusion invisible-watermark decoder (`sd_dwt` module, behind the
//! default-on `watermark-dwt` cargo feature — the WASM build opts out, so
//! there this layer honestly reports itself as not compiled in).
//!
//! Vendor watermarks with NO public decoder (Google's SynthID above all)
//! remain undetectable by this or any third-party tool; nothing here may
//! fake a lookalike classifier for them. A wrong "watermark found" is worse
//! than none.

use crate::pipeline::{Asset, Layer, LayerFinding};

/// Plain owned pixel buffer every detector consumes, so detectors never do
/// their own image decoding: RGB, 8 bits per channel, row-major.
pub struct DecodedImage {
    pub rgb: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

/// A positive detection. `source` is the user-facing sentence fragment that
/// becomes the Indication finding (rendered after "indication from …" in the
/// CLI), so it must name the scheme precisely and stay descriptive.
pub struct WatermarkHit {
    pub source: String,
}

/// One vendor-specific watermark probe. Implementations must be pure
/// functions of the pixels: no I/O, no models fetched at probe time.
pub trait WatermarkDetector {
    /// Short stable scheme name (diagnostics, FPR reports).
    fn vendor(&self) -> &'static str;
    /// `Some` only on a positive detection; absence of a watermark is `None`,
    /// never an error.
    fn probe(&self, image: &DecodedImage) -> Option<WatermarkHit>;
}

pub struct WatermarkLayer {
    detectors: Vec<Box<dyn WatermarkDetector>>,
}

impl WatermarkLayer {
    /// The standard detector set for this build: the Stable Diffusion
    /// invisible-watermark decoder when compiled in, nothing otherwise.
    pub fn standard() -> Self {
        #[cfg(feature = "watermark-dwt")]
        {
            WatermarkLayer {
                detectors: vec![Box::new(crate::layers::sd_dwt::SdInvisibleWatermark)],
            }
        }
        #[cfg(not(feature = "watermark-dwt"))]
        {
            WatermarkLayer {
                detectors: Vec::new(),
            }
        }
    }

    /// Caller-supplied detectors (tests, future configuration).
    pub fn with_detectors(detectors: Vec<Box<dyn WatermarkDetector>>) -> Self {
        WatermarkLayer { detectors }
    }

    /// Add one detector to this layer (CLI: a runtime-supplied model joins
    /// the standard set).
    pub fn push_detector(&mut self, detector: Box<dyn WatermarkDetector>) {
        self.detectors.push(detector);
    }
}

impl Layer for WatermarkLayer {
    fn name(&self) -> &'static str {
        "watermark"
    }

    fn examine(&self, asset: &Asset) -> LayerFinding {
        if self.detectors.is_empty() {
            return LayerFinding::NotEvaluated {
                reason: "no watermark detector is compiled into this build".to_string(),
            };
        }
        let image = match decode_rgb(asset.bytes) {
            Ok(image) => image,
            Err(reason) => return LayerFinding::NotEvaluated { reason },
        };
        for detector in &self.detectors {
            if let Some(hit) = detector.probe(&image) {
                // Ceiling by construction: a watermark hit is an Indication,
                // never Proof (only Layer 1 may prove).
                return LayerFinding::Indication { source: hit.source };
            }
        }
        LayerFinding::NoSignal
    }
}

#[cfg(feature = "watermark-dwt")]
fn decode_rgb(bytes: &[u8]) -> Result<DecodedImage, String> {
    let decoded = image::load_from_memory(bytes)
        .map_err(|err| format!("image could not be decoded for watermark analysis: {err}"))?
        .to_rgb8();
    let (width, height) = (decoded.width() as usize, decoded.height() as usize);
    Ok(DecodedImage {
        rgb: decoded.into_raw(),
        width,
        height,
    })
}

#[cfg(not(feature = "watermark-dwt"))]
fn decode_rgb(_bytes: &[u8]) -> Result<DecodedImage, String> {
    Err("image decoding is not compiled into this build".to_string())
}
