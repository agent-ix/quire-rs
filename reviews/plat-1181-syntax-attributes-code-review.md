---
id: SR-20261102118101
title: "PLAT-1181 Rust implements attribute code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs PR566; FR062AC12..17; module guard, adapter-owned syntax, complete matching, diagnostics and declared fixture bank"
review_set: subset
---

## Summary

Frozen original quire-rsPR566 source5eca81c22123b51be8261757b6516e2398f5b530 overmain699860d. PLAT1181 ticket, current AGENTS/CLAUDE, FR062AC12..17 and TC1983..1990 procedures were read, along with actual source/test/fixture diff. Existing CR191/FR062 spec is unchanged, so no new spec-authoring/sub-analysis review is opened. One code review with Rust lane plus one scoped gap pass; no reviewed source/worktree edits, Cargo/heavy/Kani/backend jobs, hosted dispatch or merge by reviewer. Hosted checks and Luna merge ownership remain explicit.

Adapter retains verbatim outer attribute nodes with actual start lines and associates them through sibling syntax, not line overlap or a second lexer. Only attributes with no local parse error become candidates; fields/variants are not extracted production owners, inner attributes are excluded, and macro token/fuzz synthetic symbols receive no candidate attributes. Binder selects candidates only for guarded forms, requires full matched byte span and preserves old attached-source matching for omitted guards. Module pattern/template and marker-id/range logic remain authoritative; no hardcoded implements pattern or implicit guard is supplied. Module load rejects unknown syntax, non-Rust guards and marker/legacy placement with named forms. Partial matches are retained once per form/candidate in their own channel and projected into CoverageReport diagnostics, with reason inventory/guidance added. Scope/evidence relation types and production/evidence kind complement remain separate. No concrete runtime logic defect was established in this scope; ERROR ancestor behavior is not guessed as correct without the missing control.

The original declarative bank is attributed to PLAT1181 and holds six source scenarios with actual/data, valid layout, ownership, unrelated healthy, range and partial cases. Python/TypeScript are explicitly covered by unguarded comment regression; no upstream corpus was copied. No corpus-wide census or rule widening was performed. New public module-load-to-bind controls exercise exact edge sets, round-trip guard data, invalid guard inputs, substring rejection/cfg_attr, range findings and per-file failure. However the merged acceptance procedure requires stronger independent recovery and separation controls, recorded below. Frozen computed matrix tags every FR062AC12..17 row, but tagged status alone does not discharge the missing semantic subclauses.

Supplied focused implements_relation target11PASS and default all-target Clippy warnings-denied PASS precede the rebase. Final per-head aggregate/required gates and hosted checks remain owed and are not inferred green. Release/install/consumer adoption are not accepted by this review. Plan completion: not assessed

## Verdict

Scoped verdict: **CONDITIONAL** pending the two medium acceptance-evidence findings and final gates.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The mandatory TC1990 independent oracles are incomplete. The test compares IDs after a blank-line insertion, not guard/no-guard symbol id/path/kind/declaration-position equality; it has no genuine positive evidence-symbol control comparing the complete verifies set, no public before/after backed-ids/totals/untracked_symbols equality, and determinism does not include partial-match diagnostics. Existing TC939 covers only an unguarded manually extended model and does not replace these guarded public-path controls. Add the specified independently failing oracles without weakening AC17 or the scope/evidence boundary. | tests/implements_relation.rs:syntax_guard_preserves_identity_range_locations_and_file_failure_isolation |
| FND-002 | medium | The normative TC1987 bank requires an ERROR-recovered attribute beside a valid real outer-attribute control, but no malformed/recovered attribute appears in this bank. The separate bad.rs test rejects an entire broken declaration and does not exercise recovered attribute exclusion in a file whose healthy control remains bound. Add the syntax-recovery adverse/healthy fixture and assert exact owners/no recovered relation; no runtime recovery defect is asserted without that counterexample. | tests/fixtures/corpus_cases/rust-implements-syntax.json:ownership case |

## Examined scope

