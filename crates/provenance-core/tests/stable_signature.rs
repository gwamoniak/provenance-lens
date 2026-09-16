//! Known-answer test for the IMATAG Stable Signature bzh classifier
//! (roadmap plan, W2), against the vendor's OWN example pair (MIT — see
//! tests/vectors/watermark/bzh/ATTRIBUTION.md).
//!
//! The model file is never committed (bare-machine rule), so this test is
//! explicitly ignored by default. Export the model once with
//!     py scripts/export_stable_signature_onnx.py <model-dir> <out.onnx>
//! then run
//!     PROVENANCE_LENS_BZH_ONNX=<out.onnx> cargo test -p provenance-core --features stable-signature --test stable_signature -- --ignored
//! With --ignored, a missing model fails rather than silently passing.
#![cfg(feature = "stable-signature")]

use std::path::{Path, PathBuf};

use provenance_core::layers::stable_signature::StableSignatureBzh;
use provenance_core::layers::watermark::{DecodedImage, DetectorResult, WatermarkDetector};

fn load(path: &Path) -> DecodedImage {
    let decoded = image::open(path).expect("read example image").to_rgb8();
    let (width, height) = (decoded.width() as usize, decoded.height() as usize);
    DecodedImage {
        rgb: decoded.into_raw(),
        width,
        height,
    }
}

#[test]
#[ignore = "requires PROVENANCE_LENS_BZH_ONNX; run with --ignored"]
fn vendor_example_pair_classifies_exactly_as_the_vendor_says() {
    let model_path = std::env::var("PROVENANCE_LENS_BZH_ONNX")
        .expect("PROVENANCE_LENS_BZH_ONNX required when running the ignored model test");
    let detector =
        StableSignatureBzh::from_onnx_bytes(&std::fs::read(&model_path).expect("read model"))
            .expect("load ONNX model");

    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectors/watermark/bzh");
    let hit = detector.probe(&load(&dir.join("watermarked.png")));
    match &hit {
        DetectorResult::Detected(hit) => assert!(
            hit.source.contains("IMATAG Stable Signature"),
            "hit must name the scheme, got: {}",
            hit.source
        ),
        other => panic!("the vendor's watermarked example must be detected: {other:?}"),
    }
    assert_eq!(
        detector.probe(&load(&dir.join("not_watermarked.png"))),
        DetectorResult::NotDetected,
        "the vendor's clean example must not be detected"
    );
    assert!(matches!(
        detector.probe(&DecodedImage {
            rgb: vec![0; 3],
            width: 1,
            height: 1
        }),
        DetectorResult::Unavailable { .. }
    ));
}
