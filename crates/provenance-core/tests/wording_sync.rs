//! The M4 wording audit, as CI: the four approved verdict phrases are
//! canonical in `verdict.rs` and must stay present, character-identical (the
//! body after the "Tier: " prefix), in every other wording location —
//! README.md, the extension popup legend, and the verdict-language skill.
//! A drive-by "wording improvement" in any copy fails this test.

use provenance_core::Verdict;

fn repo_file(rel: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {rel}: {err}"))
}

#[test]
fn approved_phrases_are_identical_in_all_wording_locations() {
    let locations = [
        "README.md",
        "extension/popup/popup.html",
        ".claude/skills/verdict-language/SKILL.md",
        // The npm package README (U6) — wasm-pack copies it into the
        // published package, so its tier wording is user-facing too.
        "crates/provenance-wasm/README.md",
    ];
    let verdicts = [
        Verdict::Verified,
        Verdict::Indicated,
        Verdict::Inconclusive,
        Verdict::Tampered,
    ];
    for rel in locations {
        let content = repo_file(rel);
        for verdict in verdicts {
            let phrase = verdict.approved_phrase();
            // The canonical phrase is "Tier: body"; copies may typeset the
            // tier name (bold, em-dash) but the body must be verbatim.
            let body = phrase.split_once(": ").expect("phrase has a tier prefix").1;
            assert!(
                content.contains(body),
                "{rel} drifted from the canonical {:?} phrasing; expected the exact text {body:?}",
                verdict.id()
            );
        }
    }
}
