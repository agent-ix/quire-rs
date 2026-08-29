//! Grade a coverage payload against declared corpus expectations.

use super::execution::validate_report;
use super::{
    Case, CaseExpect, ExpectGroup, ExpectUnbackedRow, ExpectUntracked, Level, Mismatch, Outcome,
};

/// Find a diagnostic by `reason`, or by `declaration/reason` when the fixture
/// needs to disambiguate two declarations emitting the same token (FR-065;
/// history in #270 and #331).
fn find_diagnostic<'a>(
    report: &'a quire_rs::CoverageReport,
    key: &str,
) -> Option<&'a quire_rs::coverage::CoverageDiagnostic> {
    let (declaration, reason) = match key.rsplit_once('/') {
        Some((d, r)) => (Some(d), r),
        None => (None, key),
    };
    // Keep this MSRV 1.75 compatible; `Option::is_none_or` is newer.
    report
        .diagnostics
        .iter()
        .find(|d| d.reason == reason && declaration.map_or(true, |want| d.declaration == want))
}

fn diagnostic_path_matches(actual: &str, expected: &str) -> bool {
    actual == expected
        || (!std::path::Path::new(expected).is_absolute()
            && std::path::Path::new(actual).ends_with(expected))
}

fn diagnostic_text(diagnostic: &quire_rs::coverage::CoverageDiagnostic) -> String {
    let mut fields = vec![diagnostic.message.as_str()];
    if let Some(guidance) = &diagnostic.guidance {
        fields.push(guidance.subject.as_str());
        fields.push(guidance.change_target.as_str());
        match &guidance.next_move {
            quire_rs::finding::FindingNextMove::Remedy { remedy } => {
                fields.push(remedy.as_str());
            }
            quire_rs::finding::FindingNextMove::NextDiagnosticStep {
                next_diagnostic_step,
            } => {
                fields.push(next_diagnostic_step.as_str());
            }
        }
    }
    fields.join(" ")
}

pub fn grade(case: &Case, report: &quire_rs::CoverageReport) -> Outcome {
    grade_with(case, report, &case.expect)
}

/// Grade one payload against an ARBITRARY expectation block, so the live
/// contract (`expect.yaml`) and the forward one (`expect-pending.yaml`) are
/// graded by the same code rather than by two that can drift.
pub fn grade_with(case: &Case, report: &quire_rs::CoverageReport, e: &CaseExpect) -> Outcome {
    grade_against(case, report, e, ValidateSource::OwnTree)
}

/// The tree used for `validate_*` assertions. Coverage payloads cannot carry
/// structural validation findings, so differential grading must validate the
/// compared tree rather than silently reusing the subject tree (FR-065;
/// historical correction in CR-132 and #286).
#[derive(Clone, Copy)]
pub enum ValidateSource<'a> {
    /// Validate the tree of the case being graded.
    OwnTree,
    /// Validate another case's tree — the control, in the differential.
    Tree(&'a Case),
}

