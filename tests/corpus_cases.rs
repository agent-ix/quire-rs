//! The controlled corpus (FR-050-AC-29, FR-065, CR-098 / CR-106) —
//! `agent-ix/quire-rs#232`, `#233`, `#267`.
//!
//! One parameterized test over **`agent-ix/qa-corpus`**, pinned as a submodule
//! at `corpus/`. It was `include_str!` of one hardcoded JSON file; it is now a
//! walk of `corpus/cases/`, so adding a case is adding a directory and costs no
//! `.rs` edit.
//!
//! The inputs are read **in place**. They used to be strings materialised into
//! a tempdir under a hardcoded `module/`/`spec/`/`src/` layout, which meant no
//! case could express a `tests/` topology or exercise `source_exclude` — and
//! meant no case could be read at all without running this file.
//!
//! Each case carries an `issue_ref`. That is the bug-to-fixture link
//! (`agent-ix/quire-rs#234`) and it is required, not decorative: a fixture
//! whose origin is unrecorded becomes a fixture nobody dares change, which is
//! how a corpus rots into a set of assertions everybody works around.

mod corpus_case;

use std::collections::{BTreeMap, BTreeSet};

use ix_trace_rs::trace;

use corpus_case::{grade, grade_with, load_cases, run, Level};

#[trace("TC-992", "FR-050-AC-29")]
// marker-form mismatch, and its control. (CR-098)
#[trace("TC-993", "FR-050-AC-29")]
// a stale test name over a correct marker.
#[trace("TC-994", "FR-050-AC-29")]
// a greenfield corpus reports 0% honestly.
#[trace("TC-995", "FR-050-AC-29")]
// `implements` was never asked, not answered none.
#[trace("TC-996", "FR-050-AC-29")]
// the catch-all-only properties headline.
#[trace("TC-1011", "FR-065-AC-1")]
// read in place: no case is materialised.
#[test]
fn corpus_cases_hold() {
    let cases = load_cases();
    assert!(!cases.is_empty(), "the corpus is not empty");

    let mut failures = String::new();
    let mut pending_now_passing = String::new();
    let mut pending = 0usize;

    for case in &cases {
        // EVERY case, not the pending ones. The first version of this assert
        // sat inside the `(Some(ticket), Some(forward))` arm below, so it ran
        // for six of twenty-nine while its message stated a universal rule —
        // the same mis-scope the round before had just found twice.
        assert!(
            case.expect.asserts_something(),
            "{}: expect.yaml asserts nothing — a case that asserts nothing \
             about its own payload still counts its cell covered",
            case.meta.id,
        );
        let report = run(case);

        // `expect.yaml` is the LIVE contract and is graded the same way for
        // every case, pending or not. Previously a pending case's whole
        // expectation block was allowed to fail, so the rule became "a pending
        // fixture asserts only what is pending" and every fact that held today
        // went unasserted — `unbacked_rows` among them, which is the only
        // field distinguishing the two minting fixtures from each other.
        let outcome = grade(case, &report);
        if !outcome.passed() {
            failures.push_str(&outcome.report());
        }

        // `expect-pending.yaml` is the FORWARD contract: what the named ticket
        // will make true, and what must not be true yet.
        match (&case.meta.pending, &case.expect_pending) {
            (Some(ticket), Some(forward)) => {
                // Both readers enforce this, which is the point of there being
                // two. A block grading zero assertions trivially holds, and
                // this loop would then report the ticket as landed.
                assert!(
                    forward.asserts_something(),
                    "{}: expect-pending.yaml asserts nothing. An empty forward \
                     block always holds, which reads as `{ticket} landed`",
                    case.meta.id,
                );
                let ahead = grade_with(case, &report, forward);
                if ahead.passed() {
                    // The fix landed and the marker is now lying about the
                    // engine. Failing here is what stops a corpus filling up
                    // with stale `pending:` markers nobody revisits.
                    pending_now_passing.push_str(&format!(
                        "  {} now satisfies expect-pending.yaml — {ticket} appears to \
                         have landed. Fold it into expect.yaml and drop `pending:`.\n",
                        case.meta.id
                    ));
                } else {
                    pending += 1;
                }
            }
            // Both halves of the pairing, because either alone is a fixture
            // whose forward claim nothing grades.
            (Some(_), None) => failures.push_str(&format!(
                "{}: declares `pending:` and ships no expect-pending.yaml — the \
                 behaviour it is waiting on is asserted nowhere.\n",
                case.meta.id
            )),
            (None, Some(_)) => failures.push_str(&format!(
                "{}: ships expect-pending.yaml and declares no `pending:` — a \
                 forward claim naming no ticket.\n",
                case.meta.id
            )),
            (None, None) => {}
        }
    }

    if pending > 0 {
        // Reported, not hidden — and on STDERR, which libtest does not capture
        // on a passing run. As a `println!` this line never reached anyone on a
        // green run, so the Rust runner satisfied FR-065's "count and report
        // every pending case" only when it was already failing.
        eprintln!("{pending} case(s) pending a fix — expected to fail, and did.");
    }
    assert!(
        pending_now_passing.is_empty(),
        "a pending case started passing:\n{pending_now_passing}"
    );
    assert!(
        failures.is_empty(),
        "corpus cases lost a detection level:\n{failures}"
    );
}

#[trace("TC-1016", "FR-065-AC-11")]
// a failing case names the level it lost, and the
// deepest level it reached. Driven by MUTATING a real case's expectation
// rather than by a synthetic fixture: the claim is about the grader's reading
// of real corpus data, and a hand-built `Outcome` would assert the enum
// ordering and nothing else.
#[test]
fn tc1016_a_lost_level_is_named_and_graded() {
    let mut cases = load_cases();
    let case = cases
        .iter_mut()
        .find(|c| !c.expect.diagnostic_message_contains.is_empty())
        .expect("a case asserting an L3 message");
    let report = run(case);

    // Baseline: it passes as authored.
    assert!(
        grade(case, &report).passed(),
        "{}",
        grade(case, &report).report()
    );

    // Break ONLY the L3 assertion. L1 and L2 must still hold, so the grader
    // has to distinguish them — a grader that failed everything at once would
    // pass a test that only checked "it failed".
    let reason = case
        .expect
        .diagnostic_message_contains
        .keys()
        .next()
        .expect("a reason")
        .clone();
    case.expect.diagnostic_message_contains.insert(
        reason,
        vec!["a phrase no diagnostic will ever carry".to_string()],
    );

    let outcome = grade(case, &report);
    assert!(!outcome.passed());
    assert_eq!(outcome.level_lost(), Some(Level::L3Actionable));
    assert_eq!(outcome.level_reached(), Some(Level::L2Localised));

    let text = outcome.report();
    assert!(text.contains("LOST L3 actionable"), "{text}");
    assert!(text.contains(&outcome.issue_ref), "{text}");
    assert!(text.contains("reached L2 localised"), "{text}");
}

