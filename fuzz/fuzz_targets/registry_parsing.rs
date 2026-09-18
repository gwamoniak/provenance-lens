#![no_main]
use libfuzzer_sys::fuzz_target;
use provenance_log::*;

fuzz_target!(|data: &[u8]| {
    if let Ok(entry) = Entry::decode(data) {
        assert_eq!(entry.encode().unwrap(), data);
    }
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = hex_decode(text);
        let key = verifying_key_from_hex("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a").unwrap();
        let _ = Checkpoint::from_note(text, &key);
        let _ = signing_key_from_hex(text);
        let _ = verifying_key_from_hex(text);
    }
    if data.len() >= 16 {
        let size = u64::from_le_bytes(data[..8].try_into().unwrap()) as usize;
        let index = u64::from_le_bytes(data[8..16].try_into().unwrap()) as usize;
        let proof: Vec<[u8;32]> = data[16..].chunks_exact(32).take(65).map(|c| c.try_into().unwrap()).collect();
        let _ = verify_inclusion(&[0;32], index, size, &proof, &[0;32]);
    }
});
