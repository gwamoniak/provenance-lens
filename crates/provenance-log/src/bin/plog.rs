//! `plog` — the git-backed pilot log's operator tool (registry plan, G2a).
//! Std-only argument handling, same as `lens`. The log repo's CI runs
//! `verify` on every change and `index` + `checkpoint` after merges; humans
//! run `keygen` and `append`.
//!
//!     plog keygen <priv-out> <pub-out>
//!     plog append <log-dir> --key <priv-file> --registrant <id> --source-type <uri> --image <file>
//!     plog index <log-dir>
//!     plog checkpoint <log-dir> --key-env <ENV_VAR>|--key <priv-file>
//!     plog verify <log-dir> [--previous-note <file>] [--log-key <independently-acquired-pub>]
//!
//! Log repo layout (pinned in the provenance-log crate docs and the log
//! repo README): entries/NNNNNNNN.bin, index/hashes.bin, log-key.pub,
//! registrants/<id>.pub, checkpoints/latest.note + checkpoints/<size>.note.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use provenance_log::*;

// ponytail: one local writer per pilot; use a service transaction only if this scale is outgrown.
struct WriterLock(PathBuf);
impl Drop for WriterLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
fn lock(dir: &str) -> Result<WriterLock, String> {
    let path = Path::new(dir).join(".plog.lock");
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| {
            format!(
                "writer lock {}: {e}; inspect stale locks before manual removal",
                path.display()
            )
        })?;
    Ok(WriterLock(path))
}
fn read_bounded(path: impl AsRef<Path>, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    std::fs::File::open(path.as_ref())
        .map_err(|e| e.to_string())?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        return Err(format!("{} exceeds size limit", path.as_ref().display()));
    }
    Ok(bytes)
}
fn read_text(path: impl AsRef<Path>, limit: usize) -> Result<String, String> {
    String::from_utf8(read_bounded(path, limit)?).map_err(|_| "file is not UTF-8".into())
}
fn publish(path: &Path, bytes: &[u8], replace: bool) -> Result<(), String> {
    let pending = path.with_extension("pending");
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&pending)
        .map_err(|e| format!("stage {}: {e}", pending.display()))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    drop(file);
    // Hard-link publication refuses existing targets atomically; never truncate history or keys.
    let result = if replace {
        std::fs::rename(&pending, path)
    } else {
        std::fs::hard_link(&pending, path)
    };
    if !replace || result.is_err() {
        let _ = std::fs::remove_file(&pending);
    }
    result.map_err(|e| format!("publish {}: {e}", path.display()))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("keygen") => keygen(&args[1..]),
        Some("append") => append(&args[1..]),
        Some("index") => index(&args[1..]),
        Some("checkpoint") => checkpoint(&args[1..]),
        Some("verify") => verify(&args[1..]),
        _ => Err("usage: plog keygen|append|index|checkpoint|verify …".to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("plog: {err}");
            ExitCode::from(2)
        }
    }
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

fn positional(args: &[String]) -> Vec<&String> {
    let mut out = Vec::new();
    let mut skip = false;
    for (i, arg) in args.iter().enumerate() {
        if skip {
            skip = false;
            continue;
        }
        if arg.starts_with("--") {
            skip = true;
            continue;
        }
        let _ = i;
        out.push(arg);
    }
    out
}

fn keygen(args: &[String]) -> Result<(), String> {
    let pos = positional(args);
    let [priv_out, pub_out] = pos.as_slice() else {
        return Err("usage: plog keygen <priv-out> <pub-out>".to_string());
    };
    if priv_out == pub_out || Path::new(priv_out).exists() || Path::new(pub_out).exists() {
        return Err("key output already exists or paths are identical; refusing overwrite".into());
    }
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).map_err(|e| format!("OS randomness failed: {e}"))?;
    let key = ed25519_dalek::SigningKey::from_bytes(&seed);
    publish(
        Path::new(priv_out),
        (hex_encode(&seed) + "\n").as_bytes(),
        false,
    )?;
    publish(
        Path::new(pub_out),
        (hex_encode(key.verifying_key().as_bytes()) + "\n").as_bytes(),
        false,
    )?;
    println!("wrote {priv_out} (KEEP PRIVATE) and {pub_out}");
    Ok(())
}