#[trace("TC-1016", "FR-065-AC-12")]
// losing L1 reports no level reached — the case that
// distinguishes "the detector stopped firing" from "the message got worse".
#[test]
fn tc1016_losing_l1_reports_no_level_reached() {
    let mut cases = load_cases();
    let case = cases
        .iter_mut()
        .find(|c| !c.expect.diagnostic_reasons.is_empty())
        .expect("a case asserting an L1 reason");
    let report = run(case);

    case.expect
        .diagnostic_reasons
        .push("a-reason-no-engine-emits".to_string());
    let outcome = grade(case, &report);

    assert_eq!(outcome.level_lost(), Some(Level::L1Detected));
    assert_eq!(outcome.level_reached(), None, "{}", outcome.report());
    assert!(outcome.report().contains("reached no level"));
}

/// Every case names the filing it is the regression for, and every case is
/// uniquely named — the two properties that keep the corpus navigable as it
/// grows past the point where anyone remembers all of it.
#[trace("TC-1012", "FR-065-AC-3")]
// attribution is required, not decorative.
#[test]
fn every_case_is_attributed_and_uniquely_named() {
    let cases = load_cases();
    let mut ids: Vec<&str> = cases.iter().map(|c| c.meta.id.as_str()).collect();
    ids.sort_unstable();
    let before = ids.len();
    ids.dedup();
    assert_eq!(before, ids.len(), "duplicate case id: {ids:?}");

    for case in &cases {
        assert!(
            case.meta.issue_ref.contains('#'),
            "{}: issue_ref must name a filing, got {:?}",
            case.meta.id,
            case.meta.issue_ref
        );
    }
}

/// A control names a case that exists, and does not claim a finding is
/// expected on it.
#[trace("TC-1017", "FR-065-AC-13")]
// every control's partner resolves.
#[test]
fn tc1017_every_control_names_a_case_that_exists() {
    let cases = load_cases();
    // A control pairs with its failure case IN THE SAME LANGUAGE. With language
    // sets the ids carry a `-<language>` suffix while `control_for` names the
    // case, so resolution is by (case, language) — matching on the bare id
    // would have said a perfectly-paired set names no case.
    // Resolved against a case's `id` OR its `case`. A single-language fixture
    // is named by its id; a set is named by the inventory row it claims, and a
    // fixture that reclaimed a row (#268) has an id that is neither.
    // FAILURE cases only. Building this from every case put each control's own
    // `case` into the set, so `control_for` resolved against ITSELF and the
    // check became self-satisfying: deleting the flagship failure fixture —
    // the only ecosystem-bound minting case, the 3,514-id defect — left all
    // eight tests green. `bounds.py` already skips non-failure kinds for
    // exactly this reason and carries a comment saying so; this did not.
    let mut pairs: BTreeSet<(String, &str)> = BTreeSet::new();
    for c in cases.iter().filter(|c| c.meta.kind == "failure") {
        pairs.insert((c.meta.id.clone(), c.meta.language.as_str()));
        if let Some(case) = &c.meta.case {
            pairs.insert((case.clone(), c.meta.language.as_str()));
        }
    }

    for case in cases.iter().filter(|c| c.meta.kind == "control") {
        let partners = case
            .meta
            .control_for
            .as_deref()
            .unwrap_or_else(|| panic!("{}: a control declares control_for", case.meta.id));
        assert!(
            !partners.is_empty(),
            "{}: control_for is empty — a control pairs with something",
            case.meta.id
        );
        for partner in partners {
            assert!(
                pairs.contains(&(partner.to_string(), case.meta.language.as_str())),
                "{}: control_for names `{partner}`, which is no case in {}",
                case.meta.id,
                case.meta.language
            );
        }
        // A control is input on which nothing may be found. `findable: true`
        // on one tells a recall-scoring consumer to expect a finding there.
        assert!(
            !case.meta.findable,
            "{}: a control cannot be findable",
            case.meta.id
        );
    }
}

/// The corpus is deterministic: the same case run twice produces the same
/// report. Mirrors the `filament_core` corpus's own determinism guard, which is
/// the pattern this generalizes.
#[trace("TC-1019", "FR-065-AC-17")]
// two runs over unchanged input agree byte for byte.
#[test]
fn corpus_cases_are_deterministic() {
    for case in &load_cases() {
        let first = run(case).to_json();
        let second = run(case).to_json();
        assert_eq!(first, second, "{} is not deterministic", case.meta.id);
    }
}

/// The vocabularies are read from `corpus.yaml`, not compiled in here.
///
/// This is what makes FR-065's single-definition claim checkable from ONE
/// repository. A runner carrying its own copy of the mode families agrees with
/// the corpus only by coincidence, and nothing detects the day it stops.
#[trace("TC-1021", "FR-065-AC-19")]
// the bounds enum comes from corpus.yaml.
#[trace("TC-1021", "FR-065-AC-21")]
// so do the mode families, and a case naming an
// undeclared one is rejected.
#[test]
fn tc1021_the_vocabularies_come_from_the_corpus_not_from_this_file() {
    let declared: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(corpus_case::corpus_root().join("corpus.yaml"))
            .expect("read corpus.yaml"),
    )
    .expect("corpus.yaml parses");

    let list = |key: &str| -> BTreeSet<String> {
        declared[key]
            .as_sequence()
            .unwrap_or_else(|| panic!("corpus.yaml declares `{key}`"))
            .iter()
            .map(|v| v.as_str().expect("a string").to_string())
            .collect()
    };
    let families = list("mode_families");
    let kinds = list("case_kinds");
    let states = list("bounds_states");
    let levels = list("grading_levels");

    // Non-vacuous: an empty declaration would make every assertion below pass.
    assert!(!families.is_empty() && !kinds.is_empty());
    // The ladder this file implements is the one the corpus declares. If the
    // corpus renamed a level, this file's `Level` enum would be a second
    // spelling — the exact thing FR-065-AC-20 forbids.
    assert_eq!(
        levels,
        ["L1", "L2", "L3"].iter().map(|s| s.to_string()).collect(),
        "the harness ladder and the declared ladder have diverged",
    );
    assert!(states.contains("GAP") && states.contains("covered"));

    for case in &load_cases() {
        assert!(
            families.contains(&case.meta.mode),
            "{}: mode `{}` is not a declared family {families:?}",
            case.meta.id,
            case.meta.mode,
        );
        assert!(
            kinds.contains(&case.meta.kind),
            "{}: kind `{}` is not declared {kinds:?}",
            case.meta.id,
            case.meta.kind,
        );
        // The module a case names must exist under `modules/`. The port shipped
        // `module: variants/bench-legacy` on two cases whose in-directory
        // manifest was a DIFFERENT synthetic module, so the field was a claim
        // nothing checked.
        let module = corpus_case::corpus_root()
            .join("modules")
            .join(&case.meta.module);
        // One module, or a PATH of them — `ecosystem` is the second.
        let resolves = module.join("manifest.yaml").is_file()
            || std::fs::read_dir(&module).is_ok_and(|entries| {
                entries
                    .filter_map(Result::ok)
                    .any(|e| e.path().join("manifest.yaml").is_file())
            });
        assert!(
            resolves,
            "{}: module `{}` names no manifest under {}",
            case.meta.id,
            case.meta.module,
            module.display(),
        );
        assert!(
            case.meta.tags.iter().any(|t| t.starts_with("TC-")),
            "{}: at least one tracking id, got {:?}",
            case.meta.id,
            case.meta.tags,
        );
        // The declared language is one the walker knows, and it agrees with
        // what the case's own census expects. A case labelled `python` whose
        // expectation names the `rust` census is a bounds-matrix entry filed
        // under a column it does not measure.
        assert!(
            ["rust", "python", "typescript"].contains(&case.meta.language.as_str()),
            "{}: language `{}` is not one the symbol walker reads",
            case.meta.id,
            case.meta.language,
        );
        for census in &case.expect.binding_census {
            assert_eq!(
                census.language, case.meta.language,
                "{}: declared language and expected census disagree",
                case.meta.id,
            );
        }
    }
}

