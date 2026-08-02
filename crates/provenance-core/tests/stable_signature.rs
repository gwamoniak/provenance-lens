//! Known-answer test for the IMATAG Stable Signature bzh classifier
//! (roadmap plan, W2), against the vendor's OWN example pair (MIT — see
//! tests/vectors/watermark/bzh/ATTRIBUTION.md).
//!
//! The model file is never committed (bare-machine rule), so this test is
//! env-gated: export the model once with
//!     py scripts/export_stable_signature_onnx.py <model-dir> <out.onnx>
//! then run
//!     PROVENANCE_LENS_BZH_ONNX=<out.onnx> cargo test -p provenance-core --test stable_signature
//! Without the variable the test skips and everything else stays green.
#![cfg(feature = "stable-signature")]

use std::path::{Path, PathBuf};

use provenance_core::layers::stable_signature::StableSignatureBzh;
use provenance_core::layers::watermark::{DecodedImage, WatermarkDetector};

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
fn vendor_example_pair_classifies_exactly_as_the_vendor_says() {
    let model_path = match std::env::var("PROVENANCE_LENS_BZH_ONNX") {
        Ok(path) => path,
        Err(_) => {
            eprintln!(
                "skipping: PROVENANCE_LENS_BZH_ONNX not set (no model committed; \
                 export one with scripts/export_stable_signature_onnx.py)"
            );
            return;
        }
    };
    let detector =
        StableSignatureBzh::from_onnx_path(Path::new(&model_path)).expect("load ONNX model");

    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectors/watermark/bzh");
    let hit = detector.probe(&load(&dir.join("watermarked.png")));
    match &hit {
        Some(hit) => assert!(
            hit.source.contains("IMATAG Stable Signature"),
            "hit must name the scheme, got: {}",
            hit.source
        ),
        None => panic!("the vendor's watermarked example must be detected"),
    }
    assert!(
        detector
            .probe(&load(&dir.join("not_watermarked.png")))
            .is_none(),
        "the vendor's clean example must not be detected"
    );
}
