//! PDQ calibration (registry plan, G1): measures, over the W2 calibration
//! corpus's `clean` set, (a) how far the SAME content drifts under each
//! transformation (robustness / would-it-still-match), (b) how close
//! DIFFERENT content ever gets (false-match risk at candidate thresholds),
//! and (c) multi-index nomination recall (share of same-content pairs with
//! at least one exact 16-bit word in common — the G2 lookup's recall).
//! Output is markdown for docs/CALIBRATION.md.
//!
//!     cargo run --release -p provenance-core --example pdq_calibrate -- <corpus-root>

use std::collections::BTreeMap;
use std::path::Path;

use provenance_core::layers::pdq::{hamming, pdq_hash, words, PdqHash};
use provenance_core::layers::watermark::DecodedImage;

const THRESHOLDS: [u32; 4] = [24, 31, 47, 63];

fn main() {
    let Some(root) = std::env::args().nth(1) else {
        eprintln!("usage: pdq_calibrate <calibration-corpus-root>");
        std::process::exit(2);
    };
    let clean = Path::new(&root).join("clean");

    // (transform, stem) -> hash
    let mut hashes: BTreeMap<(String, String), PdqHash> = BTreeMap::new();
    for transform_entry in std::fs::read_dir(&clean).expect("list clean set").flatten() {
        if !transform_entry.path().is_dir() {
            continue;
        }
        let transform = transform_entry.file_name().to_string_lossy().into_owned();
        for file in std::fs::read_dir(transform_entry.path())
            .expect("list transform")
            .flatten()
        {
            let path = file.path();
            let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
            let Some(image) = load(&path) else {
                eprintln!("undecodable: {}", path.display());
                continue;
            };
            let Some(hash) = pdq_hash(&image) else {
                eprintln!("unhashable: {}", path.display());
                continue;
            };
            hashes.insert((transform.clone(), stem), hash);
        }
        eprintln!("hashed: {transform}");
    }

    let transforms: Vec<String> = {
        let mut t: Vec<String> = hashes.keys().map(|(t, _)| t.clone()).collect();
        t.dedup();
        t.retain(|t| t != "orig");
        t
    };

    // (a) + (c): same content, orig vs each transform.
    println!("\n### PDQ: same content across transformations\n");
    println!("| transform | pairs | min | median | max | ≤24 | ≤31 | ≤47 | ≤63 | word-nominated |");
    println!("|---|---|---|---|---|---|---|---|---|---|");
    for transform in &transforms {
        let mut dists: Vec<u32> = Vec::new();
        let mut nominated = 0u32;
        for ((t, stem), hash) in &hashes {
            if t != transform {
                continue;
            }
            let Some(orig) = hashes.get(&("orig".to_string(), stem.clone())) else {
                continue;
            };
            dists.push(hamming(orig, hash));
            let (wa, wb) = (words(orig), words(hash));
            if wa.iter().zip(wb.iter()).any(|(a, b)| a == b) {
                nominated += 1;
            }
        }
        dists.sort_unstable();
        let n = dists.len();
        let counts: Vec<String> = THRESHOLDS
            .iter()
            .map(|t| format!("{}", dists.iter().filter(|d| **d <= *t).count()))
            .collect();
        println!(
            "| {transform} | {n} | {} | {} | {} | {} | {}/{n} |",
            dists.first().unwrap(),
            dists[n / 2],
            dists.last().unwrap(),
            counts.join(" | "),
            nominated
        );
    }

    // (b): different content, all pairs across every transform variant.
    let all: Vec<(&String, &PdqHash)> = hashes.iter().map(|((_, s), h)| (s, h)).collect();
    let mut cross_pairs = 0u64;
    let mut min_cross = u32::MAX;
    let mut cross_counts = [0u64; THRESHOLDS.len()];
    for i in 0..all.len() {
        for j in (i + 1)..all.len() {
            if all[i].0 == all[j].0 {
                continue; // same content, different transform — not a false-match case
            }
            let d = hamming(all[i].1, all[j].1);
            cross_pairs += 1;
            min_cross = min_cross.min(d);
            for (idx, t) in THRESHOLDS.iter().enumerate() {
                if d <= *t {
                    cross_counts[idx] += 1;
                }
            }
        }
    }
    println!("\n### PDQ: different content (false-match risk)\n");
    println!("| pairs | min distance | ≤24 | ≤31 | ≤47 | ≤63 |");
    println!("|---|---|---|---|---|---|");
    println!(
        "| {cross_pairs} | {min_cross} | {} | {} | {} | {} |",
        cross_counts[0], cross_counts[1], cross_counts[2], cross_counts[3]
    );
}

fn load(path: &Path) -> Option<DecodedImage> {
    let bytes = std::fs::read(path).ok()?;
    let decoded = image::load_from_memory(&bytes).ok()?.to_rgb8();
    let (width, height) = (decoded.width() as usize, decoded.height() as usize);
    Some(DecodedImage {
        rgb: decoded.into_raw(),
        width,
        height,
    })
}
