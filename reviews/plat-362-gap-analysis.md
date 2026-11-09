---
id: SR-16435
title: "Gap analysis of agent-ix/quire-rs PR 624 (PLAT-362 part 2): FR-051 shell clauses"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md (shell adapter, AC-34..46), spec/tests.md (TC-2101..TC-2113), tests/it/shell_symbols.rs, src/symbols/shell.rs, src/symbols/shell_trace.rs (PR 624 diff only, head 1b3bafd)"
review_set: subset
---

## Summary

Ticket: PLAT-362. Planless gap analysis over PR 624 at head `1b3bafd`. Plan completion: not assessed.

The matrix comes from `quoin matrix --json` (quoin 0.29.1, module spec-artifacts-process@b82e42e). FR-051-AC-34 through FR-051-AC-46 are each `static_status: tagged`, bound by exactly one test in `tests/it/shell_symbols.rs` (TC-2101..TC-2113, in order). No criterion in scope is untagged. All 13 tests pass at the head. The qa-corpus cases `outside-entry-paths`, `function-not-evidence` and `not-in-tag-channel`, with their controls, also pass and exercise AC-44 through `quire coverage`.

Examined, with each binding checked against its AC text: TC-2101/AC-34, TC-2102/AC-35, TC-2103/AC-36, TC-2104/AC-37, TC-2105/AC-38, TC-2106/AC-39, TC-2107/AC-40, TC-2108/AC-41, TC-2109/AC-42, TC-2110/AC-43, TC-2111/AC-44, TC-2112/AC-45, TC-2113/AC-46. Every binding is correct, meaning each test asserts the clause it is tagged with. The findings below are clauses that a correct binding leaves unasserted.

The production code has no requirement-less code: every new function in `shell.rs` and `shell_trace.rs` serves an AC in 34..46. There are no stubs and no coverage inflation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-2109 does not exercise the `syntax: comment` half of FR-051-AC-42. It covers a model with no form and a model with only a Rust form, but not a `language: shell` markers form that omits `syntax: comment`. That is exactly the case where the implementation binds against the AC (SR-16434 FND-001). Add it: a shell form without `syntax` mints zero `verifies`, or is refused at load. | tests/it/shell_symbols.rs:349-364 |
| FND-002 | low | TC-2111 asserts `graph.shell_findings`, not `CoverageReport.diagnostics`. It never asserts AC-44's "no census count moves". The report channel is covered elsewhere, by TC-2113 for `outside-entry-paths` and by the qa-corpus cases for all three reasons. The census clause is covered nowhere. Add a census-equality assertion (before and after the non-binding marker) to TC-2111. | tests/it/shell_symbols.rs:411-442 |
| FND-003 | low | TC-2112 checks that the diagnostic reason contains `"line "`, but does not check the line number. FR-051-AC-45 says the diagnostic names "the line the error sits at". Assert the expected line for the `bad.sh` fixture. | tests/it/shell_symbols.rs:458-463 |
| FND-004 | low | TC-2104 asserts the `SymbolKind` methods (`binds_trace_ids`, `carries_implements`, `as_str`), not the surfaces FR-051-AC-37 names: the symbol-table report's binding-kind columns, and the FR-045 record `kind` (`schemas/output/assurance-v1.schema.json` gained `entry_point`). Add an assertion over `symbol_table` output and an assurance record for one `.sh` file. | tests/it/shell_symbols.rs:173-204 |
| FND-005 | low | The FR-051 shell narrative says the adapter mints "a function symbol for every `function_definition` node the grammar yields". `tree-sitter-bash` does yield `function_definition` nodes inside `$( )` and backticks, and FR-051-AC-36 lists those as decoys that mint nothing. The implementation follows AC-36, which is also correct shell semantics, because the function is subshell-scoped. The narrative contradicts the AC, though. Add a CR note qualifying the narrative with "outside a command substitution". | spec/functional/FR-051-source-symbol-extraction.md:98-103 |

## Verdict

Coverage is complete at the binding level: 13 of 13 ACs are tagged and each binding is correct. FND-001 is the gap that matters, because it hides a spec violation that exists in the code. The low findings are test-strength and spec-wording gaps; none hides a defect the reviewer found. The coder's AC-36 reading (lead point 1) is right, and only the narrative needs a CR line (FND-005).

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | aa205f7: TC-2109 adds a shell markers form without `syntax`. It asserts that load refuses the form, naming it and `syntax: comment`, and that `bind` on the unvalidated model mints zero `verifies`. The TC-2109 row text is updated to match. |
| FND-002 | fixed | aa205f7: TC-2111 asserts that `binding_census` carries no `shell` row, which is how AC-44's "no census count moves" holds under the new CR-213 census exclusion. |
| FND-003 | fixed | aa205f7: TC-2112 asserts that the reason starts with `line 2:`, the line of the unterminated string in `bad.sh`. |
| FND-004 | fixed | aa205f7: TC-2104 asserts the `symbol_table::build` record for `entry_point` (`binds_trace_ids && !carries_implements`) and an FR-045 graph node from `graph_records` carrying `"kind":"entry_point"`. |
| FND-005 | fixed | aa205f7: a CR-213 note in the Shell adapter section qualifies the narrative. A function is minted for every `function_definition` node except one inside a command substitution, citing AC-36. |
