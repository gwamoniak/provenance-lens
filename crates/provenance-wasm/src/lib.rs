//! WASM boundary. One exported function: bytes in, JSON report out. The
//! extension owns fetching the asset and bundling its trust-anchor list;
//! this crate owns nothing but the call into `provenance-core`.
//!
//! JSON is hand-rolled for now to keep the artifact small; if the report
//! shape grows past this flat structure, switch to serde + serde-wasm-bindgen
//! (Milestone 2 decision point — recorded in the ExecPlan Decision Log).

use provenance_core::{Asset, LayerFinding, Pipeline};
use wasm_bindgen::prelude::*;

/// Examine `bytes` and return the report as a JSON string.
///
/// `trust_anchors_pem` is a PEM bundle of root certificates that signature
/// chains may validate against; without it no chain can reach "trusted", so
/// signed assets report as unverifiable provenance (tampered tier).
///
/// ```text
/// {
///   "verdict": "inconclusive",
///   "phrase": "Inconclusive: ...",
///   "findings": [ { "layer": "c2pa", "status": "no_signal", "detail": "" }, ... ]
/// }
/// ```
#[wasm_bindgen]
pub fn verify_bytes(
    bytes: &[u8],
    media_type: Option<String>,
    trust_anchors_pem: Option<String>,
) -> String {
    let asset = Asset {
        bytes,
        media_type: media_type.as_deref(),
    };
    let pipeline = match trust_anchors_pem {
        Some(pem) => Pipeline::with_trust_anchors(pem),
        None => Pipeline::standard(),
    };
    let report = pipeline.examine(&asset);

    let findings = report
        .findings
        .iter()
        .map(|(layer, finding)| {
            let (status, detail) = match finding {
                LayerFinding::NotEvaluated { reason } => ("not_evaluated", reason.clone()),
                LayerFinding::NoSignal => ("no_signal", String::new()),
                LayerFinding::Proof { issuer } => ("proof", issuer.clone()),
                LayerFinding::Indication { source } => ("indication", source.clone()),
                LayerFinding::TamperEvidence { detail } => ("tamper_evidence", detail.clone()),
            };
            format!(
                r#"{{"layer":{},"status":{},"detail":{}}}"#,
                json_string(layer),
                json_string(status),
                json_string(&detail)
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    format!(
        r#"{{"verdict":{},"phrase":{},"findings":[{}]}}"#,
        json_string(report.verdict.id()),
        json_string(report.verdict.approved_phrase()),
        findings
    )
}

/// Minimal JSON string encoder (quotes, backslashes, control characters).
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_json_is_wellformed_for_empty_asset() {
        let json = verify_bytes(&[], None, None);
        assert!(json.starts_with('{') && json.ends_with('}'));
        assert!(json.contains(r#""verdict":"inconclusive""#));
        assert!(json.contains(r#""layer":"c2pa""#));
    }

    #[test]
    fn json_string_escapes() {
        assert_eq!(json_string("a\"b\\c\nd"), r#""a\"b\\c\nd""#);
    }
}