/// A case's declared module and its documented invocation name the same thing.
///
/// The review of #290 found nothing cross-checked them: `verify.py` never read
/// `module:`, and this harness never read `reproduce:`. So
/// `module: ecosystem` beside `--module modules/variants/bench-legacy` was
/// accepted by both runners — each testing a *different* module, with
/// `bounds.py` crediting the cell as ecosystem-covered. That is #266's defect
/// ("the field named one thing and the file loaded another") relocated rather
/// than removed.
#[trace("TC-1018", "FR-065-AC-15")]
// a variant binding names its relaxation ticket.
#[trace("TC-1020", "FR-065-AC-18")]
// the documented invocation names the module that loads.
#[test]
fn tc1020_the_documented_invocation_names_the_module_that_loads() {
    for case in &load_cases() {
        let reproduce = case
            .meta
            .reproduce
            .as_deref()
            .unwrap_or_else(|| panic!("{}: no `reproduce` invocation", case.meta.id));

        // FR-065-AC-18: the invocation names a module. Without one no model
        // loads, the run reports 0/0, and the case cannot exhibit the
        // declaration defect it exists for.
        // A single module is selected with `--module`; a module PATH with
        // `IX_FILAMENT_MODULES_PATH`, because `--module` takes one directory.
        // Either way the invocation must NAME the module the case declares, or
        // two runners test different things and neither notices (#266).
        let by_flag = format!("--module modules/{}", case.meta.module);
        let by_path = format!("IX_FILAMENT_MODULES_PATH=modules/{}", case.meta.module);
        assert!(
            reproduce.contains(&by_flag) || reproduce.contains(&by_path),
            "{}: declares `module: {}` but its invocation says `{reproduce}`. \
             Two runners would test different modules and neither would notice.",
            case.meta.id,
            case.meta.module,
        );
        assert!(
            reproduce.contains(&format!(
                "--scope {}",
                case.dir
                    .strip_prefix(corpus_case::corpus_root())
                    .unwrap()
                    .display()
            )),
            "{}: its invocation does not scope to its own directory: {reproduce}",
            case.meta.id,
        );

        // FR-065-CON-3 / AC-15: a variant binding names the ticket sizing it.
        if case.meta.module != "ecosystem" {
            assert!(
                case.meta.relaxation_ticket.is_some(),
                "{}: binds variant `{}` and names no `relaxation_ticket`",
                case.meta.id,
                case.meta.module,
            );
        }

        // A `pending:` marker with no stated reason is one nobody can decide
        // whether to remove, which is how stale markers accumulate.
        if case.meta.pending.is_some() {
            let reason = case.meta.pending_reason.as_deref().unwrap_or("");
            assert!(
                !reason.trim().is_empty(),
                "{}: is pending with no `pending_reason`",
                case.meta.id,
            );
        }

        // The same argument as `issue_ref`: a fixture nobody explained is a
        // fixture nobody dares change.
        let comment = case.meta.comment.as_deref().unwrap_or("");
        assert!(
            !comment.trim().is_empty(),
            "{}: carries no `comment` saying what it is about",
            case.meta.id,
        );
    }
}

/// A language set's variant declares only what varies, and every reader derives
/// one identity for it.
///
/// Both properties were violated at once and neither gate noticed: a variant
/// could override `case` — silently re-pointing the cell the fixture credits,
/// with `gap_count` unmoved — and `bounds.py` honoured a variant-declared `id`
/// verbatim while this harness overwrote it, so one fixture had two identities
/// and nothing keyed on `id` could be joined across runners.
#[trace("TC-1022", "FR-065-AC-22")]
// a variant may not re-point its own case.
#[trace("TC-1022", "FR-065-AC-23")]
// one variant, one id, in every reader.
#[test]
fn tc1022_a_variant_varies_expectations_not_identity() {
    let cases = load_cases();
    let sets: Vec<_> = cases
        .iter()
        .filter(|c| {
            c.dir
                .parent()
                .is_some_and(|p| p.join("case.yaml").is_file())
        })
        .collect();
    assert!(
        !sets.is_empty(),
        "no language set in the corpus, so this asserts nothing",
    );

    for case in &sets {
        // AC-23: `<shared id>-<language>`, derived the same way by every reader.
        assert!(
            case.meta.id.ends_with(&format!("-{}", case.meta.language)),
            "{}: a variant's id must be `<shared id>-<language>`",
            case.meta.id,
        );
        // AC-22: the identity fields come from the SHARED file, so a variant
        // that declared one would be a second claim about one fact.
        let variant: serde_yaml::Value = if case.dir.join("case.yaml").is_file() {
            serde_yaml::from_str(
                &std::fs::read_to_string(case.dir.join("case.yaml")).expect("read case.yaml"),
            )
            .expect("variant case.yaml parses")
        } else {
            serde_yaml::Value::Null
        };
        // PRESENCE, not disagreement. The first version compared the two only
        // when BOTH files carried the field, so a variant could INJECT one the
        // shared file omitted and nothing fired. Measured: adding `pending:`
        // to one control variant and then breaking that control left
        // `32/32, 0 mismatches, rc 0` and the bounds matrix unmoved — the
        // control that exists to prove a check stays silent on healthy input
        // had been converted into an expected failure by one line.
        for field in ["case", "mode", "module", "kind", "pending"] {
            assert!(
                variant.get(field).is_none(),
                "{}: a variant may not declare `{field}` at all — it declares \
                 WHICH case this is, and the shared `case.yaml` is where that \
                 claim lives (FR-065-AC-22)",
                case.meta.id,
            );
        }
    }
}

