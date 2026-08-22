//! The shared corpus-case harness (FR-050-AC-29, CR-098).
//!
//! `tests/fixtures/filament_core/graph_cases.json` was the only data-driven
//! scenario corpus in this repository — an 18-case `{name, tags, input, expect}`
//! array behind one parameterized test. Everything else was hand-authored
//! directory convention, which is why every new regression cost a new `.rs`
//! file and why the battletest failure families had nowhere to land.
//!
//! This generalizes that pattern to the **tool** surfaces: a case declares a
//! whole miniature repository — module manifest, spec documents, source files —
//! and the envelope it expects out. The harness materializes it, runs the real
//! `compute` path, and asserts. Adding a regression is adding a JSON object.
//!
//! **Directory corpora stay for what genuinely needs them.** A case here has no
//! filesystem topology beyond the paths it lists, so anything about walking,
//! exclusion globs or symlinks belongs in a real fixture tree. The point is not
//! to eliminate those; it is that a scenario expressible as data should not
//! cost a file.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

/// One declarative case.
#[derive(Debug, Deserialize)]
pub struct CorpusCase {
    /// Stable, human-readable — it is what an assertion failure names.
    pub name: String,
    /// The filing this case is the regression for. Required, and the reason
    /// the harness exists: a fixture whose origin is not recorded becomes a
    /// fixture nobody dares change (CR-098 / `agent-ix/quire-rs#234`).
    pub issue_ref: String,
    /// Tracking ids and free-form labels. Ids here are what bind the case to
    /// the matrix.
    #[serde(default)]
    pub tags: Vec<String>,
    pub input: CaseInput,
    pub expect: CaseExpect,
}

/// A miniature repository, entirely in data.
#[derive(Debug, Deserialize)]
pub struct CaseInput {
    /// The module manifest, verbatim YAML.
    pub module: String,
    /// Scope-relative path → contents, under `spec/`.
    #[serde(default)]
    pub documents: BTreeMap<String, String>,
    /// Scope-relative path → contents, under the code root.
    #[serde(default)]
    pub sources: BTreeMap<String, String>,
}

/// What the emitted envelope must say.
///
/// Every field is optional: a case asserts the facts it is about and stays
/// silent on the rest, so an unrelated engine change does not fail forty cases
/// that were never about it.
#[derive(Debug, Default, Deserialize)]
pub struct CaseExpect {
    pub backed: Option<usize>,
    pub total: Option<usize>,
    /// `reason` tokens that MUST be present.
    #[serde(default)]
    pub diagnostic_reasons: Vec<String>,
    /// `reason` tokens that must NOT be — the half a fixture usually forgets,
    /// and the half that catches a check firing on healthy input.
    #[serde(default)]
    pub absent_diagnostic_reasons: Vec<String>,
    /// Per-language binding census expectations.
    #[serde(default)]
    pub binding_census: Vec<ExpectCensus>,
    /// Per-metric expectations, keyed on the metric name.
    #[serde(default)]
    pub metrics: Vec<ExpectMetric>,
}

#[derive(Debug, Deserialize)]
pub struct ExpectCensus {
    pub language: String,
    pub candidates: Option<usize>,
    pub bound: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ExpectMetric {
    pub name: String,
    /// `measured` or `not_computed`.
    pub state: Option<String>,
    pub value: Option<u64>,
    pub population: Option<u64>,
    pub examined: Option<u64>,
    pub matched: Option<u64>,
    /// Whether the metric is a ratio over input it could not read.
    pub hollow: Option<bool>,
}

/// Materialize `case` into `root` and run the real coverage path over it.
pub fn run(case: &CorpusCase, root: &Path) -> quire_rs::CoverageReport {
    let module = root.join("module");
    let scope = root.join("spec");
    let source = root.join("src");
    for dir in [&module, &scope, &source] {
        std::fs::create_dir_all(dir).expect("mkdir");
    }
    write(&module.join("manifest.yaml"), &case.input.module);
    for (rel, body) in &case.input.documents {
        write(&scope.join(rel), body);
    }
    for (rel, body) in &case.input.sources {
        write(&source.join(rel), body);
    }

    let registry = quire_rs::Registry::load_module(&module)
        .unwrap_or_else(|e| panic!("{}: module load failed: {e}", case.name));
    let spec = quire_rs::Spec::from_path(&scope);
    let extraction = quire_rs::symbols::extract_tree(&source);
    let model = registry.traceability().cloned().unwrap_or_default();
    let graph = quire_rs::symbols::trace::bind(&extraction, &model);
    quire_rs::coverage::compute(&spec, &registry, &graph, &scope)
        .unwrap_or_else(|e| panic!("{}: compute failed: {e}", case.name))
}

fn write(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, body).expect("write");
}

