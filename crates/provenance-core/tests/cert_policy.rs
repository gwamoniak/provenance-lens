//! Cert-policy pinning tests — the backlog item accepted at the M1 review.
//!
//! Layer 1 inherits its certificate policy (expiry, algorithm allowlist)
//! from the c2pa crate. These tests pin that policy so a future c2pa
//! upgrade cannot silently weaken it: if an upgrade drops the expiry check
//! or widens the algorithm allowlist, this suite fails.
//!
//! The chains are built by an in-test X.509 builder (adapted from the c2pa
//! crate's own ephemeral-cert generation, Apache-2.0/MIT) because the
//! public `EphemeralSigner` offers no control over validity dates or
//! signature algorithms. Keys are deterministic test seeds — never trusted,
//! never persisted. Every crate used here is already in the dependency
//! tree via c2pa itself; versions are pinned to the same ones (see
//! Cargo.toml dev-dependencies).

use std::io::Cursor;
use std::thread;
use std::time::Duration;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use c2pa::{
    create_signer,
    crypto::cose::{
        check_end_entity_certificate_profile, CertificateProfileError, CertificateTrustPolicy,
    },
    status_tracker::StatusTracker,
    Builder, Context, SigningAlg,
};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use ed25519_dalek::{Signer as _, SigningKey};
use pkcs8::{EncodePrivateKey, LineEnding};
use provenance_core::{Asset, LayerFinding, Pipeline, Verdict};
use rasn::types::{
    Any, BitString, Ia5String, Integer, ObjectIdentifier, OctetString, PrintableString, SetOf,
};
use rasn_pkix::{
    AlgorithmIdentifier, AttributeTypeAndValue, AuthorityKeyIdentifier, BasicConstraints,
    Certificate, Extension, Extensions, GeneralName, GeneralNames, Name, RelativeDistinguishedName,
    SubjectPublicKeyInfo, TbsCertificate, Time, Validity, Version,
};
use sha1::{Digest, Sha1};

const PLAIN_JPG: &[u8] = include_bytes!("fixtures/plain.jpg");

const MANIFEST_DEF: &str = r#"{
    "claim_generator_info": [{ "name": "provenance-lens cert-policy tests", "version": "0.1.0" }],
    "title": "cert-policy test asset",
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

const ED25519_OID: &[u32] = &[1, 3, 101, 112];
/// sha1WithRSAEncryption — deliberately OUTSIDE the C2PA algorithm allowlist.
const SHA1_RSA_OID: &[u32] = &[1, 2, 840, 113549, 1, 1, 5];
const CN_OID: &[u32] = &[2, 5, 4, 3];
const ORG_OID: &[u32] = &[2, 5, 4, 10];
const KEY_USAGE_OID: &[u32] = &[2, 5, 29, 15];
const BASIC_CONSTRAINTS_OID: &[u32] = &[2, 5, 29, 19];
const SUBJECT_KEY_ID_OID: &[u32] = &[2, 5, 29, 14];
const AUTH_KEY_ID_OID: &[u32] = &[2, 5, 29, 35];
const SUBJECT_ALT_NAME_OID: &[u32] = &[2, 5, 29, 17];
const EXT_KEY_USAGE_OID: &[u32] = &[2, 5, 29, 37];
const EKU_EMAIL_PROTECTION_OID: &[u32] = &[1, 3, 6, 1, 5, 5, 7, 3, 4];

fn oid(components: &[u32]) -> ObjectIdentifier {
    ObjectIdentifier::new(components.to_vec()).expect("valid OID")
}

/// Subject with CN and O. The organization attribute is REQUIRED on the
/// end-entity subject: the c2pa verifier extracts it as the issuer name and
/// fails signature validation outright when it is absent.
fn name(cn: &str, org: &str) -> Name {
    let attr = |attr_oid: &[u32], value: &str| {
        let mut set = SetOf::new();
        set.insert(AttributeTypeAndValue {
            r#type: oid(attr_oid),
            value: Any::new(
                rasn::der::encode(
                    &PrintableString::try_from(value.to_string()).expect("printable"),
                )
                .expect("encode DN attribute"),
            ),
        });
        RelativeDistinguishedName::from(set)
    };
    Name::RdnSequence(vec![attr(CN_OID, cn), attr(ORG_OID, org)])
}

