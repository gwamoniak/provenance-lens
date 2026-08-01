//! The pipeline: an [`Asset`] flows through the four [`Layer`]s, their
//! [`LayerFinding`]s combine into a [`Report`] with one [`Verdict`].

use crate::verdict::Verdict;

/// An asset under examination. Bytes only — the core never does I/O; callers
/// (CLI, WASM wrapper) read files or fetch URLs and hand the bytes in.
pub struct Asset<'a> {
    pub bytes: &'a [u8],
    /// MIME hint when the caller knows it (e.g. "image/jpeg"). Layers must
    /// tolerate `None` and must not trust the hint over the bytes.
    pub media_type: Option<&'a str>,
}

/// What a single layer concluded about one asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayerFinding {
    /// The layer could not run: not yet implemented, or its prerequisite
    /// (detector model, registry endpoint) is absent. Distinct from
    /// `NoSignal` so reports never conflate "didn't look" with "looked and
    /// found nothing".
    NotEvaluated { reason: String },
    /// The layer ran and found no usable signal.
    NoSignal,
    /// A cryptographically valid provenance chain. By project rule, only
    /// Layer 1 (C2PA) may return this variant.
    Proof { issuer: String },
    /// A non-cryptographic indication of AI involvement (watermark hit,
    /// registry match, heuristic score above threshold).
    Indication { source: String },
    /// Provenance data present but invalid: bad signature, truncated
    /// manifest, content-hash mismatch.
    TamperEvidence { detail: String },
}

/// One examination stage. Layers are sans-IO: they inspect the bytes they are
/// given and return a finding; anything requiring the network (registry
/// lookups) receives its transport by injection at construction time.
pub trait Layer {
    fn name(&self) -> &'static str;
    fn examine(&self, asset: &Asset) -> LayerFinding;
}

/// What a Verified credential *claims* — descriptive metadata read from the
/// already-validated manifest (U2). This never affects the verdict: it
/// reports the credential's own statements, and the wording must stay
/// descriptive ("the credential declares…"), never an endorsement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialSummary {
    /// Issuing authority of the validated signature (also on the Proof finding).
    pub issuer: String,
    /// Claim generator as "name/version" when the manifest names one.
    pub claim_generator: Option<String>,
    /// Signature time as reported by the validator, when present.
    pub signing_time: Option<String>,
    /// The declared digitalSourceType, verbatim (an IPTC/C2PA URI).
    pub digital_source_type: Option<String>,
    /// Fixed descriptive phrase for the generative-AI source types, so every
    /// surface (CLI, JSON, popup) prints the identical sentence from one
    /// definition here in core.
    pub source_type_note: Option<&'static str>,
}

/// The full result: per-layer findings plus the combined verdict.
#[derive(Debug)]
pub struct Report {
    pub verdict: Verdict,
    /// `(layer name, finding)` in pipeline order, so a report always shows
    /// which layers actually ran.
    pub findings: Vec<(String, LayerFinding)>,
    /// `Some` only when the verdict is Verified — a summary must never
    /// appear beside a non-Verified verdict (pinned by tests).
    pub credentials: Option<CredentialSummary>,
}

/// The ordered pipeline.
pub struct Pipeline {
    layers: Vec<Box<dyn Layer>>,
    /// The concrete Layer-1 instance, kept so `examine` can ask it what a
    /// Verified credential claims. `None` for `with_layers` pipelines, which
    /// therefore never carry a summary.
    c2pa: Option<crate::layers::c2pa::C2paLayer>,
}

impl Pipeline {
    /// The standard four-layer pipeline in canonical order:
    /// C2PA → watermark → registry → heuristics. No trust anchors are
    /// configured, so no signature chain can validate as trusted.
    pub fn standard() -> Self {
        Self::build(crate::layers::c2pa::C2paLayer::new())
    }

    /// The standard pipeline with Layer 1 trusting the given PEM bundle of
    /// anchor root certificates (CLI: `lens verify --trust-anchors <file>`).
    pub fn with_trust_anchors(anchors_pem: impl Into<String>) -> Self {
        Self::build(crate::layers::c2pa::C2paLayer::with_trust_anchors(
            anchors_pem,
        ))
    }

