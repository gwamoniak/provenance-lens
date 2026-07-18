//! Vector-corpus generator (dev tool, not product code — file I/O is fine
//! here). Regenerates `tests/vectors/` from `tests/fixtures/plain.jpg` and
//! `tests/fixtures/plain.png`:
//!
//!   plain.jpg              never signed                      → inconclusive
//!   valid_signed.jpg       ephemeral-CA signed               → verified (with test_ca.pem)
//!   stripped.jpg           signed, then APP11 removed        → inconclusive (the platform-laundering case)
//!   manifest_corrupted.jpg signed, byte flipped in manifest  → tampered
//!   content_tampered.jpg   signed, image bytes edited after  → tampered
//!   valid_signed.png       same signer, PNG container        → verified      (U3: format
//!   stripped.png           signed, caBX chunk removed        → inconclusive   coverage is
//!   manifest_corrupted.png signed, byte flipped in caBX      → tampered       proven, not
//!                          (chunk CRC refreshed so the container parses)      assumed)
//!   test_ca.pem            the run's ephemeral CA root (no private key is ever written)
//!   manifest.tsv           catalogue: file, expected verdict, provenance of each byte change
//!
//! The generator SELF-VERIFIES: every vector is run through the real pipeline
//! and the process aborts if any expected verdict does not hold, so a
//! committed corpus can never lie. Signatures are fresh per run (ephemeral
//! keys, ECDSA nondeterminism) — regenerate and commit together with
//! test_ca.pem.
//!
//! Run from the repo root:
//!
//! ```text
//! cargo run -p provenance-core --example gen_vectors
//! ```

use std::io::Cursor;
use std::path::PathBuf;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use c2pa::{Builder, Context, EphemeralSigner, Signer};
use provenance_core::{Asset, Pipeline, Verdict};

const MANIFEST_DEF: &str = r#"{
    "claim_generator_info": [{ "name": "provenance-lens test vectors", "version": "0.1.0" }],
    "title": "provenance-lens test vector",
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

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures = root.join("tests/fixtures");
    let vectors = root.join("tests/vectors");
    std::fs::create_dir_all(&vectors).expect("create tests/vectors");

    let plain = std::fs::read(fixtures.join("plain.jpg")).expect("read tests/fixtures/plain.jpg");
    let plain_png =
        std::fs::read(fixtures.join("plain.png")).expect("read tests/fixtures/plain.png");

    // Sign with a fresh ephemeral CA + end-entity chain; only the CA root
    // certificate (public) is persisted. One chain signs both containers.
    let signer = EphemeralSigner::new("provenance-lens.test").expect("ephemeral signer");
    let ca_pem = ca_root_pem(&signer);

    let signed = sign(&signer, "image/jpeg", &plain);
    let signed_png = sign(&signer, "image/png", &plain_png);

    let stripped = strip_app11(&signed);
    let (manifest_corrupted, corrupt_note) = corrupt_manifest(&signed, &ca_pem);
    let (content_tampered, tamper_note) = tamper_content(&signed, &ca_pem);

    let stripped_png = strip_cabx(&signed_png);
    let (manifest_corrupted_png, corrupt_png_note) = corrupt_png_manifest(&signed_png, &ca_pem);

    // Self-verification: the corpus must prove its own manifest.tsv.
    let rows = [
        (
            "plain.jpg",
            &plain,
            Verdict::Inconclusive,
            "source image, never signed".to_string(),
        ),
        (
            "valid_signed.jpg",
            &signed,
            Verdict::Verified,
            "signed by the ephemeral chain in test_ca.pem".to_string(),
        ),
        (
            "stripped.jpg",
            &stripped,
            Verdict::Inconclusive,
            "valid_signed.jpg with all APP11 segments removed".to_string(),
        ),
        (
            "manifest_corrupted.jpg",
            &manifest_corrupted,
            Verdict::Tampered,
            corrupt_note,
        ),
        (
            "content_tampered.jpg",
            &content_tampered,
            Verdict::Tampered,
            tamper_note,
        ),
        (
            "valid_signed.png",
            &signed_png,
            Verdict::Verified,
            "plain.png signed by the same ephemeral chain".to_string(),
        ),
        (
            "stripped.png",
            &stripped_png,
            Verdict::Inconclusive,
            "valid_signed.png with all caBX chunks removed".to_string(),
        ),
        (
            "manifest_corrupted.png",
            &manifest_corrupted_png,
            Verdict::Tampered,
            corrupt_png_note,
        ),
    ];
    for (name, bytes, expected, _) in &rows {
        let got = verdict_of(bytes, media_type_of(name), &ca_pem);
        assert_eq!(got, *expected, "self-verification failed for {name}");
    }

    let mut tsv = String::from("file\texpected_verdict\tnotes\n");
    for (name, bytes, expected, note) in &rows {
        std::fs::write(vectors.join(name), bytes).expect("write vector");
        tsv.push_str(&format!("{name}\t{}\t{note}\n", expected.id()));
    }
    std::fs::write(vectors.join("test_ca.pem"), &ca_pem).expect("write test_ca.pem");
    std::fs::write(vectors.join("manifest.tsv"), tsv).expect("write manifest.tsv");
    println!(
        "wrote {} vectors + test_ca.pem + manifest.tsv to {}",
        rows.len(),
        vectors.display()
    );
}