```yaml
scope:
- id: FR-062-AC-12
  path: spec/functional/FR-062-implements-relation.md
  role: examined
  excerpt: 'A module-loaded Rust form retains `syntax: rust_attribute` through the
    traceability model and serialization; omission preserves the old declared-text
    behavior. A partial attribute match produces no relation and one `partial-implements-attribute-match`
    warning in `CoverageReport.diagnostics`, naming the form, file, 1-based candidate
    start line, qualified symbol and complete attribute text; no-match and unrelated-attribute
    cases produce no such finding. An unknown syntax value or this guard on a non-Rust
    form or outside `trace_tags.implements` is refused at module load, identifying
    the form, with no text-matching fallback.'
- id: FR-062-AC-13
  path: spec/functional/FR-062-implements-relation.md
  role: examined
  excerpt: A real production attribute carrying two declared ids produces exactly
    those two `implements` relations and no `verifies` relation. Valid leading, multiline,
    trailing-comment and same-line-item attributes are eligible through the same module-loaded
    form; each form matches only the exact attribute-node text.
- id: FR-062-AC-14
  path: spec/functional/FR-062-implements-relation.md
  role: examined
  excerpt: Attribute-shaped text in doc, line, block or nested block comments, ordinary
    or byte strings, and single-line or multiline raw strings produces no guarded
    attribute-form relation, including a column-zero raw-string line; the matching
    real attribute control remains bound. A substring inside another attribute payload
    creates no relation.
- id: FR-062-AC-15
  path: spec/functional/FR-062-implements-relation.md
  role: examined
  excerpt: 'Guarded attributes bind only to their actual production declaration: a
    nested declaration''s attribute binds neither parent nor sibling, and an attribute
    on an evidence symbol produces no `implements` relation. Field, variant, statement,
    expression and closure attributes do not bind to their enclosing symbol. Inner
    attributes, macro-token attributes and recovered-error attributes do not bind;
    cfg_attr payloads remain unexpanded data rather than nested candidates. Same-line
    siblings retain distinct owners.'
- id: FR-062-AC-16
  path: spec/functional/FR-062-implements-relation.md
  role: examined
  excerpt: Unguarded Rust comment forms and Python/TypeScript declared comment forms
    retain their existing relations; syntax-guard adoption supplies no implicit pattern
    and does not change the module's declared ids or templates.
- id: FR-062-AC-17
  path: spec/functional/FR-062-implements-relation.md
  role: examined
  excerpt: Guarded binding preserves symbol identity, actual finding source locations,
    existing range refusal and per-file syntax-failure behavior, and deterministic
    relation/diagnostic ordering. It changes no `verifies` edge or coverage backing
    count.
- id: TC-1990-procedure
  path: spec/tests.md
  role: examined
  excerpt: TC-1990 SHALL have the following independently failing subcases; each SHALL
- id: TC-1987-procedure
  path: spec/tests.md
  role: examined
  excerpt: attribute control. Assert no nested/inner/token/error relation; an explicitly
- id: src/symbols/rust.rs
  path: src/symbols/rust.rs
  role: examined
  excerpt: //! Rust source adapter (FR-051).
- id: src/symbols/mod.rs
  path: src/symbols/mod.rs
  role: examined
  excerpt: //! Source-symbol extraction (FR-051).
- id: src/symbols/trace.rs
  path: src/symbols/trace.rs
  role: examined
  excerpt: //! Trace binding and graph relations (FR-051).
- id: src/traceability.rs
  path: src/traceability.rs
  role: examined
  excerpt: //! The declarative traceability model (FR-050).
- id: src/coverage/reconcile.rs
  path: src/coverage/reconcile.rs
  role: examined
  excerpt: //! Reconcile declared targets and references with the bound symbol graph.
- id: src/coverage/diagnostics.rs
  path: src/coverage/diagnostics.rs
  role: examined
  excerpt: //! Diagnostics derived after coverage reconciliation.
- id: src/coverage.rs
  path: src/coverage.rs
  role: examined
  excerpt: //! Declarative coverage computation (FR-050).
- id: tests/implements_relation.rs
  path: tests/implements_relation.rs
  role: examined
  excerpt: "//! FR-062 \u2014 the requirement\u2192production-code relation (TC-936..TC-938)."
- id: tests/fixtures/corpus_cases/rust-implements-syntax.json
  path: tests/fixtures/corpus_cases/rust-implements-syntax.json
  role: examined
  excerpt: '{'
```

