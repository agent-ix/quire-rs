//! Diagnostics derived from trace binding rather than declaration scanning.

use std::collections::{BTreeMap, BTreeSet};

use crate::symbols::shell_trace::{ShellFinding, FUNCTION_NOT_EVIDENCE, OUTSIDE_ENTRY_PATHS};
use crate::symbols::trace::{
    quoted_id_list, BindingCensus, LegacyImplementsUse, OutsideMarkerId, OutsideMarkerOwner,
};
use crate::traceability::{SourceLanguage, TraceMarkerSyntax, TraceabilityModel};

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

/// A warning per id written in a comment or docstring attached to a declaration,
/// naming what to write instead (FR-051-AC-49, AC-22; CR-215).
///
/// Diagnostic only: nothing binds and no total moves. The text is rendered
/// from the loaded module's declared `template`, never from a form the engine
/// carries. A module declaring no marker form for the file's language gets no
/// finding, because there is nothing to write. An evidence symbol is told the
/// marker; a production function or container is told the `implements`
/// attribute when the module declares one with a template (FR-050-AC-62's
/// rendering), and its kind alone otherwise; a file header is named by kind
/// alone, because no marker of either relation can sit on it.
pub(super) fn outside_marker_diagnostics(
    found: &[OutsideMarkerId],
    model: &TraceabilityModel,
) -> Vec<CoverageDiagnostic> {
    // The first `rust_attribute` implements form declaring a template, for the
    // finding's own language (FR-051-AC-22): a Python or TypeScript function is
    // never told a Rust attribute.
    // For a Python or TypeScript symbol, the structural form of its own position
    // (FR-062-AC-26); a position with no structural form is named by kind alone.
    let implements_template = |finding: &OutsideMarkerId| {
        let wanted = match finding.language {
            SourceLanguage::Rust => Some(TraceMarkerSyntax::RustAttribute),
            _ => finding.position,
        }?;
        model
            .trace_tags
            .implements
            .iter()
            .filter(|form| form.language == finding.language && form.syntax == Some(wanted))
            .find_map(|form| form.template.as_deref())
    };
    found
        .iter()
        .filter_map(|finding| {
            let marker = model
                .trace_tags
                .markers
                .iter()
                .find(|form| form.language == finding.language)?;
            let ids = quoted_id_list(std::slice::from_ref(&finding.trace_id));
            let render = |template: &str| template.replace("{ids}", &ids);
            let here = format!(
                "`{}` is written in a comment or docstring on `{}` at {}:{}",
                finding.trace_id, finding.symbol, finding.path, finding.line
            );
            let (instruction, value) = match finding.owner {
                OutsideMarkerOwner::Evidence => match marker.template.as_deref().map(render) {
                    Some(text) => (
                        format!("a comment never binds; write `{text}` on the test instead"),
                        text,
                    ),
                    None => (
                        format!(
                            "a comment never binds; write the module's `{}` marker on the test instead",
                            marker.name
                        ),
                        marker.name.clone(),
                    ),
                },
                OutsideMarkerOwner::Production(kind) => {
                    let kind = kind.as_str();
                    match implements_template(finding).map(render) {
                        Some(text) => (
                            format!(
                                "a {kind} binds no trace id, so a `trace` marker would bind nothing; to say what it implements write `{text}`"
                            ),
                            text,
                        ),
                        None => (
                            format!("a {kind} binds no trace id, so a `trace` marker would bind nothing"),
                            kind.to_string(),
                        ),
                    }
                }
                OutsideMarkerOwner::Module => (
                    "a module binds no trace id, so a `trace` marker would bind nothing".to_string(),
                    "module".to_string(),
                ),
            };
            Some(CoverageDiagnostic {
                declaration: "traceability.trace_tags.markers".to_string(),
                reason: "trace-id-outside-marker".to_string(),
                message: format!("{here}; {instruction}"),
                path: Some(finding.path.clone()),
                line: Some(finding.line),
                value: Some(value),
                guidance: None,
            })
        })
        .collect()
}

/// One diagnostic per shell marker that matched a comment and bound nothing,
/// (FR-051-AC-44).
pub(super) fn shell_finding_diagnostics(findings: &[ShellFinding]) -> Vec<CoverageDiagnostic> {
    findings
        .iter()
        .map(|finding| {
            let message = match finding.reason {
                OUTSIDE_ENTRY_PATHS => format!(
                    "trace id `{}` sits in the header of {}:{}, but that path matches no \
                     `entry_paths` glob of a declared shell marker form, so it binds nothing",
                    finding.value, finding.path, finding.line
                ),
                FUNCTION_NOT_EVIDENCE => format!(
                    "trace id `{}` sits in a shell function's comment block at {}:{}; a shell \
                     function is production and binds no verifies",
                    finding.value, finding.path, finding.line
                ),
                _ => format!(
                    "trace id `{}` sits in a comment at {}:{} that is neither a shell \
                     function's block nor the file header, so it binds nothing",
                    finding.value, finding.path, finding.line
                ),
            };
            CoverageDiagnostic {
                declaration: "traceability.trace_tags".to_string(),
                reason: finding.reason.to_string(),
                message,
                path: Some(finding.path.clone()),
                line: Some(finding.line),
                value: Some(finding.value.clone()),
                guidance: None,
            }
        })
        .collect()
}

/// A warning per textual Rust `implements` use, naming the attribute form the
/// loaded module declares instead (FR-050-AC-62, CR-209).
///
/// Diagnostic only: the use keeps binding. Forms are identified by their
/// declared `syntax` and `template`, never by name. With no
/// `syntax: rust_attribute` implements form carrying a template there is
/// nothing to name, so nothing is reported.
pub(super) fn legacy_implements_diagnostics(
    uses: &[LegacyImplementsUse],
    model: &TraceabilityModel,
) -> Vec<CoverageDiagnostic> {
    let Some(template) = model
        .trace_tags
        .implements
        .iter()
        .filter(|form| {
            form.language == SourceLanguage::Rust
                && form.syntax == Some(TraceMarkerSyntax::RustAttribute)
        })
        .find_map(|form| form.template.as_deref())
    else {
        return Vec::new();
    };
    uses.iter()
        .map(|found| {
            let replacement = template.replace("{ids}", &quoted_id_list(&found.trace_ids));
            CoverageDiagnostic {
                declaration: "traceability.trace_tags.implements".to_string(),
                reason: "legacy-implements-form".to_string(),
                message: format!(
                    "`{}` at {}:{} implements {} through a textual comment form; it still \
                     binds, but the module declares an attribute form, so write `{replacement}` \
                     instead",
                    found.symbol,
                    found.path,
                    found.line,
                    found.trace_ids.join(", ")
                ),
                path: Some(found.path.clone()),
                line: Some(found.line),
                value: Some(replacement),
                guidance: None,
            }
        })
        .collect()
}

pub(super) fn binding_diagnostics(census: &[BindingCensus]) -> Vec<CoverageDiagnostic> {
    census
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
        .collect()
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

    #[trace("FR-050-AC-42")]
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
            strict_exempt: false,
        }
    }

    #[trace("FR-050-AC-37")]
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

    #[trace("FR-050-AC-37")]
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
