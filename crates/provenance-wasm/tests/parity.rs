//! Native/WASM-wrapper parity: every committed vector must produce, through
//! `verify_bytes` (the function the extension calls), exactly the verdict,
//! approved phrase, and per-layer findings that the native pipeline produces.
//! This runs the wrapper compiled natively (same Rust, same logic); executing
//! the compiled .wasm inside a JS engine is covered by the M3 extension
//! smoke script.

use provenance_core::{Asset, LayerFinding, Pipeline};
use provenance_wasm::verify_bytes;
use serde_json::Value;

fn vectors_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../provenance-core/tests/vectors")
}

fn finding_as_status_detail(finding: &LayerFinding) -> (&'static str, String) {
    match finding {
        LayerFinding::NotEvaluated { reason } => ("not_evaluated", reason.clone()),
        LayerFinding::NoSignal => ("no_signal", String::new()),
        LayerFinding::Proof { issuer } => ("proof", issuer.clone()),
        LayerFinding::Indication { source } => ("indication", source.clone()),
        LayerFinding::TamperEvidence { detail } => ("tamper_evidence", detail.clone()),
    }
}

#[test]
fn wrapper_matches_native_pipeline_on_every_vector() {
    let dir = vectors_dir();
    let ca_pem = std::fs::read_to_string(dir.join("test_ca.pem")).expect("read test_ca.pem");
    let tsv = std::fs::read_to_string(dir.join("manifest.tsv")).expect("read manifest.tsv");

    let mut checked = 0;
    for line in tsv.lines().skip(1).filter(|l| !l.trim().is_empty()) {
        let file = line.split('\t').next().expect("file column");
        let bytes = std::fs::read(dir.join(file)).expect("read vector");

        // Both sides get identical inputs, each with and without anchors.
        for anchors in [Some(ca_pem.clone()), None] {
            // No MIME hint on either side: parity must hold for the sniffing
            // path too (the corpus is JPEG + PNG since U3).
            let native = match &anchors {
                Some(pem) => Pipeline::with_trust_anchors(pem.as_str()),
                None => Pipeline::standard(),
            }
            .examine(&Asset {
                bytes: &bytes,
                media_type: None,
            });

            let json = verify_bytes(&bytes, None, anchors.clone());
            let parsed: Value = serde_json::from_str(&json).expect("wrapper emits valid JSON");

            assert_eq!(
                parsed["verdict"].as_str().expect("verdict is a string"),
                native.verdict.id(),
                "verdict parity failed for {file} (anchors: {})",
                anchors.is_some()
            );
            assert_eq!(
                parsed["phrase"].as_str().expect("phrase is a string"),
                native.verdict.approved_phrase(),
                "phrase parity failed for {file}"
            );

            // U2: the credentials object appears in the JSON exactly when the
            // native report carries a summary, with the same issuer.
            assert_eq!(
                parsed["credentials"].is_object(),
                native.credentials.is_some(),
                "credentials presence parity for {file} (anchors: {})",
                anchors.is_some()
            );
            if let Some(summary) = &native.credentials {
                assert_eq!(
                    parsed["credentials"]["issuer"].as_str(),
                    Some(summary.issuer.as_str()),
                    "credentials issuer parity for {file}"
                );
            }

            let json_findings = parsed["findings"].as_array().expect("findings array");
            assert_eq!(
                json_findings.len(),
                native.findings.len(),
                "finding count for {file}"
            );
            for (json_finding, (layer, finding)) in json_findings.iter().zip(&native.findings) {
                let (status, detail) = finding_as_status_detail(finding);
                assert_eq!(
                    json_finding["layer"].as_str(),
                    Some(layer.as_str()),
                    "layer name for {file}"
                );
                assert_eq!(
                    json_finding["status"].as_str(),
                    Some(status),
                    "status for {file} [{layer}]"
                );
                assert_eq!(
                    json_finding["detail"].as_str(),
                    Some(detail.as_str()),
                    "detail for {file} [{layer}]"
                );
            }
        }
        checked += 1;
    }
    assert!(
        checked >= 5,
        "expected the full corpus, checked only {checked}"
    );
}
