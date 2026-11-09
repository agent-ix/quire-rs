---
id: SR-16380
title: "Code and Rust review of PLAT-1170 legacy-implements-form warning (quire-rs PR 615)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/coverage.rs, src/coverage/binding_diagnostics.rs, src/coverage/diagnostics.rs, src/coverage/reconcile.rs, src/symbols/trace.rs, tests/it/legacy_implements_warning.rs, tests/it/main.rs, tests/fixtures/coverage_baseline/expected.json, spec/tests.md (PR 615 diff only)"
review_set: subset
---

## Summary

Ticket: PLAT-1170. PR head `abc3875`. Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD`. The diagnostic is declaration-driven: the binder records a use only for a Rust implements form with no `syntax`, and the renderer takes the first Rust `syntax: rust_attribute` form that declares a `template`. No form name is compared anywhere. Binding, relations and counts are unchanged because the record is appended beside the existing `ImplementsRelation` push. No `RewriteSuggestion` is created. Multi-id rendering reuses `quoted_id_list`, which is also the existing rewrite renderer, so the two cannot drift. The work rides inside the existing `bind_implements` loop, so there is no extra scan. `LegacyImplementsUse` and `SymbolGraph.legacy_implements` derive no `Serialize`. The only new output is one `CoverageDiagnostic` per use plus the new reason in `diagnostic_reason_registry`, so no new output schema appears. The six TC-2090 tests pass locally. A probe outside the branch reproduced FND-001. No hash, pin, digest, tracking record, vendored file or compatibility layer was introduced.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A legacy `/// Implements:` line inside an inline `mod`, a `trait` or a function body produces two `legacy-implements-form` findings for one occurrence. One names the member. The other names the enclosing container, because a container's attached span covers its members. The container finding names the wrong symbol and tells the author to put the attribute where it does not belong. Reproduced: `pub mod m { /// Implements: FR-007-AC-2  pub fn a() {} }` gives findings for `m` and `m::a` at lib.rs:2. The trait `T`/`T::b` and the function `outer`/`outer::inner` cases do the same. The `LegacyImplementsUse` doc says one record per authored occurrence, and CR-209 says one warning per use. Range findings already handle this by attributing each occurrence to its innermost symbol (`innermost_symbol_at`, greatest `leading_line`). | src/symbols/trace.rs:1316-1328, src/symbols/trace.rs:875-878 |

## Verdict

Examined: FR-050-AC-62 (every clause), the CR-209 note, the binder change, the renderer, guidance, the reason registry, the baseline fixture and the tests.md row edits. Correct and clean: declaration-driven selection, the first template wins, no template gives no finding, `"a", "b"` rendering, non-Rust bindings and `rust_attribute` bindings stay silent, counts are unchanged, no rewrite is created, no new scan and no schema leak. The tests can fail. Removing the emission fails the first, renaming and first-template tests. Dropping either the `syntax.is_none()` or the Rust guard fails the attribute-binding or Python test. Setting the line offset to zero fails the line asserts. The `spec/tests.md` change is consistent: TC-2090 is now ✅ and fully bound by six tagged tests, so the FR-050 summary row can drop "except AC-62". `corpus_cases::tc1026` fails only because corpus.yaml `emitted` lacks `legacy-implements-form` (confirmed). It is not a finding. Not mergeable until FND-001 is fixed by attributing each occurrence to its innermost symbol, with a nested-fixture test.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | Commit 99268c4 deduplicates by `(path, line)` and keeps the innermost owner. That is correct when the nested doc line directly follows code, and the new nested test passes. A mutation check also confirms the test fails when the old `dedup()` is restored (six findings instead of three). The fix misses the most common layout: a nested item after a blank line. With `(?m)^\s*///`, `^` matches at the start of the blank line and `\s*` consumes the newline, so the container's match starts one line early. The container record gets line L-1 and the member's gets line L, and the `(path, line)` dedup never merges them. Reproduced at 99268c4: `pub mod m {\n\n    /// Implements: FR-007-AC-2\n    pub fn a() {}\n}` gives `m` at lib.rs:2 and `m::a` at lib.rs:3. The container finding also carries a wrong line. Still missing: take the line from the id capture (`caps.get(1)`), not from the match start, or dedup by innermost-owner containment. Then add a blank-line-preceded nested case to the test. |
| FND-001 | fixed | - |