/// The two expectation blocks, and the pairing between them.
///
/// `pending:` used to excuse a case's WHOLE expectation block, so the working
/// rule was "a pending fixture asserts only what is pending" and every fact
/// true today went unasserted. Measured on the two minting rows: both fixtures
/// could have regressed to minting nothing at all, in three languages, and
/// stayed green.
#[trace("TC-1023", "FR-065-AC-25")]
// the live block holds for a pending case too.
#[trace("TC-1023", "FR-065-AC-26")]
// `pending:` and `expect-pending.yaml` imply each other.
#[trace("TC-1023", "FR-065-AC-27")]
// a forward block that starts holding fails the run.
#[test]
fn tc1023_a_pending_case_still_asserts_what_is_true_today() {
    let cases = load_cases();

    // AC-26, over the real corpus, both directions.
    for case in &cases {
        assert_eq!(
            case.meta.pending.is_some(),
            case.expect_pending.is_some(),
            "{}: `pending:` and `expect-pending.yaml` imply each other — one \
             without the other is either a forward claim nothing grades or a \
             forward claim naming no ticket",
            case.meta.id,
        );
    }

    let pending: Vec<_> = cases.iter().filter(|c| c.meta.pending.is_some()).collect();
    assert!(
        !pending.is_empty(),
        "no pending case, so this asserts nothing"
    );

    for case in pending {
        let report = run(case);
        // AC-25. The live block is graded like any other case's.
        let live = grade(case, &report);
        assert!(live.passed(), "{}", live.report());
        // AC-27's precondition: the forward block does NOT hold yet. When it
        // does, `corpus_cases_hold` fails naming the ticket — asserted here so
        // a forward block that was vacuous from the start cannot hide as
        // "pending, failed as expected".
        let ahead = grade_with(
            case,
            &report,
            case.expect_pending.as_ref().expect("forward"),
        );
        assert!(
            !ahead.passed(),
            "{}: expect-pending.yaml already holds — {} appears to have landed",
            case.meta.id,
            case.meta.pending.as_deref().unwrap_or(""),
        );
    }
}

/// `unbacked_rows` and `groups` are exact in BOTH directions.
///
/// This is the pair that tells the two minting defects apart. A wrong section
/// name strands the whole table so nothing mints; a wrong id column still reads
/// it and mints a row whose identity is null. Every other key of those two
/// payloads is byte-identical, and a subset match would let either case pass on
/// the other's payload.
#[trace("TC-1024", "FR-065-AC-28")]
// exact, both directions; `[]` is an assertion.
#[test]
fn tc1024_unbacked_rows_and_groups_are_exact() {
    let mut cases = load_cases();

    // A case asserting a NON-empty `unbacked_rows`: emptying it must fail.
    let case = cases
        .iter_mut()
        .find(|c| {
            c.expect
                .unbacked_rows
                .as_ref()
                .is_some_and(|r| !r.is_empty())
        })
        .expect("a case asserting unbacked rows");
    let report = run(case);
    assert!(
        grade(case, &report).passed(),
        "{}",
        grade(case, &report).report()
    );
    case.expect.unbacked_rows = Some(Vec::new());
    let outcome = grade(case, &report);
    assert!(
        !outcome.passed(),
        "an empty list must be an assertion, not an omission"
    );
    assert_eq!(outcome.level_lost(), Some(corpus_case::Level::L2Localised));

    // And a case asserting an EMPTY one: it must reject a payload that mints.
    let mut cases = load_cases();
    let empty = cases
        .iter_mut()
        .find(|c| {
            c.expect
                .unbacked_rows
                .as_ref()
                .is_some_and(|r| r.is_empty())
        })
        .expect("a case asserting no unbacked rows");
    let report = run(empty);
    assert!(grade(empty, &report).passed());
    // The mutation is the point. The first version stopped at the line above —
    // "it passes as authored" — which `corpus_cases_hold` already guarantees
    // for every case, so no change to the grader could have failed it. This
    // feeds it the row its partner fixture really produces.
    empty.expect.unbacked_rows = Some(vec![corpus_case::ExpectUnbackedRow {
        document: "spec/tests.md".to_string(),
        row_id: None,
        target_ids: vec!["FR-001-AC-1".to_string()],
    }]);
    let outcome = grade(empty, &report);
    assert!(
        !outcome.passed(),
        "a case asserting `unbacked_rows: []` must reject a payload that mints"
    );
    assert_eq!(outcome.level_lost(), Some(corpus_case::Level::L2Localised));

    // `untracked_symbols` — the only field row 4's defect moves, and the field
    // #272's own body names as where the evidence already lives.
    let mut cases = load_cases();
    let untracked = cases
        .iter_mut()
        .find(|c| {
            c.expect
                .untracked_symbols
                .as_ref()
                .is_some_and(|u| !u.is_empty())
        })
        .expect("a case asserting untracked symbols");
    let report = run(untracked);
    assert!(grade(untracked, &report).passed());
    untracked.expect.untracked_symbols = Some(Vec::new());
    let outcome = grade(untracked, &report);
    assert!(
        !outcome.passed(),
        "an empty list must be an assertion: a defect whose only trace is \
         `untracked_symbols` disappears if `[]` is read as `unasserted`"
    );
    assert_eq!(outcome.level_lost(), Some(corpus_case::Level::L2Localised));

    // `groups` names WHAT minted, which `total` cannot: `total: 2` is satisfied
    // by any two backed ids from anywhere, so a control asserting it could not
    // say that the TC row mints at all.
    let mut cases = load_cases();
    let grouped = cases
        .iter_mut()
        .find(|c| c.expect.groups.as_ref().is_some_and(|g| g.len() > 1))
        .expect("a case asserting more than one group");
    let report = run(grouped);
    assert!(grade(grouped, &report).passed());
    grouped.expect.groups.as_mut().expect("groups").pop();
    let outcome = grade(grouped, &report);
    assert!(
        !outcome.passed(),
        "a dropped group must fail: the count is the claim"
    );
    assert_eq!(outcome.level_lost(), Some(corpus_case::Level::L1Detected));
}

/// Every substring in an L3 list is asserted, not just the first.
///
/// For a mismatch, L3 is TWO facts — what was found and what was declared — and
/// a message naming either one satisfies a single-substring assertion while
/// leaving its reader unable to act. `agent-ix/identity` has 606 ids stranded
/// on exactly this: told only "the declared id column was not found", its
/// reader edits the heading, which is already correct.
#[trace("TC-1024", "FR-065-AC-29")]
// each fragment in the list must be present.
#[test]
fn tc1024_every_l3_fragment_is_asserted() {
    let mut cases = load_cases();
    let case = cases
        .iter_mut()
        .find(|c| !c.expect.diagnostic_message_contains.is_empty())
        .expect("a case asserting an L3 message");
    let report = run(case);
    assert!(grade(case, &report).passed());

    // Append a SECOND fragment that no message carries. A grader checking only
    // the first would still pass.
    let reason = case
        .expect
        .diagnostic_message_contains
        .keys()
        .next()
        .expect("a reason")
        .clone();
    case.expect
        .diagnostic_message_contains
        .get_mut(&reason)
        .expect("fragments")
        .push("a phrase no diagnostic will ever carry".to_string());

    let outcome = grade(case, &report);
    assert!(!outcome.passed(), "every fragment in the list is asserted");
    assert_eq!(outcome.level_lost(), Some(corpus_case::Level::L3Actionable));
}

