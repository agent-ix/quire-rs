//! Diagnostics derived from trace binding rather than declaration scanning.

use std::collections::{BTreeMap, BTreeSet};

use crate::symbols::trace::{BindingCensus, NonBindingTag};

use super::{CoverageDiagnostic, UnbackedRow, UntrackedSymbol};

/// Observation boundary declared by MP-201 (`coverage.binding-read-v1`).
/// It selects an uncertainty-shaped diagnostic; it is not a coverage target
/// and does not choose between sparse tagging and an unreadable convention.
const LOW_BINDING_OBSERVATION_FLOOR: f64 = 0.05;

/// A bound id that is not itself minted, beside exact descendant targets the
/// active model does mint (#328).
///
/// The engine must not join a coarse tag to every descendant—that would let
/// one `FR-006` tag back every `FR-006-AC-n` criterion. It can still name the
/// useful context from facts it already holds. A direct parent can safely name
/// exact children as the authored form. A nested unminted class must not tell
/// the author to substitute a sibling obligation merely to clear coverage.
pub(super) fn minted_child_diagnostics(
    untracked: &[UntrackedSymbol],
    minted_ids: &BTreeSet<String>,
) -> Vec<CoverageDiagnostic> {
    const SHOWN: usize = 5;
    let mut out = Vec::new();
    for symbol in untracked {
        let parts: Vec<&str> = symbol.trace_id.split('-').collect();
        let mut match_set: Vec<&str> = Vec::new();
        let mut parent = String::new();
        for length in (2..=parts.len()).rev() {
            let candidate_parent = parts[..length].join("-");
            let prefix = format!("{candidate_parent}-");
            match_set = minted_ids
                .iter()
                .map(String::as_str)
                .filter(|id| id.starts_with(&prefix))
                .collect();
            if !match_set.is_empty() {
                parent = candidate_parent;
                break;
            }
        }
        if match_set.is_empty() {
            continue;
        }
        let shown = match_set
            .iter()
            .take(SHOWN)
            .map(|id| format!("`{id}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let remainder = match_set.len().saturating_sub(SHOWN);
        let more = if remainder == 0 {
            String::new()
        } else {
            format!(" and {remainder} more")
        };
        let remedy = if parent == symbol.trace_id {
            "tag the exact child id that states the obligation this evidence verifies".to_string()
        } else {
            format!(
                "`{}` is not their parent, so do not substitute one merely to clear coverage; \
                 correct the authored id or declare a trace target for its class",
                symbol.trace_id
            )
        };
        out.push(CoverageDiagnostic {
            declaration: "traceability.trace_tags".to_string(),
            reason: "untracked-id-has-minted-children".to_string(),
            message: format!(
                "`{}` on `{}` is not a minted trace target. Under `{parent}`, the active \
                 model mints {shown}{more}; {remedy}. Nothing is joined automatically because \
                 one coarse or unrelated tag cannot back every child",
                symbol.trace_id, symbol.symbol
            ),
            path: Some(symbol.path.clone()),
            line: symbol.line,
            value: Some(symbol.trace_id.clone()),
            guidance: None,
        });
    }
    out
}

/// A trace id normalised so two spellings of one id compare equal (#307).
///
/// Upper-cased, separators dropped, and each run of digits stripped of leading
/// zeros. `TC-1`, `TC-001`, `tc_001` and `tc001` all become `TC1` (CR-136).
fn normalized_id(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    let mut digits = String::new();
    let flush = |digits: &mut String, out: &mut String| {
        let trimmed = digits.trim_start_matches('0');
        out.push_str(if trimmed.is_empty() && !digits.is_empty() {
            "0"
        } else {
            trimmed
        });
        digits.clear();
    };
    for ch in id.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
        } else {
            flush(&mut digits, &mut out);
            if ch.is_alphanumeric() {
                out.extend(ch.to_uppercase());
            }
        }
    }
    flush(&mut digits, &mut out);
    out
}

