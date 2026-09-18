//! Transparency-log core for the git-backed registry pilot (registry plan,
//! G2a). One binary entry format, RFC 6962 Merkle hashing, and a signed
//! checkpoint note — IDENTICAL for the pilot (flat files in a public git
//! repository) and the future G2b service, so migration is data movement,
//! not redesign. The `plog` binary wraps this library for the log repo's CI.
//!
//! Trust model, restated from the registry plan: the log operator and the
//! files it serves are UNTRUSTED. Verification is local arithmetic —
//! rebuild the tree from entries, compare the root against the signed
//! checkpoint, verify entry signatures against the registrant manifest.
//! At pilot scale clients fetch ALL entries and recompute the full tree
//! (the strongest, degenerate form of an inclusion proof); the
//! inclusion-proof functions here exist and are tested so the G2b/G3
//! remote-proof mode needs no new cryptography.
//!
//! NOTE (sign-off perimeter): when G3's Layer-3 client starts consuming
//! this crate, its verification paths become signature-validation surface
//! under the project's standing cryptography sign-off rule.
//!
//! Formats (pinned here, referenced by the log repo's README):
//!
//! Entry file (`entries/NNNNNNNN.bin`, N = 8-digit zero-padded leaf index):
//!
//! ```text
//! magic          4 bytes  "PLE1"
//! pdq_hash      32 bytes  canonical PDQ packing (provenance-core)
//! registrant     1-byte length + UTF-8 (the registrant manifest id)
//! source_type    1-byte length + UTF-8 (declared digitalSourceType URI)
//! created_at     1-byte length + UTF-8 (RFC 3339)
//! signature     64 bytes  Ed25519 over ALL preceding bytes
//! ```
//!
//! Leaf hash = SHA-256(0x00 || entry file bytes) — the signature is part of
//! the logged record. Node hash = SHA-256(0x01 || left || right), RFC 6962
//! unbalanced-tree construction.
//!
//! Checkpoint note (`checkpoints/latest.note`, Sigstore signed-note shape):
//!
//! ```text
//! provenance-lens-log/pilot\n
//! <tree_size>\n
//! <root_hash_hex>\n
//! <RFC 3339 timestamp>\n
//! \n
//! — provenance-lens-log <base64 Ed25519 signature over the 4 lines above incl. their newlines>\n
//! ```

use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

pub const ENTRY_MAGIC: &[u8; 4] = b"PLE1";
pub const NOTE_ORIGIN: &str = "provenance-lens-log/pilot";
pub const NOTE_KEY_NAME: &str = "provenance-lens-log";
pub const MAX_ENTRY_BYTES: usize = 4 + 32 + 3 + 3 * 255 + 64;
pub const MAX_NOTE_BYTES: usize = 512;
pub const MAX_PROOF_HASHES: usize = 64;

pub fn validate_registrant(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > 64
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(
            "registrant must be 1..=64 ASCII letters, digits, hyphens or underscores".into(),
        );
    }
    Ok(())
}

fn validate_timestamp(value: &str) -> Result<(), String> {
    chrono::DateTime::parse_from_rfc3339(value).map_err(|_| "invalid RFC 3339 timestamp")?;
    if value.bytes().any(|b| b.is_ascii_control()) {
        return Err("control character in timestamp".into());
    }
    Ok(())
}

// ---------- entries ----------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub pdq_hash: [u8; 32],
    pub registrant: String,
    pub source_type: String,
    pub created_at: String,
    pub signature: [u8; 64],
}

impl Entry {
    /// The bytes the registrant signs (everything before the signature).
    fn signed_body(
        pdq_hash: &[u8; 32],
        registrant: &str,
        source_type: &str,
        created_at: &str,
    ) -> Result<Vec<u8>, String> {
        validate_registrant(registrant)?;
        validate_timestamp(created_at)?;
        if source_type.chars().any(char::is_control) {
            return Err("control character in source type".into());
        }
        let mut body = Vec::with_capacity(4 + 32 + 3 + 96);
        body.extend_from_slice(ENTRY_MAGIC);
        body.extend_from_slice(pdq_hash);
        for field in [registrant, source_type, created_at] {
            let bytes = field.as_bytes();
            if bytes.is_empty() || bytes.len() > 255 {
                return Err(format!("field length {} out of range 1..=255", bytes.len()));
            }
            body.push(bytes.len() as u8);
            body.extend_from_slice(bytes);
        }
        Ok(body)
    }

