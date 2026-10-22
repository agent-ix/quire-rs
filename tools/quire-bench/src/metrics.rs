//! Read the declared benchmark metrics from a Quire coverage payload.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::{Number, Value};

use crate::BenchError;

/// The fields of a coverage payload consumed by the benchmark.
///
/// The engine emits other fields as well, so Serde intentionally ignores
/// unknown envelope fields here. Known fields remain typed and validated.
#[derive(Debug, Default, Deserialize)]
pub struct CoveragePayload {
    /// Overall coverage counts.
    #[serde(default)]
    pub totals: Totals,
    /// An absent census differs from an empty census.
    pub binding_census: Option<Vec<BindingCensus>>,
    /// Source markers without a minted target.
    #[serde(default)]
    pub untracked_symbols: Vec<Value>,
    /// Published metric rows.
    #[serde(default)]
    pub metrics: Vec<PublishedMetric>,
    /// Published diagnostics.
    #[serde(default)]
    pub diagnostics: Vec<Diagnostic>,
    /// Skeptic findings.
    #[serde(default)]
    pub suspicions: Vec<Value>,
}

/// Coverage totals used by the benchmark.
#[derive(Debug, Default, Deserialize)]
pub struct Totals {
    /// Backed rows.
    #[serde(default)]
    pub backed: u64,
    /// All rows.
    #[serde(default)]
    pub total: u64,
    /// Binding criteria, when published by this engine version.
    pub criteria: Option<u64>,
    /// Legacy specific-shaped count.
    pub specific_shaped: Option<u64>,
}

/// One language's source-symbol census.
#[derive(Debug, Deserialize)]
pub struct BindingCensus {
    /// Evidence symbols examined.
    pub candidates: u64,
    /// Evidence symbols bound.
    pub bound: u64,
    /// Tagged evidence symbols, when published.
    pub tagged: Option<u64>,
}

/// One metric emitted by the coverage engine.
#[derive(Debug, Deserialize)]
pub struct PublishedMetric {
    /// Metric name.
    #[serde(default)]
    pub name: String,
    /// `measured` or another published state.
    #[serde(default)]
    pub state: String,
    /// Ratio or count; absent means legacy ratio.
    pub shape: Option<String>,
    /// Metric numerator or count.
    pub value: Option<Number>,
    /// Metric population.
    pub population: Option<u64>,
    /// Inputs examined.
    pub examined: Option<u64>,
    /// Inputs matched.
    pub matched: Option<u64>,
}

/// One coverage diagnostic.
#[derive(Debug, Deserialize)]
pub struct Diagnostic {
    /// Stable reason code.
    pub reason: Option<String>,
}

fn decimal_percent(numerator: u128, denominator: u128) -> Number {
    let value = if denominator == 0 {
        0.0
    } else {
        ((numerator as f64 * 100.0 / denominator as f64) * 100.0).round_ties_even() / 100.0
    };
    Number::from_f64(value).expect("bounded finite percent")
}

fn count_number(value: usize) -> Result<Number, BenchError> {
    let value = u64::try_from(value)
        .map_err(|_| BenchError::Input("metric count exceeds u64".to_owned()))?;
    Ok(Number::from(value))
}

fn published<'a>(payload: &'a CoveragePayload, name: &str) -> Option<&'a PublishedMetric> {
    payload.metrics.iter().find(|metric| metric.name == name)
}

/// Count ratios that the engine published over unread input without a refusal.
pub fn silent_zeros(payload: &CoveragePayload) -> usize {
    let covered = payload.diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic.reason.as_deref(),
            Some("hollow-denominator" | "no-symbol-bound")
        )
    });
    payload
        .metrics
        .iter()
        .filter(|metric| {
            metric.state == "measured"
                && metric.shape.as_deref().unwrap_or("ratio") == "ratio"
                && metric.population.unwrap_or(0) > 0
                && metric.examined.unwrap_or(0) > 0
                && metric.matched.unwrap_or(0) == 0
                && !covered
        })
        .count()
}

