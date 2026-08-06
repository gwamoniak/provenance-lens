//! `plog` — the git-backed pilot log's operator tool (registry plan, G2a).
//! Std-only argument handling, same as `lens`. The log repo's CI runs
//! `verify` on every change and `index` + `checkpoint` after merges; humans
//! run `keygen` and `append`.
//!
//!     plog keygen <priv-out> <pub-out>
//!     plog append <log-dir> --key <priv-file> --registrant <id> --source-type <uri> --image <file>
//!     plog index <log-dir>
//!     plog checkpoint <log-dir> --key-env <ENV_VAR>|--key <priv-file>
//!     plog verify <log-dir> [--previous-note <file-or->]
//!
//! Log repo layout (pinned in the provenance-log crate docs and the log
//! repo README): entries/NNNNNNNN.bin, index/hashes.bin, log-key.pub,
//! registrants/<id>.pub, checkpoints/latest.note + checkpoints/<size>.note.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use provenance_log::*;

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
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).map_err(|e| format!("OS randomness failed: {e}"))?;
    let key = ed25519_dalek::SigningKey::from_bytes(&seed);
    std::fs::write(priv_out, hex_encode(&seed) + "\n").map_err(|e| e.to_string())?;
    std::fs::write(
        pub_out,
        hex_encode(key.verifying_key().as_bytes()) + "\n",
    )
    .map_err(|e| e.to_string())?;
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

    let key = signing_key_from_hex(
        &std::fs::read_to_string(key_file).map_err(|e| format!("read {key_file}: {e}"))?,
    )?;
    let bytes = std::fs::read(image_path).map_err(|e| format!("read {image_path}: {e}"))?;
    let decoded = image::load_from_memory(&bytes)
        .map_err(|e| format!("decode {image_path}: {e}"))?
        .to_rgb8();
    let (width, height) = (decoded.width() as usize, decoded.height() as usize);
    let pdq = provenance_core::layers::pdq::pdq_hash(
        &provenance_core::layers::watermark::DecodedImage {
            rgb: decoded.into_raw(),
            width,
            height,
        },
    )
    .ok_or("image too small for PDQ")?;

    let created_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let entry = Entry::sign(pdq.bits, registrant, source_type, &created_at, &key)?;

    let entries_dir = Path::new(log_dir).join("entries");
    std::fs::create_dir_all(&entries_dir).map_err(|e| e.to_string())?;
    let next = entry_paths(log_dir)?.len();
    let path = entries_dir.join(format!("{next:08}.bin"));
    std::fs::write(&path, entry.encode()).map_err(|e| e.to_string())?;
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
    let entries = load_entries(log_dir)?;
    let mut hashes = Vec::with_capacity(entries.len() * 32);
    for (_, entry, _) in &entries {
        hashes.extend_from_slice(&entry.pdq_hash);
    }
    let index_dir = Path::new(log_dir).join("index");
    std::fs::create_dir_all(&index_dir).map_err(|e| e.to_string())?;
    std::fs::write(index_dir.join("hashes.bin"), hashes).map_err(|e| e.to_string())?;
    println!("wrote index/hashes.bin ({} entries)", entries.len());
    Ok(())
}