/// Diagnostics for an id that binds to nothing and a row backed by nothing,
/// one spelling apart, both already in the payload (#307).
///
/// Emit once per pair and name both spellings and loci; an exact match belongs
/// to a different defect class (FR-050-AC-37, CR-136).
pub(super) fn near_miss_diagnostics(
    untracked: &[UntrackedSymbol],
    unbacked: &[UnbackedRow],
) -> Vec<CoverageDiagnostic> {
    // Every unbacked target id, by normalised key, with the document it sits in.
    let mut rows: BTreeMap<String, (&str, &str)> = BTreeMap::new();
    for row in unbacked {
        for id in &row.target_ids {
            rows.entry(normalized_id(id))
                .or_insert((id.as_str(), row.document.as_str()));
        }
    }
    let mut out = Vec::new();
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    for symbol in untracked {
        let key = normalized_id(&symbol.trace_id);
        let Some((row_id, document)) = rows.get(&key) else {
            continue;
        };
        // An EXACT match is not a near miss. It is a different defect — the id
        // bound and the row still went unbacked — and reporting it here would
        // name two identical strings and call them a discrepancy.
        if *row_id == symbol.trace_id {
            continue;
        }
        if !seen.insert((symbol.trace_id.clone(), (*row_id).to_string())) {
            continue;
        }
        out.push(CoverageDiagnostic {
            declaration: "traceability.trace_tags".to_string(),
            reason: "untracked-id-near-miss".to_string(),
            message: format!(
                "`{}` is written on `{}` at {} and matches no minted row, while `{}` in {} is \
                 reported unbacked — the two differ only in zero-padding, case or separator, \
                 so they are the same id written twice. Both halves were already in this \
                 payload and nothing joined them; the census reads healthy because the tag \
                 bound fine, to the wrong id",
                symbol.trace_id, symbol.symbol, symbol.path, row_id, document
            ),
            path: Some(symbol.path.clone()),
            line: symbol.line,
            value: Some(symbol.trace_id.clone()),
            guidance: None,
        });
    }
    out
}

/// Diagnostics for a trace tag written where it cannot bind (#312).
///
/// Report the exact tag and symbol kind without changing binding or census
/// semantics: production symbols are not evidence (FR-051-AC-22, CR-061).
pub(super) fn non_binding_tag_diagnostics(tags: &[NonBindingTag]) -> Vec<CoverageDiagnostic> {
    tags.iter()
        .map(|tag| CoverageDiagnostic {
            declaration: "traceability.trace_tags".to_string(),
            reason: "tag-on-non-binding-symbol".to_string(),
            message: format!(
                "trace id `{}` is written on `{}` at {}:{}, a {} — a kind that does not bind \
                 trace ids (CR-061), so the tag reached no channel and the row it names is \
                 reported unbacked, indistinguishable from a test nobody wrote. The form `{}` \
                 matched, so this is an authored tag rather than prose. A production symbol \
                 records what it is about with `Implements:`; a trace id binds on an evidence \
                 symbol — a test, a benchmark or a fuzz target",
                tag.trace_id, tag.symbol, tag.path, tag.line, tag.kind, tag.form
            ),
            path: Some(tag.path.clone()),
            line: Some(tag.line),
            value: Some(tag.trace_id.clone()),
            guidance: None,
        })
        .collect()
}