fn ed25519_alg() -> AlgorithmIdentifier {
    AlgorithmIdentifier {
        algorithm: oid(ED25519_OID),
        parameters: None,
    }
}

fn spki(key: &SigningKey) -> SubjectPublicKeyInfo {
    SubjectPublicKeyInfo {
        algorithm: ed25519_alg(),
        subject_public_key: BitString::from_slice(key.verifying_key().as_bytes()),
    }
}

fn ski_ext(spki_der: &[u8]) -> Extension {
    Extension {
        extn_id: oid(SUBJECT_KEY_ID_OID),
        critical: false,
        extn_value: OctetString::from(Sha1::digest(spki_der).to_vec()),
    }
}

fn validity(not_before: DateTime<Utc>, not_after: DateTime<Utc>) -> Validity {
    Validity {
        not_before: Time::Utc(not_before),
        not_after: Time::Utc(not_after),
    }
}

/// A CA + end-entity chain with caller-controlled EE validity and signature
/// algorithm OID. The CA itself is always currently valid, so any failure a
/// test provokes is attributable to the EE certificate alone.
struct TestChain {
    ee_der: Vec<u8>,
    ca_pem: String,
    chain_pem: String,
    key_pem: String,
}

impl TestChain {
    /// `sig_alg_override`: when set, the EE certificate's signature
    /// AlgorithmIdentifier claims this OID instead of Ed25519. The TBS bytes
    /// are still Ed25519-signed — the certificate profile check inspects only
    /// the declared OID, which is exactly the policy being pinned.
    fn new(
        ee_not_before: DateTime<Utc>,
        ee_not_after: DateTime<Utc>,
        sig_alg_override: Option<&[u32]>,
    ) -> Self {
        // Deterministic keys: this is test material, never trusted, never persisted.
        let ca_key = SigningKey::from_bytes(&[0x11; 32]);
        let ee_key = SigningKey::from_bytes(&[0x22; 32]);
        let now = Utc::now();

        // CA: self-signed, currently valid.
        let ca_subject = name(
            "provenance-lens cert-policy test CA",
            "provenance-lens cert-policy tests",
        );
        let ca_spki = spki(&ca_key);
        let ca_spki_der = rasn::der::encode(&ca_spki).expect("encode CA SPKI");
        let ca_tbs = TbsCertificate {
            version: Version::V3,
            serial_number: Integer::from(1i64),
            signature: ed25519_alg(),
            issuer: ca_subject.clone(),
            validity: validity(
                now - ChronoDuration::days(1),
                now + ChronoDuration::days(30),
            ),
            subject: ca_subject.clone(),
            subject_public_key_info: ca_spki,
            issuer_unique_id: None,
            subject_unique_id: None,
            extensions: Some(Extensions::from(vec![
                Extension {
                    extn_id: oid(BASIC_CONSTRAINTS_OID),
                    critical: true,
                    extn_value: rasn::der::encode(&BasicConstraints {
                        ca: true,
                        path_len_constraint: None,
                    })
                    .expect("encode basicConstraints")
                    .into(),
                },
                Extension {
                    extn_id: oid(KEY_USAGE_OID),
                    critical: true,
                    // digitalSignature, keyCertSign, cRLSign
                    extn_value: rasn::der::encode(&BitString::from_slice(&[0x86]))
                        .expect("encode keyUsage")
                        .into(),
                },
                ski_ext(&ca_spki_der),
            ])),
        };
        let ca_der = sign_cert(ca_tbs, &ca_key, None);

        // End entity: caller-controlled validity and signature algorithm.
        let ee_alg = sig_alg_override.map(|o| AlgorithmIdentifier {
            algorithm: oid(o),
            parameters: None,
        });
        let ee_spki = spki(&ee_key);
        let ee_spki_der = rasn::der::encode(&ee_spki).expect("encode EE SPKI");
        let eku = rasn::der::encode(&vec![oid(EKU_EMAIL_PROTECTION_OID)]).expect("encode EKU");
        let ee_tbs = TbsCertificate {
            version: Version::V3,
            serial_number: Integer::from(2i64),
            signature: ee_alg.clone().unwrap_or_else(ed25519_alg),
            issuer: ca_subject,
            validity: validity(ee_not_before, ee_not_after),
            subject: name(
                "cert-policy.provenance-lens.test",
                "provenance-lens cert-policy tests -- NEVER TRUSTED",
            ),
            subject_public_key_info: ee_spki,
            issuer_unique_id: None,
            subject_unique_id: None,
            extensions: Some(Extensions::from(vec![
                Extension {
                    extn_id: oid(BASIC_CONSTRAINTS_OID),
                    critical: true,
                    // Minimal DER for cA=FALSE: SEQUENCE { BOOLEAN FALSE } —
                    // rasn omits the default-false boolean, which strict
                    // validators reject (same workaround as the c2pa crate).
                    extn_value: OctetString::from([0x30, 0x03, 0x01, 0x01, 0x00].to_vec()),
                },
                Extension {
                    extn_id: oid(KEY_USAGE_OID),
                    critical: true,
                    // digitalSignature
                    extn_value: rasn::der::encode(&BitString::from_slice(&[0x80]))
                        .expect("encode keyUsage")
                        .into(),
                },
                Extension {
                    extn_id: oid(EXT_KEY_USAGE_OID),
                    critical: false,
                    extn_value: eku.into(),
                },
                ski_ext(&ee_spki_der),
                Extension {
                    extn_id: oid(AUTH_KEY_ID_OID),
                    critical: false,
                    extn_value: rasn::der::encode(&AuthorityKeyIdentifier {
                        key_identifier: Some(OctetString::from(
                            Sha1::digest(&ca_spki_der).to_vec(),
                        )),
                        authority_cert_issuer: None,
                        authority_cert_serial_number: None,
                    })
                    .expect("encode AKI")
                    .into(),
                },
                Extension {
                    extn_id: oid(SUBJECT_ALT_NAME_OID),
                    critical: false,
                    extn_value: rasn::der::encode(&GeneralNames::from(vec![GeneralName::DnsName(
                        Ia5String::try_from("cert-policy.provenance-lens.test".to_string())
                            .expect("ia5"),
                    )]))
                    .expect("encode SAN")
                    .into(),
                },
            ])),
        };
        let ee_der = sign_cert(ee_tbs, &ca_key, ee_alg);

        let ca_pem = der_to_pem(&ca_der);
        let chain_pem = format!("{}\n{}", der_to_pem(&ee_der), ca_pem);
        let key_pem = ee_key
            .to_pkcs8_pem(LineEnding::LF)
            .expect("EE key PKCS#8 PEM")
            .to_string();

        TestChain {
            ee_der,
            ca_pem,
            chain_pem,
            key_pem,
        }
    }

