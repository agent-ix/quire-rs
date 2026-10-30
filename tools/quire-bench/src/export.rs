//! Governed measurement export.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use time::OffsetDateTime;

use crate::collection::{Collection, RawEvidence};
use crate::engine::build_engine;
use crate::{BenchError, Manifest};

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
    /// Consumer and module.
    pub environment: Environment,
    /// Plan-derived observations.
    pub observations: Vec<Observation>,
    /// Original coverage payloads.
    pub raw_evidence: BTreeMap<String, RawEvidence>,
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
    consumer: &Path,
    module: Option<&Path>,
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
        environment: Environment {
            consumer: consumer.display().to_string(),
            module: format!(
                "per-manifest with default {}",
                module.map_or_else(|| "default".to_owned(), |path| path.display().to_string())
            ),
        },
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

/// Export one governed collection.
pub fn export(
    root: &Path,
    manifest: &Manifest,
    consumer: &Path,
    module: Option<&Path>,
) -> Result<MeasurementCollection, BenchError> {
    let quire = build_engine(consumer, true)?;
    let measurements = crate::collection::collect(root, manifest, &quire, module, true)?;
    build_collection(
        root,
        manifest,
        measurements,
        &utc_timestamp(),
        consumer,
        module,
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