pub(super) fn binding_diagnostics(census: &[BindingCensus]) -> Vec<CoverageDiagnostic> {
    let mut diagnostics: Vec<CoverageDiagnostic> = census
        .iter()
        .filter(|entry| entry.candidates > 0)
        .filter_map(|entry| {
            let forms = if entry.forms.is_empty() {
                "none declared".to_string()
            } else {
                entry.forms.join(", ")
            };
            // One unbound symbol, named. A census is a count and a count
            // cannot be opened: this diagnostic named the LANGUAGE and nothing
            // else, so a reader holding 1,292 unbound Rust symbols was told to
            // search Rust (#256). `at` is the sentence a reader can act on;
            // `path` is the same fact as a field, so `path:line` output and the
            // benchmark's positional scoring both work.
            let at = entry.unbound_example.as_ref().map(|e| {
                format!(
                    " — for example `{}` at {}:{}, whose annotation carries no \
                     matching form",
                    e.symbol, e.path, e.line
                )
            });
            let at = at.unwrap_or_default();
            let (reason, message) = if entry.bound == 0 {
                (
                    "no-symbol-bound",
                    format!(
                        "{} {} evidence symbols were examined and none carried a tag any \
                         declared form matched (forms: {forms}){at}; every row those symbols \
                         verify is reported unbacked, which is indistinguishable from a \
                         missing test",
                        entry.candidates, entry.language
                    ),
                )
            } else if (entry.bound as f64)
                < (entry.candidates as f64) * LOW_BINDING_OBSERVATION_FLOOR
            {
                (
                    "low-symbol-binding",
                    format!(
                        "{} of {} {} evidence symbols bound a trace id (forms: {forms}){at}; \
                         below {}% this observation cannot distinguish sparse tagging \
                         from a marker-form mismatch; inspect the unbound examples and \
                         declared forms",
                        entry.bound,
                        entry.candidates,
                        entry.language,
                        (LOW_BINDING_OBSERVATION_FLOOR * 100.0) as usize
                    ),
                )
            } else {
                return None;
            };
            Some(CoverageDiagnostic {
                declaration: "traceability.trace_tags".to_string(),
                reason: reason.to_string(),
                message,
                path: entry.unbound_example.as_ref().map(|e| e.path.clone()),
                line: entry.unbound_example.as_ref().map(|e| e.line),
                value: Some(entry.language.clone()),
                guidance: None,
            })
        })
        .collect();
    diagnostics.extend(census.iter().filter_map(|entry| {
        if entry.self_named == 0 || entry.self_named_bound > 0 {
            return None;
        }
        let example = entry.self_named_unbound_example.as_ref()?;
        Some(CoverageDiagnostic {
            declaration: "traceability.trace_tags".to_string(),
            reason: "marker-form-mismatch".to_string(),
            message: format!(
                "{} {} evidence symbols carry an id in their own name and no declared name form read one; for example `{}` at {}:{} (declared forms: {}). Other tag forms may bind in the same language, so the aggregate binding census cannot clear this subpopulation",
                entry.self_named,
                entry.language,
                example.symbol,
                example.path,
                example.line,
                entry.forms.join(", ")
            ),
            path: Some(example.path.clone()),
            line: Some(example.line),
            value: Some(entry.language.clone()),
            guidance: None,
        })
    }));
    diagnostics
}

#[cfg(test)]
mod cr137_minted_children {
    use std::collections::BTreeSet;

    use ix_trace_rs::trace;

    use super::{minted_child_diagnostics, UntrackedSymbol};

    fn symbol(trace_id: &str) -> UntrackedSymbol {
        UntrackedSymbol {
            path: "src/lib.rs".to_string(),
            symbol: "tests::criterion_evidence".to_string(),
            trace_id: trace_id.to_string(),
            line: Some(12),
        }
    }

    #[trace("TC-1077", "FR-050-AC-42")]
    #[test]
    fn tc1077_an_unminted_parent_names_real_children_without_backing_them() {
        let minted = BTreeSet::from([
            "FR-001-AC-1".to_string(),
            "FR-001-AC-2".to_string(),
            "TC-001".to_string(),
        ]);
        let finding = minted_child_diagnostics(&[symbol("FR-001")], &minted)
            .into_iter()
            .next()
            .expect("the model can name the exact children it mints");
        assert_eq!(finding.reason, "untracked-id-has-minted-children");
        assert_eq!(finding.path.as_deref(), Some("src/lib.rs"));
        assert_eq!(finding.line, Some(12));
        assert!(finding.message.contains("FR-001-AC-1"));
        assert!(finding.message.contains("FR-001-AC-2"));
        assert!(finding.message.contains("cannot back every child"));

        assert!(
            minted_child_diagnostics(&[symbol("TC-999")], &minted).is_empty(),
            "an unrelated typo has no model-grounded repair to invent"
        );

        let nested = minted_child_diagnostics(&[symbol("FR-001-INV-1")], &minted);
        assert_eq!(nested.len(), 1, "the nearest useful ancestor is FR-001");
        assert!(nested[0].message.contains("Under `FR-001`"));
        assert!(nested[0]
            .message
            .contains("declare a trace target for its class"));
        assert!(!nested[0].message.contains("tag the exact child id"));
    }
}

