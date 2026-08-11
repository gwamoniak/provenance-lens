//! Developer-only inspection helper for formats the production CLI does not
//! yet infer from a filename. This keeps C0 compatibility probes on the exact
//! same `Pipeline` and JSON renderer as shipped surfaces without changing CLI
//! behavior.

use std::{env, error::Error, fs};

use provenance_core::{render_json, Asset, Pipeline};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let path = args
        .next()
        .ok_or("usage: inspect_with_mime <FILE> <MIME> [TRUST_ANCHORS_PEM]")?;
    let media_type = args
        .next()
        .ok_or("usage: inspect_with_mime <FILE> <MIME> [TRUST_ANCHORS_PEM]")?;
    let anchors = args.next().map(fs::read_to_string).transpose()?;
    if args.next().is_some() {
        return Err("too many arguments".into());
    }

    let bytes = fs::read(&path)?;
    let pipeline = match anchors {
        Some(pem) => Pipeline::with_trust_anchors(pem),
        None => Pipeline::standard(),
    };
    let report = pipeline.examine(&Asset {
        bytes: &bytes,
        media_type: Some(&media_type),
    });

    println!("{}", render_json(&report, Some(&path)));
    Ok(())
}
