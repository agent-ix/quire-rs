//! Check Criterion's mean change estimates against a regression band.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::BenchError;

#[derive(Deserialize)]
struct Estimates {
    mean: Mean,
}

#[derive(Deserialize)]
struct Mean {
    point_estimate: f64,
}

/// Human report and gate status for one Criterion output tree.
pub struct PerfReport {
    /// Lines printed by the gate.
    pub lines: Vec<String>,
    /// Whether any readable estimate exceeded the band.
    pub failed: bool,
    /// Number of readable estimates.
    pub seen: usize,
}

fn estimates_files(root: &Path) -> Result<Vec<PathBuf>, BenchError> {
    let mut directories = vec![root.to_path_buf()];
    let mut found = Vec::new();
    while let Some(directory) = directories.pop() {
        let entries = fs::read_dir(&directory).map_err(|source| BenchError::Io {
            context: format!("reading {}", directory.display()),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| BenchError::Io {
                context: format!("reading {}", directory.display()),
                source,
            })?;
            let kind = entry.file_type().map_err(|source| BenchError::Io {
                context: format!("reading type of {}", entry.path().display()),
                source,
            })?;
            if kind.is_dir() {
                directories.push(entry.path());
            } else if kind.is_file()
                && entry.file_name() == "estimates.json"
                && entry
                    .path()
                    .parent()
                    .and_then(Path::file_name)
                    .is_some_and(|name| name == "change")
            {
                found.push(entry.path());
            }
        }
    }
    found.sort();
    Ok(found)
}

/// Scan Criterion change estimates, skipping unreadable files loudly.
pub fn check(root: &Path, band: f64) -> Result<PerfReport, BenchError> {
    if !band.is_finite() || band < 0.0 {
        return Err(BenchError::Input(
            "regression band must be a finite nonnegative number".to_owned(),
        ));
    }
    if !root.is_dir() {
        return Ok(PerfReport {
            lines: vec![
                "check_perf_regression: no criterion output found; first run, skipping.".to_owned(),
            ],
            failed: false,
            seen: 0,
        });
    }
    let mut lines = Vec::new();
    let mut seen = 0;
    let mut failed = false;
    for path in estimates_files(root)? {
        let rel = path
            .parent()
            .and_then(Path::parent)
            .and_then(|parent| parent.strip_prefix(root).ok())
            .ok_or_else(|| {
                BenchError::Input(format!("invalid Criterion path {}", path.display()))
            })?;
        let estimate = fs::read(&path)
            .map_err(|source| source.to_string())
            .and_then(|bytes| {
                serde_json::from_slice::<Estimates>(&bytes).map_err(|source| source.to_string())
            });
        let change = match estimate {
            Ok(value) if value.mean.point_estimate.is_finite() => value.mean.point_estimate,
            Ok(_) => {
                lines.push(format!(
                    "  skip {} (unreadable: non-finite mean)",
                    rel.display()
                ));
                continue;
            }
            Err(reason) => {
                lines.push(format!("  skip {} (unreadable: {reason})", rel.display()));
                continue;
            }
        };
        seen += 1;
        if change > band {
            failed = true;
            lines.push(format!(
                "::error::regression in {}: +{:.1}% (band: +{:.0}%)",
                rel.display(),
                change * 100.0,
                band * 100.0
            ));
        } else if change < -band {
            lines.push(format!(
                "  speedup {}: {:+.1}%",
                rel.display(),
                change * 100.0
            ));
        } else {
            lines.push(format!(
                "  ok      {}: {:+.1}%",
                rel.display(),
                change * 100.0
            ));
        }
    }
    if seen == 0 {
        lines.push("check_perf_regression: no change/ files found (first run with --save-baseline?), skipping.".to_owned());
    } else if failed {
        lines.push(format!(
            "check_perf_regression: regression band {:.0}% exceeded by one or more benches.",
            band * 100.0
        ));
    } else {
        lines.push("check_perf_regression: OK".to_owned());
    }
    Ok(PerfReport {
        lines,
        failed,
        seen,
    })
}
