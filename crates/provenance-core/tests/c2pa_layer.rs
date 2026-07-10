//! Layer 1 integration tests — known answers by construction. Every signed
//! asset is built in-memory with a fresh ephemeral CA per run (no committed
//! keys), so each expectation is validated against ground truth we control.

use std::io::Cursor;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use c2pa::{Builder, Context, EphemeralSigner, Signer};
use provenance_core::{Asset, LayerFinding, Pipeline, Verdict};

const PLAIN_JPG: &[u8] = include_bytes!("fixtures/plain.jpg");

const MANIFEST_DEF: &str = r#"{
    "claim_generator_info": [{ "name": "provenance-lens tests", "version": "0.1.0" }],
    "title": "in-memory test asset",
    "assertions": [
        {
            "label": "c2pa.actions",
            "data": {
                "actions": [{
                    "action": "c2pa.created",
                    "digitalSourceType": "http://cv.iptc.org/newscodes/digitalsourcetype/trainedAlgorithmicMedia"
                }]
            }
        }
    ]
}"#;

/// Sign the fixture with a fresh ephemeral chain; return (signed bytes, CA root PEM).
fn signed_asset() -> (Vec<u8>, String) {
    let signer = EphemeralSigner::new("provenance-lens.test").expect("ephemeral signer");
    let chain = signer.certs().expect("cert chain");
    let ca_der = chain.last().expect("CA root");
    let b64 = BASE64.encode(ca_der);
    let mut ca_pem = String::from("-----BEGIN CERTIFICATE-----\n");
    for chunk in b64.as_bytes().chunks(64) {
        ca_pem.push_str(std::str::from_utf8(chunk).unwrap());
        ca_pem.push('\n');
    }
    ca_pem.push_str("-----END CERTIFICATE-----\n");

    let mut signed = Cursor::new(Vec::new());
    Builder::from_context(Context::new())
        .with_definition(MANIFEST_DEF)
        .expect("manifest definition")
        .sign(
            &signer,
            "image/jpeg",
            &mut Cursor::new(PLAIN_JPG),
            &mut signed,
        )
        .expect("sign fixture");
    (signed.into_inner(), ca_pem)
}

fn examine(bytes: &[u8], anchors: Option<&str>) -> provenance_core::Report {
    let pipeline = match anchors {
        Some(pem) => Pipeline::with_trust_anchors(pem),
        None => Pipeline::standard(),
    };
    pipeline.examine(&Asset {
        bytes,
        media_type: Some("image/jpeg"),
    })
}

fn layer1(report: &provenance_core::Report) -> &LayerFinding {
    &report
        .findings
        .iter()
        .find(|(name, _)| name == "c2pa")
        .expect("c2pa layer ran")
        .1
}

#[test]
fn trusted_chain_is_verified_with_issuer() {
    let (signed, ca_pem) = signed_asset();
    let report = examine(&signed, Some(&ca_pem));
    assert_eq!(
        report.verdict,
        Verdict::Verified,
        "findings: {:?}",
        report.findings
    );
    match layer1(&report) {
        LayerFinding::Proof { issuer } => assert!(!issuer.is_empty()),
        other => panic!("expected Proof, got {other:?}"),
    }
}

#[test]
fn valid_but_unanchored_is_tampered_never_proof() {
    // Same signed asset, but NO trust anchors configured: cryptographically
    // valid, unverifiable provenance. Project rule: TamperEvidence, not Proof.
    let (signed, _ca_pem) = signed_asset();
    let report = examine(&signed, None);
    assert_eq!(
        report.verdict,
        Verdict::Tampered,
        "findings: {:?}",
        report.findings
    );
    match layer1(&report) {
        LayerFinding::TamperEvidence { detail } => {
            assert!(
                detail.contains("trust anchor"),
                "detail should name the cause: {detail}"
            )
        }
        other => panic!("expected TamperEvidence, got {other:?}"),
    }
}

#[test]
fn unsigned_asset_is_no_signal_and_inconclusive() {
    let report = examine(PLAIN_JPG, None);
    assert_eq!(report.verdict, Verdict::Inconclusive);
    assert_eq!(*layer1(&report), LayerFinding::NoSignal);
}

#[test]
fn content_edit_after_signing_is_tampered() {
    let (signed, ca_pem) = signed_asset();
    let mut edited = signed.clone();
    let len = edited.len();
    edited[len - 8] ^= 0x01; // entropy data near EOI: hard-binding mismatch
    let report = examine(&edited, Some(&ca_pem));
    assert_eq!(
        report.verdict,
        Verdict::Tampered,
        "findings: {:?}",
        report.findings
    );
}

#[test]
fn wrong_anchor_does_not_verify() {
    // Signed by chain A, validated against anchor B: must not be Verified.
    let (signed, _ca_a) = signed_asset();
    let (_other, ca_b) = signed_asset();
    let report = examine(&signed, Some(&ca_b));
    assert_ne!(report.verdict, Verdict::Verified);
    assert_eq!(
        report.verdict,
        Verdict::Tampered,
        "findings: {:?}",
        report.findings
    );
}

#[test]
fn hostile_bytes_never_panic_and_never_prove() {
    // Deterministic pseudo-random garbage, magic-byte prefixes, and
    // truncations of a real signed asset: whatever comes back, the layer
    // must not panic and must never mint a Proof.
    let (signed, ca_pem) = signed_asset();

    let mut buffers: Vec<Vec<u8>> = Vec::new();
    let mut state: u64 = 0x5EED_CAFE_F00D_D00D;
    for len in [0usize, 1, 2, 16, 256, 4096] {
        let mut buf = Vec::with_capacity(len + 3);
        buf.extend_from_slice(&[0xFF, 0xD8, 0xFF]); // JPEG magic so parsing engages
        for _ in 0..len {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            buf.push((state >> 33) as u8);
        }
        buffers.push(buf);
    }
    for cut in 1..signed.len().min(64) {
        buffers.push(signed[..cut].to_vec());
    }
    for cut in [signed.len() / 4, signed.len() / 2, signed.len() - 1] {
        buffers.push(signed[..cut].to_vec());
    }

    for (i, bytes) in buffers.iter().enumerate() {
        let report = examine(bytes, Some(&ca_pem));
        assert_ne!(
            report.verdict,
            Verdict::Verified,
            "hostile buffer {i} must never verify"
        );
        for (_, finding) in &report.findings {
            assert!(
                !matches!(finding, LayerFinding::Proof { .. }),
                "hostile buffer {i} minted a Proof"
            );
        }
    }
}