/// Assert `report` against the case's expectations.
pub fn assert_expected(case: &CorpusCase, report: &quire_rs::CoverageReport) {
    let name = &case.name;
    let e = &case.expect;

    if let Some(backed) = e.backed {
        assert_eq!(report.totals.backed, backed, "{name}: backed");
    }
    if let Some(total) = e.total {
        assert_eq!(report.totals.total, total, "{name}: total");
    }

    let reasons: Vec<&str> = report
        .diagnostics
        .iter()
        .map(|d| d.reason.as_str())
        .collect();
    for want in &e.diagnostic_reasons {
        assert!(
            reasons.contains(&want.as_str()),
            "{name}: expected diagnostic `{want}`, got {reasons:?}"
        );
    }
    for unwanted in &e.absent_diagnostic_reasons {
        assert!(
            !reasons.contains(&unwanted.as_str()),
            "{name}: `{unwanted}` fired on a case that is not about it: {reasons:?}"
        );
    }

    for want in &e.binding_census {
        let got = report
            .binding_census
            .iter()
            .find(|c| c.language == want.language)
            .unwrap_or_else(|| {
                panic!(
                    "{name}: no `{}` census in {:?}",
                    want.language, report.binding_census
                )
            });
        if let Some(candidates) = want.candidates {
            assert_eq!(
                got.candidates, candidates,
                "{name}: {} candidates",
                got.language
            );
        }
        if let Some(bound) = want.bound {
            assert_eq!(got.bound, bound, "{name}: {} bound", got.language);
        }
    }

    for want in &e.metrics {
        let got = report
            .metrics
            .iter()
            .find(|m| m.name == want.name)
            .unwrap_or_else(|| panic!("{name}: no `{}` metric", want.name));
        if let Some(hollow) = want.hollow {
            assert_eq!(got.is_hollow(), hollow, "{name}: {} hollow", want.name);
        }
        match &got.measurement {
            quire_rs::metric::Measurement::Measured {
                value,
                population,
                examined,
                matched,
            } => {
                if let Some(state) = &want.state {
                    assert_eq!(state, "measured", "{name}: {} state", want.name);
                }
                check(name, &want.name, "value", want.value, Some(*value));
                check(
                    name,
                    &want.name,
                    "population",
                    want.population,
                    Some(*population),
                );
                check(name, &want.name, "examined", want.examined, Some(*examined));
                check(name, &want.name, "matched", want.matched, Some(*matched));
            }
            quire_rs::metric::Measurement::NotComputed { .. } => {
                if let Some(state) = &want.state {
                    assert_eq!(state, "not_computed", "{name}: {} state", want.name);
                }
                for (field, v) in [
                    ("value", want.value),
                    ("population", want.population),
                    ("examined", want.examined),
                    ("matched", want.matched),
                ] {
                    assert!(
                        v.is_none(),
                        "{name}: {} is not computed and cannot assert {field}",
                        want.name
                    );
                }
            }
        }
    }
}

fn check(case: &str, metric: &str, field: &str, want: Option<u64>, got: Option<u64>) {
    if let (Some(want), Some(got)) = (want, got) {
        assert_eq!(want, got, "{case}: {metric}.{field}");
    }
}
