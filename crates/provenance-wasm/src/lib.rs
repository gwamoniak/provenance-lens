//! WASM boundary. One exported function: bytes in, JSON report out. The
//! extension owns fetching the asset and bundling its trust-anchor list;
//! this crate owns nothing but the call into `provenance-core` — since U1
//! even the JSON shape lives there (`provenance_core::render_json`), shared
//! with the CLI's `--json` output so the two surfaces cannot drift.

use provenance_core::{render_json, Asset, Pipeline};
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
    render_json(&report, None)
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
}