    fn build(c2pa: crate::layers::c2pa::C2paLayer) -> Self {
        Pipeline {
            layers: vec![
                Box::new(c2pa.clone()),
                Box::new(crate::layers::watermark::WatermarkLayer::standard()),
                Box::new(crate::layers::registry::RegistryLayer),
                Box::new(crate::layers::heuristics::HeuristicsLayer),
            ],
            c2pa: Some(c2pa),
        }
    }

    /// A pipeline with caller-supplied layers (tests, future configuration).
    pub fn with_layers(layers: Vec<Box<dyn Layer>>) -> Self {
        Pipeline { layers, c2pa: None }
    }

    /// Run every layer and combine the findings. All layers always run —
    /// tamper evidence in a later layer must not be masked by an early exit.
    pub fn examine(&self, asset: &Asset) -> Report {
        let findings: Vec<(String, LayerFinding)> = self
            .layers
            .iter()
            .map(|layer| (layer.name().to_string(), layer.examine(asset)))
            .collect();
        let verdict = combine(&findings);
        // ponytail: the summary re-parses the asset (second Reader run), but
        // only on the Verified path; plumb it through examine() instead if a
        // profile ever cares.
        let credentials = if verdict == Verdict::Verified {
            self.c2pa
                .as_ref()
                .and_then(|layer| layer.credential_summary(asset))
        } else {
            None
        };
        Report {
            verdict,
            findings,
            credentials,
        }
    }
}

/// The combination rule, in strict precedence order:
///
/// 1. any `TamperEvidence` → `Tampered` — broken provenance outranks
///    everything, including a valid chain elsewhere in the asset; the
///    conservative reading wins.
/// 2. any `Proof`          → `Verified`
/// 3. any `Indication`     → `Indicated`
/// 4. otherwise            → `Inconclusive` — including when every layer was
///    `NotEvaluated`. No data ≠ authentic.
pub fn combine(findings: &[(String, LayerFinding)]) -> Verdict {
    let any = |pred: fn(&LayerFinding) -> bool| findings.iter().any(|(_, f)| pred(f));

    if any(|f| matches!(f, LayerFinding::TamperEvidence { .. })) {
        Verdict::Tampered
    } else if any(|f| matches!(f, LayerFinding::Proof { .. })) {
        Verdict::Verified
    } else if any(|f| matches!(f, LayerFinding::Indication { .. })) {
        Verdict::Indicated
    } else {
        Verdict::Inconclusive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(finding: LayerFinding) -> (String, LayerFinding) {
        ("test-layer".to_string(), finding)
    }

    #[test]
    fn no_findings_is_inconclusive_not_authentic() {
        assert_eq!(combine(&[]), Verdict::Inconclusive);
        assert_eq!(
            combine(&[f(LayerFinding::NotEvaluated {
                reason: "gated".into()
            })]),
            Verdict::Inconclusive
        );
        assert_eq!(combine(&[f(LayerFinding::NoSignal)]), Verdict::Inconclusive);
    }

    #[test]
    fn tamper_outranks_proof() {
        let findings = [
            f(LayerFinding::Proof {
                issuer: "Example CA".into(),
            }),
            f(LayerFinding::TamperEvidence {
                detail: "hash mismatch".into(),
            }),
        ];
        assert_eq!(combine(&findings), Verdict::Tampered);
    }

    #[test]
    fn proof_outranks_indication() {
        let findings = [
            f(LayerFinding::Indication {
                source: "watermark".into(),
            }),
            f(LayerFinding::Proof {
                issuer: "Example CA".into(),
            }),
        ];
        assert_eq!(combine(&findings), Verdict::Verified);
    }

    #[test]
    fn indication_alone_is_indicated() {
        let findings = [
            f(LayerFinding::NoSignal),
            f(LayerFinding::Indication {
                source: "registry".into(),
            }),
        ];
        assert_eq!(combine(&findings), Verdict::Indicated);
    }

    #[test]
    fn standard_pipeline_on_bare_machine_is_inconclusive() {
        // With every layer still gated/stubbed, an empty asset must come back
        // Inconclusive and the report must show all four layers.
        let pipeline = Pipeline::standard();
        let report = pipeline.examine(&Asset {
            bytes: &[],
            media_type: None,
        });
        assert_eq!(report.verdict, Verdict::Inconclusive);
        let names: Vec<&str> = report.findings.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["c2pa", "watermark", "registry", "heuristics"]);
    }
}
