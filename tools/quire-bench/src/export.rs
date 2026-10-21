//! Governed measurement export using an attested consumer and corpus.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::collection::{read_json, Collection, RawEvidence};
use crate::engine::build_engine;
use crate::{BenchError, Manifest};

/// One repository recorded by the verification stack.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceAttestation {
    /// Full Git revision.
    pub revision: String,
    /// Must be `clean`.
    pub source_state: String,
    /// Authoritative origin URL.
    pub remote: String,
}

/// Toolchain versions recorded by the verification stack.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Toolchains {
    /// Node version.
    pub node: String,
    /// Rust version.
    pub rust: String,
    /// Python version.
    pub python: String,
}

/// Attestation governing one measurement collection.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VerificationStack {
    /// Attestation schema identifier.
    pub schema_version: String,
    /// Digest of the dependency lock.
    pub lock_digest: String,
    /// Digest of the engine executable.
    pub executable_digest: String,
    /// Required Cargo profile.
    pub build_profile: String,
    /// Repository revisions.
    pub sources: BTreeMap<String, SourceAttestation>,
    /// Sorted engine capability tokens.
    pub capabilities: Vec<String>,
    /// Digests of dependent artifacts.
    pub artifacts: BTreeMap<String, String>,
    /// Toolchain versions.
    pub toolchains: Toolchains,
}

/// The complete population identity of one observation.
#[derive(Debug, Serialize)]
pub struct Population {
    /// Every declared input was measured.
    pub complete: bool,
    /// Corpus identity.
    pub identity: CorpusIdentity,
}

/// Corpus dimension of an observation.
#[derive(Debug, Serialize)]
pub struct CorpusIdentity {
    /// Manifest corpus name.
    pub corpus: String,
}

/// One result linked to its measurement plan.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    /// Metric dictionary key.
    pub metric: String,
    /// Plan identifier.
    pub plan_id: String,
    /// Plan definition version.
    pub definition_version: String,
    /// Measurement state.
    pub state: &'static str,
    /// Observed number.
    pub value: Number,
    /// Declared unit.
    pub unit: String,
    /// Count or ratio.
    pub shape: &'static str,
    /// Complete population identity.
    pub population: Population,
    /// Corpus dimension.
    pub dimensions: CorpusIdentity,
}

/// The observed corpus and metric keys.
#[derive(Debug, Serialize)]
pub struct Scope {
    /// Measured corpus names.
    pub corpora: Vec<String>,
    /// Measured metric keys.
    pub metrics: Vec<String>,
}

/// Inputs used to produce the collection.
#[derive(Debug, Serialize)]
pub struct Environment {
    /// Consumer checkout.
    pub consumer: String,
    /// Default module choice.
    pub module: String,
}

/// Governed measurement collection, version 2.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasurementCollection {
    /// Collection schema version.
    pub schema_version: u8,
    /// Stable collection identifier derived from timestamp.
    pub collection_id: String,
    /// Measurement subject.
    pub subject: &'static str,
    /// Measured scope.
    pub scope: Scope,
    /// Producer name.
    pub tool_identity: &'static str,
    /// UTC collection time.
    pub timestamp: String,
    /// Attested source revision.
    pub source_revision: String,
    /// Consumer and module.
    pub environment: Environment,
    /// Input attestation.
    pub verification_stack: VerificationStack,
    /// Plan-derived observations.
    pub observations: Vec<Observation>,
    /// Original coverage payloads.
    pub raw_evidence: BTreeMap<String, RawEvidence>,
}

/// Full lowercase SHA-1 string, used only to validate the existing attestation.
fn is_full_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn is_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

/// Normalize GitHub SSH and HTTPS origin spellings before comparison.
pub fn normalized_remote(value: &str) -> String {
    let value = value.trim();
    let value = value.strip_prefix("git@github.com:").map_or_else(
        || value.to_owned(),
        |path| format!("https://github.com/{path}"),
    );
    value
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .to_owned()
}

