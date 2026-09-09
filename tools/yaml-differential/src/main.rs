use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

const EXCLUDED_DIRS: &[&str] = &[".git", ".worktrees", "node_modules", "target", "dist"];

#[derive(Debug)]
struct SourceArg {
    kind: String,
    name: String,
    revision: String,
    root: PathBuf,
}

#[derive(Serialize)]
struct Report {
    schema: &'static str,
    producer: Producer,
    exclusions: &'static [&'static str],
    injected_difference: bool,
    sources: Vec<SourceReport>,
    focused_cases: Vec<InputResult>,
    population: Population,
    input_manifest_sha256: String,
    differences: Vec<Difference>,
}

#[derive(Serialize)]
struct Producer {
    package: &'static str,
    version: &'static str,
    rustc: String,
    cargo: String,
    comparator_sha256: String,
    comparator_files: Vec<FileDigest>,
    old_engine: &'static str,
    selected_engine: &'static str,
}

#[derive(Serialize)]
struct SourceReport {
    kind: String,
    name: String,
    revision: String,
    root: String,
    markdown_documents: usize,
    complete_frontmatter_blocks: usize,
    inputs: Vec<InputResult>,
}

#[derive(Serialize)]
struct FileDigest {
    path: String,
    sha256: String,
}

#[derive(Serialize)]
struct InputResult {
    identity: String,
    input_sha256: String,
    byte_length: usize,
    old: Outcome,
    selected: Outcome,
    equal: bool,
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum Outcome {
    Success {
        value: Value,
        canonical_json_sha256: String,
    },
    Failure,
}

#[derive(Serialize)]
struct Population {
    markdown_documents: usize,
    complete_frontmatter_blocks: usize,
    focused_cases: usize,
    compared_inputs: usize,
    differences: usize,
}

#[derive(Serialize)]
struct Difference {
    identity: String,
    reason: &'static str,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn command_version(program: &str) -> String {
    Command::new(program)
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .unwrap_or_else(|| "unavailable".to_owned())
}

fn comparator_identity() -> Result<(String, Vec<FileDigest>), String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut hasher = Sha256::new();
    let mut files = Vec::new();
    for relative in ["Cargo.toml", "Cargo.lock", "src/main.rs"] {
        let bytes = fs::read(root.join(relative))
            .map_err(|error| format!("read comparator {relative}: {error}"))?;
        hasher.update(relative.as_bytes());
        hasher.update([0]);
        hasher.update(&bytes);
        files.push(FileDigest {
            path: relative.to_owned(),
            sha256: sha256(&bytes),
        });
    }
    Ok((format!("{:x}", hasher.finalize()), files))
}

fn parse_args() -> Result<(Vec<SourceArg>, bool), String> {
    let mut args = env::args().skip(1);
    let mut sources = Vec::new();
    let mut injected = false;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--source" => {
                let kind = args.next().ok_or("--source requires kind")?;
                let name = args.next().ok_or("--source requires name")?;
                let revision = args.next().ok_or("--source requires revision")?;
                let root = PathBuf::from(args.next().ok_or("--source requires path")?);
                sources.push(SourceArg {
                    kind,
                    name,
                    revision,
                    root,
                });
            }
            "--inject-difference" => injected = true,
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    if sources.is_empty() {
        return Err("at least one --source is required".to_owned());
    }
    Ok((sources, injected))
}

