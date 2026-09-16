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

pub const MAX_DECODED_PIXELS: usize = 40_000_000;

impl DecodedImage {
    /// Validate caller-supplied buffers before any detector indexes pixels.
    pub fn validate(&self) -> Result<(), String> {
        let pixels = self
            .width
            .checked_mul(self.height)
            .filter(|&n| n > 0 && n <= MAX_DECODED_PIXELS)
            .ok_or("image dimensions exceed the 40 million pixel limit or are empty")?;
        if pixels.checked_mul(3) != Some(self.rgb.len()) {
            return Err("RGB buffer does not match image dimensions".into());
        }
        Ok(())
    }
}

/// A positive detection. `source` is the user-facing sentence fragment that
/// becomes the Indication finding (rendered after "indication from …" in the
/// CLI), so it must name the scheme precisely and stay descriptive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatermarkHit {
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectorResult {
    Detected(WatermarkHit),
    NotDetected,
    Unavailable { reason: String },
}

/// One vendor-specific watermark probe. Implementations must be pure
/// functions of the pixels: no I/O, no models fetched at probe time.
pub trait WatermarkDetector {
    /// Short stable scheme name (diagnostics, FPR reports).
    fn vendor(&self) -> &'static str;
    /// A failed or unsupported examination is never a completed negative.
    fn probe(&self, image: &DecodedImage) -> DetectorResult;
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
                detectors: vec![
                    Box::new(crate::layers::sd_dwt::SdInvisibleWatermark),
                    #[cfg(feature = "stable-signature")]
                    Box::new(crate::layers::stable_signature::StableSignatureBzh::default()),
                ],
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
        self.detectors
            .retain(|existing| existing.vendor() != detector.vendor());
        self.detectors.push(detector);
    }

    fn probe(&self, image: &DecodedImage) -> LayerFinding {
        let mut incomplete = Vec::new();
        for detector in &self.detectors {
            match detector.probe(image) {
                DetectorResult::Detected(hit) => {
                    return LayerFinding::Indication { source: hit.source }
                }
                DetectorResult::NotDetected => {}
                DetectorResult::Unavailable { reason } => {
                    incomplete.push(format!("{}: {reason}", detector.vendor()))
                }
            }
        }
        if incomplete.is_empty() {
            LayerFinding::NoSignal
        } else {
            LayerFinding::NotEvaluated {
                reason: incomplete.join("; "),
            }
        }
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
        self.probe(&image)
    }
}

#[cfg(feature = "watermark-dwt")]
pub fn decode_rgb(bytes: &[u8]) -> Result<DecodedImage, String> {
    use image::ImageDecoder;
    if bytes.len() > crate::pipeline::MAX_ASSET_BYTES {
        return Err("asset exceeds the 25 MiB examination limit".into());
    }
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|err| err.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_DECODED_PIXELS as u32);
    limits.max_image_height = Some(MAX_DECODED_PIXELS as u32);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    let decoder = reader.into_decoder().map_err(|err| err.to_string())?;
    let (width, height) = decoder.dimensions();
    if u64::from(width) * u64::from(height) > MAX_DECODED_PIXELS as u64 {
        return Err("image exceeds the 40 million decoded pixel limit".into());
    }
    let decoded = image::DynamicImage::from_decoder(decoder)
        .map_err(|err| format!("image could not be decoded for watermark analysis: {err}"))?
        .to_rgb8();
    let (width, height) = (decoded.width() as usize, decoded.height() as usize);
    Ok(DecodedImage {
        rgb: decoded.into_raw(),
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Mock(DetectorResult);
    impl WatermarkDetector for Mock {
        fn vendor(&self) -> &'static str {
            "mock"
        }
        fn probe(&self, _: &DecodedImage) -> DetectorResult {
            self.0.clone()
        }
    }
    #[test]
    fn incomplete_checks_never_become_negative_but_hits_survive() {
        let image = DecodedImage {
            rgb: vec![],
            width: 0,
            height: 0,
        };
        let layer = |results: Vec<DetectorResult>| {
            WatermarkLayer::with_detectors(
                results
                    .into_iter()
                    .map(|result| Box::new(Mock(result)) as Box<dyn WatermarkDetector>)
                    .collect(),
            )
        };
        let unavailable = DetectorResult::Unavailable {
            reason: "runtime failure".into(),
        };
        assert!(
            matches!(layer(vec![DetectorResult::NotDetected, unavailable.clone()]).probe(&image),
            LayerFinding::NotEvaluated { reason } if reason.contains("mock: runtime failure"))
        );
        assert_eq!(
            layer(vec![DetectorResult::NotDetected]).probe(&image),
            LayerFinding::NoSignal
        );
        assert!(matches!(
            layer(vec![
                unavailable,
                DetectorResult::Detected(WatermarkHit {
                    source: "test watermark".into()
                })
            ])
            .probe(&image),
            LayerFinding::Indication { .. }
        ));
    }
    #[test]
    fn invalid_pixel_buffers_and_overflow_are_rejected() {
        for (width, height) in [(usize::MAX, 2), (40_000_001, 1), (0, 1), (256, 256)] {
            assert!(DecodedImage {
                rgb: vec![],
                width,
                height
            }
            .validate()
            .is_err());
        }
    }

    #[test]
    #[cfg(feature = "watermark-dwt")]
    fn large_declared_png_dimensions_are_rejected_before_pixel_allocation() {
        let mut bytes = include_bytes!("../../tests/vectors/stripped.png").to_vec();
        bytes[16..20].copy_from_slice(&10_000u32.to_be_bytes());
        bytes[20..24].copy_from_slice(&4_001u32.to_be_bytes());
        let mut crc = !0u32;
        for &byte in &bytes[12..29] {
            crc ^= u32::from(byte);
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
            }
        }
        bytes[29..33].copy_from_slice(&(!crc).to_be_bytes());
        let error = decode_rgb(&bytes)
            .err()
            .expect("must refuse large dimensions");
        assert!(error.contains("40 million"), "{error}");
    }
}

#[cfg(not(feature = "watermark-dwt"))]
fn decode_rgb(_bytes: &[u8]) -> Result<DecodedImage, String> {
    Err("image decoding is not compiled into this build".to_string())
}
