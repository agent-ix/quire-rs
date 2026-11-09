---
id: SR-16437
title: "Code review — PLAT-847 only marker forms bind, trace-id-outside-marker advisory (quire-rs PR 627)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@6f257a9; git diff origin/main...HEAD (80 files): src/symbols/trace.rs, src/coverage/binding_diagnostics.rs, src/coverage/reconcile.rs, src/coverage/diagnostics.rs, src/coverage.rs, src/traceability.rs, src/loader/mod.rs, src/assurance.rs, schemas/output/*.json, the 197-symbol retag (commit 2ad3e38), tests/it/outside_marker_advisory.rs and the other changed tests"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references}
---

## Summary

Ticket: PLAT-847. PR head: `6f257a9`. This is a code review with the rust-review lane folded in. The PR implements CR-215: only a marker form binds a `verifies` relation. One advisory, `trace-id-outside-marker`, reports every id-shaped token in a comment or docstring attached to a declaration. `TraceLegacyForm`, `trace_tags.legacy`, `id_format` capture reconstruction, `rewrite_to`, `RewriteSuggestion`, `TraceProvenance`, `SymbolGraph.rewrites`, `non_binding_tags`, the `self_named*` census fields and the `marker-form-mismatch` and `tag-on-non-binding-symbol` reasons are deleted. 197 symbols were retagged to `#[trace]` / `@pytest.mark.trace`. The `corpus` submodule pointer points at the qa-corpus#49 branch head, as expected, and is not treated as vendoring.

## Verdict

**FAIL: two medium and three low findings. Each fix is small.**

