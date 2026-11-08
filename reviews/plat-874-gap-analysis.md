---
id: SR-16311
title: "Gap analysis — PLAT-874 unextractable language refuses measurement (quire-rs PR 596)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md (FR-051-AC-29), spec/tests.md (TC-2010), src/symbols/mod.rs, tests/it/coverage_rollup.rs, Makefile, .github/workflows/ci.yml"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references}
---

## Summary

Ticket: PLAT-874. Frozen PR head: `5b129915de90eb833c198b86d87ce9288243c1cc`. Planless gap analysis scoped to the PR diff: FR-051-AC-29 against TC-2010's two tagged tests and the code that implements the AC. Plan completion: not assessed.

TC-2010 is bound by two tests. `tc2010_unextractable_language_refuses_coverage` (tests/it) hand-sets `extraction.unavailable_languages` and checks that `compute` refuses with the named language, and that a capable build still measures. `missing_adapter_is_unavailable_not_a_diagnostic` (lib) branches on `is_compiled_in` and asserts the matching arm.

## Verdict

**FAIL**. One medium gap. The arm the ticket is about (an adapter feature off) is never executed by any gate. Two AC-29 clauses have no assertion.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | No gate runs the no-adapter arm. Makefile and CI build the `--no-default-features` legs with `cargo check` only, so the unit test's `else` branch (`unavailable_languages == [Rust]`, no symbols, no diagnostic) runs only when someone builds that configuration by hand. The integration test injects the field instead of deriving it. As a result, nothing that gates checks the real extractor→`bind`→`compute` chain refusing over a tree, AC-29's "symbol-table report" record, or AC-29's "a tree with no file in the unavailable language is unaffected" clause. Add a focused `cargo test --no-default-features --features resolve-file` leg running the TC-2010 tests. Alternatively, make capability injectable so default-feature tests drive the recording arm, plus assertions on `symbol_table::build(..).unavailable_languages` and on a tree holding only an available language. | src/symbols/mod.rs:676-688 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-051-AC-29: "records that language as unavailable on the extraction, the symbol graph and the symbol-table report, with no symbols and no per-file diagnostic for those files, and coverage computed over that graph is refused with an error naming the language … A tree with no file in the unavailable language is unaffected." | examined | Refusal clause backed by an injected-field test; recording arm, symbol-table clause and unaffected clause unexercised in any gate (FND-001) |
| TC-2010 → FR-051-AC-29 bindings (both tests) | examined | Trace tags correct |
| Production code with no owning requirement | examined | None. All new code traces to AC-29 |