/// Calculate available metrics and explain every omitted measurement.
pub fn metrics_from(
    payload: &CoveragePayload,
) -> Result<(BTreeMap<String, Number>, Vec<String>), BenchError> {
    let mut out = BTreeMap::new();
    let mut omitted = Vec::new();
    let candidates: u128 = payload
        .binding_census
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|row| u128::from(row.candidates))
        .sum();
    let bound: u128 = payload
        .binding_census
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|row| u128::from(row.bound))
        .sum();
    out.insert(
        "coverage.backed_pct".to_owned(),
        decimal_percent(
            u128::from(payload.totals.backed),
            u128::from(payload.totals.total),
        ),
    );
    out.insert(
        "coverage.dead_tags".to_owned(),
        count_number(payload.untracked_symbols.len())?,
    );

    match published(payload, "minting.section_hit_rate") {
        Some(metric) if metric.state == "measured" => {
            let positive = metric
                .value
                .as_ref()
                .and_then(Number::as_f64)
                .unwrap_or(0.0)
                > 0.0;
            out.insert(
                "coverage.minting_repos".to_owned(),
                Number::from(u8::from(positive)),
            );
        }
        _ => omitted.push(
            "omitted coverage.minting_repos: payload carries no measured minting.section_hit_rate"
                .to_owned(),
        ),
    }

    if payload.binding_census.is_none() {
        omitted.push("omitted coverage.binding_read_pct: payload carries no binding_census (engine predates FR-050-AC-27)".to_owned());
    } else if candidates > 0 {
        out.insert(
            "coverage.binding_read_pct".to_owned(),
            decimal_percent(bound, candidates),
        );
    } else {
        omitted.push("omitted coverage.binding_read_pct: no evidence symbols examined".to_owned());
    }

    let census = payload.binding_census.as_deref().unwrap_or_default();
    if !census.is_empty() && census.iter().any(|row| row.tagged.is_none()) {
        omitted.push("omitted authoring.tag_rate: binding_census carries no tagged count (engine predates agent-ix/quire-rs#271)".to_owned());
    } else if candidates > 0 {
        let tagged: u128 = census
            .iter()
            .map(|row| u128::from(row.tagged.unwrap_or(0)))
            .sum();
        out.insert(
            "authoring.tag_rate".to_owned(),
            decimal_percent(tagged, candidates),
        );
    } else {
        omitted.push("omitted authoring.tag_rate: no evidence symbols examined".to_owned());
    }

    let specific = published(payload, "coverage.specific_shaped");
    if let Some(metric) = specific.filter(|metric| metric.state == "measured") {
        let population = metric.population.unwrap_or(0);
        if population > 0 {
            let value = metric.value.as_ref().and_then(Number::as_u64).unwrap_or(0);
            out.insert(
                "properties.specific_shaped_pct".to_owned(),
                decimal_percent(u128::from(value), u128::from(population)),
            );
        } else {
            omitted.push("omitted properties.specific_shaped_pct: no binding criteria".to_owned());
        }
    } else if let Some(specific) = payload.totals.specific_shaped {
        if let Some(criteria) = payload.totals.criteria.filter(|criteria| *criteria > 0) {
            out.insert(
                "properties.specific_shaped_pct".to_owned(),
                decimal_percent(u128::from(specific), u128::from(criteria)),
            );
        } else {
            omitted.push("omitted properties.specific_shaped_pct: no binding criteria".to_owned());
        }
    } else {
        omitted.push("omitted properties.specific_shaped_pct: payload carries neither a measured coverage.specific_shaped metric nor the legacy totals.specific_shaped field".to_owned());
    }

    out.insert(
        "sentinel.silent_zero".to_owned(),
        count_number(silent_zeros(payload))?,
    );
    if candidates > 0 {
        out.insert(
            "skeptic.suspicion_rate".to_owned(),
            decimal_percent(
                u128::try_from(payload.suspicions.len())
                    .map_err(|_| BenchError::Input("suspicion count exceeds u128".to_owned()))?,
                candidates,
            ),
        );
    } else {
        omitted.push("omitted skeptic.suspicion_rate: no evidence symbols examined".to_owned());
    }
    Ok((out, omitted))
}
