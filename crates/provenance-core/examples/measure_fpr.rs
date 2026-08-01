//! False-positive-rate measurement for the watermark detectors (roadmap
//! plan, W1 gate: measured FPR ≤ 0.1% on ≥ 2,000 clean images before W2).
//!
//! Point it at a directory of images KNOWN to carry no Stable Diffusion
//! watermark (personal photos, a stock-photo dump, …); it runs the exact
//! production probe on every .jpg/.jpeg/.png and reports the hit rate.
//! Every hit on a clean corpus is a false positive. The corpus is NOT part
//! of the repo (bare-machine rule: repo tests use only the committed
//! vectors); record the measured number in the roadmap plan.
//!
//!     cargo run --release -p provenance-core --example measure_fpr -- <dir>

use std::path::Path;

use provenance_core::layers::sd_dwt::SdInvisibleWatermark;
use provenance_core::layers::watermark::{DecodedImage, WatermarkDetector};

fn main() {
    let dir = match std::env::args().nth(1) {
        Some(dir) => dir,
        None => {
            eprintln!("usage: measure_fpr <directory of clean images>");
            std::process::exit(2);
        }
    };
    let detector = SdInvisibleWatermark;
    let (mut examined, mut skipped, mut hits) = (0u64, 0u64, 0u64);

    let mut stack = vec![std::path::PathBuf::from(&dir)];
    while let Some(current) = stack.pop() {
        let entries = match std::fs::read_dir(&current) {
            Ok(entries) => entries,
            Err(err) => {
                eprintln!("skipping {}: {err}", current.display());
                continue;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if !matches!(
                path.extension()
                    .and_then(|e| e.to_str())
                    .map(str::to_ascii_lowercase)
                    .as_deref(),
                Some("jpg" | "jpeg" | "png")
            ) {
                continue;
            }
            match load_rgb(&path) {
                Some(image) => {
                    examined += 1;
                    if let Some(hit) = detector.probe(&image) {
                        hits += 1;
                        println!("FALSE POSITIVE  {}  ({})", path.display(), hit.source);
                    }
                }
                None => skipped += 1,
            }
        }
    }

    println!(
        "examined {examined} images, skipped {skipped} (undecodable/too small), {hits} false positives"
    );
    if examined > 0 {
        println!(
            "false-positive rate: {:.4}%",
            hits as f64 / examined as f64 * 100.0
        );
    }
    if examined < 2000 {
        println!("note: the W1 gate wants >= 2000 clean images; this corpus has fewer.");
    }
}

fn load_rgb(path: &Path) -> Option<DecodedImage> {
    let bytes = std::fs::read(path).ok()?;
    let decoded = image::load_from_memory(&bytes).ok()?.to_rgb8();
    let (width, height) = (decoded.width() as usize, decoded.height() as usize);
    Some(DecodedImage {
        rgb: decoded.into_raw(),
        width,
        height,
    })
}