## Fix dispositions

Disposition verified at frozen test-only fix008119b0878c7af6ce647e22134b9fb6b28c3bf4, originalPR566. Actual diff changes only the original declarative bank and implements_relation tests; adapter/binder/module/coverage production source remains5eca81c. Both original medium acceptance-evidence findings are fixed. The new recovered-attribute witness is now grounded by the actual parser shape, not a guessed syntax string or whole-file rejection. The new guarded-path separation control has positive evidence/backing/untracked oracles, complete identities/endpoints and deterministic partial diagnostics, so all previously missing TC1990 subcases have meaningful controls.

Preserved focused implements_relation target13PASS/0FAIL is credited at this fix. Prior symbol131PASS and default-all-target Clippy PASS are earlier evidence, not a replacement for final per-head gates. The one final aggregate and required hosted checks remain owed; reviewer ran no Cargo/heavy/backend jobs and edited no source/worktree. No new findings, scope narrowing or spec waiver. Method verdicts nowPASS, with merge readiness CONDITIONAL on final gates/hosted checks; Luna retains merge ownership. No release/install/consumer adoption claimed.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | New public module-load control compares all symbol id/path/kind/declaration positions with guard omitted/present, exact eligible endpoints and original multiline attribute text/line8/byte prefix. It preserves a genuine two-verifies positive control, one backed of one row plus nonempty untracked control, exact backed IDs/totals/untracked sets, and complete graph/coverage diagnostic determinism including a partial match. |
| FND-002 | fixed | Original attributed corpus now includes malformed body-local attributeFR801 beside healthyFR802. Real parser oracle establishes diagnosticNone, actual attribute_item.has_error and MISSING token with exact malformed candidate text; existing public module-load bank harness independently asserts only healthyFR802 relation, no recovered edge. |

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  after_excerpt: "assert_eq!(after.verifies, before.verifies);\n    assert_eq!(after.backed_trace_ids(),\
    \ before.backed_trace_ids());"
  reason: New public module-load control compares all symbol id/path/kind/declaration
    positions with guard omitted/present, exact eligible endpoints and original multiline
    attribute text/line8/byte prefix. It preserves a genuine two-verifies positive
    control, one backed of one row plus nonempty untracked control, exact backed IDs/totals/untracked
    sets, and complete graph/coverage diagnostic determinism including a partial match.
- fnd: FND-002
  outcome: fixed
  after_excerpt: assert!(node.to_sexp().contains("MISSING"));
  reason: Original attributed corpus now includes malformed body-local attributeFR801
    beside healthyFR802. Real parser oracle establishes diagnosticNone, actual attribute_item.has_error
    and MISSING token with exact malformed candidate text; existing public module-load
    bank harness independently asserts only healthyFR802 relation, no recovered edge.
