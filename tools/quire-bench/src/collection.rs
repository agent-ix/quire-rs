//! Collect benchmark measurements from the declared corpus entries.

use crate::export::{validate_repository_against_stack, VerificationStack};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use crate::engine::run_bounded;
use crate::metrics::{metrics_from, CoveragePayload};
use crate::{BenchError, CorpusEntry, Manifest, ModuleSource, Observed};

/// Coverage payload and module path saved for a governed export.
#[derive(Debug, Serialize)]
pub struct RawEvidence {
    /// Selected declaration module.
    pub module: Option<String>,
    /// Original coverage payload.
    pub payload: Value,
}

/// Measurements and optional raw evidence from one collection run.
pub struct Collection {
    /// Scores keyed by corpus and metric.
    pub observed: Observed,
    /// Unmodified input payloads keyed by corpus.
    pub raw_evidence: BTreeMap<String, RawEvidence>,
}

fn canonical(path: &Path) -> Result<PathBuf, BenchError> {
    path.canonicalize().map_err(|source| BenchError::Io {
        context: format!("resolving {}", path.display()),
        source,
    })
}

/// Refuse a declared input whose Git tree contains local edits.
pub fn require_clean(
    root: &Path,
    name: &str,
    path: &str,
    required: bool,
) -> Result<(), BenchError> {
    if !required {
        return Ok(());
    }
    let path = canonical(&root.join(path)).map_err(|_| {
        BenchError::Input(format!(
            "{name}: corpus absent at {}",
            root.join(path).display()
        ))
    })?;
    let output = Command::new("git")
        .arg("-C")
        .arg(&path)
        .args(["status", "--porcelain=v1", "--untracked-files=all"])
        .output()
        .map_err(|source| BenchError::Io {
            context: format!("checking {name} Git tree"),
            source,
        })?;
    if !output.status.success() || !output.stdout.is_empty() {
        return Err(BenchError::Input(format!(
            "{name}: corpus tree is dirty. Refusing to score an input with uncommitted edits."
        )));
    }
    Ok(())
}

/// Resolve the declared default module and reject an operator-selected path.
pub fn resolve_default_module(
    root: &Path,
    source: &ModuleSource,
    requested: Option<&Path>,
) -> Result<PathBuf, BenchError> {
    for (name, value) in [
        ("name", &source.name),
        ("path", &source.path),
        ("module", &source.module),
    ] {
        if value.is_empty() {
            return Err(BenchError::Input(format!(
                "benchmark module_source declares no {name}"
            )));
        }
    }
    if !source.require_clean {
        return Err(BenchError::Input(
            "benchmark module_source must require a clean tree".to_owned(),
        ));
    }
    require_clean(root, &source.name, &source.path, true)?;
    let expected = canonical(&root.join(&source.path).join(&source.module))?;
    if let Some(requested) = requested {
        let requested = if requested.is_absolute() {
            canonical(requested)?
        } else {
            canonical(&root.join(requested))?
        };
        if requested != expected {
            return Err(BenchError::Input(format!(
                "module path {} does not equal pinned module {}",
                requested.display(),
                expected.display()
            )));
        }
    }
    if !expected.is_dir() {
        return Err(BenchError::Input(format!(
            "pinned module is absent at {}",
            expected.display()
        )));
    }
    Ok(expected)
}

/// Run `quire coverage` for one corpus entry.
pub fn coverage_payload(
    quire: &Path,
    scope: &Path,
    module: Option<&Path>,
) -> Result<Value, BenchError> {
    let mut command = Command::new(quire);
    command
        .arg("coverage")
        .arg("--scope")
        .arg(scope)
        .arg("--json");
    if let Some(module) = module {
        command.arg("--module").arg(module);
    }
    let output = run_bounded(&mut command, Duration::from_secs(900))?;
    if !output.status.success() || output.stdout.trim().is_empty() {
        let detail = output.stderr.trim().lines().last().unwrap_or("no output");
        return Err(BenchError::Process(format!(
            "quire coverage failed: {}",
            detail.chars().take(200).collect::<String>()
        )));
    }
    serde_json::from_str(&output.stdout).map_err(|source| BenchError::Json {
        context: "decoding quire coverage payload".to_owned(),
        source,
    })
}