fn git(root: &Path, args: &[&str]) -> Result<String, BenchError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|source| BenchError::Io {
            context: format!("running git in {}", root.display()),
            source,
        })?;
    if !output.status.success() {
        return Err(BenchError::Process(format!(
            "git {} failed in {}: {}",
            args.join(" "),
            root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|_| BenchError::Input("Git output is not UTF-8".to_owned()))
}

/// Read and validate an existing verification-stack attestation.
pub fn load_verification_stack(
    path: &Path,
    source_name: &str,
    source_remote: &str,
) -> Result<VerificationStack, BenchError> {
    let stack: VerificationStack = read_json(path)?;
    if stack.schema_version != "verification-stack-attestation-v1" {
        return Err(BenchError::Input(
            "verification-stack attestation has unsupported schemaVersion".to_owned(),
        ));
    }
    for (name, digest) in [
        ("lockDigest", &stack.lock_digest),
        ("executableDigest", &stack.executable_digest),
    ] {
        if !is_sha256(digest) {
            return Err(BenchError::Input(format!(
                "verification-stack {name} is not a full sha256 digest"
            )));
        }
    }
    if stack.build_profile != "release" {
        return Err(BenchError::Input(
            "verification-stack buildProfile must be release".to_owned(),
        ));
    }
    if stack.sources.is_empty() {
        return Err(BenchError::Input(
            "verification-stack sources must be a non-empty object".to_owned(),
        ));
    }
    for (name, source) in &stack.sources {
        if !is_full_sha(&source.revision)
            || source.source_state != "clean"
            || source.remote.is_empty()
        {
            return Err(BenchError::Input(format!(
                "verification-stack source {name} is not clean and immutable"
            )));
        }
    }
    let own = stack.sources.get(source_name).ok_or_else(|| {
        BenchError::Input(format!("verification-stack has no {source_name} source"))
    })?;
    if normalized_remote(&own.remote) != normalized_remote(source_remote) {
        return Err(BenchError::Input(format!(
            "verification-stack {source_name} remote does not match exporter origin"
        )));
    }
    if stack.capabilities.is_empty()
        || stack.capabilities.iter().any(String::is_empty)
        || stack.capabilities.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(BenchError::Input(
            "verification-stack capabilities must be non-empty, unique, and sorted".to_owned(),
        ));
    }
    if stack.artifacts.is_empty() {
        return Err(BenchError::Input(
            "verification-stack artifacts must be a non-empty object".to_owned(),
        ));
    }
    for (name, digest) in &stack.artifacts {
        if !is_sha256(digest) {
            return Err(BenchError::Input(format!(
                "verification-stack artifact {name} is not a full sha256 digest"
            )));
        }
    }
    if [
        &stack.toolchains.node,
        &stack.toolchains.rust,
        &stack.toolchains.python,
    ]
    .iter()
    .any(|version| version.is_empty())
    {
        return Err(BenchError::Input(
            "verification-stack toolchains must pin node, rust, and python".to_owned(),
        ));
    }
    Ok(stack)
}

/// Read the repository's origin URL.
pub fn git_remote(root: &Path) -> Result<String, BenchError> {
    let remote = git(root, &["remote", "get-url", "origin"])?;
    if remote.trim().is_empty() {
        return Err(BenchError::Input(
            "the Quire origin remote is unavailable".to_owned(),
        ));
    }
    Ok(normalized_remote(&remote))
}

/// Validate a checkout against the attestation, allowing only a linear evidence overlay.
pub fn validate_repository_against_stack(
    root: &Path,
    stack: &VerificationStack,
    source_name: &str,
    allowed_overlay_paths: &[&str],
) -> Result<String, BenchError> {
    let source = stack.sources.get(source_name).ok_or_else(|| {
        BenchError::Input(format!("verification-stack has no {source_name} source"))
    })?;
    let head = git(root, &["rev-parse", "HEAD"])?;
    let head = head.trim();
    if !is_full_sha(head) {
        return Err(BenchError::Input(format!(
            "{source_name} checkout revision is unavailable"
        )));
    }
    let status = git(root, &["status", "--porcelain=v1", "--untracked-files=all"])?;
    if !status.trim().is_empty() {
        return Err(BenchError::Input(format!(
            "{source_name} checkout is dirty"
        )));
    }
    if git_remote(root)? != normalized_remote(&source.remote) {
        return Err(BenchError::Input(format!(
            "{source_name} checkout remote does not match the attestation"
        )));
    }
    if head == source.revision {
        return Ok(source.revision.clone());
    }
    if allowed_overlay_paths.is_empty() {
        return Err(BenchError::Input(format!(
            "{source_name} checkout does not equal the attested revision"
        )));
    }
    let ancestor = git(root, &["merge-base", &source.revision, head])?;
    if ancestor.trim() != source.revision {
        return Err(BenchError::Input(format!(
            "{source_name} evidence overlay does not descend from the attested revision"
        )));
    }
    let commits = git(
        root,
        &[
            "rev-list",
            "--parents",
            &format!("{}..{head}", source.revision),
        ],
    )?;
    if commits
        .lines()
        .any(|line| line.split_whitespace().count() != 2)
    {
        return Err(BenchError::Input(format!(
            "{source_name} evidence overlay is not a linear, merge-free chain"
        )));
    }
    let changed = git(root, &["diff", "--name-only", &source.revision, head])?;
    let unexpected: Vec<_> = changed
        .lines()
        .filter(|path| {
            !allowed_overlay_paths
                .iter()
                .any(|prefix| *path == *prefix || path.starts_with(&format!("{prefix}/")))
        })
        .collect();
    if !unexpected.is_empty() {
        return Err(BenchError::Input(format!(
            "{source_name} evidence overlay changes non-evidence paths: {}",
            unexpected.join(", ")
        )));
    }
    Ok(source.revision.clone())
}

/// Refuse a toolchain mismatch with the attestation.
pub fn validate_toolchains(stack: &VerificationStack) -> Result<(), BenchError> {
    let python = Command::new("python3")
        .args(["-c", "import platform; print(platform.python_version())"])
        .output()
        .map_err(|source| BenchError::Io {
            context: "reading Python version".to_owned(),
            source,
        })?;
    let observed_python = String::from_utf8_lossy(&python.stdout).trim().to_owned();
    if !python.status.success() || observed_python != stack.toolchains.python {
        return Err(BenchError::Input(format!(
            "Python drift: expected {}, observed {observed_python}",
            stack.toolchains.python
        )));
    }
    let rust = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|source| BenchError::Io {
            context: "reading Rust version".to_owned(),
            source,
        })?;
    let observed_rust = String::from_utf8_lossy(&rust.stdout)
        .split_whitespace()
        .nth(1)
        .unwrap_or("unavailable")
        .to_owned();
    if !rust.status.success() || observed_rust != stack.toolchains.rust {
        return Err(BenchError::Input(format!(
            "Rust drift: expected {}, observed {observed_rust}",
            stack.toolchains.rust
        )));
    }
    Ok(())
}