```

## QA corpus amendment

Narrow same-review amendment: original enginePR566 at4e34014b94c5ad6fc82539851e1b72943173c74a selects companion qa-corpusPR42 candidate0cd2548cc8270c17fa181e2721a1cab6eeab65ad overc720e13. Engine Rust production remains unchanged from reviewed5eca81c and test-only fixes008119b; originalFND001/002 remain fixed. QA source/guidance, declared diagnostic vocabulary, original failure/control pair, module manifest, inventory and derived counters were inspected. No source/worktree edit or Cargo/heavy/backend run by reviewer.

The new exact diagnostic reason is appropriately added to authoritative QA vocabulary. The failure and healthy control preserve genuine production multi-ID bindings while differing only in a matching substring inside doc-attribute data; both assert backed0 and native metric examined2/matched1, so scope does not become verification. Required L1reason/L2path+line11/L3message fragments identify the module form, payload owner and exact offending attribute semantics. The custom form owns its Rust syntax guard, regex and template. Rust inventory is explicit, Python/TypeScript exclusions are justified by the Rust-only opt-in position and point to existing unguarded regressions. Attachment witness channels permit these diagnostic oracles. Derived fixture/pair counters advance consistently without attributing unrelated historical measurements.

Preserved QA18selftests and complete bounds (189fixtures,118cells,90covered,28out_of_scope,GAP0) pass. Actual engine corpus_cases run now reports20PASS/1FAIL at required missing-comment metadata check, recorded as newFND003; registry parity and functional corpus cases themselves pass. This is not a green focused or aggregate claim. Final engine aggregate/hosted checks remain owed; QA42 must land before engine gitlink is changed to its actual squash result and final hosted engine checks. Luna merge ownership remains explicit. No release/consumer qualification inferred.

### New finding

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | Both new QA failure/control root case.yaml files omit required nonempty comment explaining the fixture. Actual engine corpus_cases at the candidate gitlink fails tc1020 at tests/corpus_cases.rs803: guarded-implements-partial-match-rust carries no comment saying what it is about (20PASS/1FAIL). Add meaningful failure/control comments in QA42 and retain the exact gitlink update; do not relax the authoritative metadata guard. | agent-ix/qa-corpus/cases/attachment/guarded-implements-partial-match/case.yaml |

Current verdict: **CONDITIONAL** pendingFND003 and final gates.

## QA metadata fix disposition

Final scoped QA amendment disposition at engine7dede62d29c23393468dee9c197c0834f9e3839e / QA bdbb5baf8bdd40844b5d6ac835ad7240dd94bf6a. Diff from reviewed QA0cd adds only the two missing explanatory comments. Actual engine gitlink selects that repaired candidate. Preserved focused logs show corpus_cases21PASS/0FAIL and implements_relation13PASS/0FAIL; QA18selftests pass. The prior complete bounds declaration is unchanged by comment-only repair. FND003 is fixed; earlier FND001/002 remain fixed; no new amendment finding.

Both code and scoped gap method verdicts are nowPASS for the reviewed source/tests/QA vocabulary. Final remaining aggregate and hosted engine checks remain owed. QA42 must merge first under Luna ownership, followed by updating engine566 gitlink to the actual QA squash commit and final checks; this candidate review does not authorize a merge/release or silently credit an untested changed gitlink. No Rust production change since reviewed source5eca81c; only source/tests and subsequent QA metadata/gitlink carries described in these same artifacts. Reviewer ran no Cargo/heavy/backend jobs and edited no reviewed tree.

| FND | outcome | reason |
| --- | --- | --- |
| FND-003 | fixed | QA bdbb5baf8bdd40844b5d6ac835ad7240dd94bf6a adds meaningful failure/control comments only. Engine7dede62d29c23393468dee9c197c0834f9e3839e selects that exact gitlink; preserved corpus_cases21PASS and implements_relation13PASS, including previously failing tc1020 and exact diagnostic vocabulary parity. QA18selftests pass; prior complete bounds is unchanged by comment-only fix. |

```yaml
dispositions:
- fnd: FND-003
  outcome: fixed
  after_excerpt: A guarded Rust form must warn on the proper substring inside the
    doc attribute, while its genuine multi-ID production attribute binds and block-comment/raw-string
    examples produce no additional implementation edges.
  reason: QA bdbb5baf8bdd40844b5d6ac835ad7240dd94bf6a adds meaningful failure/control
    comments only. Engine7dede62d29c23393468dee9c197c0834f9e3839e selects that exact
    gitlink; preserved corpus_cases21PASS and implements_relation13PASS, including
    previously failing tc1020 and exact diagnostic vocabulary parity. QA18selftests
    pass; prior complete bounds is unchanged by comment-only fix.
```
