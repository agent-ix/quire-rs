//! Diagnostics derived after coverage reconciliation.

use std::collections::BTreeMap;

use crate::metric::Metric;
use crate::registry::Registry;

use super::{CoverageDiagnostic, CriteriaCounts};

/// Structured action contract for every registered coverage diagnostic.
///
/// The producer already owns the typed reason/value/locus facts. Keeping the
/// mapping here prevents downstream consumers from reverse-engineering them
/// from `message`, while the fallback makes an unknown future reason explicit
/// and uncertainty-shaped instead of inventing a repair.
pub(super) fn guidance_for(diagnostic: &CoverageDiagnostic) -> crate::finding::FindingGuidance {
    use crate::finding::FindingGuidance;

    let declaration = format!("declaration `{}`", diagnostic.declaration);
    let locus = match (&diagnostic.path, diagnostic.line) {
        (Some(path), Some(line)) => format!("{path}:{line}"),
        (Some(path), None) => path.clone(),
        (None, _) => declaration.clone(),
    };
    let value = diagnostic
        .value
        .as_deref()
        .unwrap_or(&diagnostic.declaration);

    match diagnostic.reason.as_str() {
        "archetype-matches-nothing" => FindingGuidance::diagnostic(
            declaration,
            "the trace-target archetype in the module manifest",
            "confirm whether the missing document is intentional; otherwise add the document or correct the declared archetype",
        ),
        "catch-all-universal" => FindingGuidance::diagnostic(
            format!("criterion represented by `{value}` at {locus}"),
            locus,
            "review whether the criterion is intentionally universal; otherwise name a concrete supported property shape",
        ),
        "hollow-denominator" => FindingGuidance::diagnostic(
            format!("metric `{value}`"),
            "the source binding census or declared trace-tag forms",
            "inspect the binding census, its unbound example, and the declared trace-tag forms before trusting or repairing the ratio",
        ),
        "id-column-matches-nothing" => FindingGuidance::remedy(
            declaration,
            locus,
            "align the declared id_column with the table's actual identifier header",
        ),
        "low-symbol-binding" => FindingGuidance::diagnostic(
            format!("{value} evidence-symbol binding"),
            locus,
            "inspect the unbound example and declared trace-tag forms to distinguish sparse tagging from a marker-form mismatch",
        ),
        "model-mints-nothing" => FindingGuidance::remedy(
            "the traceability model",
            "traceability.trace_targets in the module manifest",
            "declare at least one trace target, or remove the misleading traceability model if no ids should be minted",
        ),
        "no-symbol-bound" => FindingGuidance::diagnostic(
            format!("{value} evidence-symbol binding"),
            locus,
            "compare the example annotation with the declared trace-tag forms; correct the annotation or the declaration according to authored intent",
        ),
        "obligation-row-states-nothing" => FindingGuidance::remedy(
            declaration,
            locus,
            "fill the declared statement cell or remove the row if it states no obligation",
        ),
        "section-holds-no-table" => FindingGuidance::remedy(
            declaration,
            locus,
            "add the expected table under the matched section or correct the declaration's section selector",
        ),
        "section-matches-nothing" => FindingGuidance::remedy(
            declaration,
            locus,
            "align the document heading with the declared section selector",
        ),
        "status-column-matches-nothing" => FindingGuidance::diagnostic(
            declaration,
            locus,
            "compare the configured status column with the observed table headers; rename the document column or change the configuration, and do not guess when both are plausible",
        ),
        "tag-on-non-binding-symbol" => FindingGuidance::remedy(
            format!("trace id `{value}`"),
            locus,
            "move the trace id to an evidence symbol, or use an Implements marker when the production symbol is the intended subject",
        ),
        "untracked-id-near-miss" => FindingGuidance::remedy(
            format!("trace id `{value}`"),
            locus,
            "change the annotation to the exact minted row id",
        ),
        "untracked-id-has-minted-children" => FindingGuidance::diagnostic(
            format!("trace id `{value}`"),
            locus,
            "choose the exact minted child that this evidence verifies, or correct the authored id/declaration; do not substitute a sibling merely to clear coverage",
        ),
        "uncatalogued-verification-method" => FindingGuidance::remedy(
            format!("verification method `{value}`"),
            locus,
            "add the method to the verification catalog or write an already declared method in the cell",
        ),
        "undeclared-coverage-vocabulary" => FindingGuidance::remedy(
            declaration,
            "the vocabulary_coverage field/archetype declaration in the module manifest",
            "correct the declared field/archetype or declare the referenced schema enum",
        ),
        _ => FindingGuidance::diagnostic(
            declaration,
            locus,
            "inspect the causal evidence and producer declaration before changing the source",
        ),
    }
}

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
        guidance: None,
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
                guidance: None,
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
            guidance: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coverage::COVERAGE_DIAGNOSTIC_REASONS;

    #[test]
    fn every_registered_reason_has_complete_exclusive_guidance() {
        for reason in COVERAGE_DIAGNOSTIC_REASONS {
            let diagnostic = CoverageDiagnostic {
                declaration: "acceptance-criterion".to_string(),
                reason: (*reason).to_string(),
                message: "causal evidence".to_string(),
                path: Some("spec/requirements.md".to_string()),
                line: Some(12),
                value: Some("authored-value".to_string()),
                guidance: None,
            };
            let guidance = guidance_for(&diagnostic);
            assert!(!guidance.subject.trim().is_empty(), "{reason}: subject");
            assert!(
                !guidance.change_target.trim().is_empty(),
                "{reason}: change target"
            );

            let wire = serde_json::to_value(guidance).expect("guidance serializes");
            let actions = ["remedy", "next_diagnostic_step"]
                .iter()
                .filter(|key| wire.get(**key).is_some())
                .count();
            assert_eq!(actions, 1, "{reason}: exactly one action: {wire}");
        }
    }
}
