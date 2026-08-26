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
    // the only ecosystem-bound minting case, the section defect — left all
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
///
/// **The grading ladder is a different claim, and this test used to overstate
/// it.** AC-20 said the runner reads the level NAMES from `corpus.yaml` rather
/// than from a compiled-in list, "so a level added there is accepted without a
/// code edit". That is not achievable and was never implemented: `Level`'s three
/// variants and the assignment of each mismatch to one of them are the grading
/// rule itself, not a vocabulary — a fourth declared level would have no variant
/// to carry it and no mismatch would ever be filed under it. What this test
/// actually did was assert the declaration equalled a literal `["L1","L2","L3"]`
/// written here, which is a THIRD copy of the ladder checked against the second
/// while the first — the enum that grades — was checked against neither.
///
/// So CR-129 narrowed AC-20 to what is true and worth having: **code and
/// declaration must agree**. The comparison below is now `Level::ALL` rendered
/// through `Level::token`, in ladder order, against `grading_levels` in
/// declaration order — the enum that grades, against the file that declares.
/// Order is asserted because `Outcome::level_lost` is a MINIMUM over the ladder;
/// a declaration that reordered it would mean something different.
#[trace("TC-1021", "FR-065-AC-19")]
// the bounds enum comes from corpus.yaml.
#[trace("TC-1021", "FR-065-AC-20")]
// the compiled ladder and the declared ladder agree,
// in name and in order.
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
    // ORDERED, because the ladder's order is load-bearing.
    let ordered = |key: &str| -> Vec<String> {
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
    let levels = ordered("grading_levels");

    // Non-vacuous: an empty declaration would make every assertion below pass.
    assert!(!families.is_empty() && !kinds.is_empty() && !states.is_empty());
    // FR-065-AC-20 as narrowed by CR-129: the ladder this file GRADES with and
    // the ladder the corpus DECLARES must agree, in name and in order. Derived
    // from `Level::ALL` rather than from a literal, so this compares the enum to
    // the declaration instead of comparing two literals to each other.
    let compiled: Vec<String> = corpus_case::Level::ALL
        .iter()
        .map(|level| level.token().to_string())
        .collect();
    assert_eq!(
        levels, compiled,
        "the ladder `Level` grades with and the ladder `corpus.yaml` declares \
         have diverged. They are not derived from one another — AC-20 is an \
         agreement between a compiled enum and a declaration, and this is where \
         the disagreement surfaces.",
    );
    // The bounds enum is read by `bounds.py`, which derives its counters, its
    // sum invariant and its rejection of an undeclared state from this list
    // (FR-065-AC-19). Asserted here only as far as this reader can: the two
    // states the matrix vocabulary cannot be missing.
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
        // Source fixtures name a walker language. Declaration-only fixtures
        // use `n/a` and cannot make symbol-census claims.
        assert!(
            ["rust", "python", "typescript", "n/a"].contains(&case.meta.language.as_str()),
            "{}: language `{}` is not a corpus language",
            case.meta.id,
            case.meta.language,
        );
        if case.meta.language == "n/a" {
            assert!(
                case.expect.binding_census.is_empty(),
                "{}: a language-neutral case cannot assert a symbol census",
                case.meta.id,
            );
        }
        for census in &case.expect.binding_census {
            assert_eq!(
                census.language, case.meta.language,
                "{}: declared language and expected census disagree",
                case.meta.id,
            );
        }
    }
}

