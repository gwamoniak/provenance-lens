//! Layer 4 — optional statistical heuristics (frequency-domain artifacts,
//! metadata anomalies). Deliberately last and deliberately weakest: heuristic
//! output may contribute an `Indication` at most, may never produce `Proof`
//! or `TamperEvidence`, and the whole layer must be trivially removable —
//! nothing upstream may depend on it.
//!
//! OPTIONAL (post-wedge, may never ship): only worth building if a heuristic
//! with a published, reproducible false-positive rate exists. "Feels
//! AI-generated" is not a finding.

use crate::pipeline::{Asset, Layer, LayerFinding};

pub struct HeuristicsLayer;

impl Layer for HeuristicsLayer {
    fn name(&self) -> &'static str {
        "heuristics"
    }

    fn examine(&self, _asset: &Asset) -> LayerFinding {
        LayerFinding::NotEvaluated {
            reason: "optional layer, not implemented".to_string(),
        }
    }
}