pub fn grade_against(
    case: &Case,
    report: &quire_rs::CoverageReport,
    e: &CaseExpect,
    validate: ValidateSource<'_>,
) -> Outcome {
    let mut m: Vec<Mismatch> = Vec::new();
    // Every mismatch is born here, so this is the single runtime guard that a
    // grading level also appears in the declared ladder (FR-065-AC-20, CR-132).
    let mut fail = |level: Level, detail: String| {
        assert!(
            Level::ALL.contains(&level),
            "`{}` grades a mismatch but is absent from `Level::ALL`, so it is \
             declared in no `grading_levels` and TC-1021 cannot see it \
             (FR-065-AC-20)",
            level.as_str(),
        );
        m.push(Mismatch { level, detail })
    };

    if let Some(backed) = e.backed {
        if report.totals.backed != backed {
            fail(
                Level::L1Detected,
                format!("backed: expected {backed}, got {}", report.totals.backed),
            );
        }
    }
    if let Some(total) = e.total {
        if report.totals.total != total {
            fail(
                Level::L1Detected,
                format!("total: expected {total}, got {}", report.totals.total),
            );
        }
    }

    // Both spellings: the bare reason, and `declaration/reason` for a fixture
    // that scopes its claim to one declaration. See `find_diagnostic`.
    let mut reasons: Vec<String> = report
        .diagnostics
        .iter()
        .map(|d| d.reason.clone())
        .collect();
    reasons.extend(
        report
            .diagnostics
            .iter()
            .map(|d| format!("{}/{}", d.declaration, d.reason)),
    );
    for want in &e.diagnostic_reasons {
        if !reasons.contains(want) {
            fail(
                Level::L1Detected,
                format!("expected diagnostic `{want}`, got {reasons:?}"),
            );
        }
    }
    for unwanted in &e.absent_diagnostic_reasons {
        if reasons.contains(unwanted) {
            fail(
                Level::L1Detected,
                format!("`{unwanted}` fired on a case that is not about it: {reasons:?}"),
            );
        }
    }

    // Suspicions are a second finding channel. Grade kind at L1, locus at L2,
    // and message/evidence at L3, matching diagnostic semantics (#358).
    let kinds: Vec<&str> = report.suspicions.iter().map(|s| s.kind.as_str()).collect();
    for want in &e.suspicions {
        let Some(got) = report.suspicions.iter().find(|s| s.kind == want.kind) else {
            fail(
                Level::L1Detected,
                format!("expected suspicion `{}`, got {kinds:?}", want.kind),
            );
            continue;
        };
        if let Some(path) = &want.path {
            if &got.path != path {
                fail(
                    Level::L2Localised,
                    format!(
                        "suspicion `{}`: expected path `{path}`, got `{}`",
                        want.kind, got.path
                    ),
                );
            }
        }
        if let Some(line) = want.line {
            if got.line != line {
                fail(
                    Level::L2Localised,
                    format!(
                        "suspicion `{}`: expected line {line}, got {}",
                        want.kind, got.line
                    ),
                );
            }
        }
        if let Some(symbol) = &want.symbol {
            if &got.symbol != symbol {
                fail(
                    Level::L2Localised,
                    format!(
                        "suspicion `{}`: expected symbol `{symbol}`, got `{}`",
                        want.kind, got.symbol
                    ),
                );
            }
        }
        for fragment in &want.message_contains {
            // The EVIDENCE counts as message for this purpose. `Suspicion`
            // splits the prose from the numbers behind it, and a fixture
            // asserting "it told me 1 of 1 assertions were guarded" is making
            // an L3 claim about the same rendered output a reader sees.
            if !got.message.contains(fragment) && !got.evidence.contains(fragment) {
                fail(
                    Level::L3Actionable,
                    format!(
                        "suspicion `{}`: message and evidence name neither `{fragment}` — got `{}` / `{}`",
                        want.kind, got.message, got.evidence
                    ),
                );
            }
        }
    }
    for unwanted in &e.absent_suspicions {
        if kinds.contains(&unwanted.as_str()) {
            fail(
                Level::L1Detected,
                format!("suspicion `{unwanted}` fired on a case that is not about it: {kinds:?}"),
            );
        }
    }

    for want in &e.binding_census {
        let Some(got) = report
            .binding_census
            .iter()
            .find(|c| c.language == want.language)
        else {
            fail(
                Level::L1Detected,
                format!(
                    "no `{}` census in {:?}",
                    want.language, report.binding_census
                ),
            );
            continue;
        };
        if let Some(candidates) = want.candidates {
            if got.candidates != candidates {
                fail(
                    Level::L1Detected,
                    format!(
                        "{} candidates: expected {candidates}, got {}",
                        got.language, got.candidates
                    ),
                );
            }
        }
        if let Some(bound) = want.bound {
            if got.bound != bound {
                fail(
                    Level::L1Detected,
                    format!(
                        "{} bound: expected {bound}, got {}",
                        got.language, got.bound
                    ),
                );
            }
        }
        if let Some(tagged) = want.tagged {
            if got.tagged != tagged {
                fail(
                    Level::L1Detected,
                    format!(
                        "{} tagged: expected {tagged}, got {}",
                        got.language, got.tagged
                    ),
                );
            }
        }
        if let Some(self_named) = want.self_named {
            if got.self_named != self_named {
                fail(
                    Level::L1Detected,
                    format!(
                        "{} self_named: expected {self_named}, got {}",
                        got.language, got.self_named
                    ),
                );
            }
        }
        if let Some(self_named_bound) = want.self_named_bound {
            if got.self_named_bound != self_named_bound {
                fail(
                    Level::L1Detected,
                    format!(
                        "{} self_named_bound: expected {self_named_bound}, got {}",
                        got.language, got.self_named_bound
                    ),
                );
            }
        }
        // L2: the census names WHERE, and that is the level being claimed.
        if let Some(at) = &want.unbound_example {
            match &got.unbound_example {
                None => fail(
                    Level::L2Localised,
                    format!("{} census carries no unbound example", got.language),
                ),
                Some(example) => {
                    let actual = format!("{}:{}", example.path, example.line);
                    if actual != *at {
                        fail(
                            Level::L2Localised,
                            format!(
                                "{} unbound example: expected {at}, got {actual}",
                                got.language
                            ),
                        );
                    }
                }
            }
        }
        if let Some(at) = &want.unmatched_example {
            match &got.unmatched_example {
                None => fail(
                    Level::L2Localised,
                    format!("{} census carries no unmatched example", got.language),
                ),
                Some(example) => {
                    let actual = format!("{}:{}", example.path, example.line);
                    if actual != *at {
                        fail(
                            Level::L2Localised,
                            format!(
                                "{} unmatched example: expected {at}, got {actual}",
                                got.language
                            ),
                        );
                    }
                }
            }
        }
        if let Some(at) = &want.self_named_unbound_example {
            match &got.self_named_unbound_example {
                None => fail(
                    Level::L2Localised,
                    format!(
                        "{} census carries no self-named unbound example",
                        got.language
                    ),
                ),
                Some(example) => {
                    let actual = format!("{}:{}", example.path, example.line);
                    if actual != *at {
                        fail(
                            Level::L2Localised,
                            format!(
                                "{} self-named unbound example: expected {at}, got {actual}",
                                got.language
                            ),
                        );
                    }
                }
            }
        }
    }

    for (reason, want) in &e.diagnostic_paths {
        let actual = find_diagnostic(report, reason).and_then(|d| d.path.clone());
        if !actual
            .as_deref()
            .is_some_and(|path| diagnostic_path_matches(path, want))
        {
            fail(
                Level::L2Localised,
                format!("{reason} path: expected {want:?}, got {actual:?}"),
            );
        }
    }

    for (reason, want) in &e.diagnostic_lines {
        let actual = find_diagnostic(report, reason).and_then(|d| d.line);
        if actual != Some(*want) {
            fail(
                Level::L2Localised,
                format!("{reason} line: expected {want:?}, got {actual:?}"),
            );
        }
    }

    for (reason, fragments) in &e.diagnostic_message_contains {
        let message = find_diagnostic(report, reason).map(diagnostic_text);
        for fragment in fragments {
            match &message {
                Some(text) if text.contains(fragment.as_str()) => {}
                other => fail(
                    Level::L3Actionable,
                    format!("{reason} message lacks {fragment:?}; got {other:?}"),
                ),
            }
        }
    }

    // EXACT, both directions. A subset match would let the id-column case pass
    // on a payload that minted nothing — which is the section case, and telling
    // those two apart is the only thing this field exists for.
    if let Some(want) = &e.unbacked_rows {
        let got: Vec<ExpectUnbackedRow> = report
            .unbacked_rows
            .iter()
            .map(|r| ExpectUnbackedRow {
                document: r.document.clone(),
                row_id: r.row_id.clone(),
                target_ids: r.target_ids.clone(),
            })
            .collect();
        if &got != want {
            fail(
                Level::L2Localised,
                format!("unbacked_rows: expected {want:?}, got {got:?}"),
            );
        }
    }

    if let Some(want) = &e.untracked_symbols {
        let got: Vec<ExpectUntracked> = report
            .untracked_symbols
            .iter()
            .map(|u| ExpectUntracked {
                symbol: u.symbol.clone(),
                trace_id: u.trace_id.clone(),
                path: u.path.clone(),
            })
            .collect();
        if &got != want {
            fail(
                Level::L2Localised,
                format!("untracked_symbols: expected {want:?}, got {got:?}"),
            );
        }
    }

    if let Some(want) = &e.groups {
        let got: Vec<ExpectGroup> = report
            .groups
            .iter()
            .map(|g| ExpectGroup {
                document: g.document.clone(),
                target: g.target.clone(),
                backed: g.backed,
                total: g.total,
            })
            .collect();
        if &got != want {
            fail(
                Level::L1Detected,
                format!("groups: expected {want:?}, got {got:?}"),
            );
        }
    }

    if let Some(want) = &e.no_symbol_rows {
        let mut got: Vec<String> = report
            .no_symbol_rows
            .iter()
            .filter_map(|r| r.row_id.clone())
            .collect();
        got.sort();
        let mut want = want.clone();
        want.sort();
        if got != want {
            fail(
                Level::L1Detected,
                format!("no_symbol_rows: expected {want:?}, got {got:?}"),
            );
        }
    }

    for want in &e.metrics {
        let Some(metric) = report.metrics.iter().find(|m| m.name == want.name) else {
            fail(
                Level::L1Detected,
                format!("metric `{}` absent from the payload", want.name),
            );
            continue;
        };
        let (state, value, population, examined, matched) = match metric.measurement {
            quire_rs::metric::Measurement::Measured {
                value,
                population,
                examined,
                matched,
            } => (
                "measured",
                Some(value),
                Some(population),
                Some(examined),
                Some(matched),
            ),
            quire_rs::metric::Measurement::NotComputed { .. } => {
                ("not_computed", None, None, None, None)
            }
        };
        if let Some(w) = &want.state {
            if w != state {
                fail(
                    Level::L1Detected,
                    format!("{}.state: expected {w}, got {state}", want.name),
                );
            }
        }
        for (label, expected, actual) in [
            ("value", want.value, value),
            ("population", want.population, population),
            ("examined", want.examined, examined),
            ("matched", want.matched, matched),
        ] {
            if let Some(expected) = expected {
                if actual != Some(expected) {
                    fail(
                        Level::L1Detected,
                        format!("{}.{label}: expected {expected}, got {actual:?}", want.name),
                    );
                }
            }
        }
        if let Some(hollow) = want.hollow {
            if metric.is_hollow() != hollow {
                fail(
                    Level::L1Detected,
                    format!(
                        "{}.hollow: expected {hollow}, got {}",
                        want.name,
                        metric.is_hollow()
                    ),
                );
            }
        }
    }

    if !e.validate_contains.is_empty() || !e.validate_absent.is_empty() {
        let report = validate_report(match validate {
            ValidateSource::OwnTree => case,
            ValidateSource::Tree(other) => other,
        });
        for want in &e.validate_contains {
            if !report.contains(want.as_str()) {
                fail(Level::L1Detected, format!("validate output lacks {want:?}"));
            }
        }
        for unwanted in &e.validate_absent {
            if report.contains(unwanted.as_str()) {
                fail(
                    Level::L1Detected,
                    format!("validate reports {unwanted:?} on input that must be clean of it"),
                );
            }
        }
    }

    Outcome {
        case: case.meta.id.clone(),
        issue_ref: case.meta.issue_ref.clone(),
        mismatches: m,
    }
}