/// Refuse a binary whose digest differs from the attestation.
pub fn validate_executable_digest(
    executable: &Path,
    stack: &VerificationStack,
) -> Result<(), BenchError> {
    let bytes = fs::read(executable).map_err(|source| BenchError::Io {
        context: format!("reading {}", executable.display()),
        source,
    })?;
    let observed = format!("sha256:{:x}", Sha256::digest(bytes));
    if observed != stack.executable_digest {
        return Err(BenchError::Input(format!(
            "built executable digest does not match attestation: expected {}, observed {observed}",
            stack.executable_digest
        )));
    }
    Ok(())
}

#[derive(Deserialize)]
struct PlanFrontmatter {
    id: String,
    definition_version: String,
    #[serde(flatten)]
    _other: BTreeMap<String, Value>,
}

/// Read the plan identity from YAML frontmatter.
pub fn plan_identity(path: &Path) -> Result<(String, String), BenchError> {
    let content = fs::read_to_string(path).map_err(|source| BenchError::Io {
        context: format!("reading {}", path.display()),
        source,
    })?;
    let body = content.strip_prefix("---\n").ok_or_else(|| {
        BenchError::Input(format!(
            "{}: measurement plan has no frontmatter",
            path.display()
        ))
    })?;
    let frontmatter = body
        .split_once("\n---")
        .ok_or_else(|| {
            BenchError::Input(format!(
                "{}: measurement plan has no frontmatter",
                path.display()
            ))
        })?
        .0;
    let plan: PlanFrontmatter = serde_yaml::from_str(frontmatter).map_err(|error| {
        BenchError::Input(format!(
            "{}: invalid measurement plan frontmatter: {error}",
            path.display()
        ))
    })?;
    Ok((plan.id, plan.definition_version))
}

