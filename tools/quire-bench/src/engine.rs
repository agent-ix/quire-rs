//! Build the consumer engine and run its bounded coverage command.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::BenchError;

/// Captured output from a bounded subprocess.
pub struct ProcessOutput {
    /// Exit status.
    pub status: ExitStatus,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
}

/// Run a command with a wall-clock ceiling and captured output.
pub fn run_bounded(command: &mut Command, ceiling: Duration) -> Result<ProcessOutput, BenchError> {
    let stdout = tempfile::NamedTempFile::new().map_err(|source| BenchError::Io {
        context: "creating stdout capture".to_owned(),
        source,
    })?;
    let stderr = tempfile::NamedTempFile::new().map_err(|source| BenchError::Io {
        context: "creating stderr capture".to_owned(),
        source,
    })?;
    command.stdout(Stdio::from(stdout.reopen().map_err(|source| {
        BenchError::Io {
            context: "opening stdout capture".to_owned(),
            source,
        }
    })?));
    command.stderr(Stdio::from(stderr.reopen().map_err(|source| {
        BenchError::Io {
            context: "opening stderr capture".to_owned(),
            source,
        }
    })?));
    let mut child = command.spawn().map_err(|source| BenchError::Io {
        context: "starting subprocess".to_owned(),
        source,
    })?;
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|source| BenchError::Io {
            context: "waiting for subprocess".to_owned(),
            source,
        })? {
            break status;
        }
        if start.elapsed() >= ceiling {
            let _ = child.kill();
            let _ = child.wait();
            return Err(BenchError::Process(format!(
                "subprocess exceeded {} seconds",
                ceiling.as_secs()
            )));
        }
        thread::sleep(Duration::from_millis(100));
    };
    Ok(ProcessOutput {
        status,
        stdout: fs::read_to_string(stdout.path()).map_err(|source| BenchError::Io {
            context: "reading stdout capture".to_owned(),
            source,
        })?,
        stderr: fs::read_to_string(stderr.path()).map_err(|source| BenchError::Io {
            context: "reading stderr capture".to_owned(),
            source,
        })?,
    })
}

/// Build `quire` from the consumer workspace and return Cargo's executable path.
pub fn build_engine(consumer: &Path, release: bool) -> Result<PathBuf, BenchError> {
    let manifest = consumer.join("Cargo.toml");
    if !manifest.is_file() {
        return Err(BenchError::Input(format!(
            "no consumer workspace at {}",
            consumer.display()
        )));
    }
    eprintln!("building the engine from {} …", consumer.display());
    let mut command = Command::new("cargo");
    command.arg("build").arg("--locked");
    if release {
        command.arg("--release");
    }
    command
        .arg("--manifest-path")
        .arg(manifest)
        .arg("--message-format")
        .arg("json");
    let output = run_bounded(&mut command, Duration::from_secs(1800))?;
    if !output.status.success() {
        let detail = if output.stderr.trim().is_empty() {
            &output.stdout
        } else {
            &output.stderr
        };
        let tail: String = detail
            .chars()
            .rev()
            .take(600)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        return Err(BenchError::Process(format!(
            "building {} failed:\n{tail}",
            consumer.display()
        )));
    }
    for line in output.stdout.lines() {
        let Ok(message) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if message.get("reason").and_then(Value::as_str) == Some("compiler-artifact")
            && message.pointer("/target/name").and_then(Value::as_str) == Some("quire")
        {
            if let Some(path) = message.get("executable").and_then(Value::as_str) {
                return Ok(PathBuf::from(path));
            }
        }
    }
    Err(BenchError::Process(format!(
        "cargo built {} but emitted no `quire` executable",
        consumer.display()
    )))
}
