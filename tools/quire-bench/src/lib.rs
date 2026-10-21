//! Corpus benchmark scoring and collection for the Quire engine.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Number;
use thiserror::Error;

pub mod collection;
pub mod engine;
pub mod metrics;

/// Failures that invalidate a benchmark run or its score.
#[derive(Debug, Error)]
pub enum BenchError {
    /// A filesystem operation failed.
    #[error("{context}: {source}")]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
    /// JSON input could not be decoded.
    #[error("{context}: {source}")]
    Json {
        context: String,
        #[source]
        source: serde_json::Error,
    },
    /// The manifest or requested population is invalid.
    #[error("{0}")]
    Input(String),
    /// A subprocess failed or exceeded its bound.
    #[error("{0}")]
    Process(String),
}

/// The declared benchmark inputs and metric dictionary.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[serde(rename = "$comment")]
    _comment: Option<String>,
    /// Metrics the benchmark may emit.
    pub metrics: BTreeMap<String, MetricSpec>,
    /// The default declaration module source.
    pub module_source: ModuleSource,
    /// Corpus entries to score.
    pub corpora: Vec<CorpusEntry>,
}

/// The metadata carried alongside each reported score.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricSpec {
    #[serde(default, rename = "measurement_plan")]
    _measurement_plan: Option<String>,
    /// Output unit.
    pub unit: String,
    /// Denominator or counted population.
    pub population: String,
    /// Calculation method.
    pub method: String,
    /// Ratchet direction.
    pub direction: Direction,
}

/// How a score may improve.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Direction {
    /// Larger values improve.
    HigherIsBetter,
    /// Smaller values improve.
    LowerIsBetter,
    /// Every nonzero value fails.
    GateZero,
}

/// The source of the default declaration module.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleSource {
    #[serde(rename = "$comment")]
    _comment: Option<String>,
    /// Human-readable source name.
    pub name: String,
    /// Repository path relative to the quire-rs root.
    pub path: String,
    /// Module directory within that repository.
    pub module: String,
    /// Whether local edits refuse a measurement.
    pub require_clean: bool,
}

/// One corpus entry in the benchmark manifest.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusEntry {
    #[serde(rename = "$comment")]
    _comment: Option<String>,
    #[serde(default, rename = "source_name")]
    _source_name: Option<String>,
    #[serde(default, rename = "tier")]
    _tier: Option<u64>,
    /// Reported corpus name.
    pub name: String,
    /// Corpus path relative to the quire-rs root.
    pub path: String,
    /// Optional module path overriding the default.
    pub module: Option<String>,
    /// Optional list of metrics this entry scores.
    pub metrics: Option<Vec<String>>,
    /// Whether local edits refuse a measurement.
    #[serde(default)]
    pub require_clean: bool,
}

/// Scores keyed by corpus and metric name.
pub type Observed = BTreeMap<String, BTreeMap<String, Number>>;

/// One metric's comparison with its current baseline.
#[derive(Debug, Serialize)]
pub struct ScoreRow {
    /// Corpus name.
    pub corpus: String,
    /// Metric dictionary key.
    pub metric: String,
    /// Declared output unit.
    pub unit: String,
    /// Declared population.
    pub population: String,
    /// Declared method.
    pub method: String,
    /// Ratchet direction.
    pub direction: Direction,
    /// Observed value.
    pub value: Number,
    /// Current baseline, if any.
    pub baseline: Option<Number>,
    /// Comparison verdict.
    pub verdict: Verdict,
    /// Value to write in an explicit update.
    pub next_baseline: Number,
}

/// Comparison verdict.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    /// The metric improved and can tighten its baseline.
    Improved,
    /// The metric equals its baseline.
    Held,
    /// The metric became worse.
    Regressed,
    /// The metric has no baseline yet.
    New,
}

/// A deterministic benchmark report.
#[derive(Debug, Serialize)]
pub struct ScoreReport {
    /// Sorted score rows.
    pub rows: Vec<ScoreRow>,
}