/// `CaseMeta` requires what `case_schema` says is required, and models every
/// field it declares — checked by behaviour, not by comparing two lists.
///
/// **Why not two lists.** The corpus has two readers on purpose, so that drift
/// between them is visible. An outside review checked whether it is and found
/// it was not: `bounds.py` had no required-field schema and no duplicate-id
/// check, so removing `issue_ref` from a `case.yaml` left it exiting 0 over all
/// 77 fixtures while serde refused the same tree (`agent-ix/quire-rs#336`). The
/// obvious repair — a second hand-written list of required fields in Python —
/// is the same defect one level up: two lists, free to disagree, with nothing
/// comparing them. So the list lives in `corpus.yaml` and BOTH readers are held
/// to it. `bounds.py` validates against it directly; this test proves serde
/// does the same thing, by deleting each declared-required field from a real
/// declaration and requiring the parse to fail.
///
/// It found two fields immediately. `findable` and `reproduce` carried
/// `#[serde(default)]`, so `case_schema` said required and this reader accepted
/// their absence — and one case had in fact omitted `findable`, arriving at
/// `false` from the derive rather than from an author, with nothing able to
/// tell the two apart. `tags` was the mirror image: TC-1021 has always required
/// a `TC-` id in it, so a gate required it and no declaration did.
///
/// The reverse direction inserts, for each declared field, a value wrong for
/// its declared **type**, and requires serde to refuse it. That catches a field
/// the corpus declares and this reader does not model — which
/// `deny_unknown_fields` would otherwise turn into a field no case could ever
/// carry — and a field whose Rust type disagrees with `case_schema.types`.
///
/// **What is and is not mutation-verified here.** Adding a field to
/// `case_schema` that `CaseMeta` does not model fails this test by name;
/// verified by adding one and reverting. Restoring `#[serde(default)]` on
/// `findable` fails it by name; verified the same way. The *type* half could
/// not be falsified by mutation: changing `control_for` to `Option<String>` or
/// `comment` to `Option<Vec<String>>` fails to **compile**, because every
/// declared field has a consumer that constrains it. So that assertion is a
/// backstop for a field with no such consumer, not a gate observed to fire —
/// said plainly rather than counted as verification it did not earn.
#[trace("TC-1043", "FR-065-AC-3")]
// every field `case_schema` requires is refused by the Rust reader when absent,
// and every field it declares is one `CaseMeta` models.
#[test]
fn tc1043_the_rust_reader_requires_what_the_corpus_declares_required() {
    let root = corpus_case::corpus_root();
    let declared: serde_yaml::Value =
        serde_yaml::from_str(&std::fs::read_to_string(root.join("corpus.yaml")).unwrap())
            .expect("corpus.yaml parses");
    let schema = &declared["case_schema"];
    let names = |key: &str| -> Vec<String> {
        schema[key]
            .as_sequence()
            .unwrap_or_else(|| panic!("case_schema declares `{key}`"))
            .iter()
            .map(|v| v.as_str().expect("a string").to_string())
            .collect()
    };
    let required = names("required");
    let optional = names("optional");
    assert!(
        !required.is_empty() && !optional.is_empty(),
        "an empty schema would make every assertion below pass",
    );

    // A REAL declaration, not a synthetic one: a hand-built mapping would only
    // prove things about the mapping. The first single-layout case that carries
    // every required field is the subject — a language set splits its
    // declaration across two files and no single file is complete.
    let mut subject: Option<(std::path::PathBuf, serde_yaml::Mapping)> = None;
    let mut case_files: Vec<_> = glob_case_files(&root.join("cases"));
    case_files.sort();
    for path in case_files {
        let value: serde_yaml::Value =
            serde_yaml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let Some(map) = value.as_mapping() else {
            continue;
        };
        if required
            .iter()
            .all(|f| map.contains_key(serde_yaml::Value::String(f.clone())))
        {
            subject = Some((path, map.clone()));
            break;
        }
    }
    let (path, complete) = subject.expect(
        "no single-layout case declares every required field — the subject of \
         this test does not exist, so it would pass vacuously",
    );

    // Control. Without it every deletion below could be failing for an
    // unrelated reason and the test would still be green.
    serde_yaml::from_value::<corpus_case::CaseMeta>(serde_yaml::Value::Mapping(complete.clone()))
        .unwrap_or_else(|e| {
            panic!(
                "{}: the unmutated subject does not parse: {e}",
                path.display()
            )
        });

    for field in &required {
        let mut mutated = complete.clone();
        mutated.remove(serde_yaml::Value::String(field.clone()));
        let result =
            serde_yaml::from_value::<corpus_case::CaseMeta>(serde_yaml::Value::Mapping(mutated));
        let error = match result {
            Ok(_) => panic!(
                "`case_schema.required` names `{field}`, and `CaseMeta` parses a \
                 declaration without it. One reader requires it and the other \
                 does not — which is the drift two readers exist to expose \
                 (agent-ix/quire-rs#336).",
            ),
            Err(e) => e.to_string(),
        };
        assert!(
            error.contains(field),
            "removing `{field}` was refused, but the error does not name it: {error}",
        );
    }

    // Reverse direction, twice over. For each declared field, insert a value
    // that is wrong FOR ITS DECLARED TYPE and require serde to refuse it.
    //
    // That catches both failures at once. A field the corpus declares and this
    // reader does not model reports `unknown field`, because
    // `deny_unknown_fields` would drop any case carrying it. A field whose
    // Rust type disagrees with `case_schema.types` ACCEPTS the wrong value —
    // measured, that is not hypothetical: `control_for` was written as a bare
    // string by the scaffolder, `bounds.py` (presence only) allowed it, and
    // `Option<Vec<String>>` refused it. The declared type is what the two
    // readers are held to; nothing here restates it.
    let types = &schema["types"];
    for field in required.iter().chain(optional.iter()) {
        let declared_type = &types[field.as_str()];
        // A value no reading of the declared type accepts.
        let wrong = match declared_type.as_str() {
            Some("str") | Some("bool") => serde_yaml::Value::Sequence(vec![]),
            None if declared_type.as_sequence().is_some() => {
                serde_yaml::Value::String("not-a-list".into())
            }
            other => panic!(
                "`case_schema.types` declares `{field}: {other:?}`, a type this \
                 test does not know how to write a wrong value for"
            ),
        };
        let mut mutated = complete.clone();
        mutated.insert(serde_yaml::Value::String(field.clone()), wrong);
        let error =
            serde_yaml::from_value::<corpus_case::CaseMeta>(serde_yaml::Value::Mapping(mutated))
                .err()
                .map(|e| e.to_string())
                .unwrap_or_default();
        assert!(
            !error.contains("unknown field"),
            "`case_schema` declares `{field}` and `CaseMeta` does not model it, \
             so `deny_unknown_fields` would refuse any case carrying it: {error}",
        );
        assert!(
            !error.is_empty(),
            "`case_schema.types` declares `{field}` as {declared_type:?}, and \
             `CaseMeta` accepted a value of the wrong shape. The two readers \
             disagree about this field's type (agent-ix/quire-rs#336).",
        );
    }

    // Every declared field carries a declared type. Without this a field could
    // be added to `required`/`optional` and silently skip the loop above.
    let typed: BTreeSet<String> = types
        .as_mapping()
        .expect("case_schema declares `types`")
        .keys()
        .map(|k| k.as_str().expect("a string").to_string())
        .collect();
    let named: BTreeSet<String> = required.iter().chain(optional.iter()).cloned().collect();
    assert_eq!(
        named, typed,
        "every field `case_schema` declares needs a type, and every typed field \
         needs to be declared",
    );
}