    pub fn sign(
        pdq_hash: [u8; 32],
        registrant: &str,
        source_type: &str,
        created_at: &str,
        key: &SigningKey,
    ) -> Result<Entry, String> {
        let body = Self::signed_body(&pdq_hash, registrant, source_type, created_at)?;
        Ok(Entry {
            pdq_hash,
            registrant: registrant.to_string(),
            source_type: source_type.to_string(),
            created_at: created_at.to_string(),
            signature: key.sign(&body).to_bytes(),
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, String> {
        let mut bytes = Self::signed_body(
            &self.pdq_hash,
            &self.registrant,
            &self.source_type,
            &self.created_at,
        )?;
        bytes.extend_from_slice(&self.signature);
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Entry, String> {
        if bytes.len() > MAX_ENTRY_BYTES {
            return Err("entry exceeds size limit".into());
        }
        let take = |offset: &mut usize, n: usize| -> Result<Vec<u8>, String> {
            let end = offset
                .checked_add(n)
                .filter(|end| *end <= bytes.len())
                .ok_or("entry truncated")?;
            let slice = bytes[*offset..end].to_vec();
            *offset = end;
            Ok(slice)
        };
        let mut offset = 0usize;
        if take(&mut offset, 4)? != ENTRY_MAGIC {
            return Err("bad entry magic".to_string());
        }
        let pdq_hash: [u8; 32] = take(&mut offset, 32)?.try_into().unwrap();
        let string_field = |offset: &mut usize| -> Result<String, String> {
            let len = take(offset, 1)?[0] as usize;
            String::from_utf8(take(offset, len)?).map_err(|_| "field is not UTF-8".to_string())
        };
        let registrant = string_field(&mut offset)?;
        let source_type = string_field(&mut offset)?;
        let created_at = string_field(&mut offset)?;
        let signature: [u8; 64] = take(&mut offset, 64)?.try_into().unwrap();
        if offset != bytes.len() {
            return Err("trailing bytes after entry".to_string());
        }
        Self::signed_body(&pdq_hash, &registrant, &source_type, &created_at)?;
        Ok(Entry {
            pdq_hash,
            registrant,
            source_type,
            created_at,
            signature,
        })
    }

    /// Verify the registrant signature. The caller supplies the public key
    /// resolved from the registrant manifest — an unknown registrant is the
    /// caller's error to report.
    pub fn verify(&self, key: &VerifyingKey) -> Result<(), String> {
        let body = Self::signed_body(
            &self.pdq_hash,
            &self.registrant,
            &self.source_type,
            &self.created_at,
        )?;
        key.verify(&body, &Signature::from_bytes(&self.signature))
            .map_err(|_| format!("entry signature invalid (registrant {})", self.registrant))
    }
}

// ---------- RFC 6962 Merkle tree ----------

// Largest power of two strictly below n (n >= 2), without rounding overflow.
fn tree_split(n: usize) -> usize {
    1usize << (usize::BITS - 1 - (n - 1).leading_zeros())
}

pub fn leaf_hash(entry_bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([0x00]);
    hasher.update(entry_bytes);
    hasher.finalize().into()
}

fn node_hash(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update([0x01]);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

/// RFC 6962 §2.1: root of the (possibly unbalanced) tree over leaf hashes.
/// The empty tree's root is SHA-256 of the empty string.
pub fn root(leaves: &[[u8; 32]]) -> [u8; 32] {
    match leaves.len() {
        0 => Sha256::digest([]).into(),
        1 => leaves[0],
        n => {
            let split = tree_split(n);
            node_hash(&root(&leaves[..split]), &root(&leaves[split..]))
        }
    }
}

/// RFC 6962 §2.1.1 inclusion proof (audit path) for `index` in a tree of
/// `leaves`. Present for G2b/G3 remote-proof mode; the pilot client rebuilds
/// the full tree instead.
pub fn inclusion_proof(leaves: &[[u8; 32]], index: usize) -> Vec<[u8; 32]> {
    fn path(leaves: &[[u8; 32]], index: usize, proof: &mut Vec<[u8; 32]>) {
        let n = leaves.len();
        if n <= 1 {
            return;
        }
        let split = tree_split(n);
        if index < split {
            path(&leaves[..split], index, proof);
            proof.push(root(&leaves[split..]));
        } else {
            path(&leaves[split..], index - split, proof);
            proof.push(root(&leaves[..split]));
        }
    }
    let mut proof = Vec::new();
    if index >= leaves.len() {
        return proof;
    }
    path(leaves, index, &mut proof);
    proof
}

/// Verify an inclusion proof: recompute the root from the leaf hash and the
/// audit path.
pub fn verify_inclusion(
    leaf: &[u8; 32],
    index: usize,
    tree_size: usize,
    proof: &[[u8; 32]],
    expected_root: &[u8; 32],
) -> bool {
    fn climb(leaf: &[u8; 32], index: usize, size: usize, proof: &[[u8; 32]]) -> Option<[u8; 32]> {
        if size == 1 {
            return proof.is_empty().then_some(*leaf);
        }
        let split = tree_split(size);
        let (rest, sibling) = proof.split_at(proof.len().checked_sub(1)?);
        if index < split {
            Some(node_hash(&climb(leaf, index, split, rest)?, &sibling[0]))
        } else {
            Some(node_hash(
                &sibling[0],
                &climb(leaf, index - split, size - split, rest)?,
            ))
        }
    }
    proof.len() <= MAX_PROOF_HASHES
        && index < tree_size
        && climb(leaf, index, tree_size, proof).is_some_and(|r| r == *expected_root)
}

// ---------- checkpoint notes ----------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkpoint {
    pub tree_size: u64,
    pub root_hex: String,
    pub timestamp: String,
}

impl Checkpoint {
    fn body(&self) -> String {
        format!(
            "{NOTE_ORIGIN}\n{}\n{}\n{}\n",
            self.tree_size, self.root_hex, self.timestamp
        )
    }

    pub fn to_note(&self, key: &SigningKey) -> String {
        let body = self.body();
        let sig =
            base64::engine::general_purpose::STANDARD.encode(key.sign(body.as_bytes()).to_bytes());
        format!("{body}\n— {NOTE_KEY_NAME} {sig}\n")
    }

    /// Parse and verify a signed note. The signature must verify against
    /// the log's public key before any field is trusted.
    pub fn from_note(note: &str, key: &VerifyingKey) -> Result<Checkpoint, String> {
        if note.len() > MAX_NOTE_BYTES {
            return Err("note exceeds size limit".into());
        }
        let mut lines = note.lines();
        let origin = lines.next().ok_or("empty note")?;
        if origin != NOTE_ORIGIN {
            return Err(format!("unknown note origin {origin:?}"));
        }
        let tree_size: u64 = lines
            .next()
            .ok_or("missing tree size")?
            .parse()
            .map_err(|_| "tree size is not a number")?;
        let root_hex = lines.next().ok_or("missing root hash")?.to_string();
        if root_hex.len() != 64 || !root_hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("root hash is not 64 hex chars".to_string());
        }
        let timestamp = lines.next().ok_or("missing timestamp")?.to_string();
        validate_timestamp(&timestamp)?;
        if lines.next() != Some("") {
            return Err("missing blank separator line".to_string());
        }
        let sig_line = lines.next().ok_or("missing signature line")?;
        let sig_b64 = sig_line
            .strip_prefix(&format!("— {NOTE_KEY_NAME} "))
            .ok_or("malformed signature line")?;
        let sig_bytes: [u8; 64] = base64::engine::general_purpose::STANDARD
            .decode(sig_b64)
            .map_err(|_| "signature is not base64")?
            .try_into()
            .map_err(|_| "signature is not 64 bytes")?;
        let checkpoint = Checkpoint {
            tree_size,
            root_hex,
            timestamp,
        };
        if lines.next().is_some()
            || note != format!("{}\n— {NOTE_KEY_NAME} {sig_b64}\n", checkpoint.body())
        {
            return Err("noncanonical or trailing note data".into());
        }
        key.verify(
            checkpoint.body().as_bytes(),
            &Signature::from_bytes(&sig_bytes),
        )
        .map_err(|_| "checkpoint signature invalid".to_string())?;
        Ok(checkpoint)
    }
}

// ---------- keys and hex ----------

pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    if !hex.len().is_multiple_of(2) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid hex".to_string());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| "bad hex".to_string()))
        .collect()
}

pub fn signing_key_from_hex(hex: &str) -> Result<SigningKey, String> {
    if hex.trim().len() != 64 {
        return Err("signing key must be 64 hex characters".into());
    }
    let seed: [u8; 32] = hex_decode(hex.trim())?
        .try_into()
        .map_err(|_| "signing key must be 32 hex-encoded bytes")?;
    Ok(SigningKey::from_bytes(&seed))
}

pub fn verifying_key_from_hex(hex: &str) -> Result<VerifyingKey, String> {
    if hex.trim().len() != 64 {
        return Err("verifying key must be 64 hex characters".into());
    }
    let bytes: [u8; 32] = hex_decode(hex.trim())?
        .try_into()
        .map_err(|_| "verifying key must be 32 hex-encoded bytes")?;
    VerifyingKey::from_bytes(&bytes).map_err(|e| format!("invalid public key: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    #[test]
    fn malformed_boundaries_are_errors() {
        for value in ["€x", "é", "0z", "a"] {
            assert!(hex_decode(value).is_err());
        }
        assert!(!verify_inclusion(&[0; 32], 0, usize::MAX, &[], &[0; 32]));
        assert!(!verify_inclusion(&[0; 32], 0, 2, &[[0; 32]; 65], &[0; 32]));
        let mut e = Entry::decode(&entry(1)).unwrap();
        for id in ["", "../key", "a.b", "é", "a/b", "a\\b"] {
            e.registrant = id.into();
            assert!(e.encode().is_err());
        }
        let mut bytes = entry(1);
        bytes[36] = 0;
        assert!(Entry::decode(&bytes).is_err());
        assert!(Entry::decode(&vec![0; MAX_ENTRY_BYTES + 1]).is_err());
        let key = test_key();
        let mut c = Checkpoint {
            tree_size: 1,
            root_hex: hex_encode(&[1; 32]),
            timestamp: "2026-09-18T00:00:00Z".into(),
        };
        let note = c.to_note(&key);
        for bad in [
            format!("{note}extra\n"),
            note.replace('\n', "\r\n"),
            note.replace("\n1\n", "\n01\n"),
            "x".repeat(MAX_NOTE_BYTES + 1),
        ] {
            assert!(Checkpoint::from_note(&bad, &key.verifying_key()).is_err());
        }
        c.timestamp = "2026-02-30T00:00:00Z".into();
        assert!(Checkpoint::from_note(&c.to_note(&key), &key.verifying_key()).is_err());
        assert!(
            Checkpoint::from_note(&note, &SigningKey::from_bytes(&[8; 32]).verifying_key())
                .is_err()
        );
    }

    #[test]
    fn independent_rfc8032_signature_known_answer() {
        // RFC 8032 section 7.1 TEST 1; expected bytes are published, not generated by this crate.
        let key = verifying_key_from_hex(
            "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
        )
        .unwrap();
        let sig: [u8; 64] = hex_decode(concat!(
            "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e06522490155",
            "5fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b"
        ))
        .unwrap()
        .try_into()
        .unwrap();
        key.verify(b"", &Signature::from_bytes(&sig)).unwrap();
        assert!(key
            .verify(b"changed", &Signature::from_bytes(&sig))
            .is_err());
    }

    fn entry(n: u8) -> Vec<u8> {
        Entry::sign(
            [n; 32],
            "test-registrant",
            "http://example/trainedAlgorithmicMedia",
            "2026-08-02T12:00:00Z",
            &test_key(),
        )
        .unwrap()
        .encode()
        .unwrap()
    }

    #[test]
    fn entry_roundtrip_and_signature() {
        let bytes = entry(1);
        let decoded = Entry::decode(&bytes).unwrap();
        assert_eq!(decoded.registrant, "test-registrant");
        decoded.verify(&test_key().verifying_key()).unwrap();
        // Any bit flip breaks either decode or the signature.
        let mut tampered = bytes.clone();
        tampered[10] ^= 1;
        assert!(Entry::decode(&tampered)
            .and_then(|e| e.verify(&test_key().verifying_key()))
            .is_err());
    }

    #[test]
    fn rfc6962_empty_and_singleton_roots() {
        // Published transparency-dev/merkle rfc6962/rfc6962_test.go (Google, Apache-2.0).
        assert_eq!(
            hex_encode(&leaf_hash(b"")),
            "6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d"
        );
        assert_eq!(
            hex_encode(&leaf_hash(b"L123456")),
            "395aa064aa4c29f7010acfe3f25db9485bbd4b91897b6ad7ad547639252b4d56"
        );
        // Empty tree root = SHA-256("") — the RFC 6962 constant.
        assert_eq!(
            hex_encode(&root(&[])),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        let leaf = leaf_hash(b"hello");
        assert_eq!(root(&[leaf]), leaf);
    }

    #[test]
    fn inclusion_proofs_verify_and_reject_tampering() {
        let leaves: Vec<[u8; 32]> = (0u8..7).map(|n| leaf_hash(&entry(n))).collect();
        let tree_root = root(&leaves);
        for (index, leaf) in leaves.iter().enumerate() {
            let proof = inclusion_proof(&leaves, index);
            assert!(
                verify_inclusion(leaf, index, leaves.len(), &proof, &tree_root),
                "leaf {index} must verify"
            );
            // Wrong index, wrong leaf, wrong root: all rejected.
            assert!(!verify_inclusion(
                leaf,
                (index + 1) % 7,
                leaves.len(),
                &proof,
                &tree_root
            ));
            let mut bad_root = tree_root;
            bad_root[0] ^= 1;
            assert!(!verify_inclusion(
                leaf,
                index,
                leaves.len(),
                &proof,
                &bad_root
            ));
        }
    }

    #[test]
    fn appending_preserves_prefix_roots() {
        // The consistency property the CI check rides on: the root over the
        // first n leaves is unchanged by later appends.
        let leaves: Vec<[u8; 32]> = (0u8..12).map(|n| leaf_hash(&entry(n))).collect();
        let old_root = root(&leaves[..8]);
        assert_eq!(root(&leaves[..8]), old_root);
        assert_ne!(root(&leaves), old_root);
    }

    #[test]
    fn checkpoint_note_roundtrip_and_tamper_rejection() {
        let key = test_key();
        let checkpoint = Checkpoint {
            tree_size: 5,
            root_hex: hex_encode(&root(&[[9u8; 32]; 5])),
            timestamp: "2026-08-02T12:00:00Z".to_string(),
        };
        let note = checkpoint.to_note(&key);
        let parsed = Checkpoint::from_note(&note, &key.verifying_key()).unwrap();
        assert_eq!(parsed, checkpoint);
        // Any edit to the body invalidates the signature.
        let forged = note.replace("\n5\n", "\n6\n");
        assert!(Checkpoint::from_note(&forged, &key.verifying_key()).is_err());
    }
}