/// Compare one observed value to its baseline.
pub fn compare(
    direction: Direction,
    value: &Number,
    baseline: Option<&Number>,
) -> (Verdict, Number) {
    if direction == Direction::GateZero {
        return (
            if value.as_f64() == Some(0.0) {
                Verdict::Held
            } else {
                Verdict::Regressed
            },
            Number::from(0),
        );
    }
    let Some(baseline) = baseline else {
        return (Verdict::New, value.clone());
    };
    let observed = value.as_f64().unwrap_or(f64::NAN);
    let previous = baseline.as_f64().unwrap_or(f64::NAN);
    let better = match direction {
        Direction::HigherIsBetter => observed > previous,
        Direction::LowerIsBetter => observed < previous,
        Direction::GateZero => unreachable!("gate-zero handled above"),
    };
    if better {
        (Verdict::Improved, value.clone())
    } else if value == baseline || observed == previous {
        (Verdict::Held, baseline.clone())
    } else {
        (Verdict::Regressed, baseline.clone())
    }
}

/// Build sorted score rows, refusing metrics absent from the dictionary.
pub fn score(
    manifest: &Manifest,
    observed: &Observed,
    baselines: &Observed,
) -> Result<ScoreReport, BenchError> {
    let mut rows = Vec::new();
    for (corpus, metrics) in observed {
        for (name, value) in metrics {
            let spec = manifest.metrics.get(name).ok_or_else(|| {
                BenchError::Input(format!(
                    "{corpus}: metric {name:?} is not in the dictionary"
                ))
            })?;
            let baseline = baselines.get(corpus).and_then(|m| m.get(name));
            let (verdict, next_baseline) = compare(spec.direction, value, baseline);
            rows.push(ScoreRow {
                corpus: corpus.clone(),
                metric: name.clone(),
                unit: spec.unit.clone(),
                population: spec.population.clone(),
                method: spec.method.clone(),
                direction: spec.direction,
                value: value.clone(),
                baseline: baseline.cloned(),
                verdict,
                next_baseline,
            });
        }
    }
    Ok(ScoreReport { rows })
}

/// Render the human summary and whether any row regressed.
pub fn render(report: &ScoreReport) -> (String, bool) {
    let mut lines = Vec::new();
    let mut failed = false;
    for row in &report.rows {
        let mark = match row.verdict {
            Verdict::Improved => "++",
            Verdict::Held => "ok",
            Verdict::New => "**",
            Verdict::Regressed => "!!",
        };
        let baseline = row
            .baseline
            .as_ref()
            .map_or_else(|| "—".to_owned(), ToString::to_string);
        lines.push(format!(
            "{mark} {}/{}: {} (baseline {baseline}) [{}]",
            row.corpus, row.metric, row.value, row.unit
        ));
        if row.verdict == Verdict::Regressed {
            failed = true;
            lines.push(format!(
                "     regressed against {baseline}; population: {}",
                row.population
            ));
        }
    }
    (lines.join("\n"), failed)
}

/// Select only the metrics declared by a corpus entry and explain omissions.
pub fn selected(
    entry: &CorpusEntry,
    measured: BTreeMap<String, Number>,
) -> (BTreeMap<String, Number>, Vec<String>) {
    let Some(declared) = &entry.metrics else {
        return (measured, Vec::new());
    };
    let allowed: BTreeSet<&str> = declared.iter().map(String::as_str).collect();
    let omissions = declared
        .iter()
        .filter(|name| !measured.contains_key(name.as_str()))
        .map(|name| {
            format!(
                "omitted {name}: declared by corpus {:?} but not measurable in its payload",
                entry.name
            )
        })
        .collect();
    (
        measured
            .into_iter()
            .filter(|(name, _)| allowed.contains(name.as_str()))
            .collect(),
        omissions,
    )
}