/// Assemble the typed, plan-derived collection.
pub fn build_collection(
    root: &Path,
    manifest: &Manifest,
    measurements: Collection,
    timestamp: &str,
    source_revision: String,
    consumer: &Path,
    module: Option<&Path>,
    verification_stack: VerificationStack,
) -> Result<MeasurementCollection, BenchError> {
    let mut observations = Vec::new();
    for (corpus, metrics) in &measurements.observed {
        for (name, value) in metrics {
            let definition = manifest.metrics.get(name).ok_or_else(|| {
                BenchError::Input(format!("{name}: metric has no manifest definition"))
            })?;
            let plan_path = root.join(&definition.measurement_plan);
            let (plan_id, definition_version) = plan_identity(&plan_path)?;
            observations.push(Observation {
                metric: name.clone(),
                plan_id,
                definition_version,
                state: "measured",
                value: value.clone(),
                unit: definition.unit.clone(),
                shape: if matches!(
                    name.as_str(),
                    "coverage.dead_tags" | "coverage.minting_repos" | "sentinel.silent_zero"
                ) {
                    "count"
                } else {
                    "ratio"
                },
                population: Population {
                    complete: true,
                    identity: CorpusIdentity {
                        corpus: corpus.clone(),
                    },
                },
                dimensions: CorpusIdentity {
                    corpus: corpus.clone(),
                },
            });
        }
    }
    if observations.is_empty() {
        return Err(BenchError::Input(
            "the benchmark produced no observations".to_owned(),
        ));
    }
    let represented: BTreeSet<_> = observations.iter().map(|row| row.metric.as_str()).collect();
    let missing: Vec<_> = manifest
        .metrics
        .keys()
        .filter(|key| !represented.contains(key.as_str()))
        .collect();
    if !missing.is_empty() {
        return Err(BenchError::Input(format!(
            "active benchmark metrics were not produced: {}",
            missing
                .into_iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }
    let compact_time: String = timestamp.chars().filter(char::is_ascii_digit).collect();
    let corpora = measurements.observed.keys().cloned().collect();
    let metrics = represented.into_iter().map(str::to_owned).collect();
    Ok(MeasurementCollection {
        schema_version: 2,
        collection_id: format!("quire-bench-{compact_time}"),
        subject: "Quire engine benchmark",
        scope: Scope { corpora, metrics },
        tool_identity: "quire-rs tools/quire-bench",
        timestamp: timestamp.to_owned(),
        source_revision,
        environment: Environment {
            consumer: consumer.display().to_string(),
            module: format!(
                "per-manifest with default {}",
                module.map_or_else(|| "default".to_owned(), |path| path.display().to_string())
            ),
        },
        verification_stack,
        observations,
        raw_evidence: measurements.raw_evidence,
    })
}

/// UTC wall-clock timestamp with exactly millisecond precision.
pub fn utc_timestamp() -> String {
    let now = OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        now.millisecond()
    )
}

/// Export one governed collection after checking every attested input.
pub fn export(
    root: &Path,
    manifest: &Manifest,
    consumer: &Path,
    module: Option<&Path>,
    stack_path: &Path,
) -> Result<MeasurementCollection, BenchError> {
    let remote = git_remote(root)?;
    let stack = load_verification_stack(stack_path, "quire", &remote)?;
    let source_revision =
        validate_repository_against_stack(root, &stack, "quire", &["spec/evidence/measurements"])?;
    validate_toolchains(&stack)?;
    validate_repository_against_stack(consumer, &stack, "quire-cli", &[])?;
    for (name, path, required) in manifest
        .corpora
        .iter()
        .map(|entry| {
            (
                entry.source_name.as_deref().unwrap_or(&entry.name),
                entry.path.as_str(),
                entry.require_clean,
            )
        })
        .chain(std::iter::once((
            manifest.module_source.name.as_str(),
            manifest.module_source.path.as_str(),
            manifest.module_source.require_clean,
        )))
    {
        if required {
            let path = root
                .join(path)
                .canonicalize()
                .map_err(|source| BenchError::Io {
                    context: format!("resolving {path}"),
                    source,
                })?;
            validate_repository_against_stack(&path, &stack, name, &[])?;
        }
    }
    let quire = build_engine(consumer, true)?;
    validate_executable_digest(&quire, &stack)?;
    let measurements = crate::collection::collect(root, manifest, &quire, module, true)?;
    build_collection(
        root,
        manifest,
        measurements,
        &utc_timestamp(),
        source_revision,
        consumer,
        module,
        stack,
    )
}

/// Resolve a CLI path relative to the repository root.
pub fn within_root(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}
