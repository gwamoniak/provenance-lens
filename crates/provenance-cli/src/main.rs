//! `lens` — the CLI front end. All I/O lives here; the core never touches
//! the filesystem.
//!
//! Exit codes are part of the contract (scripts depend on them):
//!   0 verified, 10 indicated, 20 inconclusive, 30 tampered, 2 usage/IO error.

use std::process::ExitCode;

use provenance_core::{Asset, LayerFinding, Pipeline, Verdict};

const USAGE: &str = "\
lens — honest provenance verdicts for media files

USAGE:
    lens verify [--trust-anchors <PEM>] <FILE>
                          examine a file and print a verdict report;
                          --trust-anchors loads a PEM bundle of root
                          certificates that signatures may chain to
                          (without it, no chain can validate as trusted)
    lens tiers            print the four verdict tiers and their meaning

EXIT CODES:
    0 verified    10 indicated    20 inconclusive    30 tampered    2 error
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("verify") => match parse_verify_args(&args[2..]) {
            Some((anchors_path, file)) => verify(file, anchors_path),
            None => {
                eprint!("{USAGE}");
                ExitCode::from(2)
            }
        },
        Some("tiers") if args.len() == 2 => {
            tiers();
            ExitCode::SUCCESS
        }
        _ => {
            eprint!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// `verify` accepts exactly: [--trust-anchors <PEM>] <FILE>.
fn parse_verify_args(rest: &[String]) -> Option<(Option<&String>, &String)> {
    match rest {
        [file] => Some((None, file)),
        [flag, pem, file] if flag == "--trust-anchors" => Some((Some(pem), file)),
        _ => None,
    }
}

fn verify(path: &str, anchors_path: Option<&String>) -> ExitCode {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) => {
            eprintln!("lens: cannot read {path}: {err}");
            return ExitCode::from(2);
        }
    };

    let pipeline = match anchors_path {
        Some(anchors_path) => match std::fs::read_to_string(anchors_path) {
            Ok(pem) => Pipeline::with_trust_anchors(pem),
            Err(err) => {
                eprintln!("lens: cannot read trust anchors {anchors_path}: {err}");
                return ExitCode::from(2);
            }
        },
        None => Pipeline::standard(),
    };

    let asset = Asset {
        bytes: &bytes,
        media_type: guess_media_type(path),
    };
    let report = pipeline.examine(&asset);

    println!("{path}");
    println!("  verdict: {}", report.verdict);
    for (layer, finding) in &report.findings {
        println!("  [{layer}] {}", describe(finding));
    }

    match report.verdict {
        Verdict::Verified => ExitCode::SUCCESS,
        Verdict::Indicated => ExitCode::from(10),
        Verdict::Inconclusive => ExitCode::from(20),
        Verdict::Tampered => ExitCode::from(30),
    }
}

fn tiers() {
    for verdict in [
        Verdict::Verified,
        Verdict::Indicated,
        Verdict::Inconclusive,
        Verdict::Tampered,
    ] {
        println!("{:<13} {}", verdict.id(), verdict.approved_phrase());
    }
}

fn describe(finding: &LayerFinding) -> String {
    match finding {
        LayerFinding::NotEvaluated { reason } => format!("not evaluated — {reason}"),
        LayerFinding::NoSignal => "ran, no signal".to_string(),
        LayerFinding::Proof { issuer } => format!("valid provenance chain, issuer: {issuer}"),
        LayerFinding::Indication { source } => format!("indication from {source}"),
        LayerFinding::TamperEvidence { detail } => format!("tamper evidence — {detail}"),
    }
}

fn guess_media_type(path: &str) -> Option<&'static str> {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        Some("image/jpeg")
    } else if lower.ends_with(".png") {
        Some("image/png")
    } else if lower.ends_with(".webp") {
        Some("image/webp")
    } else if lower.ends_with(".avif") {
        Some("image/avif")
    } else {
        None
    }
}
