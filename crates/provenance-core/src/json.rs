//! The flat JSON rendering of a [`Report`] — the single source of the shape
//! consumed by both the CLI (`lens verify --json`) and the browser extension
//! (via `provenance-wasm`'s `verify_bytes`). One renderer, so the two
//! surfaces cannot drift (U1 decision in `PROVENANCE_LENS_UPGRADES_EXECPLAN.md`).
//!
//! Hand-rolled rather than serde: the shape is flat and stable, and the
//! wedge's dependency rule prices a new crate higher than forty lines.
//!
//! Shape (`file` present only when the caller supplies one, e.g. the CLI):
//!
//! ```text
//! {
//!   "file": "photo.jpg",
//!   "verdict": "inconclusive",
//!   "phrase": "Inconclusive: ...",
//!   "findings": [ { "layer": "c2pa", "status": "no_signal", "detail": "" }, ... ]
//! }
//! ```

use crate::pipeline::{CredentialSummary, LayerFinding, Report};

/// Render `report` as a single flat JSON object. `file` (when given) becomes
/// a leading `"file"` key — the CLI uses it to label per-file results; the
/// WASM boundary passes `None` and the key is absent.
pub fn render_json(report: &Report, file: Option<&str>) -> String {
    let findings = report
        .findings
        .iter()
        .map(|(layer, finding)| {
            let (status, detail) = match finding {
                LayerFinding::NotEvaluated { reason } => ("not_evaluated", reason.as_str()),
                LayerFinding::NoSignal => ("no_signal", ""),
                LayerFinding::Proof { issuer } => ("proof", issuer.as_str()),
                LayerFinding::Indication { source } => ("indication", source.as_str()),
                LayerFinding::TamperEvidence { detail } => ("tamper_evidence", detail.as_str()),
            };
            format!(
                r#"{{"layer":{},"status":{},"detail":{}}}"#,
                json_string(layer),
                json_string(status),
                json_string(detail)
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let file_field = match file {
        Some(path) => format!(r#""file":{},"#, json_string(path)),
        None => String::new(),
    };

    // Present only on Verified reports (the pipeline guarantees it); absent
    // keys are omitted rather than null.
    let credentials_field = match &report.credentials {
        Some(summary) => format!(r#","credentials":{}"#, credentials_json(summary)),
        None => String::new(),
    };

    format!(
        r#"{{{file_field}"verdict":{},"phrase":{}{credentials_field},"findings":[{}]}}"#,
        json_string(report.verdict.id()),
        json_string(report.verdict.approved_phrase()),
        findings
    )
}

fn credentials_json(summary: &CredentialSummary) -> String {
    let mut fields = vec![format!(r#""issuer":{}"#, json_string(&summary.issuer))];
    let optional = [
        ("claim_generator", summary.claim_generator.as_deref()),
        ("signing_time", summary.signing_time.as_deref()),
        (
            "digital_source_type",
            summary.digital_source_type.as_deref(),
        ),
        ("source_type_note", summary.source_type_note),
    ];
    for (key, value) in optional {
        if let Some(value) = value {
            fields.push(format!(r#""{key}":{}"#, json_string(value)));
        }
    }
    format!("{{{}}}", fields.join(","))
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
    use crate::pipeline::{Asset, Pipeline};

    #[test]
    fn json_string_escapes() {
        assert_eq!(json_string("a\"b\\c\nd"), r#""a\"b\\c\nd""#);
    }

    #[test]
    fn report_json_shape_with_and_without_file() {
        let report = Pipeline::standard().examine(&Asset {
            bytes: &[],
            media_type: None,
        });

        let bare = render_json(&report, None);
        assert!(bare.starts_with(r#"{"verdict":"#), "got: {bare}");
        assert!(bare.contains(r#""verdict":"inconclusive""#));
        assert!(bare.contains(r#""layer":"c2pa""#));
        assert!(bare.ends_with("]}"));

        let with_file = render_json(&report, Some(r#"C:\img\a"b.jpg"#));
        assert!(
            with_file.starts_with(r#"{"file":"C:\\img\\a\"b.jpg","verdict":"#),
            "got: {with_file}"
        );
    }
}
