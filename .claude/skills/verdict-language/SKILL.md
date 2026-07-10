---
name: verdict-language
description: The normative wording rules for the four verdict tiers — approved phrases, forbidden implications, and styling rules. Load before writing or reviewing ANY user-facing string (CLI output, extension UI, README, store listing, error messages) that talks about a verdict.
---

# Verdict language — the wording rules

This project's credibility rests on never overclaiming in either direction. These rules are normative; the canonical machine copy is `crates/provenance-core/src/verdict.rs` (`approved_phrase()`), and this file, the extension popup, and the README must stay character-identical to it. Changing a phrase requires an ExecPlan Decision Log entry and all four locations updated in one change.

## The founding rule

**No data ≠ authentic.** Most genuine, human-made images on the web carry no provenance data. Therefore finding nothing means *nothing was found* — it licenses no claim about authenticity, in any direction. Every surface must survive this test: could a reader walk away thinking "the tool said it's real"? If yes, the wording is wrong.

## The four tiers and their approved phrases

- `verified` — "Verified: this asset carries a valid, cryptographically signed provenance chain."
- `indicated` — "Indicated: signals suggest AI involvement, but no cryptographic proof chain is present."
- `inconclusive` — "Inconclusive: no provenance data was found. This does NOT mean the asset is authentic."
- `tampered` — "Tampered: provenance data is present but fails validation. Treat this asset with suspicion."

## Forbidden phrasings, per tier

- **Inconclusive** must never be rendered as: "authentic", "real", "clean", "human-made", "no AI detected", "passed", "safe", a green checkmark, or any visual language of reassurance. Neutral gray, always.
- **Verified** verifies the *provenance chain*, not virtue: never "genuine", "trustworthy", "not AI" (a verified manifest may explicitly declare AI generation — say what the provenance asserts). Never drop "cryptographically signed" from long-form copy; it is the claim's entire basis.
- **Indicated** is not an accusation: never "fake", "deepfake", "AI-generated" as flat fact, "caught". Signals suggest; they do not prove. Watermark and registry hits have false positives and the wording must survive one.
- **Tampered** means the *provenance data* fails validation — commonly because a platform stripped or re-encoded it, not because the author lied: never "forged", "malicious", "fraud". "Treat with suspicion" is the ceiling.

## General rules

- Tier names are capitalized exactly: Verified, Indicated, Inconclusive, Tampered.
- Never present a probability or confidence score next to a verdict unless the plan has specified how it was calibrated — an uncalibrated "87% AI" is worse than no number.
- Error states (engine missing, fetch failed, unsupported format) are stated as errors, never mapped onto a tier.
- Translations are reviewed against this file's tests: the Inconclusive translation must contain an explicit "this does not mean authentic" clause.
- Marketing copy obeys the same rules. The product's brand is understatement; "detect AI fakes instantly" would be a violation, not a tagline.
