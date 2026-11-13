---
id: SR-20261113104842
title: "Code review - PLAT-1597 criticality_field on obligation sources (quire-rs PR 676)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@2930079; src/obligation.rs, src/obligation/inventory.rs, src/traceability.rs, tests/it/obligation_criticality.rs, tests/it/combinatorial.rs, tests/it/main.rs, tests/fixtures/traceability/obligations-rated/manifest.yaml, CHANGELOG.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-053", type: references}
---

# Code review - PLAT-1597 criticality_field on obligation sources (quire-rs PR 676)

## Summary

Ticket: PLAT-1597. PR head `2930079`, scope `git diff origin/main...HEAD` (11 files, +359/-20).
Code-review with the rust-review lane folded in. Focused gate run by the reviewer:
`cargo test --test it obligation` through `locked-build.sh` (48 passed, 0 failed).

## Verdict

**PASS with findings (1 medium, 5 low).** The rule is wired through every non-combinatorial
obligation builder: `derive` (via `collect` -> `row_obligation`), `for_document` (classification
and `quire properties`), and `SelectedInventory` (via `row_obligation`). One helper,
`row_criticality`, holds the precedence, so the three paths cannot disagree. A non-empty cell wins,
because `ScannedRow::cell` filters empty cells. The document value fills only absent or empty cells.
Only a non-blank JSON string counts. The engine holds no key name, and the fixture uses the invented
key `rating`. `ObligationSource` keeps `deny_unknown_fields`, so a typo is still rejected at load.
The struct-literal break is complete: the only literals are the five in `tests/it/combinatorial.rs`.
The value flows through `Obligation` and `CriterionObligation` into coverage `obligations[]`,
`properties --json` and the assurance projection, all unchanged. The coverage schema already carries
`criticality`, and a test validates against it. No digest, pin, SHA or compatibility layer was added.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A combinatorial source silently ignores a declared `criticality_field`. Both combinatorial builders hard-code `criticality: None`, and module load does not reject the key on a combinatorial source. FR-053 item 5 says the value "rates every obligation derived from that document", so a module that declares it on a configuration-space source gets no rating, and no error tells it why. Fix: apply `document_rating` in both combinatorial builders, or reject the key at load for a combinatorial source and state that in FR-053. | src/obligation.rs:255-260, src/obligation.rs:603-612, src/traceability.rs:1769-1800 |
| FND-002 | low | Doc comments cite `FR-049-AC-60`, which does not exist (FR-049 ends at AC-32). `quire trace --id FR-049-AC-60` reports it as a `production_orphan_tag` on `row_criticality` and a mention on `ObligationSource`. The intended id is FR-053-AC-24..26. | src/obligation.rs:273, src/traceability.rs:759 |
| FND-003 | low | Two doc comments are now stale. `Obligation::criticality` says it is "absent for a source declaring no criticality column", but a `criticality_field` alone now supplies it. `CriterionObligation` says criticality is "readable from the criterion's own row", but it may now come from the document frontmatter. | src/obligation.rs:60-61, src/obligation.rs:88-90 |
| FND-004 | low | An empty `criticality_field: ""` passes module load and silently matches nothing. Sibling fields such as `statement_column`, the combinatorial columns and parameter columns are checked with `check_field`. | src/traceability.rs:1762-1815 |
| FND-005 | low | `SelectedInventory` calls `document_rating` once per row inside the row loop, so the same frontmatter lookup is repeated for every criterion. `for_document` hoists it per document. Hoist it above the table loop. | src/obligation/inventory.rs:1188-1195 |
| FND-006 | low | The test helper `bundle(name, ...)` takes a `name` it never uses and discards it with `let _ = name;`. Drop the parameter. | tests/it/obligation_criticality.rs:44-53 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| src/obligation.rs `row_criticality`, `document_rating` | examined | Precedence and string-only rule correct |
| src/obligation.rs `derive` / `collect` / `row_obligation` | examined | Ratings keyed on document path; row path matches |
| src/obligation.rs `for_document` | examined | Wired |
| src/obligation.rs combinatorial builders | examined | FND-001 |
| src/obligation/inventory.rs `SelectedInventory` row path | examined | Wired; FND-005 |
| src/traceability.rs `ObligationSource` + load validation | examined | deny_unknown_fields kept; FND-004 |
| src/assurance.rs, src/coverage.rs, src/validate_document.rs consumers | context_only | Carry `obligation.criticality` unchanged |
| tests/it/combinatorial.rs struct literals | examined | Complete |
| tests/it/obligation_criticality.rs | examined | FND-006 |
| tests/fixtures/traceability/obligations-rated/manifest.yaml | examined | Invented key `rating` |
| CHANGELOG.md | examined | Breaking change noted |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | low | The module doc of the test file still reads `FR-053-AC-24..26`, but the file now carries tests for FR-053-AC-24..31 after the AC split. | tests/it/obligation_criticality.rs:1 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | f8289aa: `for_document` passes `document_rating` into `combinatorial_obligation` (`criticality: rating`). `derive` builds `ratings` before the combinatorial branch, and `collect_combinatorial` sets `criticality: ratings.get(path).cloned()`. The selected inventory passes the per-document rating into its combinatorial collector. With no `criticality_field`, `document_rating` returns None and `ratings` is empty, so the output is unchanged. FR-053 item 5 now names combinatorial obligations. New AC-29 is covered by `tc_combinatorial_obligation_carries_the_rating_on_every_surface` (derive, for_document, inventory). |
| FND-002 | fixed | f8289aa: both citations now read `FR-053 item 5, FR-053-AC-24..25`. `quire trace --id FR-049-AC-60` reports no claims and no citations. |
| FND-003 | fixed | f8289aa: the `Obligation::criticality` and `CriterionObligation` doc comments name both declarations. |
| FND-004 | fixed | f8289aa: `validate` runs `check_field(..., "criticality_field", field)` (src/traceability.rs:1904). New AC-30 is covered by `tc_blank_criticality_field_fails_module_load` ("" and "   "). |
| FND-005 | fixed | f8289aa: `let rating = super::document_rating(source, document.frontmatter());` is computed once per document (src/obligation/inventory.rs:1089) and reused for rows and combinatorial rows. |
| FND-006 | fixed | f8289aa: `fn bundle(docs: &[(&str, String)])`; the unused parameter is gone. |