#[cfg(test)]
mod cr136_near_miss {
    use ix_trace_rs::trace;

    use super::{near_miss_diagnostics, normalized_id, UnbackedRow, UntrackedSymbol};

    fn symbol(trace_id: &str) -> UntrackedSymbol {
        UntrackedSymbol {
            path: "src/lib.rs".to_string(),
            symbol: "tests::tc_1_every_finding_defaults_to_warning".to_string(),
            trace_id: trace_id.to_string(),
            line: Some(5),
        }
    }

    fn row(target: &str) -> UnbackedRow {
        UnbackedRow {
            reference: "test-case".to_string(),
            document: "spec/tests.md".to_string(),
            row_id: Some(target.to_string()),
            target_ids: vec![target.to_string()],
            line: None,
        }
    }

    #[trace("TC-1050", "FR-050-AC-37")]
    // zero-padding, case and separator are one class, and
    // the normalisation collapses all three onto one key.
    #[test]
    fn tc1050_one_id_written_four_ways_normalises_to_one_key() {
        let key = normalized_id("TC-001");
        for spelling in ["TC-1", "tc_001", "tc001", "Tc-0001", "TC-1"] {
            assert_eq!(
                normalized_id(spelling),
                key,
                "`{spelling}` is `TC-001` written differently"
            );
        }
        // And ids that genuinely differ do NOT collide, or the join would
        // manufacture pairs out of unrelated rows.
        assert_ne!(normalized_id("TC-001"), normalized_id("TC-010"));
        assert_ne!(normalized_id("TC-001"), normalized_id("FR-001"));
        // A run of zeros is a zero, not an empty string — otherwise `TC-000`
        // and `TC-` would be the same id.
        assert_ne!(normalized_id("TC-000"), normalized_id("TC-"));
    }

    #[trace("TC-1051", "FR-050-AC-37")]
    // a near miss is reported naming BOTH spellings, and an
    // EXACT match is not reported at all.
    #[test]
    fn tc1051_a_near_miss_is_reported_and_an_exact_match_is_not() {
        // The defect: `fn tc_1_…` mints `TC-1` while the row declares `TC-001`.
        // Both halves are in the payload and nothing joined them.
        let out = near_miss_diagnostics(&[symbol("TC-1")], &[row("TC-001")]);
        assert_eq!(out.len(), 1, "one pair, one diagnostic: {out:?}");
        assert_eq!(out[0].reason, "untracked-id-near-miss");
        // BOTH strings. "An id did not match" is useless here — the whole
        // defect is that the two look identical until you count zeros, so a
        // message naming one of them sends its reader to the wrong file.
        assert!(out[0].message.contains("TC-1"), "{}", out[0].message);
        assert!(out[0].message.contains("TC-001"), "{}", out[0].message);
        assert_eq!(out[0].path.as_deref(), Some("src/lib.rs"));

        // AN EXACT MATCH IS A DIFFERENT DEFECT. The id bound and the row still
        // went unbacked; reporting it here would print two identical strings
        // and call them a discrepancy.
        assert!(
            near_miss_diagnostics(&[symbol("TC-001")], &[row("TC-001")]).is_empty(),
            "an id that matches its row exactly is not a near miss"
        );

        // And an id matching no row at all is left alone — that is
        // `untracked_symbols` doing its job, not a near miss.
        assert!(near_miss_diagnostics(&[symbol("TC-1")], &[row("FR-002")]).is_empty());
    }
}
