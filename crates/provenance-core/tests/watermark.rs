//! Known-answer tests for the Stable Diffusion invisible-watermark detector
//! (roadmap plan, W1). The vectors in tests/vectors/watermark/ were produced
//! by the VERBATIM reference embedder (scripts/gen_watermark_vectors.py —
//! ShieldMnt/invisible-watermark EmbedMaxDct, method 'dwtDct'), so the Rust
//! decoder is validated against ground truth it did not generate.
//! Regenerate:  py scripts/gen_watermark_vectors.py
#![cfg(feature = "watermark-dwt")]

use std::path::PathBuf;

use provenance_core::{Asset, LayerFinding, Pipeline, Verdict};

fn vectors_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectors")
}

fn watermark_finding(report: &provenance_core::Report) -> &LayerFinding {
    &report
        .findings
        .iter()
        .find(|(name, _)| name == "watermark")
        .expect("watermark layer must appear in every report")
        .1
}

#[test]
fn reference_embedded_payloads_decode_to_indicated() {
    let cases = [
        ("watermark/wm_sdxl.png", "Stable Diffusion XL"),
        ("watermark/wm_sdv1.png", "Stable Diffusion V1"),
    ];
    let pipeline = Pipeline::standard();
    for (file, expected_scheme) in cases {
        let bytes = std::fs::read(vectors_dir().join(file)).expect("read watermark vector");
        let report = pipeline.examine(&Asset {
            bytes: &bytes,
            media_type: None,
        });
        assert_eq!(
            report.verdict,
            Verdict::Indicated,
            "{file}: expected Indicated, findings: {:?}",
            report.findings
        );
        match watermark_finding(&report) {
            LayerFinding::Indication { source } => assert!(
                source.contains(expected_scheme),
                "{file}: indication must name {expected_scheme}, got: {source}"
            ),
            other => panic!("{file}: expected Indication, got {other:?}"),
        }
    }
}

#[test]
fn clean_twin_of_the_base_image_stays_inconclusive() {
    let bytes =
        std::fs::read(vectors_dir().join("watermark/clean_base.png")).expect("read clean twin");
    let report = Pipeline::standard().examine(&Asset {
        bytes: &bytes,
        media_type: None,
    });
    assert_eq!(
        report.verdict,
        Verdict::Inconclusive,
        "clean_base.png must not be Indicated; findings: {:?}",
        report.findings
    );
    assert_eq!(
        watermark_finding(&report),
        &LayerFinding::NoSignal,
        "the detector ran and must report NoSignal, not a false hit"
    );
}

#[test]
fn c2pa_corpus_produces_zero_watermark_false_positives() {
    // Every committed C2PA vector is watermark-free; the detector must never
    // fire on any of them (their recorded verdicts already pin this
    // indirectly — this makes the zero-FP expectation explicit).
    let dir = vectors_dir();
    let pipeline = Pipeline::standard();
    for entry in std::fs::read_dir(&dir).expect("list vectors dir") {
        let path = entry.expect("dir entry").path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        if !(name.ends_with(".jpg") || name.ends_with(".png")) {
            continue;
        }
        let bytes = std::fs::read(&path).expect("read vector");
        let report = pipeline.examine(&Asset {
            bytes: &bytes,
            media_type: None,
        });
        assert!(
            !matches!(watermark_finding(&report), LayerFinding::Indication { .. }),
            "false watermark positive on {name}"
        );
    }
}
