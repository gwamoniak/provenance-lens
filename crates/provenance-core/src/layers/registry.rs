//! Layer 3 — registry lookup: compute a perceptual hash (PDQ/pHash) locally
//! and query provenance registries — C2PA recovery services, Numbers
//! Protocol, or this project's own transparency log — for assets registered
//! as AI-generated at creation time. Catches images whose metadata was
//! stripped on upload. A hit is an `Indication` (the registrant vouches, not
//! cryptography over these bytes); a miss is `NoSignal`. Details in
//! `.claude/skills/provenance-registry/SKILL.md`.
//!
//! Blockchain appears here ONLY as optional anchoring of the transparency
//! log's checkpoints — the registry must work fully without it, and no code
//! path may require a chain lookup to produce a verdict.
//!
//! GATED (post-wedge milestone): requires a deployed transparency-log
//! endpoint. The layer stays sans-IO: the lookup transport is injected at
//! construction, so tests exercise it with an in-memory fake.

use crate::pipeline::{Asset, Layer, LayerFinding};

pub struct RegistryLayer;

impl Layer for RegistryLayer {
    fn name(&self) -> &'static str {
        "registry"
    }

    fn examine(&self, _asset: &Asset) -> LayerFinding {
        LayerFinding::NotEvaluated {
            reason: "gated: no transparency-log registry deployed yet".to_string(),
        }
    }
}
