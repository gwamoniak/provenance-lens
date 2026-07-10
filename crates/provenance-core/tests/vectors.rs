//! Committed-corpus test: every vector in tests/vectors/manifest.tsv must
//! produce exactly its recorded verdict, validated against the committed
//! ephemeral CA root (test_ca.pem — a public certificate; the signing key was
//! never persisted). Regenerate the corpus with:
//!
//! ```text
//! cargo run -p provenance-core --example gen_vectors
//! ```
//!
//! Catalogue and corpus must agree bidirectionally: a row without a file or
//! an unexpected .jpg without a row fails the suite.

use std::path::PathBuf;

use provenance_core::{Asset, Pipeline, Verdict};

fn vectors_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectors")
}

#[test]
fn every_committed_vector_matches_its_recorded_verdict() {
    let dir = vectors_dir();
    let ca_pem = std::fs::read_to_string(dir.join("test_ca.pem")).expect("read test_ca.pem");
    let tsv = std::fs::read_to_string(dir.join("manifest.tsv")).expect("read manifest.tsv");

    let mut catalogued = Vec::new();
    for line in tsv.lines().skip(1).filter(|l| !l.trim().is_empty()) {
        let mut cols = line.split('\t');
        let file = cols.next().expect("file column");
        let expected = cols.next().expect("verdict column");
        catalogued.push(file.to_string());

        let bytes = std::fs::read(dir.join(file)).expect("read vector file");
        let report = Pipeline::with_trust_anchors(ca_pem.as_str()).examine(&Asset {
            bytes: &bytes,
            media_type: Some("image/jpeg"),
        });
        let expected = match expected {
            "verified" => Verdict::Verified,
            "indicated" => Verdict::Indicated,
            "inconclusive" => Verdict::Inconclusive,
            "tampered" => Verdict::Tampered,
            other => panic!("unknown verdict id {other:?} in manifest.tsv"),
        };
        assert_eq!(
            report.verdict, expected,
            "vector {file}: expected {expected:?}, findings: {:?}",
            report.findings
        );
    }
    assert!(!catalogued.is_empty(), "manifest.tsv lists no vectors");

    // No orphan vectors: every .jpg on disk must be catalogued.
    for entry in std::fs::read_dir(&dir).expect("list vectors dir") {
        let name = entry
            .expect("dir entry")
            .file_name()
            .to_string_lossy()
            .into_owned();
        if name.ends_with(".jpg") {
            assert!(
                catalogued.contains(&name),
                "vector {name} on disk but not in manifest.tsv"
            );
        }
    }
}
