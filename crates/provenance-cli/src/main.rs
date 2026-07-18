//! `lens` — the CLI front end. All I/O lives here; the core never touches
//! the filesystem.
//!
//! Exit codes are part of the contract (scripts depend on them):
//!   0 verified, 10 indicated, 20 inconclusive, 30 tampered, 2 usage/IO error.
//! With multiple files the exit code is the highest per-file code, so the
//! worst result wins (a Tampered anywhere outranks everything, mirroring the
//! pipeline's own precedence rule).

use std::process::ExitCode;

use provenance_core::{render_json, Asset, LayerFinding, Pipeline, Verdict};

const USAGE: &str = "\
lens — honest provenance verdicts for media files

USAGE:
    lens verify [--json] [--trust-anchors <PEM>] <FILE>...
                          examine each file and print a verdict report;
                          --json prints one JSON object per line (same shape
                          the WASM engine returns, plus a \"file\" key);
                          --trust-anchors loads a PEM bundle of root
                          certificates that signatures may chain to
                          (without it, no chain can validate as trusted).
                          With multiple files the exit code is the highest
                          per-file code — the worst result wins.
    lens tiers            print the four verdict tiers and their meaning

EXIT CODES:
    0 verified    10 indicated    20 inconclusive    30 tampered    2 error
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("verify") => match VerifyArgs::parse(&args[2..]) {
            Some(parsed) => verify_all(&parsed),
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

struct VerifyArgs<'a> {
    json: bool,
    anchors_path: Option<&'a str>,
    files: Vec<&'a str>,
}

impl<'a> VerifyArgs<'a> {
    /// `verify` accepts: [--json] [--trust-anchors <PEM>] <FILE>...
    /// Flags may appear in any order before or between files; anything else
    /// starting with `--` is a usage error.
    fn parse(rest: &'a [String]) -> Option<Self> {
        let mut json = false;
        let mut anchors_path = None;
        let mut files = Vec::new();
        let mut it = rest.iter();
        while let Some(arg) = it.next() {
            match arg.as_str() {
                "--json" => json = true,
                "--trust-anchors" => anchors_path = Some(it.next()?.as_str()),
                flag if flag.starts_with("--") => return None,
                file => files.push(file),
            }
        }
        if files.is_empty() {
            return None;
        }
        Some(VerifyArgs {
            json,
            anchors_path,
            files,
        })
    }
}

fn verify_all(args: &VerifyArgs) -> ExitCode {
    let pipeline = match args.anchors_path {
        Some(anchors_path) => match std::fs::read_to_string(anchors_path) {
            Ok(pem) => Pipeline::with_trust_anchors(pem),
            Err(err) => {
                eprintln!("lens: cannot read trust anchors {anchors_path}: {err}");
                return ExitCode::from(2);
            }
        },
        None => Pipeline::standard(),
    };

    let mut worst = 0u8;
    for path in &args.files {
        let code = match std::fs::read(path) {
            Ok(bytes) => {
                let report = pipeline.examine(&Asset {
                    bytes: &bytes,
                    media_type: guess_media_type(path),
                });
                if args.json {
                    println!("{}", render_json(&report, Some(path)));
                } else {
                    println!("{path}");
                    println!("  verdict: {}", report.verdict);
                    for (layer, finding) in &report.findings {
                        println!("  [{layer}] {}", describe(finding));
                    }
                    // Present only on Verified reports; descriptive of what
                    // the credential claims, never an endorsement.
                    if let Some(summary) = &report.credentials {
                        println!("  credential claims:");
                        if let Some(generator) = &summary.claim_generator {
                            println!("    claim generator: {generator}");
                        }
                        if let Some(time) = &summary.signing_time {
                            println!("    signed at: {time}");
                        }
                        if let Some(source_type) = &summary.digital_source_type {
                            println!("    declared source type: {source_type}");
                        }
                        if let Some(note) = summary.source_type_note {
                            println!("    note: {note}");
                        }
                    }
                }
                exit_code(&report.verdict)
            }
            Err(err) => {
                eprintln!("lens: cannot read {path}: {err}");
                2
            }
        };
        worst = worst.max(code);
    }
    ExitCode::from(worst)
}

fn exit_code(verdict: &Verdict) -> u8 {
    match verdict {
        Verdict::Verified => 0,
        Verdict::Indicated => 10,
        Verdict::Inconclusive => 20,
        Verdict::Tampered => 30,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(args: &[&str]) -> Vec<String> {
        args.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_accepts_flags_in_any_order_and_many_files() {
        let args = strings(&["--json", "a.jpg", "--trust-anchors", "ca.pem", "b.jpg"]);
        let parsed = VerifyArgs::parse(&args).expect("valid invocation");
        assert!(parsed.json);
        assert_eq!(parsed.anchors_path, Some("ca.pem"));
        assert_eq!(parsed.files, vec!["a.jpg", "b.jpg"]);
    }

    #[test]
    fn parse_rejects_missing_files_unknown_flags_and_dangling_value() {
        assert!(VerifyArgs::parse(&strings(&["--json"])).is_none());
        assert!(VerifyArgs::parse(&strings(&["--jsn", "a.jpg"])).is_none());
        assert!(VerifyArgs::parse(&strings(&["a.jpg", "--trust-anchors"])).is_none());
        assert!(VerifyArgs::parse(&[]).is_none());
    }
}
