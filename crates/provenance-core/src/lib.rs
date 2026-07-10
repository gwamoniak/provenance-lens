//! provenance-core — the four-layer AI-content provenance pipeline.
//!
//! One asset (image bytes today; audio/video later) flows through four layers
//! in order of evidentiary strength:
//!
//! 1. C2PA proof        — cryptographic manifest validation (`layers::c2pa`)
//! 2. Watermark         — vendor watermark detectors            (`layers::watermark`)
//! 3. Registry lookup   — transparency-log hash lookup          (`layers::registry`)
//! 4. Heuristics        — optional statistical signals          (`layers::heuristics`)
//!
//! Their findings combine into one of four honest verdict tiers
//! ([`Verdict`]): Verified, Indicated, Inconclusive, Tampered. The project's
//! founding rule is baked into the types: **absence of provenance data is not
//! evidence of authenticity**, so "nothing found" is `Inconclusive`, never
//! "authentic".
//!
//! The crate is sans-IO: layers examine bytes they are handed and never open
//! files, sockets, or processes. The CLI and WASM wrappers own all I/O.

pub mod layers;
pub mod pipeline;
pub mod verdict;

pub use pipeline::{Asset, Layer, LayerFinding, Pipeline, Report};
pub use verdict::Verdict;
