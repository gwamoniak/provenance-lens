//! Calibration measurement for the watermark detectors (roadmap plan, W2):
//! runs the EXACT production probes over the labeled corpus that
//! `scripts/gen_calibration_corpus.py` generates, and prints per-set,
//! per-transformation hit rates as a markdown table for docs/CALIBRATION.md.
//!
//! Corpus layout: `<root>/<set>/<transform>/<image>` with sets `clean`,
//! `sd_dwt` (SDXL-payload DWT watermark), `bzh` (IMATAG bzh VAE roundtrip).
//! Hits on a detector's own set are true positives; hits on `clean` (or the
//! other scheme's set) are false positives.
//!
//!     cargo run --release -p provenance-core --features stable-signature \
//!         --example calibrate -- <corpus-root> --model <bzh.onnx>

use std::collections::BTreeMap;
use std::path::Path;

use provenance_core::layers::sd_dwt::SdInvisibleWatermark;
use provenance_core::layers::watermark::{DecodedImage, WatermarkDetector};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut root = None;
    let mut model = None;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--model" => model = it.next().cloned(),
            other => root = Some(other.to_string()),
        }
    }
    let Some(root) = root else {
        eprintln!("usage: calibrate <corpus-root> [--model <bzh.onnx>]");
        std::process::exit(2);
    };

    let mut detectors: Vec<Box<dyn WatermarkDetector>> = vec![Box::new(SdInvisibleWatermark)];
    #[cfg(feature = "stable-signature")]
    if let Some(model_path) = &model {
        use provenance_core::layers::stable_signature::StableSignatureBzh;
        match StableSignatureBzh::from_onnx_path(Path::new(model_path)) {
            Ok(det) => detectors.push(Box::new(det)),
            Err(err) => {
                eprintln!("calibrate: cannot load model {model_path}: {err}");
                std::process::exit(2);
            }
        }
    }
    #[cfg(not(feature = "stable-signature"))]
    if model.is_some() {
        eprintln!("calibrate: --model needs --features stable-signature");
        std::process::exit(2);
    }

    // results[(set, transform, vendor)] = (hits, total)
    let mut results: BTreeMap<(String, String, String), (u32, u32)> = BTreeMap::new();
    let root = Path::new(&root);
    for set in list_dirs(root) {
        for transform in list_dirs(&root.join(&set)) {
            let dir = root.join(&set).join(&transform);
            for entry in std::fs::read_dir(&dir)
                .expect("read transform dir")
                .flatten()
            {
                let path = entry.path();
                let Some(image) = load_rgb(&path) else {
                    eprintln!("undecodable: {}", path.display());
                    continue;
                };
                for detector in &detectors {
                    let key = (
                        set.clone(),
                        transform.clone(),
                        detector.vendor().to_string(),
                    );
                    let counter = results.entry(key).or_insert((0, 0));
                    counter.1 += 1;
                    if detector.probe(&image).is_some() {
                        counter.0 += 1;
                    }
                }
            }
            eprintln!("done: {set}/{transform}");
        }
    }

    // Markdown: one table per detector, rows = transform, cols = sets.
    let vendors: Vec<String> = detectors.iter().map(|d| d.vendor().to_string()).collect();
    let sets: Vec<String> = list_dirs(root);
    let transforms = [
        "orig",
        "jpeg90",
        "jpeg70",
        "jpeg50",
        "resize75",
        "resize50",
        "crop80",
        "screenshot",
    ];
    for vendor in &vendors {
        println!("\n### {vendor}\n");
        print!("| transform |");
        for set in &sets {
            print!(" {set} (hits/total) |");
        }
        println!();
        print!("|---|");
        for _ in &sets {
            print!("---|");
        }
        println!();
        for transform in transforms {
            print!("| {transform} |");
            for set in &sets {
                match results.get(&(set.clone(), transform.to_string(), vendor.clone())) {
                    Some((hits, total)) => {
                        print!(
                            " {hits}/{total} ({:.1}%) |",
                            *hits as f64 / *total as f64 * 100.0
                        )
                    }
                    None => print!(" — |"),
                }
            }
            println!();
        }
    }
}

fn list_dirs(path: &Path) -> Vec<String> {
    let mut dirs: Vec<String> = std::fs::read_dir(path)
        .unwrap_or_else(|err| panic!("cannot list {}: {err}", path.display()))
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    dirs.sort();
    dirs
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
