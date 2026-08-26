//! Diagnostics derived after coverage reconciliation.

use std::collections::BTreeMap;

use crate::metric::Metric;
use crate::registry::Registry;

use super::{CoverageDiagnostic, CriteriaCounts};

/// Documents whose every extractable criterion is the `universal` catch-all
/// (FR-050-AC-28, #261).
///
/// Emit one corpus-level diagnostic with a count and the first deterministic
/// locus. This keeps the result actionable without turning a descriptive shape
/// into a per-document verdict (FR-050-CON-1; rationale in CR-095).
pub(super) fn catch_all_documents(criteria: &[CriteriaCounts]) -> Vec<CoverageDiagnostic> {
    let binding: Vec<&CriteriaCounts> = criteria.iter().filter(|c| c.property_shaped > 0).collect();
    let all_universal: Vec<&CriteriaCounts> = binding
        .iter()
        .copied()
        .filter(|c| c.specific_shaped == 0)
        .collect();
    let Some(first) = all_universal.first() else {
        return Vec::new();
    };
    // `criteria` is sorted by document, so `first` is deterministic (NFR-006).
    let at = match &first.catch_all_example {
        Some(example) => {
            let id = example.row_id.as_deref().unwrap_or("its first criterion");
            match example.line {
                Some(line) => format!(" — for example `{id}` at {}:{line}", first.document),
                None => format!(" — for example `{id}` in {}", first.document),
            }
        }
        None => String::new(),
    };
    vec![CoverageDiagnostic {
        declaration: "criteria".to_string(),
        reason: "catch-all-universal".to_string(),
        message: format!(
            "{} of {} documents binding extractable criteria named a specific property \
             shape for none of them{at}; the extractable headline counts what a generator \
             could quantify over, and this is the part it could not tell you what to write",
            all_universal.len(),
            binding.len()
        ),
        path: Some(first.document.clone()),
        line: first
            .catch_all_example
            .as_ref()
            .and_then(|example| example.line),
        value: Some("coverage.specific_shaped".to_string()),
    }]
}

/// A ratio computed over a population the measurement could not read
/// (FR-063-AC-5).
///
/// The generalization of `no-symbol-bound`: that diagnostic knows it is about
/// the trace binder and can name the declared forms to check, and it stays,
/// because a finding that says what to look at beats one that says a number is
/// wrong. This one fires for any metric, including ones added later that have
/// no bespoke check of their own — which is the point of a schema invariant.
pub(super) fn hollow_denominators(metrics: &[Metric]) -> Vec<CoverageDiagnostic> {
    metrics
        .iter()
        .filter(|metric| metric.is_hollow())
        .filter_map(|metric| {
            // `is_hollow` is false for every `NotComputed`, so this destructure
            // always succeeds — `filter_map` states that rather than rendering
            // a half-sentence for a branch that cannot happen (CR-102).
            let crate::metric::Measurement::Measured {
                population,
                examined,
                ..
            } = metric.measurement
            else {
                return None;
            };
            Some(CoverageDiagnostic {
                declaration: "metrics".to_string(),
                reason: "hollow-denominator".to_string(),
                message: format!(
                    "`{}` published a ratio over {} {}{} but read none of the {} \
                     it walked, so the number is arithmetic over nothing; {}",
                    metric.name,
                    population,
                    metric.unit,
                    if population == 1 { "" } else { "s" },
                    examined,
                    metric.method
                ),
                path: None,
                line: None,
                value: Some(metric.name.clone()),
            })
        })
        .collect()
}

/// Declared methods that are in no catalog (FR-054-AC-11).
///
/// With no catalog the question is not computed. Otherwise emit one diagnostic
/// per distinct `(source, method)` decision, preserving byte identity for
/// modules that have not adopted the catalog (FR-050-AC-7; history in #179).
pub(super) fn uncatalogued_methods(
    obligations: &[crate::obligation::Obligation],
    registry: &Registry,
) -> Vec<CoverageDiagnostic> {
    if registry.verification_catalog().is_none() {
        return Vec::new();
    }
    let methods = registry.column_vocabulary("verification_method");
    let classes = registry.column_vocabulary("verification_class");
    let known = |declared: &str| {
        let declared = declared.trim();
        methods.iter().any(|m| m.eq_ignore_ascii_case(declared))
            || classes.iter().any(|c| c.eq_ignore_ascii_case(declared))
    };

    // (source, method) -> (row count, first document). BTreeMap so the order is
    // a property of the data rather than of the walk (NFR-006).
    let mut unknown: BTreeMap<(&str, &str), (usize, &str)> = BTreeMap::new();
    for obligation in obligations {
        let Some(method) = obligation.method.as_deref() else {
            continue;
        };
        if known(method) {
            continue;
        }
        let entry = unknown
            .entry((&obligation.source, method))
            .or_insert((0, &obligation.document));
        entry.0 += 1;
    }

    unknown
        .into_iter()
        .map(|((source, method), (rows, document))| CoverageDiagnostic {
            declaration: source.to_string(),
            reason: "uncatalogued-verification-method".to_string(),
            message: format!(
                "'{method}' is neither a declared verification_catalog method id nor a \
                 declared class, so nothing can say what discharging it means ({rows} \
                 row(s), first in '{document}'). Add a catalog entry, or write a \
                 declared method in the cell"
            ),
            path: Some(document.to_string()),
            line: None,
            // Verbatim — the same string the obligation records carry in
            // `method` — so the join is equality, not prose parsing
            // (FR-054-AC-12).
            value: Some(method.to_string()),
        })
        .collect()
}
