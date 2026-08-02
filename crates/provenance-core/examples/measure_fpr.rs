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
//!
//! With `--model <bzh.onnx>` (needs `--features stable-signature`) the
//! IMATAG bzh classifier is measured over the same corpus alongside the
//! DWT detector; hits are reported per detector.

use std::path::Path;

use provenance_core::layers::sd_dwt::SdInvisibleWatermark;
use provenance_core::layers::watermark::{DecodedImage, WatermarkDetector};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut dir = None;
    let mut model = None;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--model" => model = it.next().cloned(),
            other => dir = Some(other.to_string()),
        }
    }
    let Some(dir) = dir else {
        eprintln!("usage: measure_fpr <directory of clean images> [--model <bzh.onnx>]");
        std::process::exit(2);
    };
    let mut detectors: Vec<Box<dyn WatermarkDetector>> = vec![Box::new(SdInvisibleWatermark)];
    #[cfg(feature = "stable-signature")]
    if let Some(model_path) = &model {
        use provenance_core::layers::stable_signature::StableSignatureBzh;
        match StableSignatureBzh::from_onnx_path(Path::new(model_path)) {
            Ok(det) => detectors.push(Box::new(det)),
            Err(err) => {
                eprintln!("measure_fpr: cannot load model {model_path}: {err}");
                std::process::exit(2);
            }
        }
    }
    #[cfg(not(feature = "stable-signature"))]
    if model.is_some() {
        eprintln!("measure_fpr: --model needs --features stable-signature");
        std::process::exit(2);
    }
    let (mut examined, mut skipped) = (0u64, 0u64);
    let mut hits_per: Vec<u64> = vec![0; detectors.len()];

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
                    for (idx, detector) in detectors.iter().enumerate() {
                        if let Some(hit) = detector.probe(&image) {
                            hits_per[idx] += 1;
                            println!("FALSE POSITIVE  {}  ({})", path.display(), hit.source);
                        }
                    }
                }
                None => skipped += 1,
            }
        }
    }

    println!("examined {examined} images, skipped {skipped} (undecodable/too small)");
    for (idx, detector) in detectors.iter().enumerate() {
        let hits = hits_per[idx];
        print!("{}: {hits} false positives", detector.vendor());
        if examined > 0 {
            print!(" ({:.4}%)", hits as f64 / examined as f64 * 100.0);
        }
        println!();
    }
    if examined < 2000 {
        println!("note: the gate wants >= 2000 clean images; this corpus has fewer.");
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
