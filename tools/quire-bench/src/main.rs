//! Score the Quire corpus benchmark and optionally ratchet its baselines.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use quire_bench::collection::{collect_attested, read_json};
use quire_bench::engine::build_engine;
use quire_bench::export::{git_remote, load_verification_stack};
use quire_bench::{render, score, BenchError, Manifest, Observed, Verdict};

struct Options {
    update: bool,
    observed: Option<PathBuf>,
    consumer: PathBuf,
    module: Option<PathBuf>,
    verification_stack: Option<PathBuf>,
}

fn options() -> Result<Options, BenchError> {
    let mut args = env::args().skip(1);
    let mut result = Options {
        update: false,
        observed: None,
        consumer: PathBuf::from("../quire-cli"),
        module: None,
        verification_stack: None,
    };
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--update" => result.update = true,
            "--observed" => {
                result.observed =
                    Some(PathBuf::from(args.next().ok_or_else(|| {
                        BenchError::Input("--observed requires a path".to_owned())
                    })?))
            }
            "--consumer" => {
                result.consumer =
                    PathBuf::from(args.next().ok_or_else(|| {
                        BenchError::Input("--consumer requires a path".to_owned())
                    })?)
            }
            "--module" => {
                let value = args
                    .next()
                    .ok_or_else(|| BenchError::Input("--module requires a path".to_owned()))?;
                result.module = if value.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(value))
                };
            }
            "--verification-stack" => {
                result.verification_stack = Some(PathBuf::from(args.next().ok_or_else(|| {
                    BenchError::Input("--verification-stack requires a path".to_owned())
                })?))
            }
            "-h" | "--help" => {
                println!("usage: quire-bench [--update] [--observed PATH] [--consumer DIR] [--module DIR] --verification-stack PATH");
                std::process::exit(0);
            }
            _ => return Err(BenchError::Input(format!("unknown argument {argument}"))),
        }
    }
    Ok(result)
}

fn within_root(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn run() -> Result<ExitCode, BenchError> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|source| BenchError::Io {
            context: "finding quire-rs root".to_owned(),
            source,
        })?;
    let options = options()?;
    let manifest: Manifest = read_json(&root.join("bench/manifest.json"))?;
    let baseline_path = root.join("bench/baselines.json");
    let baselines: Observed = if baseline_path.exists() {
        read_json(&baseline_path)?
    } else {
        Observed::new()
    };
    let observed: Observed = if let Some(path) = options.observed {
        read_json(&path)?
    } else {
        // FR-050-AC-52: every require_clean corpus input is checked against
        // its attested revision before it is scored.
        let stack_path = options.verification_stack.as_deref().ok_or_else(|| {
            BenchError::Input("--verification-stack is required to collect".to_owned())
        })?;
        let stack = load_verification_stack(
            &within_root(&root, stack_path),
            "quire",
            &git_remote(&root)?,
        )?;
        let consumer = within_root(&root, &options.consumer);
        let quire = build_engine(&consumer, true)?;
        collect_attested(
            &root,
            &manifest,
            &quire,
            options.module.as_deref(),
            false,
            &stack,
        )?
        .observed
    };
    let report = score(&manifest, &observed, &baselines)?;
    let (text, failed) = render(&report);
    println!(
        "{}",
        if text.is_empty() {
            "no metrics collected"
        } else {
            &text
        }
    );
    if options.update {
        let mut next = Observed::new();
        for row in report.rows {
            next.entry(row.corpus)
                .or_default()
                .insert(row.metric, row.next_baseline);
        }
        let mut bytes = serde_json::to_vec_pretty(&next).map_err(|source| BenchError::Json {
            context: "encoding baselines".to_owned(),
            source,
        })?;
        bytes.push(b'\n');
        fs::write(&baseline_path, bytes).map_err(|source| BenchError::Io {
            context: format!("writing {}", baseline_path.display()),
            source,
        })?;
        println!("\nbaselines rewritten: bench/baselines.json");
        return Ok(ExitCode::SUCCESS);
    }
    let improved = report
        .rows
        .iter()
        .filter(|row| row.verdict == Verdict::Improved)
        .count();
    if improved > 0 {
        println!("\n{improved} metric(s) improved. `make bench-update` to tighten the baselines; the diff belongs in the PR.");
    }
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("bench: {error}");
            ExitCode::from(2)
        }
    }
}
