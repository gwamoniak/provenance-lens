//! Fuzz target for manifest parsing (post-wedge hardening backlog item):
//! the pipeline must never panic on attacker-controlled bytes, and with no
//! trust anchors configured it must never mint a Proof — Trusted is
//! unreachable without anchors, so a Proof here is a bug by construction.
//!
//! Run (needs nightly + cargo-fuzz):
//!
//!     cargo +nightly fuzz run manifest_parsing fuzz/corpus_seed
//!
//! `fuzz/corpus_seed/` holds the committed vector corpus as seeds.
//!
//! Windows note: keep the default ASAN sanitizer but put MSVC's ASAN
//! runtime on PATH first, or the binary dies at launch with
//! STATUS_DLL_NOT_FOUND ("clang_rt.asan_dynamic-x86_64.dll" lives under
//! VC\Tools\MSVC\<ver>\bin\Hostx64\x64). `--sanitizer none` is NOT a
//! workaround on MSVC — libFuzzer's SanitizerCoverage needs linker
//! section-boundary symbols that only materialize in sanitized builds
//! there (link error: unresolved __stop___sancov_pcs).

#![no_main]

use libfuzzer_sys::fuzz_target;
use provenance_core::{Asset, LayerFinding, Pipeline};

fuzz_target!(|data: &[u8]| {
    let report = Pipeline::standard().examine(&Asset {
        bytes: data,
        media_type: None,
    });
    for (name, finding) in &report.findings {
        assert!(
            !matches!(finding, LayerFinding::Proof { .. }),
            "layer {name} minted a Proof with no trust anchors configured"
        );
    }
});
