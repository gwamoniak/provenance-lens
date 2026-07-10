//! Verdict tiers and the wording rules that keep them honest.
//!
//! The wording rules are normative and mirrored in
//! `.claude/skills/verdict-language/SKILL.md`; changing a phrase here requires
//! updating the skill, the extension UI, and the README in the same change.

use std::fmt;

/// The four honest verdict tiers.
///
/// The single most important rule of this project: **absence of provenance
/// data is not evidence of authenticity.** `Inconclusive` must never be
/// rendered as "authentic", "human-made", "clean", or "no AI detected" — not
/// in the CLI, not in the extension, not in a log line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// A complete, cryptographically valid C2PA manifest chain was validated
    /// against the asset's content hash. Only Layer 1 can produce this.
    Verified,
    /// A non-cryptographic signal (watermark hit, registry match) indicates
    /// AI involvement. Strong enough to surface, never claimed as proof.
    Indicated,
    /// No usable provenance signal was found. Says nothing about authenticity
    /// — most genuine photos on today's web carry no credentials either.
    Inconclusive,
    /// Provenance data is present but broken: signature mismatch, truncated
    /// manifest, or a content hash that no longer matches the asset. This is
    /// the tier that catches credential stripping and re-encoding.
    Tampered,
}

impl Verdict {
    /// The only approved one-line phrasing for each tier. UI surfaces render
    /// these strings (or a reviewed translation of them) verbatim.
    pub fn approved_phrase(&self) -> &'static str {
        match self {
            Verdict::Verified => {
                "Verified: this asset carries a valid, cryptographically signed provenance chain."
            }
            Verdict::Indicated => {
                "Indicated: signals suggest AI involvement, but no cryptographic proof chain is present."
            }
            Verdict::Inconclusive => {
                "Inconclusive: no provenance data was found. This does NOT mean the asset is authentic."
            }
            Verdict::Tampered => {
                "Tampered: provenance data is present but fails validation. Treat this asset with suspicion."
            }
        }
    }

    /// Stable machine-readable identifier (CLI exit-code mapping, JSON output,
    /// extension message passing).
    pub fn id(&self) -> &'static str {
        match self {
            Verdict::Verified => "verified",
            Verdict::Indicated => "indicated",
            Verdict::Inconclusive => "inconclusive",
            Verdict::Tampered => "tampered",
        }
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.approved_phrase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inconclusive_wording_never_implies_authenticity() {
        let phrase = Verdict::Inconclusive.approved_phrase();
        assert!(
            phrase.contains("does NOT mean the asset is authentic"),
            "the no-data-is-not-authentic rule must stay in the approved phrase"
        );
        for forbidden in ["clean", "human-made", "no AI"] {
            assert!(
                !phrase.contains(forbidden),
                "forbidden implication {forbidden:?} in Inconclusive phrasing"
            );
        }
    }

    #[test]
    fn ids_are_stable() {
        assert_eq!(Verdict::Verified.id(), "verified");
        assert_eq!(Verdict::Indicated.id(), "indicated");
        assert_eq!(Verdict::Inconclusive.id(), "inconclusive");
        assert_eq!(Verdict::Tampered.id(), "tampered");
    }
}