- **Deletion completeness (point 1).** Clean in code, the schemas and the loader. No `TraceLegacyForm`, `provenance` on `VerifiesRelation` or the assurance `verifiesRelation`, `rewrite_to`, `self_named`, `non_binding_tags`, `marker-form-mismatch` or `tag-on-non-binding-symbol` path remains in `src/`, `schemas/`, `benches/`, `examples/`, `fuzz/` or `tools/`. Repository-wide grep hits on `id_format`, `provenance` and `legacy` belong to other features: obligation `id_format`, the `RecordBinder` and metric provenance, `legacy-implements-form` (FR-050-AC-62) and the semantic `legacy_forms`. `TraceTagGrammar` keeps `#[serde(deny_unknown_fields)]`, so a manifest that still declares `trace_tags.legacy` fails to load with serde's `unknown field` error naming the key, as AC-43 requires. `merge_traceability`, `is_empty`, `validate` and `project_relation_kinds` lost their legacy branches cleanly. Two stale doc references remain (FND-003).
- **Retag (point 2).** The retag added 194 `#[trace(...)]` lines and 22 `@pytest.mark.trace(...)` lines, and no `#[implements]`. That is correct, because a non-marker binding could only ever exist on an evidence symbol. Every added marker sits on a `#[test]` function, a `proptest!` test (the Rust adapter scans `proptest!` token trees), the criterion-registered `bench_validate_document`, or the `fuzz_target!` item. None sits on a production function. All 234 distinct ids in the retag commit were checked against the ids defined in `spec/`, and 232 exist. The 2 that do not are mis-tags carried over mechanically from spurious legacy bindings (FND-002). About 25 retags were read against their test bodies (FR-002-AC-7, FR-013-AC-3/15/16/17, TC-526, TC-577, TC-579, TC-609 ×6, TC-770/771, TC-829, TC-1038, TC-1635/1644 and others), and each id matches what the test asserts. Two markers on non-test items predate this PR and are also present on origin/main (FND-005).
- **Advisory (points 3, 4, 6).** `outside_marker_ids` reads the annotation channel only: comments everywhere, plus a positional Python docstring. Code, string contents and symbol bodies are blanked, and each byte and line keeps its position. Each symbol claims `leading_line..=line` plus its docstring; the innermost symbol wins and a container loses a tie. Ranges and ids read by an `implements` form are excluded. One finding is made per (line, id). This matches AC-22, AC-24, AC-49 and AC-50. The production-symbol replacement text has a language defect (FND-001). Shell files return `None` from `annotation_channel`, so they never reach the advisory, as AC-49's last sentence requires ("a shell comment is reported only under AC-44"). The noise question is reported in the gap-analysis SR (SR-16438), not as a defect here.
- **Rust idioms.** `OutsideMarkerOwner` is an enum rather than flags, the code borrows rather than clones in the per-file scan, `BTreeMap` keeps output order deterministic, and no `unwrap` is added to library code. `CensusAccumulator::finish` keeps its pre-existing `assert!` invariant. No ceremony was added.
- **Checks run** (worktree target, debug=0, jobs 4, via locked-build.sh). `cargo test --lib -- symbols:: coverage:: traceability:: assurance` gave 162 ok. `cargo test --test it -- outside_marker trace_dogfood legacy_implements shell_symbols tc2110 corpus_case coverage_baseline` gave 54 ok. clippy and fmt were not re-run; the coder reports both clean.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `outside_marker_diagnostics` computes one `implements_template` from the Rust `syntax: rust_attribute` forms and applies it to every `OutsideMarkerOwner::Production` finding of any language. A Python or TypeScript production function or class is therefore told to write `#[implements("FR-007-AC-2")]`, a Rust attribute that cannot bind there. FR-050-AC-62, whose rendering AC-22 cites, is Rust-only. Fix: choose the template per finding with `form.language == finding.language && form.syntax == Some(RustAttribute)` (non-Rust languages fall back to the kind alone), and add Python and TypeScript production-function cases to TC-2165 that assert `value == "function"` / `"container"`. | src/coverage/binding_diagnostics.rs:189-197, 226-240 |
| FND-002 | medium | The retag carried two spurious legacy bindings forward as markers. `#[trace("TC-1")]` on `tc1051_a_near_miss_is_reported_and_an_exact_match_is_not`: `TC-1` is the test's input string (the `fn tc_1_…` near-miss example) and is not a spec id, since the test verifies TC-1051. `#[trace("FR-152")]` on `part_owner_and_allocation_operation_source_admit_any_declared_kind`: FR-152 is not an id in this repository (it is an external spec's FR cited in prose), and the test already carries `TC-1872`/`FR-075-AC-12`. Each mints a `verifies` relation to an id that no row has, so it lands in `untracked_symbols`. Delete both markers. | src/coverage/binding_diagnostics.rs:507; tests/it/semantic_systems.rs:546 |
| FND-003 | low | The deletion left stale references. A rustdoc intra-doc link points at the deleted type ("rule already proven for [`NonBindingTag`]"), so it is broken, and the `trace_dogfood` module doc still says a `//!` header "matches no declared legacy form". Point both at the current mechanism. | src/symbols/trace.rs:764; tests/it/trace_dogfood.rs:9-12 |
| FND-004 | low | The retag edited comment text mechanically. About 50 comment and docstring lines now open with a dangling parenthetical (`/// (FR-049, CR-069, Property): …`, `"""(FR-042-AC-10): …"""`, `// (no-FM half)`). Citations that never bound were deleted rather than kept (`TC-461/467` lost `TC-467`; `FR-047-AC-12 / FR-048` lost `FR-048`). New ids were added as separate `#[trace]` attributes beside an existing marker instead of being merged into it (`#[trace("FR-013-AC-15")]` above `#[trace("TC-1651")]`). Nothing binds wrongly, but the text reads as damaged. | tests/python/test_bindings.py:22-235; src/loader/mod.rs:2626-2714; src/corpus/declared_tables.rs:1202 |
| FND-005 | low | These markers predate this PR (they are on origin/main), but they are within the retag's own goal and bind nothing. `#[trace("TC-609", "FR-036-AC-6")]` sits on the helper `fn shall_rule`, a production function, so FR-036-AC-6 is bound nowhere in the repository. `#[trace("TC-562", "FR-033-AC-4")]` sits on `const ASSERT_MANIFEST`. AC-22 keeps a marker on a non-binding kind silent, so neither the advisory nor the census will ever surface these. Move each to the test that asserts it. | src/lint.rs:499; tests/it/markdown_validation.rs:120 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| src/symbols/trace.rs outside_marker_ids, claim, python_docstring_lines, annotation_channel | examined | Correct against AC-22/24/49/50; FND-003 |
| src/symbols/trace.rs bind, bind_symbol, verifies_form_ids, generic_tags, CensusAccumulator | examined | Marker-only; name blanked from census (FR-050-AC-44) |
| src/coverage/binding_diagnostics.rs outside_marker_diagnostics | examined | FND-001 |
| src/coverage/reconcile.rs, diagnostics.rs, coverage.rs reason registry | examined | `trace-id-outside-marker` registered; removed reasons gone |
| src/traceability.rs TraceTagGrammar, validate; src/loader/mod.rs merge | examined | `legacy` key rejected by deny_unknown_fields (AC-43) |
| src/assurance.rs, schemas/output/assurance-v1 and coverage-v1 | examined | provenance and self_named removed consistently |
| retag commit 2ad3e38 (49 files) | examined | FND-002, FND-004 |
| src/symbols/shell_trace.rs, shell adapter merge (#624) | context_only | shell excluded from the advisory per AC-49 |
| corpus submodule pointer | context_only | points at qa-corpus#49 head, expected |

## New findings (disposition pass 1)

Pass 1 reviewed fix head `2648663`: commit `369d89a` (review fixes, the CR-215 kind test and the AC-22 language fix), the main merge `6a09d57` (#626), and the corpus bump to `8d24c93`. The new code reviewed is `IdKinds::from_minted`, `IdKinds::admits`, `id_kind`, `SymbolGraph::retain_id_kinds`, the `reconcile` filter over `outside_marker` and `unmatched_tags`, and the fallback that admits every token when nothing was minted. The code is correct and idiomatic: a `BTreeSet` newtype, no panics, deterministic. Bound relations are untouched. The kind is the leading alphabetic run of an id, upper-cased, so `StR-14` and `STR-14` compare equal. The fallback matches the amended Advisory narrative. Checks run at `2648663` (locked-build.sh, debug=0, jobs 4): `cargo test --lib -- symbols:: coverage:: traceability:: lint::` gave 175 ok, and `cargo test --test it -- outside_marker trace_dogfood shell_symbols assurance_export markdown_validation corpus_case coverage_baseline semantic_systems` gave 90 ok.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | low | TC-2185 (`a_production_function_is_told_an_implements_form_of_its_own_language_only`) asserts only negatives: `assert_ne!(by_path("m.py"), Some(rust attribute))` and `!by_path("m.ts").unwrap_or_default().contains("#[implements")`. Both pass when no finding is emitted for the file, so the test would also pass if the advisory dropped production findings outside Rust. Assert the positive value, `Some("function")`, for both files. | tests/it/outside_marker_advisory.rs:463-476 |
| FND-007 | low | The kind test filters `unmatched_tags` and the advisory, but not the binding census. `generic_tags` still feeds `tagged`, `unmatched_example` and `tagged_not_bound` from every id-shaped token, so a test whose only annotation is `// see CR-215` counts as tagged-but-unbound. `unmatched_example` can then name a symbol that has no `unmatched_tags` record. This conforms to FR-051-AC-19, which is unchanged ("a generic id-shaped token"), but the census and `unmatched_tags` now disagree about what counts as a tag. Apply the same kinds to the census in `reconcile`, or note the difference in the AC-19 text. | src/symbols/trace.rs:596-641; src/coverage/reconcile.rs:477-484 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `implements_template` is now a closure taking `finding.language` (binding_diagnostics.rs:192-201); AC-22 was amended to "whose `language` is the symbol's own"; TC-2185 was added (see FND-006). |
| FND-002 | fixed | `#[trace("TC-1")]` was removed from binding_diagnostics.rs and `#[trace("FR-152")]` from semantic_systems.rs. |
| FND-003 | fixed | trace.rs:818 now reads "already used for the advisory"; trace_dogfood.rs:12 now reads "matches no declared marker form". |
| FND-004 | fixed | No added comment line in the PR diff now opens with a dangling `(…)`; the 7 remaining matches are mid-sentence continuation lines. No added `#[trace]` sits next to an existing one, because adjacent markers were merged. |
| FND-005 | fixed | The marker moved from `fn shall_rule` to `tc609_section_body_pattern_match_yields_no_finding` (lint.rs:512). `TC-562`/`FR-033-AC-4` moved from the const to the two registry-assert tests (markdown_validation.rs:163, 176). |
| FND-006 | fixed | Round 2 (head `54940b4`): TC-2185 now asserts `assert_eq!(by_path("m.py").as_deref(), Some("function"))` and the same for `m.ts`, so it fails when no finding is emitted. It passes (outside_marker: 9 ok). |
| FND-007 | fixed | Round 2: FR-051-AC-19 now states that the census counts every id-shaped token whatever its kind (CR-215), unlike `unmatched_tags` and the advisory, so `tagged`, `tagged_not_bound` and `unmatched_example` can count a `CR-` or `PLAT-` token with no `unmatched_tags` record. This is the "note the difference in AC-19" resolution the finding offered; code and spec now agree. |