fn checkpoint(args: &[String]) -> Result<(), String> {
    let pos = positional(args);
    let [log_dir] = pos.as_slice() else {
        return Err("usage: plog checkpoint <log-dir> --key-env <VAR>|--key <file>".to_string());
    };
    let key_hex = if let Some(var) = flag(args, "--key-env") {
        std::env::var(var).map_err(|_| format!("env var {var} not set"))?
    } else if let Some(file) = flag(args, "--key") {
        std::fs::read_to_string(file).map_err(|e| format!("read {file}: {e}"))?
    } else {
        return Err("--key-env or --key required".to_string());
    };
    let key = signing_key_from_hex(&key_hex)?;

    let entries = load_entries(log_dir)?;
    let leaves: Vec<[u8; 32]> = entries.iter().map(|(_, _, leaf)| *leaf).collect();
    let checkpoint = Checkpoint {
        tree_size: leaves.len() as u64,
        root_hex: hex_encode(&root(&leaves)),
        timestamp: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    };
    let note = checkpoint.to_note(&key);
    let dir = Path::new(log_dir).join("checkpoints");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join("latest.note"), &note).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(format!("{:08}.note", checkpoint.tree_size)), &note)
        .map_err(|e| e.to_string())?;
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

    // 1. Every entry decodes and its signature verifies against the
    //    registrant manifest.
    let entries = load_entries(log_dir)?;
    for (path, entry, _) in &entries {
        let pub_path = Path::new(log_dir)
            .join("registrants")
            .join(format!("{}.pub", entry.registrant));
        let key_hex = std::fs::read_to_string(&pub_path).map_err(|_| {
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

    // 2. index/hashes.bin matches the entries exactly.
    let expected_index: Vec<u8> = entries
        .iter()
        .flat_map(|(_, e, _)| e.pdq_hash.to_vec())
        .collect();
    let index_path = Path::new(log_dir).join("index/hashes.bin");
    let actual_index = std::fs::read(&index_path)
        .map_err(|e| format!("read {}: {e} (run plog index)", index_path.display()))?;
    if actual_index != expected_index {
        return Err("index/hashes.bin does not match the entries".to_string());
    }

    // 3. The signed checkpoint matches the recomputed tree.
    let log_key = verifying_key_from_hex(
        &std::fs::read_to_string(Path::new(log_dir).join("log-key.pub"))
            .map_err(|e| format!("read log-key.pub: {e}"))?,
    )?;
    let note = std::fs::read_to_string(Path::new(log_dir).join("checkpoints/latest.note"))
        .map_err(|e| format!("read checkpoints/latest.note: {e}"))?;
    let checkpoint = Checkpoint::from_note(&note, &log_key)?;
    let leaves: Vec<[u8; 32]> = entries.iter().map(|(_, _, leaf)| *leaf).collect();
    if checkpoint.tree_size != leaves.len() as u64 {
        return Err(format!(
            "checkpoint covers {} entries but the log has {}",
            checkpoint.tree_size,
            leaves.len()
        ));
    }
    if checkpoint.root_hex != hex_encode(&root(&leaves)) {
        return Err("checkpoint root does not match the recomputed tree".to_string());
    }

    // 4. Consistency: the previous checkpoint's tree must be a prefix of
    //    this one (history is appended to, never rewritten).
    if let Some(previous_file) = flag(args, "--previous-note") {
        let previous_text =
            std::fs::read_to_string(previous_file).map_err(|e| format!("read previous: {e}"))?;
        if previous_text.trim().is_empty() {
            println!("no previous checkpoint (first ever) — consistency vacuous");
        } else {
            let previous = Checkpoint::from_note(&previous_text, &log_key)?;
            if previous.tree_size > leaves.len() as u64 {
                return Err("log SHRANK relative to the previous checkpoint".to_string());
            }
            let prefix_root = hex_encode(&root(&leaves[..previous.tree_size as usize]));
            if prefix_root != previous.root_hex {
                return Err(
                    "HISTORY REWRITTEN: previous checkpoint is not a prefix of this log"
                        .to_string(),
                );
            }
            println!(
                "consistency ok: {} -> {} entries",
                previous.tree_size,
                leaves.len()
            );
        }
    }

    println!(
        "verify ok: {} entries, root {}, checkpoint {}",
        entries.len(),
        checkpoint.root_hex,
        checkpoint.timestamp
    );
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
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
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

#[allow(clippy::type_complexity)]
fn load_entries(log_dir: &str) -> Result<Vec<(PathBuf, Entry, [u8; 32])>, String> {
    entry_paths(log_dir)?
        .into_iter()
        .map(|path| {
            let bytes = std::fs::read(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
            let entry =
                Entry::decode(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
            let leaf = leaf_hash(&bytes);
            Ok((path, entry, leaf))
        })
        .collect()
}
