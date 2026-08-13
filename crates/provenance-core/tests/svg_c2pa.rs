//! C1 SVG known-answer tests. These vectors are signed by the repository's
//! ephemeral test CA and prove generic SVG plumbing only; they make no claim
//! about Anthropic interoperability.

use std::path::PathBuf;

use provenance_core::{Asset, LayerFinding, Pipeline, Verdict};

fn vectors_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectors")
}

fn examine(name: &str, anchors: Option<&str>) -> provenance_core::Report {
    let bytes = std::fs::read(vectors_dir().join(name)).expect("read SVG vector");
    let pipeline = match anchors {
        Some(pem) => Pipeline::with_trust_anchors(pem),
        None => Pipeline::standard(),
    };
    pipeline.examine(&Asset {
        bytes: &bytes,
        media_type: Some("image/svg+xml"),
    })
}

#[test]
fn signed_svg_is_verified_only_with_its_test_anchor() {
    let anchors = std::fs::read_to_string(vectors_dir().join("test_ca.pem")).expect("read test CA");
    assert_eq!(
        examine("valid_signed.svg", Some(&anchors)).verdict,
        Verdict::Verified
    );
    assert_eq!(examine("valid_signed.svg", None).verdict, Verdict::Tampered);
}

#[test]
fn unsigned_and_post_signing_edited_svg_have_honest_verdicts() {
    let anchors = std::fs::read_to_string(vectors_dir().join("test_ca.pem")).expect("read test CA");
    assert_eq!(
        examine("plain.svg", Some(&anchors)).verdict,
        Verdict::Inconclusive
    );
    assert_eq!(
        examine("content_tampered.svg", Some(&anchors)).verdict,
        Verdict::Tampered
    );
}

#[test]
fn malformed_svg_never_panics_or_mints_proof() {
    let signed = std::fs::read(vectors_dir().join("valid_signed.svg")).expect("read signed SVG");
    let mut hostile = vec![
        Vec::new(),
        b"<svg".to_vec(),
        b"<?xml version='1.0'?><svg><metadata><c2pa:manifest>%%%".to_vec(),
        b"<!DOCTYPE svg [<!ENTITY x SYSTEM 'file:///secret'>]><svg>&x;</svg>".to_vec(),
    ];
    for cut in [1, signed.len() / 4, signed.len() / 2, signed.len() - 1] {
        hostile.push(signed[..cut].to_vec());
    }

    for bytes in hostile {
        let report = Pipeline::standard().examine(&Asset {
            bytes: &bytes,
            media_type: Some("image/svg+xml"),
        });
        assert_ne!(report.verdict, Verdict::Verified);
        assert!(report
            .findings
            .iter()
            .all(|(_, finding)| !matches!(finding, LayerFinding::Proof { .. })));
    }
}