/// The loader's conformance checks, MUTATED.
///
/// The first version of this scanned an already-conformant corpus and asserted
/// it conforms — which is what `bounds.py` guarantees before any case is
/// returned, so deleting three of its four criteria left it reporting `ok`.
/// That is the identical defect this branch had just repaired in TC-1024.
///
/// Every case below drives the real loader, `bounds.py`, over a COPY of the
/// corpus with one file changed, and asserts it refuses — naming the case and
/// the thing that is wrong.
#[trace("TC-1025", "FR-065-AC-30")]
// a token in neither list is rejected.
#[trace("TC-1025", "FR-065-AC-31")]
// a live block may not require what no engine emits.
#[trace("TC-1025", "FR-065-AC-32")]
// a failure case may not assert its pending token absent.
#[trace("TC-1025", "FR-065-AC-33")]
// a forward block that asserts nothing is rejected.
#[trace("TC-1025", "FR-065-AC-39")]
// a live block that asserts nothing is rejected.
#[trace("TC-1025", "FR-065-AC-36")]
// a forward block must be ABOUT its ticket.
#[test]
fn tc1025_the_loader_refuses_a_block_that_asserts_the_wrong_thing() {
    // The case under mutation is CHOSEN AT RUNTIME, not named here.
    //
    // These mutations need a live PENDING case, and the set of those changes
    // every time a ticket ships. This test named `section-name-mismatch` until
    // #270 landed and retired its forward block; a mutation writing
    // `expect-pending.yaml` into a case with no `pending:` is then rejected for
    // THAT, not for the rule under test, and five assertions silently stopped
    // measuring what they name. Retargeting by hand just moves the expiry date.
    //
    // Two kinds are needed and they are picked separately: a TOKEN case (its
    // ticket introduces a diagnostic) and a BEHAVIOUR-CHANGE case (its ticket
    // changes what the payload says). The rules differ, so both must be
    // exercised, and neither can be a literal.
    let cases = load_cases();
    let corpus: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(corpus_case::corpus_root().join("corpus.yaml"))
            .expect("read corpus.yaml"),
    )
    .expect("corpus.yaml parses");
    let behaviour_change: BTreeSet<&str> = corpus
        .get("behaviour_change_tickets")
        .and_then(|v| v.as_sequence())
        .map(|s| s.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let forward_tokens: std::collections::BTreeMap<&str, &str> = corpus
        .get("diagnostic_reasons")
        .and_then(|v| v.get("forward"))
        .and_then(|v| v.as_mapping())
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| Some((v.as_str()?, k.as_str()?)))
                .collect()
        })
        .unwrap_or_default();

    // A token case: pending on a ticket that is NOT a behaviour change, and
    // whose ticket has a reserved forward token to name.
    let token_case = cases
        .iter()
        .filter(|c| c.meta.pending.is_some())
        .find(|c| {
            let t = c.meta.pending.as_deref().unwrap_or("");
            !behaviour_change.contains(t) && forward_tokens.contains_key(t)
        })
        .expect("a pending case on a token ticket — none left, so this measures nothing");
    let token_ticket = token_case.meta.pending.clone().expect("ticket");
    let token = forward_tokens[token_ticket.as_str()].to_string();
    let token_dir = token_case
        .dir
        .strip_prefix(corpus_case::corpus_root())
        .expect("under the corpus root")
        .to_string_lossy()
        .into_owned();

    // A behaviour-change case, and its OWN live block — so "identical to its
    // live block" stays identical when that block is re-measured.
    let change_case = cases
        .iter()
        .filter(|c| c.meta.pending.is_some())
        .find(|c| behaviour_change.contains(c.meta.pending.as_deref().unwrap_or("")))
        .expect("a pending case on a behaviour-change ticket");
    let change_dir = change_case
        .dir
        .strip_prefix(corpus_case::corpus_root())
        .expect("under the corpus root")
        .to_string_lossy()
        .into_owned();
    let change_live = std::fs::read_to_string(change_case.dir.join("expect.yaml"))
        .expect("read the behaviour-change case's live block");

    let pend = format!("{token_dir}/expect-pending.yaml");
    let live = format!("{token_dir}/expect.yaml");
    let change_pend = format!("{change_dir}/expect-pending.yaml");

    // Each: (file under the corpus root, its new contents, a fragment the
    // rejection must carry).
    let mutations: Vec<(String, String, String)> = vec![
        // AC-30 — a token in neither list.
        (
            pend.clone(),
            "diagnostic_reasons: [totally-bogus-token]\n".to_string(),
            "neither emitted nor forward".to_string(),
        ),
        // AC-31 — a LIVE block requiring what no engine emits yet.
        (
            live.clone(),
            format!("diagnostic_reasons: [{token}]\n"),
            "belongs in expect-pending.yaml".to_string(),
        ),
        // AC-32 — a FAILURE case asserting its own pending token absent. That
        // block is guaranteed to fail the day the ticket lands.
        (
            live.clone(),
            format!("total: 4\nabsent_diagnostic_reasons: [{token}]\n"),
            "a live block must survive the fix it waits for".to_string(),
        ),
        // AC-33 — non-empty as YAML, zero assertions when graded.
        (
            pend.clone(),
            "diagnostic_reasons: []\n".to_string(),
            "asserts nothing".to_string(),
        ),
        // AC-36 — merely FALSE, not ABOUT the ticket. This is the one that
        // survived two rounds of review: false today, false after the fix, and
        // the case sits pending forever with no gate saying so.
        (
            pend.clone(),
            "backed: 99\n".to_string(),
            format!("requires no token that {token_ticket} introduces"),
        ),
        // A BEHAVIOUR-CHANGE forward block silent on a key its live block
        // asserts. The rule is "the same measurement, after", so a block that
        // drops keys the live block pins is not that measurement.
        (
            change_pend.clone(),
            "total: 4\nbacked: 3\nunbacked_rows: []\n".to_string(),
            "silent on".to_string(),
        ),
        // And one IDENTICAL to its live block: the ticket landing would change
        // nothing the fixture can see. Read from that block rather than
        // transcribed, so it stays identical when the block is re-measured.
        (
            change_pend.clone(),
            change_live.clone(),
            "landing would change nothing".to_string(),
        ),
        // AC-36, the other half — an ALREADY-EMITTED token in a forward block.
        // The shape a partial landing takes: the token fires, the message is
        // not actionable, and the fixture reads as "not landed yet" forever.
        (
            pend.clone(),
            "diagnostic_reasons: [catch-all-universal]\n".to_string(),
            "which the engine already emits".to_string(),
        ),
        // A typo'd key in a forward block was GRADED, so the fixture's own
        // schema error read as evidence about the engine.
        (
            pend.clone(),
            format!("diagnostic_reason: [{token}]\n"),
            "unhandled key".to_string(),
        ),
        // The pairing, both directions (AC-26).
        (pend.clone(), String::new(), "asserts nothing".to_string()),
        // AC-39 — the LIVE block, which was checked by neither reader. This
        // is round one's defect reached by truncating the file: every gate
        // stayed green with the cell still `covered`.
        (
            live.clone(),
            String::new(),
            "expect.yaml asserts nothing".to_string(),
        ),
    ];

    for (index, (target, contents, fragment)) in mutations.iter().enumerate() {
        let scratch = scratch_dir("tc1025", index);
        copy_tree(&corpus_case::corpus_root(), &scratch);
        std::fs::write(scratch.join(target), contents).expect("write mutation");

        let run = std::process::Command::new("python3")
            .arg(scratch.join("bounds.py"))
            .output()
            .expect("run bounds.py");
        let stderr = String::from_utf8_lossy(&run.stderr).to_string();
        assert!(
            !run.status.success(),
            "mutation {index} ({target} <- {contents:?}) was ACCEPTED by the \
             loader. stdout:\n{}",
            String::from_utf8_lossy(&run.stdout),
        );
        assert!(
            stderr.contains(fragment),
            "mutation {index} was rejected, but not for the stated reason. \
             Expected a message carrying {fragment:?}, got:\n{stderr}",
        );
    }

    // And the unmutated corpus is accepted, so the assertions above are about
    // the mutations rather than about a loader that refuses everything.
    let baseline = std::process::Command::new("python3")
        .arg(corpus_case::corpus_root().join("bounds.py"))
        .output()
        .expect("run bounds.py");
    assert!(
        baseline.status.success(),
        "the corpus as committed must load:\n{}",
        String::from_utf8_lossy(&baseline.stderr),
    );
}