fn append(args: &[String]) -> Result<(), String> {
    let pos = positional(args);
    let [log_dir] = pos.as_slice() else {
        return Err(
            "usage: plog append <log-dir> --key <priv> --registrant <id> \
             --source-type <uri> --image <file>"
                .to_string(),
        );
    };
    let key_file = flag(args, "--key").ok_or("--key required")?;
    let registrant = flag(args, "--registrant").ok_or("--registrant required")?;
    let source_type = flag(args, "--source-type").ok_or("--source-type required")?;
    let image_path = flag(args, "--image").ok_or("--image required")?;
    validate_registrant(registrant)?;
    let _lock = lock(log_dir)?;

    let key = signing_key_from_hex(&read_text(key_file, 128)?)?;
    let bytes = read_bounded(image_path, provenance_core::pipeline::MAX_ASSET_BYTES)?;
    let decoded = provenance_core::layers::watermark::decode_rgb(&bytes)?;
    let pdq = provenance_core::layers::pdq::pdq_hash(&decoded).ok_or("image too small for PDQ")?;

    let created_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let entry = Entry::sign(pdq.bits, registrant, source_type, &created_at, &key)?;

    let entries_dir = Path::new(log_dir).join("entries");
    std::fs::create_dir_all(&entries_dir).map_err(|e| e.to_string())?;
    let next = entry_paths(log_dir)?.len();
    let path = entries_dir.join(format!("{next:08}.bin"));
    publish(&path, &entry.encode()?, false)?;
    println!(
        "appended {} (pdq {}, quality {})",
        path.display(),
        hex_encode(&pdq.bits),
        pdq.quality
    );
    Ok(())
}

fn index(args: &[String]) -> Result<(), String> {
    let pos = positional(args);
    let [log_dir] = pos.as_slice() else {
        return Err("usage: plog index <log-dir>".to_string());
    };
    let _lock = lock(log_dir)?;
    let entries = load_entries(log_dir)?;
    let mut hashes = Vec::with_capacity(entries.len() * 32);
    for (_, entry, _) in &entries {
        hashes.extend_from_slice(&entry.pdq_hash);
    }
    let index_dir = Path::new(log_dir).join("index");
    std::fs::create_dir_all(&index_dir).map_err(|e| e.to_string())?;
    publish(&index_dir.join("hashes.bin"), &hashes, true)?;
    println!("wrote index/hashes.bin ({} entries)", entries.len());
    Ok(())
}

