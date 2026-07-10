//! The four layers, in canonical pipeline order. Each is honest about what it
//! can and cannot do yet: an unimplemented or gated layer returns
//! `LayerFinding::NotEvaluated` — never a fake `NoSignal`.

pub mod c2pa;
pub mod heuristics;
pub mod registry;
pub mod watermark;