/// Copy a directory tree, skipping `.git`.
fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).expect("create scratch");
    for entry in std::fs::read_dir(from).expect("read tree").flatten() {
        let (source, target) = (entry.path(), to.join(entry.file_name()));
        if entry.file_name() == ".git" {
            continue;
        }
        if source.is_dir() {
            copy_tree(&source, &target);
        } else {
            std::fs::copy(&source, &target).expect("copy file");
        }
    }
}

/// The declared vocabulary is checked against the ENGINE, not hand-maintained.
///
/// Round three of this review found the defect had moved from the fixtures to
/// `corpus.yaml`: a token could be added to `emitted` that no engine emits, or
/// a token the engine already emits could be parked in `forward`, and every
/// gate stayed green. A second hand-written list drifts exactly the way the
/// first one did.
///
/// The `forward` direction is the forcing function, and it has now fired once
/// in anger. The day `#270` landed, `"section-matches-nothing"` became a
/// literal in `src/`, this test FAILED, and it stayed failing until both of
/// that ticket's tokens moved to `emitted` — which is the same edit that made
/// the nine fixtures waiting on them go green. Nobody can land the fix and
/// leave the corpus describing a world where it has not landed. The same
/// applies next to `"tag-on-non-binding-symbol"` (`#312`).
///
/// A source scan, not a registry read. Both directions are exact TODAY —
/// verified token by token, every `emitted` resolves to a literal and every
/// `forward` resolves to none — but it is a proxy: a reason assembled at
/// runtime rather than written as a literal would read as absent. The engine
/// should publish its reason registry, which is `agent-ix/quire-rs#300`.
#[trace("TC-1026", "FR-065-AC-34")]
// `emitted` names only what the engine emits.
#[trace("TC-1026", "FR-065-AC-35")]
// `forward` names only what it does not.
#[test]
fn tc1026_the_declared_vocabulary_matches_the_engine() {
    let declaration: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(corpus_case::corpus_root().join("corpus.yaml"))
            .expect("read corpus.yaml"),
    )
    .expect("corpus.yaml parses");
    let vocabulary = declaration
        .get("diagnostic_reasons")
        .expect("corpus.yaml declares `diagnostic_reasons`");

    // Every `.rs` under `src/`, concatenated once.
    let mut sources = String::new();
    let mut stack = vec![std::path::PathBuf::from("src")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read src").flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                sources.push_str(&std::fs::read_to_string(&path).expect("read source"));
            }
        }
    }
    assert!(sources.len() > 10_000, "the source scan read something");

    let emitted: Vec<&str> = vocabulary
        .get("emitted")
        .and_then(|v| v.as_sequence())
        .expect("`emitted`")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(!emitted.is_empty(), "`emitted` is not empty");
    for token in &emitted {
        assert!(
            sources.contains(&format!("\"{token}\"")),
            "corpus.yaml declares `{token}` emitted, but no literal by that name \
             appears in src/. Either the engine stopped emitting it — in which \
             case every fixture asserting it is now vacuous — or it belongs in \
             `forward` with the ticket that will add it.",
        );
    }

    // A SUPPRESSED token is the inverse of a forward one: its literal IS in
    // the engine — the finding is computed — and what the ticket changes is
    // that it reaches the payload. Asserting it is absent from `src/` would be
    // exactly backwards, so it is asserted PRESENT, like an emitted token.
    let suppressed: Vec<(&str, &str)> = vocabulary
        .get("suppressed")
        .and_then(|v| v.as_mapping())
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| Some((k.as_str()?, v.as_str()?)))
                .collect()
        })
        .unwrap_or_default();
    assert!(!suppressed.is_empty(), "`suppressed` is read");
    for (token, ticket) in &suppressed {
        assert!(
            sources.contains(&format!("\"{token}\"")),
            "corpus.yaml declares `{token}` suppressed, waiting on {ticket}, but no \
             literal by that name appears in src/. A suppressed token is one the \
             engine COMPUTES and discards — if the engine no longer has it, the \
             fixtures waiting on it are waiting for something that cannot arrive.",
        );
    }

    let forward = vocabulary
        .get("forward")
        .and_then(|v| v.as_mapping())
        .expect("`forward`");
    assert!(!forward.is_empty(), "`forward` is not empty");
    for (token, ticket) in forward {
        let (token, ticket) = (
            token.as_str().expect("token"),
            ticket.as_str().expect("ticket"),
        );
        assert!(
            !sources.contains(&format!("\"{token}\"")),
            "corpus.yaml declares `{token}` forward, waiting on {ticket}, but the \
             engine already carries a literal by that name. {ticket} appears to \
             have landed: move the token to `emitted` and fold every forward \
             block waiting on it into its expect.yaml.",
        );
    }
}