fn entry_module(
    root: &Path,
    entry: &CorpusEntry,
    default: Option<&Path>,
) -> Result<Option<PathBuf>, BenchError> {
    if let Some(module) = &entry.module {
        return canonical(&root.join(module)).map(Some);
    }
    Ok(default.map(Path::to_path_buf))
}

/// Collect all reachable corpus entries, or require every entry in strict mode.
pub fn collect(
    root: &Path,
    manifest: &Manifest,
    quire: &Path,
    module_override: Option<&Path>,
    strict: bool,
) -> Result<Collection, BenchError> {
    collect_inputs(root, manifest, quire, module_override, strict, None)
}

/// Collect as [`collect`] does, and also refuse a `require_clean` entry whose
/// checkout is not its attested revision (FR-050-AC-52): a normal run skips
/// that entry loudly, a strict run fails, and a `require_clean` module source
/// at another revision refuses the run.
pub fn collect_attested(
    root: &Path,
    manifest: &Manifest,
    quire: &Path,
    module_override: Option<&Path>,
    strict: bool,
    stack: &VerificationStack,
) -> Result<Collection, BenchError> {
    if manifest.module_source.require_clean {
        let path = canonical(&root.join(&manifest.module_source.path))?;
        validate_repository_against_stack(&path, stack, &manifest.module_source.name, &[])?;
    }
    collect_inputs(root, manifest, quire, module_override, strict, Some(stack))
}

fn collect_inputs(
    root: &Path,
    manifest: &Manifest,
    quire: &Path,
    module_override: Option<&Path>,
    strict: bool,
    stack: Option<&VerificationStack>,
) -> Result<Collection, BenchError> {
    let default_module = if manifest.corpora.iter().any(|entry| entry.module.is_none()) {
        Some(resolve_default_module(
            root,
            &manifest.module_source,
            module_override,
        )?)
    } else {
        None
    };
    let mut observed = Observed::new();
    let mut raw_evidence = BTreeMap::new();
    for entry in &manifest.corpora {
        let measurement = (|| {
            require_clean(root, &entry.name, &entry.path, entry.require_clean)?;
            if let (Some(stack), true) = (stack, entry.require_clean) {
                let path = canonical(&root.join(&entry.path))?;
                let source = entry.source_name.as_deref().unwrap_or(&entry.name);
                validate_repository_against_stack(&path, stack, source, &[])?;
            }
            let module = entry_module(root, entry, default_module.as_deref())?;
            let scope = canonical(&root.join(&entry.path))?;
            let payload = coverage_payload(quire, &scope, module.as_deref())?;
            let typed: CoveragePayload =
                serde_json::from_value(payload.clone()).map_err(|source| BenchError::Json {
                    context: format!("{}: decoding benchmark metrics", entry.name),
                    source,
                })?;
            let (measured, omissions) = metrics_from(&typed)?;
            let (selected, selected_omissions) = crate::selected(entry, measured);
            Ok::<_, BenchError>((module, payload, selected, omissions, selected_omissions))
        })();
        match measurement {
            Ok((module, payload, selected, omissions, selected_omissions)) => {
                for omission in omissions.into_iter().chain(selected_omissions) {
                    eprintln!("     {omission}");
                }
                eprintln!("…{}", entry.name);
                observed.insert(entry.name.clone(), selected);
                raw_evidence.insert(
                    entry.name.clone(),
                    RawEvidence {
                        module: module.map(|path| path.display().to_string()),
                        payload,
                    },
                );
            }
            Err(error) if strict => {
                return Err(BenchError::Input(format!("{}: {error}", entry.name)))
            }
            Err(error) => eprintln!("skip {}: {error}", entry.name),
        }
    }
    if observed.is_empty() {
        return Err(BenchError::Input(
            "no corpus entry could be scored — refusing to report a pass over nothing".to_owned(),
        ));
    }
    Ok(Collection {
        observed,
        raw_evidence,
    })
}

/// Read a JSON file into the requested type.
pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, BenchError> {
    let bytes = fs::read(path).map_err(|source| BenchError::Io {
        context: format!("reading {}", path.display()),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| BenchError::Json {
        context: format!("decoding {}", path.display()),
        source,
    })
}
