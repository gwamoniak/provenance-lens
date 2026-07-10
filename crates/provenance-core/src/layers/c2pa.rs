//! Layer 1 — C2PA manifest validation. The only layer permitted to return
//! `LayerFinding::Proof`, because it is the only one backed by cryptography:
//! a C2PA manifest embeds a COSE signature over a claim whose hard binding
//! (content hash) ties it to these exact bytes.
//!
//! Validation runs through the `c2pa` crate (CAI Rust SDK), built with
//! default features off: rust-native crypto (no OpenSSL) and no HTTP clients,
//! so remote-manifest fetching is impossible by construction, not by
//! configuration. Any change to this file is signature-validation code and
//! requires human cryptography-reviewer sign-off before merge (see CLAUDE.md).
//!
//! Finding mapping (normative, mirrored in `.claude/skills/c2pa-spec/SKILL.md`):
//! - validation state `Trusted` (chain reaches a configured anchor) → `Proof`
//! - state `Valid` (signature checks out, chain does NOT reach an anchor,
//!   e.g. self-signed) → `TamperEvidence` — conservative by project rule
//! - state `Invalid` (bad signature, hash mismatch) → `TamperEvidence`
//! - no manifest store in the asset → `NoSignal`
//! - unparseable/unsupported asset → `NotEvaluated`. TamperEvidence comes
//!   only from validation results, never from parse errors: a corrupt plain
//!   image carries no provenance to have tampered with, and an attacker who
//!   could truncate a manifest into a parse error could strip it entirely
//!   anyway (both roads honestly end at "no verdict", not a false alarm).

use std::io::Cursor;

use c2pa::{settings::Settings, Context, Error as C2paError, Reader, ValidationState};

use crate::pipeline::{Asset, Layer, LayerFinding};

pub struct C2paLayer {
    /// PEM bundle of trust-anchor root certificates. `None` means no anchors
    /// are configured, so no chain can reach `Trusted` and signed assets cap
    /// out at `Valid` → `TamperEvidence` (unverifiable provenance).
    trust_anchors_pem: Option<String>,
}

impl C2paLayer {
    pub fn new() -> Self {
        C2paLayer {
            trust_anchors_pem: None,
        }
    }

    pub fn with_trust_anchors(pem: impl Into<String>) -> Self {
        C2paLayer {
            trust_anchors_pem: Some(pem.into()),
        }
    }

    /// Build the per-call validation context. `verify_trust` is on by the
    /// crate's default; user anchors are the caller's PEM bundle.
    fn context(&self) -> Result<Context, C2paError> {
        let mut settings = Settings::new();
        settings.trust.user_anchors = self.trust_anchors_pem.clone();
        Context::new().with_settings(settings)
    }
}

impl Default for C2paLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl Layer for C2paLayer {
    fn name(&self) -> &'static str {
        "c2pa"
    }

    fn examine(&self, asset: &Asset) -> LayerFinding {
        let format = match asset.media_type {
            Some(mime) => mime.to_string(),
            None => match sniff_media_type(asset.bytes) {
                Some(mime) => mime.to_string(),
                None => {
                    return LayerFinding::NotEvaluated {
                        reason: "unrecognized media type; cannot locate a manifest store"
                            .to_string(),
                    }
                }
            },
        };

        let context = match self.context() {
            Ok(context) => context,
            Err(err) => {
                return LayerFinding::NotEvaluated {
                    reason: format!("c2pa validation context rejected: {err}"),
                }
            }
        };

        let reader =
            match Reader::from_context(context).with_stream(&format, Cursor::new(asset.bytes)) {
                Ok(reader) => reader,
                Err(C2paError::JumbfNotFound) => return LayerFinding::NoSignal,
                Err(C2paError::UnsupportedType) => {
                    return LayerFinding::NotEvaluated {
                        reason: format!("media type {format} is not supported by the C2PA reader"),
                    }
                }
                Err(err) => {
                    // Parse errors are NOT tamper evidence (see module docs).
                    return LayerFinding::NotEvaluated {
                        reason: format!("asset could not be parsed for a manifest store: {err}"),
                    };
                }
            };

        match reader.validation_state() {
            ValidationState::Trusted => {
                let issuer = reader
                    .active_manifest()
                    .and_then(|manifest| manifest.signature_info())
                    .and_then(|info| info.issuer.clone())
                    .unwrap_or_else(|| "unknown issuer".to_string());
                LayerFinding::Proof { issuer }
            }
            ValidationState::Valid => LayerFinding::TamperEvidence {
                detail: "manifest signature verifies cryptographically but its certificate \
                         does not chain to a configured trust anchor"
                    .to_string(),
            },
            ValidationState::Invalid => {
                let codes: Vec<String> = reader
                    .validation_status()
                    .unwrap_or(&[])
                    .iter()
                    .map(|status| status.code().to_string())
                    .collect();
                let detail = if codes.is_empty() {
                    "manifest store present but fails validation".to_string()
                } else {
                    format!("manifest validation failed: {}", codes.join(", "))
                };
                LayerFinding::TamperEvidence { detail }
            }
        }
    }
}

/// Byte-signature sniffing for the formats the wedge cares about. Used only
/// when the caller supplied no MIME hint; never overrides an explicit hint.
pub(crate) fn sniff_media_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some("image/png")
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniffs_common_formats() {
        assert_eq!(
            sniff_media_type(&[0xFF, 0xD8, 0xFF, 0xE0]),
            Some("image/jpeg")
        );
        assert_eq!(
            sniff_media_type(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0]),
            Some("image/png")
        );
        assert_eq!(
            sniff_media_type(b"RIFF\x00\x00\x00\x00WEBPVP8 "),
            Some("image/webp")
        );
        assert_eq!(sniff_media_type(b"GIF89a"), Some("image/gif"));
        assert_eq!(sniff_media_type(b"not an image"), None);
        assert_eq!(sniff_media_type(&[]), None);
    }

    #[test]
    fn text_bytes_are_not_evaluated_not_tampered() {
        let layer = C2paLayer::new();
        let finding = layer.examine(&Asset {
            bytes: b"just some text, no image here",
            media_type: None,
        });
        assert!(
            matches!(finding, LayerFinding::NotEvaluated { .. }),
            "non-media bytes must be NotEvaluated, got {finding:?}"
        );
    }
}
