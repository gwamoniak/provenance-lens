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

use crate::pipeline::{LayerFinding, Report};

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

    format!(
        r#"{{{file_field}"verdict":{},"phrase":{},"findings":[{}]}}"#,
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