    /// Sign the fixture JPEG with this chain through the c2pa Builder — the
    /// same signing path production credentials use.
    fn sign(&self, source: &[u8]) -> c2pa::Result<Vec<u8>> {
        let signer = create_signer::from_keys(
            self.chain_pem.as_bytes(),
            self.key_pem.as_bytes(),
            SigningAlg::Ed25519,
            None,
        )?;
        let mut signed = Cursor::new(Vec::new());
        Builder::from_context(Context::new())
            .with_definition(MANIFEST_DEF)?
            .sign(
                &*signer,
                "image/jpeg",
                &mut Cursor::new(source),
                &mut signed,
            )?;
        Ok(signed.into_inner())
    }
}

fn sign_cert(
    tbs: TbsCertificate,
    signing_key: &SigningKey,
    alg_override: Option<AlgorithmIdentifier>,
) -> Vec<u8> {
    let tbs_der = rasn::der::encode(&tbs).expect("encode TBS");
    let sig = signing_key.sign(&tbs_der);
    let cert = Certificate {
        tbs_certificate: tbs,
        signature_algorithm: alg_override.unwrap_or_else(ed25519_alg),
        signature_value: BitString::from_slice(sig.to_bytes().as_slice()),
    };
    rasn::der::encode(&cert).expect("encode certificate")
}

fn der_to_pem(der: &[u8]) -> String {
    let b64 = BASE64.encode(der);
    let mut pem = String::from("-----BEGIN CERTIFICATE-----\n");
    for chunk in b64.as_bytes().chunks(64) {
        pem.push_str(std::str::from_utf8(chunk).expect("base64 is ascii"));
        pem.push('\n');
    }
    pem.push_str("-----END CERTIFICATE-----\n");
    pem
}

