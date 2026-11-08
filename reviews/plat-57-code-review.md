---
id: SR-20261103118128
title: "Code review — PLAT-57 inline expression extraction (quire-rs PR 600)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; schemas/output/semantic-v1.schema.json, src/semantic/clauses.rs, src/semantic/model.rs, src/semantic/scan.rs, src/semantic/surface.rs, tests/fixtures/semantic/cases.json, tests/fixtures/semantic/cases.expected.json, tests/it/semantic_boundary.rs, tests/it/semantic_model.rs, tests/it/semantic_surface.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-071", type: references}
  - {target: "ix://agent-ix/quire-rs/FR-075", type: references}
---

## Summary

Ticket: PLAT-57. PR head `964a9dafe75e0d1e82f67eb7c6a86e13b5300f31`, diff `origin/main...HEAD`. Code review with the Rust lane folded in, over the opaque inline-expression extraction (FR-071-AC-10..14, CON-3; FR-075-AC-28..30). The behaviour matches the merged spec and every in-scope criterion has a tagged test with exact assertions. One medium test-scope regression and three low findings remain.

## Verdict

**CONDITIONAL** — one medium finding (a pre-existing test now silently skips 19 cases, 9 of them pre-existing). No vendoring, duplication of upstream content, or ceremony was introduced.

Evidence: `cargo test --test it` 545 passed. `cargo fmt --check` clean. `cargo clippy --locked --all-targets -D warnings` clean. A release wheel (`maturin build --release --features python`) was built and `tests/python/test_semantic.py` passes 4/4, so Python matches the Rust output for all 21 PLAT-57 cases (TC-2043 parity).

`split_row_spans` refactor: an exhaustive differential test compared the old `split_row` with the new one on every string of length 0 to 7 over `| \ space a backtick \r é \t U+3000`, 5,380,840 inputs in all. The cell lists were identical, and every returned span, sliced from the line and with `\|` unescaped, equals its cell. Escaped pipes, `\\|`, trailing pipes and backticks therefore behave exactly as before. The pre-existing table, row-width and assert tests pass.

Judgement calls in the PR body:

- The Guard span covers the whole trimmed cell, backticks included. That agrees with FR-071 ("SOURCE bytes of the trimmed text or cell … for a `Guard` cell the `text` length may differ from the span length").
- A `Pre:`/`Post:` expression makes only `availability.operations` lossy. That agrees with FR-071 ("mark the owning kind lossy"): the line belongs to `operations`, and the `Guard` belongs to `model`.
- The CON-3 denylist oracle follows the NFR-021 precedent and is acceptable, apart from FND-002.

Schema: the change only adds the `expression`/`operationExpression` defs (`additionalProperties: false`), the `operationExpressions` property and `guardExpression`. The top-level `additionalProperties` SemanticId[] rule is unchanged. `cases.expected.json` and `cases.json` only add lines (0 removed).

## Assurance Context

AP-201 (`spec/assurance/AP-201-detection-minting.md`) applies through its "diagnostics, and published schemas" scope. Schema compatibility is additive and verified, as stated above. The new diagnostic tokens are covered by failure fixtures with healthy controls (the refusal and control cases). Unavailable: `make ci`, the controlled corpus and the mutation runs were not run, per the brief (no full gate). Active exceptions: none.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | `records_without_model_features_are_unchanged` (TC-1846, FR-075-AC-7/CON-1) now skips every case whose module declares any `mappings`. That drops the 9 pre-existing PLAT-55 cases with `mappings: [relationships]` (not a model feature; their records carry no `model`), and 10 PLAT-57 cases whose records also carry no `model`, from a check that passed on main. Exclude only cases that declare a model-feature mapping (e.g. `effect-frames`), not any mapping. | tests/it/semantic_model.rs:938-946 |
| FND-002 | low | The CON-3 audit repeats the `no_clause_parser_or_template_dependency` denylist loop in a parallel test. Its `parse_expression` symbol is already covered by the existing `parse_expr` substring. It omits the common Rust parser crates `nom`, `winnow`, `logos`, `peg`, `combine`, `pest_derive`, none of which is in Cargo.lock today. Fold the list into the existing audit (or a shared helper) and add those crates. | tests/it/semantic_boundary.rs:79-110 |
| FND-003 | low | `TableRead::guard` adds a third copy of the header-to-column lookup (`cell`, `raw_cell`, `guard`). Its `.unwrap_or(0..0)` would silently emit a 1..1 span instead of failing if the lookup ever missed. Extract one column-index helper and use it in all three. | src/semantic/model.rs:1461-1479 |
| FND-004 | low | CHANGELOG `[Unreleased]` has no entry for the consumer-visible additive record keys and diagnostics (`operationExpressions`, `guardExpression`, `semantic.expression-unchecked`, `semantic.empty-expression`) or the new public types `Expression`/`OperationExpression`. The PLAT-175 `fieldSources` entry is the precedent. | CHANGELOG.md:10 |