/// Every `case.yaml` under `cases/`, in either layout.
fn glob_case_files(cases: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(modes) = std::fs::read_dir(cases) else {
        return out;
    };
    for mode in modes.filter_map(Result::ok) {
        let Ok(dirs) = std::fs::read_dir(mode.path()) else {
            continue;
        };
        for dir in dirs.filter_map(Result::ok) {
            let file = dir.path().join("case.yaml");
            if file.is_file() {
                out.push(file);
            }
        }
    }
    out
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
        // No `unwrap_or_else(panic)` here any more: `reproduce` is required by
        // `case_schema` and by `CaseMeta`, so a case without one is refused at
        // deserialization, naming the file rather than reaching this loop.
        let reproduce = case.meta.reproduce.as_str();

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

        // FR-065-CON-3 / AC-15: a variant says whether it is temporary or is
        // itself the subject. Exactly one classification is allowed.
        if case.meta.module != "ecosystem" {
            assert!(
                case.meta.relaxation_ticket.is_some() ^ case.meta.declaration_under_test.is_some(),
                "{}: binds variant `{}` and must declare exactly one of \
                 `relaxation_ticket` or `declaration_under_test`",
                case.meta.id,
                case.meta.module,
            );
        } else {
            assert!(
                case.meta.relaxation_ticket.is_none() && case.meta.declaration_under_test.is_none(),
                "{}: ecosystem-bound case carries variant metadata",
                case.meta.id,
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

    // A PENDING CASE IS A BACKLOG ITEM, NOT A PRECONDITION FOR THIS GATE.
    //
    // This asserted `!pending.is_empty()` with the message "no pending case, so
    // this asserts nothing", and on 2026-08-25 the corpus reached zero pending
    // for the first time — #312, #304, #307 landed and #273 was answered by
    // #312 rather than implemented. Requiring a specimen would mean the reward
    // for fixing every known defect is a red gate, and would push the next
    // author to keep one fixture broken to satisfy it.
    //
    // What the assertion protected is the AC-26 pairing above, which runs over
    // EVERY case and is what makes a `pending:` without a forward block — or a
    // forward block without a `pending:` — fail. That loop needs no specimen.
    let pending: Vec<_> = cases.iter().filter(|c| c.meta.pending.is_some()).collect();
    assert!(
        !cases.is_empty(),
        "the corpus loaded no cases at all, so the pairing above ran over nothing"
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
    // The declared class is READ, so a corpus that deletes the key fails here
    // rather than silently changing what this gate exercises — the subjects
    // below are synthesized, but the class they stand in for is still declared.
    assert!(
        corpus.get("behaviour_change_tickets").is_some(),
        "corpus.yaml declares no `behaviour_change_tickets`. The class is real \
         even when empty — a ticket that changes what MINTS names no token — \
         and deleting it leaves the next such fixture nowhere to be declared."
    );
    // THE TOKEN SUBJECT IS SYNTHESIZED, NOT BORROWED FROM THE CORPUS.
    //
    // This used to pick a real case pending on a ticket with a reserved forward
    // token, and `.expect("… none left, so this measures nothing")` was already
    // written for the day there were none. That day arrived: `#312` and `#307`
    // landed within an hour of each other and both tokens graduated to
    // `emitted`, leaving `forward: {}` and this gate with no subject.
    //
    // A gate that only works while the corpus happens to hold a specimen is the
    // same defect one layer up — burning the backlog down would silently
    // disable the check written to protect it. So the subject is built inside
    // each scratch tree: a real failure case is made pending on a ticket that
    // does not exist, against a token no engine carries.
    let synth_ticket = "agent-ix/quire-rs#999999";
    let token = "synthetic-forward-token".to_string();
    let token_case = cases
        .iter()
        .find(|c| {
            c.meta.pending.is_none()
                && c.meta.kind == "failure"
                && c.dir.join("expect.yaml").is_file()
        })
        .expect("a non-pending failure case to make pending");
    let token_ticket = synth_ticket.to_string();
    let token_dir = token_case
        .dir
        .strip_prefix(corpus_case::corpus_root())
        .expect("under the corpus root")
        .to_string_lossy()
        .into_owned();
    // The case's own `case.yaml` — a language set keeps its `pending:` in the
    // shared file one level up, which is where the loader reads it from.
    let token_case_yaml = if token_case.dir.join("case.yaml").is_file() {
        format!("{token_dir}/case.yaml")
    } else {
        let parent = token_case
            .dir
            .parent()
            .expect("a variant has a parent")
            .strip_prefix(corpus_case::corpus_root())
            .expect("under the corpus root")
            .to_string_lossy()
            .into_owned();
        format!("{parent}/case.yaml")
    };

    // A behaviour-change case, and its OWN live block — so "identical to its
    // live block" stays identical when that block is re-measured.
    //
    // SYNTHESIZED, for the reason the token subject above is:
    // `behaviour_change_tickets` reached empty when #273 was answered by #312,
    // and a gate that only works while the backlog holds a specimen is disabled
    // by fixing everything.
    //
    // A SECOND case and a SECOND ticket, deliberately: one ticket cannot be both
    // a token ticket and a behaviour change, because the loader takes a
    // different path for each and the mutations below exercise both.
    let change_ticket = "agent-ix/quire-rs#999998";
    let change_case = cases
        .iter()
        .find(|c| {
            c.meta.pending.is_none()
                && c.meta.kind == "failure"
                && c.meta.id != token_case.meta.id
                && std::fs::read_to_string(c.dir.join("expect.yaml"))
                    .is_ok_and(|t| t.contains("\nbacked: "))
        })
        .expect("a second non-pending failure case whose live block asserts `backed`");
    let change_dir = change_case
        .dir
        .strip_prefix(corpus_case::corpus_root())
        .expect("under the corpus root")
        .to_string_lossy()
        .into_owned();
    let change_live = std::fs::read_to_string(change_case.dir.join("expect.yaml"))
        .expect("read the behaviour-change case's live block");
    let change_case_yaml = if change_case.dir.join("case.yaml").is_file() {
        format!("{change_dir}/case.yaml")
    } else {
        let parent = change_case
            .dir
            .parent()
            .expect("a variant has a parent")
            .strip_prefix(corpus_case::corpus_root())
            .expect("under the corpus root")
            .to_string_lossy()
            .into_owned();
        format!("{parent}/case.yaml")
    };
    // A CORRECT behaviour-change forward block: the same keys the live block
    // asserts, with one value moved. Identical would be rejected — "landing
    // would change nothing it can see" — which is mutation 6 below.
    // The live block reduced to its graded measurements: comments gone, and
    // diagnostic keys gone because a behaviour-change ticket adds no token and
    // a forward block naming one the engine already emits is refused by a
    // different rule — correct, and not the rule these mutations are about.
    let change_graded = {
        let mut out = String::new();
        let mut skipping = false;
        for line in change_live.lines() {
            let is_key = !line.starts_with([' ', '\t', '-', '#']) && line.contains(':');
            if is_key {
                skipping =
                    line.starts_with("diagnostic_") || line.starts_with("absent_diagnostic_");
            }
            if skipping || line.starts_with('#') {
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    };
    let change_forward = {
        let mut out = String::new();
        let mut moved = false;
        for line in change_graded.lines() {
            match line
                .strip_prefix("backed: ")
                .and_then(|n| n.parse::<usize>().ok())
            {
                Some(n) if !moved => {
                    moved = true;
                    out.push_str(&format!("backed: {}\n", n + 1));
                }
                _ => {
                    out.push_str(line);
                    out.push('\n');
                }
            }
        }
        assert!(moved, "the change case's live block asserts `backed`");
        out
    };

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
            // DERIVED from the case's own live block by dropping the one key
            // the forward block moves, rather than transcribed. The literal
            // this used to carry was shaped for whichever case happened to be
            // pending, and went vacuous the moment the subject changed — it was
            // "silent on" nothing, and the mutation was accepted.
            change_forward
                .lines()
                .filter(|l| !l.starts_with("backed: "))
                .map(|l| format!("{l}\n"))
                .collect::<String>(),
            "silent on".to_string(),
        ),
        // And one IDENTICAL to its live block: the ticket landing would change
        // nothing the fixture can see. Read from that block rather than
        // transcribed, so it stays identical when the block is re-measured.
        (
            change_pend.clone(),
            change_graded.clone(),
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
        // Build the token-pending subject in the copy: reserve the token
        // against a ticket nothing has landed, make the case pending on it, and
        // give it a forward block that is correct. Every mutation below then
        // breaks exactly one thing about a case that would otherwise load.
        let decl = scratch.join("corpus.yaml");
        let text = std::fs::read_to_string(&decl).expect("read corpus.yaml");
        std::fs::write(
            &decl,
            text.replace(
                "  forward: {}",
                &format!("  forward:\n    {token}: {synth_ticket}"),
            ),
        )
        .expect("reserve the synthetic token");
        let case_path = scratch.join(&token_case_yaml);
        let case_text = std::fs::read_to_string(&case_path).expect("read case.yaml");
        std::fs::write(
            &case_path,
            format!(
                "{case_text}pending: {synth_ticket}\npending_reason: >-\n                   A subject synthesized by TC-1025 so the gate does not depend on \
                 the corpus holding one.\n"
            ),
        )
        .expect("make the case pending");
        std::fs::write(
            scratch.join(&pend),
            format!("diagnostic_reasons: [{token}]\n"),
        )
        .expect("write a correct forward block");

        // And the behaviour-change subject: a second ticket, declared as one,
        // with a forward block that restates its live block with one value
        // moved.
        let decl2 = scratch.join("corpus.yaml");
        let text2 = std::fs::read_to_string(&decl2).expect("read corpus.yaml");
        std::fs::write(
            &decl2,
            text2.replace(
                "behaviour_change_tickets: []",
                &format!("behaviour_change_tickets:\n- {change_ticket}"),
            ),
        )
        .expect("declare the synthetic behaviour-change ticket");
        let change_path = scratch.join(&change_case_yaml);
        let change_text = std::fs::read_to_string(&change_path).expect("read case.yaml");
        std::fs::write(
            &change_path,
            format!(
                "{change_text}pending: {change_ticket}\npending_reason: >-\n                   A subject synthesized by TC-1025 so the gate does not depend on \
                 the corpus holding one.\n"
            ),
        )
        .expect("make the change case pending");
        std::fs::write(scratch.join(&change_pend), &change_forward)
            .expect("write a correct behaviour-change forward block");

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

/// The corpus vocabulary equals the engine's published registry (#300).
/// Forward tokens must stay outside it until their implementation lands.
#[trace("TC-1026", "FR-065-AC-34")]
// `emitted` names only what the engine emits.
#[trace("TC-1026", "FR-065-AC-35")]
// `forward` names only what it does not.
#[trace("TC-1026", "FR-065-AC-48")]
// Every emitted reason is exercised in both failure and healthy input.
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

    let emitted: Vec<&str> = vocabulary
        .get("emitted")
        .and_then(|v| v.as_sequence())
        .expect("`emitted`")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(
        emitted,
        quire_rs::coverage::COVERAGE_DIAGNOSTIC_REASONS,
        "corpus.yaml `emitted` must equal the engine registry",
    );

    // The same registry is present in an ordinary report, which makes the
    // contract reachable through `quire coverage --json`, not Rust-only.
    let cases = load_cases();
    let report = run(cases.first().expect("the corpus has a case"));
    let payload_registry: Vec<&str> = report
        .diagnostic_reason_registry
        .iter()
        .map(String::as_str)
        .collect();
    assert_eq!(payload_registry, emitted);

    let mut asserted_present = BTreeSet::new();
    let mut asserted_absent = BTreeSet::new();
    for case in &cases {
        asserted_present.extend(
            case.expect
                .diagnostic_reasons
                .iter()
                .map(|reason| reason.rsplit('/').next().expect("a reason")),
        );
        asserted_present.extend(
            case.expect
                .diagnostic_paths
                .keys()
                .map(|reason| reason.rsplit('/').next().expect("a reason")),
        );
        asserted_present.extend(
            case.expect
                .diagnostic_message_contains
                .keys()
                .map(|reason| reason.rsplit('/').next().expect("a reason")),
        );
        asserted_absent.extend(
            case.expect
                .absent_diagnostic_reasons
                .iter()
                .map(|reason| reason.rsplit('/').next().expect("a reason")),
        );
    }
    let emitted_set: BTreeSet<&str> = emitted.iter().copied().collect();
    let missing_present: Vec<&str> = emitted_set.difference(&asserted_present).copied().collect();
    let missing_absent: Vec<&str> = emitted_set.difference(&asserted_absent).copied().collect();
    assert!(
        missing_present.is_empty() && missing_absent.is_empty(),
        "every emitted diagnostic needs a failure assertion and a healthy-input \
         absence assertion; missing present {missing_present:?}; missing absent \
         {missing_absent:?}",
    );

    let suppressed: Vec<(&str, &str)> = vocabulary
        .get("suppressed")
        .and_then(|v| v.as_mapping())
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| Some((k.as_str()?, v.as_str()?)))
                .collect()
        })
        .unwrap_or_default();
    assert!(
        vocabulary.get("suppressed").is_some(),
        "corpus.yaml declares no `suppressed` key. The class is real even when \
         empty — a token the engine computes and discards is neither `emitted` \
         nor `forward` — and deleting it leaves the next ticket of that shape \
         nowhere to be declared."
    );
    for (token, ticket) in &suppressed {
        assert!(
            !emitted.contains(token),
            "corpus.yaml declares `{token}` both emitted and suppressed while waiting on \
             {ticket}",
        );
    }

    let forward = vocabulary
        .get("forward")
        .and_then(|v| v.as_mapping())
        .expect("`forward`");
    for (token, ticket) in forward {
        let (token, ticket) = (
            token.as_str().expect("token"),
            ticket.as_str().expect("ticket"),
        );
        assert!(
            !emitted.contains(&token),
            "corpus.yaml declares `{token}` forward, waiting on {ticket}, but the \
             engine registry already carries it. {ticket} appears to \
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
        //
        // DERIVED, not transcribed. This named `wrong-type-cell` as a literal,
        // and #286 gave that case a control — so the entry left the list, the
        // `replace` matched nothing, and the mutation was a silent no-op. A
        // mutation pinned to a specimen stops testing the moment the specimen
        // is fixed, which is the reward-for-fixing-things trap this suite has
        // now hit four times in one day.
        (
            "i = s.index('uncontrolled_failure_cases:'); \
             j = s.index('\\n    - ', i) + 1; \
             k = s.index('\\n', j) + 1; \
             s = s[:j] + s[k:]",
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
        //
        // The mutation matches the RULE's pattern (`#\\d+`), not the spelling the
        // corpus happens to use. It matched `agent-ix/<repo>#<n>` only, so a
        // reason carrying a BARE `#286` kept satisfying the check while the
        // mutation reported nothing to remove — a mutation that tests a
        // convention rather than the rule underneath it.
        (
            "import re; s = re.sub(r'(agent-ix/[a-z-]+)?#\\d+', 'because I said so', s)",
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
///
/// **AND IT EXISTED ONLY HERE.** An outside review checked whether the corpus's
/// two readers are actually independent and found that they stop immediately
/// before this rule: `verify.py` ran each case once against its own payload and
/// never cross-graded anything, and the Python loader checked only that a
/// control was NAMED — a predicate on shape, which is the class of defect this
/// check exists to close. It is implemented in both readers as of #337, and the
/// two resolutions of "which control is this case's" are now one rule, asserted
/// against each other at the end of this test rather than assumed to agree.
#[trace("TC-1028", "FR-065-AC-42", "FR-065-AC-46", "FR-065-AC-47")]
// a failure case separates itself from its control, THROUGH its mode's witness
// channel. CR-130 added AC-46 and AC-47 and pointed both at this TC in the
// spec/tests.md index, but left the marker naming AC-42 alone — so the two new
// criteria read as verified while the only machine-readable link back to a test
// did not mention them. The index and the marker are the two halves of the same
// claim; a criterion covered by one and not the other is covered by neither.
#[test]
fn tc1028_a_failure_case_discriminates_from_its_control() {
    let cases = load_cases();

    // FR-065-AC-46's channels come from `corpus.yaml`, not from this file — the
    // same single-definition rule the ladder and the mode families are under.
    let declared: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(corpus_case::corpus_root().join("corpus.yaml")).unwrap(),
    )
    .expect("corpus.yaml parses");
    let witness: BTreeMap<String, BTreeSet<String>> = declared["witness_channels"]
        .as_mapping()
        .expect("corpus.yaml declares `witness_channels`")
        .iter()
        .map(|(mode, keys)| {
            (
                mode.as_str().expect("a mode name").to_string(),
                keys.as_sequence()
                    .expect("a list of channel names")
                    .iter()
                    .map(|k| k.as_str().expect("a channel name").to_string())
                    .collect(),
            )
        })
        .collect();
    assert!(!witness.is_empty(), "an empty declaration asserts nothing");

    // EVERY DECLARED CHANNEL MUST BE ONE THIS HARNESS CAN RESTRICT ON.
    // `restricted_to` matches on key names; a name it does not know would be
    // silently dropped from the restricted block, which makes AC-46 quietly
    // WEAKER for precisely the mode that declared the channel. Failing loudly
    // is the difference between a rule and a rule-shaped hole.
    let known = corpus_case::CaseExpect::channel_names();
    for (mode, channels) in &witness {
        let unknown: Vec<_> = channels.difference(&known).cloned().collect();
        assert!(
            unknown.is_empty(),
            "`witness_channels.{mode}` names {unknown:?}, which `CaseExpect` \
             cannot restrict on — it would be dropped and the rule would silently \
             weaken for this mode",
        );
    }

    // WHAT A `control_for` NAME RESOLVES TO, and it is `bounds.py`'s
    // `failure_partners` — ID first, `case:` alias second, an alias never
    // displacing a real id.
    let mut partners: BTreeMap<(&str, &str), &corpus_case::Case> = BTreeMap::new();
    for case in cases.iter().filter(|c| c.meta.kind == "failure") {
        if let Some(alias) = case.meta.case.as_deref() {
            partners
                .entry((alias, case.meta.language.as_str()))
                .or_insert(case);
        }
    }
    for case in cases.iter().filter(|c| c.meta.kind == "failure") {
        partners.insert((case.meta.id.as_str(), case.meta.language.as_str()), case);
    }

    // `(failure id, language) -> EVERY control that names it`, resolved through
    // that map. Rewritten in #337 for two reasons, both of which made this
    // reader disagree with `bounds.py` about one corpus.
    //
    // ONE. It used to key on the raw `control_for` string and, failing to find
    // the failure case's id, fall back to the failure case's `case:` alias.
    // That handed `marker-mismatch` — which this corpus DECLARES under
    // `known_gaps.uncontrolled_failure_cases` — the control belonging to
    // `marker-form-mismatch`, whose id is that alias. So it never reached the
    // declared-gap branch below and was counted as controlled. FR-065 cites
    // "35 controlled failure cases at `3ff72c0`" from this count; `bounds.py`
    // counted **34** at the same revision, and 34 is right.
    //
    // TWO. A VEC, not one control. Two controls legitimately name
    // `marker-form-mismatch` — `marker-form-declared` and
    // `marker-form-mismatch-control` — and `insert` kept whichever came last in
    // load order while `bounds.py` would have kept the first. Grading against
    // EVERY control that names the case is both stronger and free of the
    // ordering question.
    let mut controls: BTreeMap<(&str, &str), Vec<&corpus_case::Case>> = BTreeMap::new();
    for case in cases.iter().filter(|c| c.meta.kind == "control") {
        for partner in case.meta.control_for.as_deref().unwrap_or(&[]) {
            let Some(failure) = partners.get(&(partner.as_str(), case.meta.language.as_str()))
            else {
                continue;
            };
            controls
                .entry((failure.meta.id.as_str(), failure.meta.language.as_str()))
                .or_default()
                .push(case);
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

    // EVERY (case, control) PAIR THIS HARNESS GRADED, as `id|language|control`.
    // The SET, not its cardinality — see the assertion at the end of this test
    // for why the count was not enough.
    let mut graded_pairs: Vec<String> = Vec::new();
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
        let mine = controls
            .get(&(case.meta.id.as_str(), case.meta.language.as_str()))
            .map(Vec::as_slice)
            .unwrap_or_default();
        if mine.is_empty() {
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
        }

        for control in mine {
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
                graded_pairs.push(format!(
                    "{}|{}|{}",
                    case.meta.id, case.meta.language, control.meta.id
                ));

                // FR-065-AC-46 — THE MODE-SPECIFIC WITNESS. The check above
                // proves the block tells these two payloads apart; this proves
                // it does so THROUGH THE CHANNEL THIS MODE IS ABOUT. Measured
                // by running every failure case and every control and comparing
                // `totals.total`, over the whole controlled population: 14 of
                // the 35 (case, control) pairs differ in `total` — equivalently
                // 14 of the 34 controlled cases. Without this, an incidental
                // global row count satisfies AC-42 for those while saying
                // nothing about the family the case is named for.
                //
                // Restriction, not mismatch inspection: dropping every
                // non-witness key and re-grading makes the claim exactly "the
                // witness channel itself discriminates".
                let channels = witness
                    .get(&case.meta.mode)
                    .unwrap_or_else(|| panic!("no witness channels for `{}`", case.meta.mode));
                let restricted = block.restricted_to(channels);
                if restricted.is_empty() {
                    blind.push_str(&format!(
                        "  {} — its {which} names no `{}` witness channel {channels:?}, so \
                     nothing it asserts constitutes detection of this defect family \
                     (FR-065-AC-46)\n",
                        case.meta.id, case.meta.mode,
                    ));
                    continue;
                }
                let witnessed = corpus_case::grade_against(
                    case,
                    &healthy,
                    &restricted,
                    corpus_case::ValidateSource::Tree(control),
                );
                if witnessed.passed() {
                    blind.push_str(&format!(
                        "  {} — separates itself from {} only OUTSIDE its `{}` witness \
                     channels {channels:?}; restricted to them its {which} holds against \
                     healthy input, so what it detects is not this defect family \
                     (FR-065-AC-46)\n",
                        case.meta.id, control.meta.id, case.meta.mode,
                    ));
                }
            }
        }
    }

    assert!(
        !graded_pairs.is_empty(),
        "no controlled failure case, so this asserts nothing"
    );

    // THE OTHER READER RESOLVES THE SAME PAIRS — THE SET, NOT HOW MANY.
    // `verify.py` implements this same differential (#337), and both resolve
    // `control_for` through one rule; "both implement it" is what was claimed
    // before the review found it implemented once, so the claim is a behaviour.
    //
    // IT COMPARED TWO INTEGERS UNTIL CR-132, UNDER THIS MESSAGE, and the
    // outside review reproduced the hole: reverting this test's resolution to
    // the exact pre-#337 form — `controls` keyed on the raw `control_for`
    // string, one control per key, the `case:` alias fallback restored at the
    // lookup — left `cargo test tc1028` GREEN. Both resolutions yield 35 pairs
    // at this corpus while the SETS differ by one each way:
    //
    //     new - old   ("marker-form-mismatch", "marker-form-declared")
    //     old - new   ("marker-mismatch", "marker-form-mismatch-control")
    //
    // The reverted harness grades `marker-mismatch` — a case DECLARED under
    // `known_gaps.uncontrolled_failure_cases` — against a stranger's control,
    // bypassing the declared-gap assertion above, and never grades
    // `marker-form-mismatch` against `marker-form-declared`. `bounds.py` does
    // the opposite. A cardinality check saw nothing.
    //
    // The gate caught the original defect at `3ff72c0` only because the counts
    // happened to differ there (34 vs 35). The defect class it exists for — an
    // alias colliding with an id — is CARDINALITY-PRESERVING whenever the
    // displaced case also has a control, which is the case here. So the count
    // was the one statistic this check could not use.
    let probe = std::process::Command::new("python3")
        .arg("-c")
        .arg(
            "import bounds; print('\\n'.join(sorted(\
             f\"{k[0]}|{k[1]}|{c['id']}\" for k, v in \
             bounds.controls_by_case(bounds.discover()).items() for c in v)))",
        )
        .current_dir(corpus_case::corpus_root())
        .output()
        .expect("run bounds.py's resolution");
    let stderr = String::from_utf8_lossy(&probe.stderr);
    let theirs: Vec<String> = String::from_utf8_lossy(&probe.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect();
    assert!(
        !theirs.is_empty(),
        "bounds.py resolved no (case, control) pair, so comparing the two \
         readers asserts nothing. stderr:\n{stderr}"
    );
    graded_pairs.sort();
    let mine: BTreeSet<&str> = graded_pairs.iter().map(String::as_str).collect();
    let theirs_set: BTreeSet<&str> = theirs.iter().map(String::as_str).collect();
    let only_here: Vec<&&str> = mine.difference(&theirs_set).collect();
    let only_there: Vec<&&str> = theirs_set.difference(&mine).collect();
    assert!(
        only_here.is_empty() && only_there.is_empty() && graded_pairs == theirs,
        "the two readers resolve different (case, control) PAIRS. One corpus, \
         two answers, which is the drift the duplicated implementation exists \
         to expose.\n  graded only by this harness: {only_here:?}\n  resolved \
         only by bounds.py: {only_there:?}\n  counts: this harness {}, \
         bounds.py {} — EQUAL COUNTS DO NOT MEAN EQUAL SETS, which is why this \
         compares the sorted list (CR-132).\nstderr:\n{stderr}",
        graded_pairs.len(),
        theirs.len(),
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
            if let Some(subject) = case.meta.declaration_under_test.as_deref() {
                for cell in &cells {
                    assert_eq!(cell["state"].as_str(), Some("out-of-scope"));
                    assert!(
                        cell["reason"]
                            .as_str()
                            .unwrap_or_default()
                            .contains(subject),
                        "{}: out-of-scope reason must carry its declaration-under-test reason",
                        case.meta.id,
                    );
                }
                continue;
            }
            let ticket = case
                .meta
                .relaxation_ticket
                .as_deref()
                .unwrap_or_else(|| panic!("{}: binds an unclassified variant", case.meta.id));
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