fn examine(bytes: &[u8], anchors: &str) -> provenance_core::Report {
    Pipeline::with_trust_anchors(anchors).examine(&Asset {
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

/// Control: a fresh, fully valid chain from THIS builder reaches Verified.
/// Without this, the negative tests below could pass vacuously (a chain that
/// fails for an unrelated structural reason would also never verify).
#[test]
fn control_fresh_chain_from_this_builder_verifies() {
    let now = Utc::now();
    let chain = TestChain::new(
        now - ChronoDuration::days(1),
        now + ChronoDuration::days(7),
        None,
    );
    let signed = chain.sign(PLAIN_JPG).expect("sign with valid chain");
    let report = examine(&signed, &chain.ca_pem);
    assert_eq!(
        report.verdict,
        Verdict::Verified,
        "findings: {:?}",
        report.findings
    );
}

/// The end-to-end expiry pin: an asset signed moments before its certificate
/// expired must never verify afterwards, even against the right anchor. This
/// exercises the exact validation path the product uses (no timestamp in the
/// manifest, so expiry is judged at validation time).
#[test]
fn expired_signing_cert_never_verifies() {
    let now = Utc::now();
    let not_after = now + ChronoDuration::seconds(5);
    let chain = TestChain::new(now - ChronoDuration::days(1), not_after, None);
    let signed = chain
        .sign(PLAIN_JPG)
        .expect("signing must succeed while the certificate is still valid");

    // Wait until the EE certificate has genuinely expired (+1s skew margin).
    while Utc::now() <= not_after + ChronoDuration::seconds(1) {
        thread::sleep(Duration::from_millis(200));
    }

    let report = examine(&signed, &chain.ca_pem);
    assert_eq!(
        report.verdict,
        Verdict::Tampered,
        "expired credential must read Tampered, findings: {:?}",
        report.findings
    );
    match layer1(&report) {
        LayerFinding::TamperEvidence { detail } => assert!(
            detail.contains("signingCredential"),
            "detail should carry the validator's credential status code: {detail}"
        ),
        other => panic!("expected TamperEvidence, got {other:?}"),
    }
}

/// The sign-time gate: the crate must refuse to even produce a signature
/// with an already-expired certificate.
#[test]
fn sign_time_gate_refuses_expired_cert() {
    let now = Utc::now();
    let chain = TestChain::new(
        now - ChronoDuration::days(30),
        now - ChronoDuration::days(1),
        None,
    );
    assert!(
        chain.sign(PLAIN_JPG).is_err(),
        "signing with an expired certificate must fail"
    );
}

/// Direct pin on the crate's certificate-profile expiry rule.
#[test]
fn profile_check_pins_expiry() {
    let now = Utc::now();
    let chain = TestChain::new(
        now - ChronoDuration::days(30),
        now - ChronoDuration::days(1),
        None,
    );
    let mut log = StatusTracker::default();
    let err = check_end_entity_certificate_profile(
        &chain.ee_der,
        &CertificateTrustPolicy::default(),
        &mut log,
        None,
    )
    .expect_err("expired certificate must fail the profile check");
    assert!(
        matches!(err, CertificateProfileError::CertificateNotValidAtTime),
        "expected CertificateNotValidAtTime, got {err:?}"
    );
}

/// Direct pin on the crate's signature-algorithm allowlist: a certificate
/// declaring sha1WithRSAEncryption must be rejected as unsupported.
#[test]
fn profile_check_pins_algorithm_allowlist() {
    let now = Utc::now();
    let chain = TestChain::new(
        now - ChronoDuration::days(1),
        now + ChronoDuration::days(7),
        Some(SHA1_RSA_OID),
    );
    let mut log = StatusTracker::default();
    let err = check_end_entity_certificate_profile(
        &chain.ee_der,
        &CertificateTrustPolicy::default(),
        &mut log,
        None,
    )
    .expect_err("weak-algorithm certificate must fail the profile check");
    assert!(
        matches!(err, CertificateProfileError::UnsupportedAlgorithm),
        "expected UnsupportedAlgorithm, got {err:?}"
    );
}
