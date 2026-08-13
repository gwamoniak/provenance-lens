//! C0 compatibility observations for maintainer-supplied Claude SVG
//! candidates. These are negative specimens: the pinned C2PA reader can
//! inspect SVG when given the MIME type, but these exact files have no
//! manifest store.

use provenance_core::{Asset, LayerFinding, Pipeline, Verdict};

const SVG_CANDIDATES: [&[u8]; 6] = [
    include_bytes!("vectors/claude/raw/flower-1-daisy.svg"),
    include_bytes!("vectors/claude/raw/flower-2-tulip.svg"),
    include_bytes!("vectors/claude/raw/flower-3-sunflower.svg"),
    include_bytes!("vectors/claude/raw/flower-1-daisy-labeled.svg"),
    include_bytes!("vectors/claude/raw/flower-2-tulip-labeled.svg"),
    include_bytes!("vectors/claude/raw/flower-3-sunflower-labeled.svg"),
];

const LABELED_CANDIDATES: [&[u8]; 3] = [
    include_bytes!("vectors/claude/raw/flower-1-daisy-labeled.svg"),
    include_bytes!("vectors/claude/raw/flower-2-tulip-labeled.svg"),
    include_bytes!("vectors/claude/raw/flower-3-sunflower-labeled.svg"),
];

fn c2pa_finding(report: &provenance_core::Report) -> &LayerFinding {
    report
        .findings
        .iter()
        .find_map(|(name, finding)| (name == "c2pa").then_some(finding))
        .expect("the standard pipeline always includes the C2PA layer")
}

#[test]
fn claude_svg_candidates_have_no_c2pa_manifest_with_explicit_mime() {
    for bytes in SVG_CANDIDATES {
        let report = Pipeline::standard().examine(&Asset {
            bytes,
            media_type: Some("image/svg+xml"),
        });

        assert_eq!(report.verdict, Verdict::Inconclusive);
        assert_eq!(c2pa_finding(&report), &LayerFinding::NoSignal);
        assert!(report.credentials.is_none());
    }
}
#[test]
fn core_without_mime_does_not_sniff_svg() {
    for bytes in SVG_CANDIDATES {
        let report = Pipeline::standard().examine(&Asset {
            bytes,
            media_type: None,
        });

        assert_eq!(report.verdict, Verdict::Inconclusive);
        assert!(matches!(
            c2pa_finding(&report),
            LayerFinding::NotEvaluated { reason }
                if reason == "unrecognized media type; cannot locate a manifest store"
        ));
    }
}

#[test]
fn editable_ai_generated_labels_do_not_become_c2pa_proof() {
    for bytes in LABELED_CANDIDATES {
        let text = std::str::from_utf8(bytes).expect("the SVG fixtures are UTF-8 XML");
        assert!(text.contains("<metadata>AI-generated image</metadata>"));
        assert!(text.contains(">AI-generated</text>"));

        let report = Pipeline::standard().examine(&Asset {
            bytes,
            media_type: Some("image/svg+xml"),
        });
        assert_eq!(report.verdict, Verdict::Inconclusive);
        assert_eq!(c2pa_finding(&report), &LayerFinding::NoSignal);
        assert!(report.credentials.is_none());
    }
}