/// A control binds its partner's declaration, every failure case has one, and
/// `known_gaps` is enforced in both directions.
///
/// `known_gaps` was read by no code. It listed three uncontrolled failure
/// cases where eleven were true — a declaration nobody checked, describing a
/// corpus it had drifted from, which is the shape this corpus exists to end.
///
/// Mutated, like TC-1025: scanning a conformant corpus and asserting it
/// conforms proves nothing about the checker.
#[trace("TC-1027", "FR-065-AC-37")]
// a control binds its partner's mode and module.
#[trace("TC-1027", "FR-065-AC-38")]
// every failure case is named by some control.
#[trace("TC-1027", "FR-065-AC-40")]
// a findable case names what finds it.
#[trace("TC-1027", "FR-065-AC-41")]
// an exemption names a ticket and a case.
#[test]
fn tc1027_a_control_binds_its_partners_declaration() {
    // (a python edit applied to the copy's corpus.yaml, a fragment the refusal
    // must carry). `bounds.py`, not `verify.py`: these are corpus conformance
    // and need no engine, which is what makes them testable this way at all.
    let mutations: &[(&str, &str)] = &[
        // AC-38 — drop an allowlisted case and the violation it was covering
        // must surface.
        (
            "s = s.replace('    - wrong-type-cell\\n', '', 1)",
            "no control names it",
        ),
        // AC-37, the other direction — an entry naming no case is a declared
        // gap that has outlived its fixture.
        (
            "s = s.replace('    - gate-that-gates-nothing\\n', \
             '    - gate-that-gates-nothing\\n    - a-case-that-does-not-exist\\n', 1)",
            "outlived its fixture",
        ),
        // AC-40 — a `findable` case that names nothing which finds it. Nine
        // do; removing one from the allowlist must surface it.
        (
            "s = s.replace('    - oracle-copy\\n', '', 1)",
            "requires no finding",
        ),
        // AC-41 — an exemption with no ticket is permanent by default.
        (
            "import re; s = re.sub(r'agent-ix/[a-z-]+#\\d+', 'because I said so', s)",
            "names no ticket",
        ),
    ];

    for (index, (edit, fragment)) in mutations.iter().enumerate() {
        let scratch = scratch_dir("tc1027", index);
        copy_tree(&corpus_case::corpus_root(), &scratch);

        let script = format!(
            "import pathlib\np = pathlib.Path({:?})\ns = p.read_text()\n{edit}\np.write_text(s)\n",
            scratch.join("corpus.yaml").to_string_lossy(),
        );
        let applied = std::process::Command::new("python3")
            .arg("-c")
            .arg(&script)
            .output()
            .expect("apply mutation");
        assert!(
            applied.status.success(),
            "mutation {index} did not apply: {}",
            String::from_utf8_lossy(&applied.stderr),
        );

        let run = std::process::Command::new("python3")
            .arg(scratch.join("bounds.py"))
            .output()
            .expect("run bounds.py");
        let text = String::from_utf8_lossy(&run.stderr).to_string();
        assert!(
            !run.status.success() && text.contains(fragment),
            "mutation {index} ({edit}) was ACCEPTED, or refused for another \
             reason. Expected a message carrying {fragment:?}, got:\n{text}",
        );
    }
}