fn checkpoint(args: &[String]) -> Result<(), String> {
    let pos = positional(args);
    let [log_dir] = pos.as_slice() else {
        return Err("usage: plog checkpoint <log-dir> --key-env <VAR>|--key <file>".to_string());
    };
    let _lock = lock(log_dir)?;
    let key_hex = if let Some(var) = flag(args, "--key-env") {
        std::env::var(var).map_err(|_| format!("env var {var} not set"))?
    } else if let Some(file) = flag(args, "--key") {
        read_text(file, 128)?
    } else {
        return Err("--key-env or --key required".to_string());
    };
    let key = signing_key_from_hex(&key_hex)?;
    let public = verifying_key_from_hex(&read_text(Path::new(log_dir).join("log-key.pub"), 128)?)?;
    if public != key.verifying_key() {
        return Err("operator signing key does not match log-key.pub".into());
    }

    let entries = load_entries(log_dir)?;
    verify_entries(log_dir, &entries)?;
    let leaves: Vec<[u8; 32]> = entries.iter().map(|(_, _, leaf)| *leaf).collect();
    let checkpoint = Checkpoint {
        tree_size: leaves.len() as u64,
        root_hex: hex_encode(&root(&leaves)),
        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    };
    let dir = Path::new(log_dir).join("checkpoints");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    if dir.join("latest.note").exists() {
        let old = Checkpoint::from_note(
            &read_text(dir.join("latest.note"), MAX_NOTE_BYTES)?,
            &public,
        )?;
        verify_prefix(&old, &leaves)?;
    }
    let history = dir.join(format!("{:08}.note", checkpoint.tree_size));
    // Recover a fully staged history file after interruption, never replace it with a new timestamp.
    let note = if history.exists() {
        let note = read_text(&history, MAX_NOTE_BYTES)?;
        let old = Checkpoint::from_note(&note, &public)?;
        if old.tree_size != checkpoint.tree_size || old.root_hex != checkpoint.root_hex {
            return Err("history conflict".into());
        }
        note
    } else {
        checkpoint.to_note(&key)
    };
    let hashes: Vec<u8> = entries.iter().flat_map(|(_, e, _)| e.pdq_hash).collect();
    let snapshot_index = dir.join(format!("{:08}.hashes", checkpoint.tree_size));
    if snapshot_index.exists() {
        if read_bounded(&snapshot_index, hashes.len())? != hashes {
            return Err("snapshot index conflict".into());
        }
    } else {
        publish(&snapshot_index, &hashes, false)?;
    }
    if !history.exists() {
        publish(&history, note.as_bytes(), false)?;
    }
    publish(&dir.join("latest.note"), note.as_bytes(), true)?;
    println!(
        "checkpoint: size {} root {}",
        checkpoint.tree_size, checkpoint.root_hex
    );
    Ok(())
}

fn verify(args: &[String]) -> Result<(), String> {
    let pos = positional(args);
    let [log_dir] = pos.as_slice() else {
        return Err("usage: plog verify <log-dir> [--previous-note <file>]".to_string());
    };

    let key_path = flag(args, "--log-key")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(log_dir).join("log-key.pub"));
    let log_key = verifying_key_from_hex(&read_text(key_path, 128)?)?;
    let note = read_text(
        Path::new(log_dir).join("checkpoints/latest.note"),
        MAX_NOTE_BYTES,
    )?;
    let checkpoint = Checkpoint::from_note(&note, &log_key)?;
    let count = usize::try_from(checkpoint.tree_size).map_err(|_| "checkpoint size overflow")?;
    let paths = entry_paths(log_dir)?;
    if count > paths.len() {
        return Err("checkpoint count exceeds available entries".into());
    }
    let pending = paths.len() - count;
    let entries = read_entries(paths.into_iter().take(count))?;
    verify_entries(log_dir, &entries)?;
    let leaves: Vec<[u8; 32]> = entries.iter().map(|(_, _, leaf)| *leaf).collect();
    verify_prefix(&checkpoint, &leaves)?;
    let expected_index: Vec<u8> = entries.iter().flat_map(|(_, e, _)| e.pdq_hash).collect();
    let snapshot = Path::new(log_dir).join(format!("checkpoints/{count:08}.hashes"));
    let index_path = if snapshot.exists() {
        snapshot
    } else {
        Path::new(log_dir).join("index/hashes.bin")
    };
    if read_bounded(index_path, expected_index.len())? != expected_index {
        return Err("snapshot index does not match entries".into());
    }
    if let Some(previous_file) = flag(args, "--previous-note") {
        let previous = Checkpoint::from_note(&read_text(previous_file, MAX_NOTE_BYTES)?, &log_key)?;
        verify_prefix(&previous, &leaves)?;
    }
    let scope = if flag(args, "--log-key").is_some() {
        "operator signature checked against supplied key; trust depends on its independent acquisition"
    } else {
        "directory self-consistency only; operator key is not independently authenticated"
    };
    println!("verify ok: {count} published entries, {pending} pending; {scope}; registrant keys remain directory-supplied");
    Ok(())
}