fn media_type_of(name: &str) -> &'static str {
    if name.ends_with(".png") {
        "image/png"
    } else {
        "image/jpeg"
    }
}

fn sign(signer: &EphemeralSigner, media_type: &str, source: &[u8]) -> Vec<u8> {
    let mut signed = Cursor::new(Vec::new());
    Builder::from_context(Context::new())
        .with_definition(MANIFEST_DEF)
        .expect("manifest definition")
        .sign(signer, media_type, &mut Cursor::new(source), &mut signed)
        .unwrap_or_else(|err| panic!("sign {media_type}: {err}"));
    signed.into_inner()
}

fn verdict_of(bytes: &[u8], media_type: &str, ca_pem: &str) -> Verdict {
    Pipeline::with_trust_anchors(ca_pem)
        .examine(&Asset {
            bytes,
            media_type: Some(media_type),
        })
        .verdict
}

/// PEM-encode the last certificate in the signer's chain (EE first, CA last).
fn ca_root_pem(signer: &EphemeralSigner) -> String {
    let chain = signer.certs().expect("signer cert chain");
    let ca_der = chain.last().expect("chain has a CA root");
    let b64 = BASE64.encode(ca_der);
    let mut pem = String::from("-----BEGIN CERTIFICATE-----\n");
    for chunk in b64.as_bytes().chunks(64) {
        pem.push_str(std::str::from_utf8(chunk).expect("base64 is ascii"));
        pem.push('\n');
    }
    pem.push_str("-----END CERTIFICATE-----\n");
    pem
}

/// Byte ranges (start, end) of every APP11 (0xFFEB) segment — where JPEG
/// carries the JUMBF manifest store.
fn app11_segments(jpeg: &[u8]) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    let mut i = 2; // past SOI
    while i + 4 <= jpeg.len() {
        if jpeg[i] != 0xFF {
            break; // desynchronized; stop walking
        }
        let marker = jpeg[i + 1];
        match marker {
            0xD8 | 0x01 | 0xD0..=0xD7 => i += 2, // standalone markers
            0xDA => break,                       // start of scan: entropy data follows
            _ => {
                let len = u16::from_be_bytes([jpeg[i + 2], jpeg[i + 3]]) as usize;
                let end = i + 2 + len;
                if end > jpeg.len() {
                    break;
                }
                if marker == 0xEB {
                    segments.push((i, end));
                }
                i = end;
            }
        }
    }
    segments
}

/// The platform-laundering case: drop every APP11 segment.
fn strip_app11(jpeg: &[u8]) -> Vec<u8> {
    let segments = app11_segments(jpeg);
    assert!(
        !segments.is_empty(),
        "signed JPEG must contain APP11 segments"
    );
    let mut out = Vec::with_capacity(jpeg.len());
    let mut cursor = 0;
    for (start, end) in segments {
        out.extend_from_slice(&jpeg[cursor..start]);
        cursor = end;
    }
    out.extend_from_slice(&jpeg[cursor..]);
    out
}

