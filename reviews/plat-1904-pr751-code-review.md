---
id: SR-967175026302197768900474181941935084891
title: "PLAT-1904 PR 751 evidence coverage code review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; PLAT-1904; PR 751; src/coverage.rs, src/obligation/inventory.rs, tests/it/coverage_matrix.rs, tests/it/coverage_rollup.rs, tests/it/formalizes_relation.rs, tests/it/relation_report.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-092", type: references}
---

## Summary

Independently reviewed the full six-file diff from `d7d418fa2727f7de87a16dfec5b38ada7d9ee776` to `a3c0d694da9a34b85e076441c4554303e1ac6896` for PLAT-1904, PR #751. Code review with the Rust lane found no defect. The explicit evidence mode and default-strict inventory policy extend the existing selective design-matrix path without changing generic caller behavior.

## Verdict

PASS for the changed surface. The public API signature changes deliberately require callers to select an evidence mode; all repository callers select Strict except the new selective regression. This engine review does not establish acceptance of the downstream Quoin host command.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Assurance Context

AP-201 (`spec/assurance/AP-201-detection-minting.md`) applies because this change controls which declarations enter coverage and inventory. The material scenario is an apparently complete coverage population after dropped declarations. Both new paths test the exact RequiredRelation discriminator; every other current and future declaration kind remains rejected by the inequality predicate. InventoryOptions defaults to Strict. The loader retains dropped declarations in the model, so the subsequent design matrix can still emit their errors.

The fixture is a real inline Registry load, real Spec filesystem walk, inventory, evidence coverage reconciliation, and design_matrix join. It begins with the attributed PLAT-1856 fixture and adds only target/obligation declarations and one criterion to exercise retained evidence. It asserts one retained inventory row and matrix entry plus the named relation error. A second real manifest adds a dropped Obligation and proves both inventory incompleteness and coverage refusal. The existing FlowRule generic refusal regression supplies another fatal kind and a healthy control. No detector threshold, source-extraction rule, dependency revision, or workflow changes. No new stub, vendored definition, test double, or weakened assertion was found.

Independent QA corpus implementation, new measurement records, and a full make ci run were not produced for this review; no claim of independent producer qualification or repository-wide coverage is made. Existing base and integrity reviews already cover FR-092-AC-20; this PR changes no specification.

## Validation

`make fmt-check` and `git diff --check` passed after initializing the pinned corpus submodule. `quire trace --scope . --file src/coverage.rs` recognized compute_with_evidence, compute_inner, and compute_coverage_for_design_matrix as structural implements claims for FR-092-AC-20; the corresponding inventory query recognized inventory. The real-engine regression carries the criterion trace tag. Generic evidence tests received only the explicit Strict argument.

A fresh exact-worktree build passed `cargo test --locked --test it dropped_record_key_relation_is_reported_with_its_valid_sibling -- --nocapture` (1/1). The resulting integration binary also passed `every_consumer_of_the_model_refuses_when_declarations_were_dropped` (1/1) and `selected_inventory` (12/12). The cargo command ran under locked-build.sh with CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, and CARGO_INCREMENTAL=0; target directory `/home/peter/dev/worktrees/codex-plat-1904-evidence-partial/target`. No broad test suite or clippy run was requested or performed. The build reports the existing optional architecture-fixture npm fetch warning; this regression does not use that package.

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

GitHub head remained `a3c0d694da9a34b85e076441c4554303e1ac6896`. At the final status check all reported checks passed or were neutral/skipped: CodeQL actions/Python/Rust and CLA gates were successful. GitHub reports MERGEABLE with mergeStateStatus BLOCKED; branch-protection approval remains separate from this clean source review.