fn verify_prefix(checkpoint: &Checkpoint, leaves: &[[u8; 32]]) -> Result<(), String> {
    let count = usize::try_from(checkpoint.tree_size).map_err(|_| "checkpoint size overflow")?;
    if count > leaves.len() {
        return Err("log SHRANK relative to checkpoint".into());
    }
    if checkpoint.root_hex != hex_encode(&root(&leaves[..count])) {
        return Err("HISTORY REWRITTEN or checkpoint root mismatch".into());
    }
    Ok(())
}

type StoredEntry = (PathBuf, Entry, [u8; 32]);
fn verify_entries(log_dir: &str, entries: &[StoredEntry]) -> Result<(), String> {
    for (path, entry, _) in entries {
        let pub_path = Path::new(log_dir)
            .join("registrants")
            .join(format!("{}.pub", entry.registrant));
        let key_hex = read_text(&pub_path, 128).map_err(|_| {
            format!(
                "{}: registrant {:?} has no key at {}",
                path.display(),
                entry.registrant,
                pub_path.display()
            )
        })?;
        entry
            .verify(&verifying_key_from_hex(&key_hex)?)
            .map_err(|e| format!("{}: {e}", path.display()))?;
    }

    Ok(())
}

/// Entry files in leaf order, with strict contiguous naming: 00000000.bin …
fn entry_paths(log_dir: &str) -> Result<Vec<PathBuf>, String> {
    let dir = Path::new(log_dir).join("entries");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .take(100_002)
        .map(|e| {
            e.map(|e| e.file_name().to_string_lossy().into_owned())
                .map_err(|e| e.to_string())
        })
        .collect::<Result<_, _>>()?;
    if names.len() > 100_000 {
        return Err("pilot limit is 100000 entries".into());
    }
    // An interrupted create-new publication may leave a staged file, never a committed entry.
    names.retain(|name| !name.ends_with(".pending"));
    names.sort();
    for (i, name) in names.iter().enumerate() {
        let expected = format!("{i:08}.bin");
        if *name != expected {
            return Err(format!(
                "entries/ must be contiguous: expected {expected}, found {name}"
            ));
        }
    }
    Ok(names.iter().map(|n| dir.join(n)).collect())
}

