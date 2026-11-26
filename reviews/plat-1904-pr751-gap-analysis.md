---
id: SR-967175026302197768900474181941935084892
title: "PLAT-1904 PR 751 evidence coverage gap analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; PLAT-1904; PR 751; spec/, src/, tests/ with changed-surface reverse inspection"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-092", type: references}
---

## Summary

Repository-driven computed-matrix and changed-surface reverse-gap inspection at `a3c0d694da9a34b85e076441c4554303e1ac6896`, compared with `d7d418fa2727f7de87a16dfec5b38ada7d9ee776`. No new trace gap, unowned behavior, source stub, or test stub was found in the PR. Plan completion: not assessed.

## Verdict

CONDITIONAL for repository assurance: the baseline already contains 229 untagged criteria, and this bounded PR review does not independently inventory every internal behavior. The computed criterion-status map is identical at base and head. No new changed-surface gap blocks PR #751.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Assurance Context

AP-201 applies to the inventory and coverage population boundary. The real-loader test and explicit strictness controls support implementation conformance. Independent QA corpus producer evidence and a full make ci run are unavailable for this review; existing repository assurance gaps are not represented as newly introduced defects.

## Coverage

Reconciliation: `quoin matrix --repo <root> --json` using quoin 0.36.8 and quire 0.52.0 (engine 0.62.2), with the pinned corpus submodule initialized. The base and head have identical criterion status maps: 1,141 tagged, 229 untagged, and 141 method-without-symbol. No ignored-only, qualified-record, or retired status appeared. Every criterion reports no run evidence; execution evidence was not stored by this read-only matrix command. FR-092-AC-20 is tagged by `tests/it/relation_report.rs::dropped_record_key_relation_is_reported_with_its_valid_sibling`.

`quire coverage --scope <root> --json` reports 269 untracked symbols, 119 unmatched tags, 1,402 diagnostics, and 18 suspicions. These pre-existing signals are not treated as defects introduced by this PR. The CLI engine version is older than the PR source; this records traceability and does not claim to run the changed engine behavior. The fresh Rust integration run provides that behavior check.

Changed-surface reverse inspection inventories two new policy types, two changed public functions, the default options behavior, and strict sibling callers. FR-092-AC-20 owns the selective behavior. No changed-surface untraced behavior, source stub, test stub, hollow assertion, or duplication found. A complete manual inventory of the whole repository's internal behaviors was not established, so no repository-wide reverse-gap PASS is claimed. No dependency move or specification change requires additional review methods.

Plan completion: not assessed

Optional semantic gap-analysis fanout: skipped; not requested. Code-review independently checked the changed criterion and concrete test assertions.

## Examined scope

```yaml
scope:
- id: FR-092-AC-20
  path: spec/functional/FR-092-matrix-design-column.md
  role: examined
  excerpt: '`compute_coverage_for_design_matrix` returns coverage from retained declarations
    when only required relations were dropped, so `design_matrix` can retain valid
    siblings and report each dropped relation; other dropped declaration kinds and
    ordinary coverage remain strict.'
- id: FR-092-AC-19
  path: spec/functional/FR-092-matrix-design-column.md
  role: context_only
  excerpt: When one required relation is dropped and a valid sibling relation remains,
    `design_matrix` returns the valid sibling's entries and reports the dropped relation
    in `errors`.
- id: src/coverage.rs
  path: src/coverage.rs
  role: examined
  excerpt: EvidenceCoverageMode selects strict or relation-only tolerance in compute_with_evidence;
    compute and compute_coverage_for_design_matrix retain their existing behavior.
- id: src/obligation/inventory.rs
  path: src/obligation/inventory.rs
  role: examined
  excerpt: InventoryModelPolicy defaults to Strict; DesignMatrix rejects every dropped
    declaration kind except RequiredRelation before enumerating selected sources.
- id: tests/it/relation_report.rs
  path: tests/it/relation_report.rs
  role: examined
  excerpt: The real-engine fixture adds a selected criterion and tests complete inventory,
    strict evidence refusal, retained matrix entry and dropped-relation error, then
    fatal dropped Obligation.
- id: tests/it/coverage_matrix.rs
  path: tests/it/coverage_matrix.rs
  role: examined
  excerpt: All existing evidence coverage callers explicitly select Strict; their
    assertions and fixtures are unchanged.
- id: tests/it/coverage_rollup.rs
  path: tests/it/coverage_rollup.rs
  role: examined
  excerpt: Both evidence coverage callers explicitly select Strict; strict exemption
    assertions are unchanged.
- id: tests/it/formalizes_relation.rs
  path: tests/it/formalizes_relation.rs
  role: examined
  excerpt: The retained obligation evidence caller explicitly selects Strict; the
    source-only path is unchanged.
- id: tests/fixtures/corpus_cases/plat-1856.json
  path: tests/fixtures/corpus_cases/plat-1856.json
  role: context_only
  excerpt: The original module has valid requirement-has-design and dropped connection-carries-classification
    with undeclared record key regulatedCategories.
- id: tests/it/declaration_drop.rs
  path: tests/it/declaration_drop.rs
  role: context_only
  excerpt: The existing generic consumer regression checks exact FlowRule refusal
    and the healthy complete-model control.
```
