//! Layer 1 — C2PA manifest validation. The only layer permitted to return
//! `LayerFinding::Proof`, because it is the only one backed by cryptography:
//! a C2PA manifest embeds a COSE signature over a claim whose hard binding
//! (content hash) ties it to these exact bytes.
//!
//! Milestone 1 replaces this stub with real validation via the `c2pa` crate
//! (Content Authenticity Initiative Rust SDK): parse the JUMBF manifest
//! store, validate the signature chain against the C2PA trust list, check the
//! hard binding, and map the result to `Proof` / `TamperEvidence` /
//! `NoSignal`. Any change to that validation code requires human
//! cryptography-reviewer sign-off before merge (see CLAUDE.md).

use crate::pipeline::{Asset, Layer, LayerFinding};

pub struct C2paLayer;

impl Layer for C2paLayer {
    fn name(&self) -> &'static str {
        "c2pa"
    }

    fn examine(&self, _asset: &Asset) -> LayerFinding {
        LayerFinding::NotEvaluated {
            reason: "C2PA validation via the c2pa crate lands in Milestone 1".to_string(),
        }
    }
}