fn load_entries(log_dir: &str) -> Result<Vec<StoredEntry>, String> {
    read_entries(entry_paths(log_dir)?.into_iter())
}
fn read_entries(paths: impl Iterator<Item = PathBuf>) -> Result<Vec<StoredEntry>, String> {
    paths
        .map(|path| {
            let bytes = read_bounded(&path, MAX_ENTRY_BYTES)?;
            let entry = Entry::decode(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
            let leaf = leaf_hash(&bytes);
            Ok((path, entry, leaf))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operator_pilot_recovery_and_rejection() {
        let dir = std::env::temp_dir().join(format!(
            "plog-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&dir).unwrap();
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                assert!(self.0.starts_with(std::env::temp_dir()));
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(dir.clone());
        std::fs::create_dir(dir.join("entries")).unwrap();
        std::fs::create_dir(dir.join("registrants")).unwrap();
        let key = ed25519_dalek::SigningKey::from_bytes(&[7; 32]);
        let private = dir.join("operator.key").to_string_lossy().into_owned();
        let public = dir.join("log-key.pub").to_string_lossy().into_owned();
        std::fs::write(&private, hex_encode(&key.to_bytes())).unwrap();
        std::fs::write(&public, hex_encode(key.verifying_key().as_bytes())).unwrap();
        std::fs::copy(&public, dir.join("registrants/test.pub")).unwrap();
        assert!(keygen(&[private.clone(), public.clone()]).is_err());
        assert_eq!(
            read_text(&private, 128).unwrap(),
            hex_encode(&key.to_bytes())
        );
        let log = dir.to_string_lossy().into_owned();
        let args = vec![log.clone(), "--key".into(), private];
        let image = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../provenance-core/tests/vectors/watermark/clean_base.png");
        let put = |n: u8| {
            let e = Entry::sign(
                [n; 32],
                "test",
                "http://example/ai",
                "2026-09-18T00:00:00Z",
                &key,
            )
            .unwrap()
            .encode()
            .unwrap();
            publish(&dir.join(format!("entries/{n:08}.bin")), &e, false).unwrap();
        };
        put(0);
        index(std::slice::from_ref(&log)).unwrap();
        checkpoint(&args).unwrap();
        let before = read_bounded(dir.join("checkpoints/latest.note"), MAX_NOTE_BYTES).unwrap();
        let previous = dir.join("previous.note");
        std::fs::write(&previous, &before).unwrap();
        checkpoint(&args).unwrap(); // idempotent history, unchanged timestamp
        assert_eq!(
            before,
            read_bounded(dir.join("checkpoints/latest.note"), MAX_NOTE_BYTES).unwrap()
        );
        assert!(publish(&dir.join("entries/00000000.bin"), b"bad", false).is_err());
        put(1);
        std::fs::write(dir.join("entries/00000002.pending"), b"interrupted").unwrap();
        verify(std::slice::from_ref(&log)).unwrap(); // unpublished append/staging does not break prior snapshot
        let staged = dir.join("checkpoints/latest.pending");
        std::fs::write(&staged, b"interrupt before commit point").unwrap();
        assert!(checkpoint(&args).is_err());
        verify(std::slice::from_ref(&log)).unwrap();
        assert_eq!(
            before,
            read_bounded(dir.join("checkpoints/latest.note"), MAX_NOTE_BYTES).unwrap()
        );
        std::fs::remove_file(staged).unwrap();
        checkpoint(&args).unwrap(); // reuses staged immutable history
        let check = vec![
            log.clone(),
            "--previous-note".into(),
            previous.to_string_lossy().into_owned(),
            "--log-key".into(),
            public.clone(),
        ];
        verify(&check).unwrap();
        let wrong = dir.join("wrong.pub");
        std::fs::write(
            &wrong,
            hex_encode(
                ed25519_dalek::SigningKey::from_bytes(&[9; 32])
                    .verifying_key()
                    .as_bytes(),
            ),
        )
        .unwrap();
        assert!(verify(&[
            log.clone(),
            "--log-key".into(),
            wrong.to_string_lossy().into_owned()
        ])
        .is_err());
        std::fs::write(dir.join("checkpoints/00000002.hashes"), b"bad").unwrap();
        assert!(verify(&check).is_err());
        assert!(checkpoint(&args).is_err());
        std::fs::remove_file(dir.join("entries/00000001.bin")).unwrap();
        assert!(verify(&check).is_err());
        assert!(checkpoint(&args).is_err());
        let forged = Entry::sign(
            [99; 32],
            "test",
            "http://example/ai",
            "2026-09-18T00:00:00Z",
            &key,
        )
        .unwrap()
        .encode()
        .unwrap();
        std::fs::write(dir.join("entries/00000000.bin"), forged).unwrap();
        assert!(verify_prefix(
            &Checkpoint::from_note(std::str::from_utf8(&before).unwrap(), &key.verifying_key())
                .unwrap(),
            &[leaf_hash(
                &read_bounded(dir.join("entries/00000000.bin"), MAX_ENTRY_BYTES).unwrap()
            )]
        )
        .is_err());
        // Exercise the actual image -> PDQ -> signed-entry operator path too.
        let append_args = vec![
            log.clone(),
            "--key".into(),
            args[2].clone(),
            "--registrant".into(),
            "test".into(),
            "--source-type".into(),
            "http://example/ai".into(),
            "--image".into(),
            image.to_string_lossy().into_owned(),
        ];
        append(&append_args).unwrap();
        let appended = Entry::decode(
            &read_bounded(dir.join("entries/00000001.bin"), MAX_ENTRY_BYTES).unwrap(),
        )
        .unwrap();
        appended.verify(&key.verifying_key()).unwrap();
        assert!(appended
            .verify(&ed25519_dalek::SigningKey::from_bytes(&[9; 32]).verifying_key())
            .is_err());
    }
}