/// A scratch directory unique to this process, cleaned up whether or not the
/// test asserts its way out.
///
/// The first version used a fixed `temp_dir()/qa-corpus-<test>-<n>` and removed
/// it only after the asserts, so two concurrent `cargo test` runs — two
/// worktrees, one shared runner — collided, and a failure left 5 MB behind.
struct Scratch(std::path::PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl std::ops::Deref for Scratch {
    type Target = std::path::Path;
    fn deref(&self) -> &std::path::Path {
        &self.0
    }
}

fn scratch_dir(test: &str, index: usize) -> Scratch {
    let path =
        std::env::temp_dir().join(format!("qa-corpus-{test}-{}-{index}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    Scratch(path)
}

/// THE DIFFERENTIAL CHECK — a fixture's assertions must SEPARATE its own input
/// from its control's.
///
/// Five review rounds each found the vacuity in a new place: an excused expect
/// block, a forward block naming any token, one naming a real token with a
/// false companion, a truncated file, a live block asserting only a row count.
/// Every gate written to catch those is a predicate on the SHAPE of a
/// declaration — non-empty, token declared, ticket named, control exists — and
/// shape has an unbounded supply of forms that are non-empty and mean nothing.
/// That is the structural reason the sequence kept producing another round.
///
/// This is a predicate on DISCRIMINATION instead. Grade each failure case's
/// `expect.yaml` against its CONTROL's payload — healthy input, same tree, the
/// defect repaired — and require at least one mismatch. A block that cannot
/// tell the two apart is not about its defect, whatever its shape.
///
/// It subsumes every earlier hole at once: an empty block cannot mismatch, a
/// row count true of the corpus is true of the control too, and a fixture
/// whose `input/` was swapped for a sibling's stops separating anything.
///
/// A pending case's FORWARD block is held to the same rule, so the behaviour
/// its ticket adds must also be behaviour the control does not exhibit.
#[trace("TC-1028", "FR-065-AC-42")]
// a failure case separates itself from its control.
#[test]
fn tc1028_a_failure_case_discriminates_from_its_control() {
    let cases = load_cases();

    let mut controls: BTreeMap<(String, String), &corpus_case::Case> = BTreeMap::new();
    for case in &cases {
        if case.meta.kind != "control" {
            continue;
        }
        for partner in case.meta.control_for.as_deref().unwrap_or(&[]) {
            controls.insert((partner.clone(), case.meta.language.clone()), case);
        }
    }

    let declaration: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(corpus_case::corpus_root().join("corpus.yaml"))
            .expect("read corpus.yaml"),
    )
    .expect("corpus.yaml parses");
    let uncontrolled: BTreeSet<&str> = declaration
        .get("known_gaps")
        .and_then(|g| g.get("uncontrolled_failure_cases"))
        .and_then(|e| e.get("cases"))
        .and_then(|c| c.as_sequence())
        .map(|s| s.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let behaviour_change: BTreeSet<&str> = declaration
        .get("behaviour_change_tickets")
        .and_then(|v| v.as_sequence())
        .map(|s| s.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let mut checked = 0usize;
    let mut blind = String::new();
    for case in &cases {
        if case.meta.kind != "failure" {
            continue;
        }
        let row = case
            .meta
            .case
            .clone()
            .unwrap_or_else(|| case.meta.id.clone());
        // ID FIRST, alias second — matching `bounds.py`, which was made
        // id-first in this same commit while this twin was not. One case's
        // `case:` alias can equal another case's `id`, and measured, the
        // alias-first order graded `catch-all-properties` against
        // `catch-all-headline-control` instead of its declared `clean-control`,
        // and handed `marker-mismatch` — a DECLARED uncontrolled gap — a
        // stranger's control, bypassing the branch below and inflating the
        // controlled count from 10 to 11.
        let control = controls
            .get(&(case.meta.id.clone(), case.meta.language.clone()))
            .or_else(|| controls.get(&(row.clone(), case.meta.language.clone())));
        let Some(control) = control else {
            // No control, so there is nothing to discriminate AGAINST. That is
            // exactly why an uncontrolled failure case is a declared gap and
            // not a matter of taste: it is a fixture no rule of this kind can
            // reach. Burning `known_gaps.uncontrolled_failure_cases` down is
            // what brings these under the check (agent-ix/quire-rs#286).
            assert!(
                uncontrolled.contains(row.as_str()) || uncontrolled.contains(case.meta.id.as_str()),
                "{}: has no control and is not a declared gap",
                case.meta.id,
            );
            continue;
        };

        let healthy = run(control);
        // A BEHAVIOUR-CHANGE forward block is held to the OPPOSITE rule, and
        // it is the strongest check available to one. The control is the
        // healthy repair of its partner — for a fix that changes what MINTS,
        // that repaired tree is exactly what the engine should produce once
        // the fix lands. So the forward block must HOLD against the control's
        // payload.
        //
        // Without it the shape rule ("re-state the live block's graded keys
        // with one different value") is satisfied by `total: 999` — different
        // from today, and wrong after the fix too. Measured: accepted by the
        // loader. Against the control it fails immediately.
        //
        // A token-ticket forward block is NOT held to this. AC-36 requires it
        // to name a token AC-35 guarantees no engine emits, so it cannot hold
        // against any payload and grading it here would be a theorem restated
        // as a test.
        if let Some(forward) = &case.expect_pending {
            if behaviour_change.contains(case.meta.pending.as_deref().unwrap_or("")) {
                let ahead = corpus_case::grade_against(
                    case,
                    &healthy,
                    forward,
                    corpus_case::ValidateSource::Tree(control),
                );
                if !ahead.passed() {
                    blind.push_str(&format!(
                        "  {} — its expect-pending.yaml does NOT hold against {}'s \
                         payload. That control is the repaired tree, which is what \
                         the engine should produce once {} lands, so a forward block \
                         that fails against it describes no reachable state:\n{}",
                        case.meta.id,
                        control.meta.id,
                        case.meta.pending.as_deref().unwrap_or(""),
                        ahead.report(),
                    ));
                }
            }
        }

        // The LIVE block. Grading a TOKEN forward block here was a theorem
        // dressed as a test: AC-36 requires it to name a token AC-35 guarantees
        // no engine emits, so it cannot hold against any payload. TC-1023
        // already makes the claim that has content.
        for (block, which) in [(Some(&case.expect), "expect.yaml")] {
            let Some(block) = block else { continue };
            let verdict = corpus_case::grade_against(
                case,
                &healthy,
                block,
                corpus_case::ValidateSource::Tree(control),
            );
            if verdict.passed() {
                blind.push_str(&format!(
                    "  {} — its {which} holds against {}'s payload, so it does not \
                     separate its own input from healthy input\n",
                    case.meta.id, control.meta.id,
                ));
            }
            checked += 1;
        }
    }

    assert!(
        checked > 0,
        "no controlled failure case, so this asserts nothing"
    );
    assert!(
        blind.is_empty(),
        "a fixture's assertions must tell its own input from its control's:\n{blind}"
    );
}

/// A `regression` case: no control, not findable, pinning a LANDED ticket.
///
/// Added when the first Wave 3 fix landed. `agent-ix/quire-rs#274`'s fixture
/// had a control that was the same content written the way the BROKEN parser
/// could handle — so once the parser was fixed, both spellings parsed
/// identically and the pair had nothing left to separate. AC-42 rejected it,
/// correctly. The input is still the shape that used to break, and asserting it
/// parses is worth keeping; it is simply no longer a defect.
///
/// Not every fix leaves one behind. Where a fix makes a diagnostic FIRE, the
/// control stays silent and the pair keeps discriminating — those fixtures fold
/// their forward block in and stay `failure` cases.
#[trace("TC-1032", "FR-065-AC-43")]
// accepted with no control; findable/control_for/pending rejected.
#[trace("TC-1032", "FR-065-AC-44")]
// an ecosystem-bound one credits its cell, so a fixed defect does not revert.
#[trace("TC-1032", "FR-065-AC-45")]
// a variant-bound one credits none, and its GAP reason names the ticket.
#[test]
fn tc1032_a_regression_case_pins_a_landed_fix() {
    let cases = load_cases();
    let pins: Vec<_> = cases
        .iter()
        .filter(|c| c.meta.kind == "regression")
        .collect();
    assert!(
        !pins.is_empty(),
        "no regression case in the corpus, so this asserts nothing"
    );

    for case in &pins {
        assert!(
            !case.meta.findable,
            "{}: a regression case pins behaviour that WORKS — `findable` says a \
             finding is expected on this input, and none is",
            case.meta.id,
        );
        assert!(
            case.meta.control_for.is_none(),
            "{}: a regression case has no partner to be the control of",
            case.meta.id,
        );
        assert!(
            case.meta.pending.is_none() && case.expect_pending.is_none(),
            "{}: a regression case pins a LANDED ticket; `pending:` says the opposite",
            case.meta.id,
        );
        assert!(
            case.expect.asserts_something(),
            "{}: a regression case that asserts nothing pins nothing",
            case.meta.id,
        );
    }

    // AC-44: the cell is credited. Read the DERIVED matrix rather than
    // re-deriving it here, so this asserts what `bounds.py` actually publishes.
    let run = std::process::Command::new("python3")
        .arg(corpus_case::corpus_root().join("bounds.py"))
        .arg("--json")
        .output()
        .expect("run bounds.py --json");
    assert!(run.status.success(), "bounds.py --json must succeed");
    let derived: serde_json::Value =
        serde_json::from_slice(&run.stdout).expect("bounds --json parses");
    for case in &pins {
        let row = case
            .meta
            .case
            .clone()
            .unwrap_or_else(|| case.meta.id.clone());
        let cells: Vec<&serde_json::Value> = derived["bounds"]["matrix"]
            .as_array()
            .expect("matrix")
            .iter()
            .filter(|r| r["case"].as_str() == Some(row.as_str()))
            .map(|r| &r["cells"][&case.meta.language])
            .collect();
        // An empty `cells` makes every claim below vacuous — the variant arm
        // would assert nothing at all over zero cells.
        assert!(
            !cells.is_empty(),
            "{}: claims inventory row `{row}` in `{}`, which the derived matrix has no \
             cell for",
            case.meta.id,
            case.meta.language,
        );

        // A case binding a RELAXATION VARIANT credits no cell whatever its
        // kind — CON-3 / AC-45, because a corpus whose manifest always matches
        // cannot exhibit an ecosystem defect. That is orthogonal to
        // `regression`, which is about whether the behaviour is broken. AC-44
        // is about a cell reverting to GAP when its defect is FIXED — a
        // different thing from one that never credited.
        //
        // ASSERTED, not skipped. Until CR-123 this branch was `continue`, so
        // the narrower of the two rules was the one nothing checked: a
        // `bounds.py` that started crediting variant-bound cases would have
        // moved `gap_count` down by one with every gate green, which is the
        // "credited itself for a manifest that cannot fail" failure that
        // `agent-ix/qa-corpus` was created to end.
        if case.meta.module != "ecosystem" {
            let ticket =
                case.meta.relaxation_ticket.as_deref().unwrap_or_else(|| {
                    panic!("{}: binds a variant and names no ticket", case.meta.id)
                });
            for cell in &cells {
                assert_eq!(
                    cell["state"].as_str(),
                    Some("GAP"),
                    "{}: binds `{}` and must credit NO cell — CON-3 outranks AC-44, \
                     because a fixture on a manifest that always matches exhibits no \
                     ecosystem mode",
                    case.meta.id,
                    case.meta.module,
                );
                let reason = cell["reason"].as_str().unwrap_or_default();
                assert!(
                    reason.contains(ticket),
                    "{}: its GAP reason must name the relaxation ticket `{ticket}` that \
                     would remove the variant, so a reader of the matrix can act on it; \
                     got {reason:?}",
                    case.meta.id,
                );
            }
            continue;
        }
        assert!(
            cells
                .iter()
                .any(|cell| cell["state"].as_str() == Some("covered")),
            "{}: a regression case must credit its cell — otherwise a cell reverts to \
             GAP the moment its defect is fixed, and `gap_count` counts unfixed defects \
             rather than unmeasured modes",
            case.meta.id,
        );
    }
}
