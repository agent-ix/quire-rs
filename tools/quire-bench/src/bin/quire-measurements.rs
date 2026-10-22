//! Export a governed Quire benchmark measurement collection.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use quire_bench::collection::read_json;
use quire_bench::export::{export, within_root};
use quire_bench::{BenchError, Manifest};

struct Options {
    consumer: PathBuf,
    module: Option<PathBuf>,
    output: PathBuf,
    verification_stack: PathBuf,
}

fn options() -> Result<Options, BenchError> {
    let mut args = env::args().skip(1);
    let mut consumer = PathBuf::from("../quire-cli");
    let mut module = None;
    let mut output = None;
    let mut verification_stack = None;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--consumer" => {
                consumer =
                    PathBuf::from(args.next().ok_or_else(|| {
                        BenchError::Input("--consumer requires a path".to_owned())
                    })?);
            }
            "--module" => {
                let value = args
                    .next()
                    .ok_or_else(|| BenchError::Input("--module requires a path".to_owned()))?;
                module = if value.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(value))
                };
            }
            "--output" => {
                output = Some(PathBuf::from(args.next().ok_or_else(|| {
                    BenchError::Input("--output requires a path".to_owned())
                })?));
            }
            "--verification-stack" => {
                verification_stack = Some(PathBuf::from(args.next().ok_or_else(|| {
                    BenchError::Input("--verification-stack requires a path".to_owned())
                })?));
            }
            "-h" | "--help" => {
                println!("usage: quire-measurements [--consumer DIR] [--module DIR] --output PATH --verification-stack PATH");
                std::process::exit(0);
            }
            _ => return Err(BenchError::Input(format!("unknown argument {argument}"))),
        }
    }
    Ok(Options {
        consumer,
        module,
        output: output.ok_or_else(|| BenchError::Input("--output is required".to_owned()))?,
        verification_stack: verification_stack
            .ok_or_else(|| BenchError::Input("--verification-stack is required".to_owned()))?,
    })
}

fn run() -> Result<(), BenchError> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|source| BenchError::Io {
            context: "finding quire-rs root".to_owned(),
            source,
        })?;
    let options = options()?;
    let manifest: Manifest = read_json(&root.join("bench/manifest.json"))?;
    let consumer = within_root(&root, &options.consumer)
        .canonicalize()
        .map_err(|source| BenchError::Io {
            context: "resolving consumer workspace".to_owned(),
            source,
        })?;
    let output = within_root(&root, &options.output);
    let stack = within_root(&root, &options.verification_stack);
    let collection = export(
        &root,
        &manifest,
        &consumer,
        options.module.as_deref(),
        &stack,
    )?;
    let mut bytes = serde_json::to_vec_pretty(&collection).map_err(|source| BenchError::Json {
        context: "encoding measurement collection".to_owned(),
        source,
    })?;
    bytes.push(b'\n');
    fs::write(&output, bytes).map_err(|source| BenchError::Io {
        context: format!("writing {}", output.display()),
        source,
    })?;
    println!("{}", output.display());
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("measurement export: {error}");
            ExitCode::FAILURE
        }
    }
}