/// Flip one byte inside the manifest store, searching for an offset that the
/// validator reports as Tampered (not a parse failure).
fn corrupt_manifest(signed: &[u8], ca_pem: &str) -> (Vec<u8>, String) {
    let (start, end) = *app11_segments(signed).first().expect("APP11 present");
    // Skip the segment header region; probe the payload.
    for offset in ((start + 32)..end.min(start + 4096)).step_by(97) {
        let mut copy = signed.to_vec();
        copy[offset] ^= 0x01;
        if verdict_of(&copy, "image/jpeg", ca_pem) == Verdict::Tampered {
            return (copy, format!("valid_signed.jpg with byte at offset {offset} XOR 0x01 (inside the manifest store)"));
        }
    }
    panic!("no manifest byte-flip produced a Tampered verdict");
}

/// Flip one image byte AFTER signing so the hard binding (content hash) no
/// longer matches.
fn tamper_content(signed: &[u8], ca_pem: &str) -> (Vec<u8>, String) {
    // Probe backwards from just before EOI (last two bytes), through entropy data.
    for back in [3, 8, 16, 32, 64, 128] {
        if back + 2 > signed.len() {
            continue;
        }
        let offset = signed.len() - back;
        let mut copy = signed.to_vec();
        copy[offset] ^= 0x01;
        if verdict_of(&copy, "image/jpeg", ca_pem) == Verdict::Tampered {
            return (copy, format!("valid_signed.jpg with image byte at offset {offset} XOR 0x01 (content-hash mismatch)"));
        }
    }
    panic!("no content byte-flip produced a Tampered verdict");
}

/// CRC-32 (ISO 3309, as PNG uses) — bitwise, no table; fast enough for one
/// chunk in a dev tool.
fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                0xEDB8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// (start, end, type) span of every PNG chunk — start at the length field,
/// end just past the CRC.
fn png_chunks(png: &[u8]) -> Vec<(usize, usize, [u8; 4])> {
    let mut chunks = Vec::new();
    let mut i = 8; // past the PNG signature
    while i + 8 <= png.len() {
        let len = u32::from_be_bytes([png[i], png[i + 1], png[i + 2], png[i + 3]]) as usize;
        let Some(end) = i.checked_add(8 + len + 4) else {
            break;
        };
        if end > png.len() {
            break;
        }
        chunks.push((i, end, [png[i + 4], png[i + 5], png[i + 6], png[i + 7]]));
        i = end;
    }
    chunks
}

/// The platform-laundering case for PNG: drop every caBX chunk (where PNG
/// carries the JUMBF manifest store).
fn strip_cabx(png: &[u8]) -> Vec<u8> {
    let cabx: Vec<(usize, usize)> = png_chunks(png)
        .into_iter()
        .filter(|(_, _, kind)| kind == b"caBX")
        .map(|(start, end, _)| (start, end))
        .collect();
    assert!(!cabx.is_empty(), "signed PNG must contain caBX chunks");
    let mut out = Vec::with_capacity(png.len());
    let mut cursor = 0;
    for (start, end) in cabx {
        out.extend_from_slice(&png[cursor..start]);
        cursor = end;
    }
    out.extend_from_slice(&png[cursor..]);
    out
}

/// Flip one byte inside the caBX chunk data and REFRESH the chunk CRC, so the
/// container still parses and the failure is manifest validation (Tampered),
/// never a parse error. Probes offsets until the validator agrees.
fn corrupt_png_manifest(signed: &[u8], ca_pem: &str) -> (Vec<u8>, String) {
    let (start, end, _) = png_chunks(signed)
        .into_iter()
        .find(|(_, _, kind)| kind == b"caBX")
        .expect("caBX chunk present");
    let data_start = start + 8;
    let data_end = end - 4; // CRC field
    for offset in ((data_start + 32)..data_end.min(data_start + 4096)).step_by(97) {
        let mut copy = signed.to_vec();
        copy[offset] ^= 0x01;
        let crc = crc32(&copy[start + 4..data_end]); // over type + data
        copy[data_end..end].copy_from_slice(&crc.to_be_bytes());
        if verdict_of(&copy, "image/png", ca_pem) == Verdict::Tampered {
            return (
                copy,
                format!(
                    "valid_signed.png with byte at offset {offset} XOR 0x01 inside the caBX \
                     chunk, chunk CRC refreshed so the container parses"
                ),
            );
        }
    }
    panic!("no caBX byte-flip produced a Tampered verdict");
}
