//! Enforce the Criterion change-estimate regression band.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use quire_bench::perf::check;
use quire_bench::BenchError;

fn run() -> Result<ExitCode, BenchError> {
    if env::args_os().len() != 1 {
        return Err(BenchError::Input(
            "usage: quire-perf-gate (set QUIRE_PERF_BAND to override 0.10)".to_owned(),
        ));
    }
    let band = env::var("QUIRE_PERF_BAND")
        .unwrap_or_else(|_| "0.10".to_owned())
        .parse::<f64>()
        .map_err(|_| BenchError::Input("QUIRE_PERF_BAND must be a number".to_owned()))?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("target/criterion");
    let report = check(&root, band)?;
    for line in report.lines {
        println!("{line}");
    }
    Ok(if report.failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("check_perf_regression: {error}");
            ExitCode::from(2)
        }
    }
}