fn walk_markdown(root: &Path) -> Result<Vec<PathBuf>, String> {
    fn visit(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
        let mut entries = fs::read_dir(dir)
            .map_err(|error| format!("read directory {}: {error}", dir.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("read directory entry {}: {error}", dir.display()))?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| format!("inspect {}: {error}", path.display()))?;
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                let name = entry.file_name();
                if !EXCLUDED_DIRS.iter().any(|excluded| name == *excluded) {
                    visit(&path, files)?;
                }
            } else if path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| matches!(extension, "md" | "markdown"))
            {
                files.push(path);
            }
        }
        Ok(())
    }

    let mut files = Vec::new();
    visit(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn complete_frontmatter(bytes: &[u8]) -> Option<&[u8]> {
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    let first_end = bytes.iter().position(|byte| *byte == b'\n')? + 1;
    if trim_line_ending(&bytes[..first_end]) != b"---" {
        return None;
    }
    let mut start = first_end;
    while start < bytes.len() {
        let end = bytes[start..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(bytes.len(), |offset| start + offset + 1);
        if trim_line_ending(&bytes[start..end]) == b"---" {
            return Some(&bytes[first_end..start]);
        }
        start = end;
    }
    None
}

fn trim_line_ending(line: &[u8]) -> &[u8] {
    let line = line.strip_suffix(b"\n").unwrap_or(line);
    line.strip_suffix(b"\r").unwrap_or(line)
}

fn old_outcome(input: &str) -> Outcome {
    match old_yaml::from_str::<Value>(input) {
        Ok(value) => success(value),
        Err(_) => Outcome::Failure,
    }
}

fn selected_outcome(input: &str) -> Outcome {
    match new_yaml::from_str::<Value>(input) {
        Ok(value) => success(value),
        Err(_) => Outcome::Failure,
    }
}

fn success(value: Value) -> Outcome {
    let encoded = serde_json::to_vec(&value).expect("serde_json::Value always serializes");
    Outcome::Success {
        value,
        canonical_json_sha256: sha256(&encoded),
    }
}

fn outcome_equal(left: &Outcome, right: &Outcome) -> bool {
    match (left, right) {
        (Outcome::Failure, Outcome::Failure) => true,
        (Outcome::Success { value: left, .. }, Outcome::Success { value: right, .. }) => {
            left == right
        }
        _ => false,
    }
}

fn compare(identity: String, input: &[u8]) -> Result<InputResult, String> {
    let text = std::str::from_utf8(input)
        .map_err(|error| format!("{identity}: frontmatter is not UTF-8: {error}"))?;
    let old = old_outcome(text);
    let selected = selected_outcome(text);
    let equal = outcome_equal(&old, &selected);
    Ok(InputResult {
        identity,
        input_sha256: sha256(input),
        byte_length: input.len(),
        old,
        selected,
        equal,
    })
}

fn focused_cases() -> Result<Vec<InputResult>, String> {
    [
        ("focused/duplicate-key", "key: first\nkey: final\n"),
        (
            "focused/merge-key",
            "base: &base\n  a: 1\nitem:\n  <<: *base\n  b: 2\n",
        ),
        (
            "focused/implicit-scalars",
            "values: [y, no, on, 2026-09-07]\n",
        ),
        ("focused/alias", "base: &base [1, 2]\ncopy: *base\n"),
        ("focused/timestamp", "timestamp: 2026-09-07T12:34:56Z\n"),
        ("focused/non-string-key", "? [a, b]\n: value\n"),
    ]
    .into_iter()
    .map(|(identity, input)| compare(identity.to_owned(), input.as_bytes()))
    .collect()
}

fn run() -> Result<(Report, bool), String> {
    let (source_args, injected_difference) = parse_args()?;
    let (comparator_sha256, comparator_files) = comparator_identity()?;
    let mut source_reports = Vec::new();
    let mut differences = Vec::new();
    let mut manifest_hasher = Sha256::new();
    let mut markdown_documents = 0;
    let mut complete_frontmatter_blocks = 0;

    for source in source_args {
        let root = source
            .root
            .canonicalize()
            .map_err(|error| format!("canonicalize {}: {error}", source.root.display()))?;
        let markdown = walk_markdown(&root)?;
        let mut inputs = Vec::new();
        for path in &markdown {
            let bytes =
                fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
            let Some(frontmatter) = complete_frontmatter(&bytes) else {
                continue;
            };
            let relative = path
                .strip_prefix(&root)
                .map_err(|error| format!("relativize {}: {error}", path.display()))?;
            let identity = format!("{}/{}", source.name, relative.display());
            let result = compare(identity.clone(), frontmatter)?;
            manifest_hasher.update(identity.as_bytes());
            manifest_hasher.update([0]);
            manifest_hasher.update(frontmatter);
            if !result.equal {
                differences.push(Difference {
                    identity,
                    reason: "outcome-or-value",
                });
            }
            inputs.push(result);
        }
        markdown_documents += markdown.len();
        complete_frontmatter_blocks += inputs.len();
        source_reports.push(SourceReport {
            kind: source.kind,
            name: source.name,
            revision: source.revision,
            root: root.display().to_string(),
            markdown_documents: markdown.len(),
            complete_frontmatter_blocks: inputs.len(),
            inputs,
        });
    }

    let mut focused = focused_cases()?;
    for result in &focused {
        manifest_hasher.update(result.identity.as_bytes());
        manifest_hasher.update([0]);
        manifest_hasher.update(result.input_sha256.as_bytes());
        if !result.equal {
            differences.push(Difference {
                identity: result.identity.clone(),
                reason: "outcome-or-value",
            });
        }
    }
    if injected_difference {
        let first = focused
            .first_mut()
            .ok_or("focused case population unexpectedly empty")?;
        first.equal = false;
        differences.push(Difference {
            identity: first.identity.clone(),
            reason: "injected-negative-control",
        });
    }

    let focused_count = focused.len();
    let difference_count = differences.len();
    let report = Report {
        schema: "quire-yaml-differential/v1",
        producer: Producer {
            package: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
            rustc: command_version("rustc"),
            cargo: command_version("cargo"),
            comparator_sha256,
            comparator_files,
            old_engine: "serde_yaml 0.9.34",
            selected_engine: "yaml_serde 0.10.7",
        },
        exclusions: EXCLUDED_DIRS,
        injected_difference,
        sources: source_reports,
        focused_cases: focused,
        population: Population {
            markdown_documents,
            complete_frontmatter_blocks,
            focused_cases: focused_count,
            compared_inputs: complete_frontmatter_blocks + focused_count,
            differences: difference_count,
        },
        input_manifest_sha256: format!("{:x}", manifest_hasher.finalize()),
        differences,
    };
    Ok((report, difference_count != 0))
}

fn main() {
    match run() {
        Ok((report, differs)) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).expect("report serialization cannot fail")
            );
            if differs {
                process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("quire-yaml-differential: {error}");
            process::exit(2);
        }
    }
}
