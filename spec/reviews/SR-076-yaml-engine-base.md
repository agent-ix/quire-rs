---
id: SR-076
title: "Base review of the YAML engine maintenance decision (#414)"
type: SpecReview
analysis: base
scope: "ADR-0012, NFR-009, TC-330..TC-332"
review_set: subset
---

## Summary

> **Superseded by SR-087.** This review evaluates the withdrawn 0.10.2
> candidate and remains only as decision history.

ADR-0012 and NFR-009 are structurally valid and their identifiers are well
formed, but the proposed decision is not ready for acceptance. The normative
dependency requirement still names the obsolete package choices, its current
executable audit does not enforce its pinning rule, and the matrix does not yet
trace the decision-specific compatibility and dependency controls. This review
was rendered after an invalid AP-201 appeared to require the subset; it is a
provisional review input, not evidence of an owner-selected review set.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-427 | high | ADR-0012 selects the `yaml_serde` package under the existing `serde_yaml` import name, while NFR-009 still defines the load-bearing dependency as `serde_yaml` or `serde_yml`, directs an inactive-upstream swap to `serde_yml`, and repeats those names in AC-1 and M-1. The ADR cannot become accepted while its owning normative requirement contradicts the decision. | ADR-0012 (Decision); NFR-009 (Load-bearing dependencies, AC-1, M-1) | wrong-requirement |
| FND-428 | high | The matrix maps NFR-009-AC-1..3 to TC-330..TC-332 but explicitly leaves AC-4 as an untested process criterion; all four criteria are currently unbacked. No criterion or TC covers the selected-package identity, rejection of the deprecated package, two-engine corpus differential, typed YAML consumers, Rust 1.75 build, advisory/license gates, or lock/tree resolution required by ADR-0012. | NFR-009-AC-1..4; spec/tests.md TC-330..TC-332; ADR-0012 (Required implementation evidence) | missing-requirement |
| FND-429 | high | `scripts/audits/check_dep_pins.sh` rejects wildcard forms only and says caret is permitted, while NFR-009 requires tilde or exact pins for every load-bearing dependency; `Cargo.toml` currently declares `serde_yaml = \"^0.9\"`. TC-330 and TC-332 therefore name controls that the executable audit does not perform. | NFR-009-AC-1, NFR-009-AC-3; TC-330, TC-332; scripts/audits/check_dep_pins.sh; Cargo.toml | implementation-bug-despite-evidence |
| FND-436 | high | The mandatory full-scope validation fails 11 existing documents. The governing AP-201 is itself invalid against the installed AssuranceProfile schema: `review_selection`, `impact`, and `lifecycle` are rejected, its impact-assessment shape is obsolete, and four required body sections are absent. Eight MP-201..MP-208 documents also miss the current MeasurementPlan sections, and two untyped assets fail. The enforced review cannot advance to `validated` until the owning assurance artifacts or pinned schema contract are reconciled. | AP-201; MP-201..MP-208; spec/assets/external-blockers.md; spec/assets/render-parity-notes.md | wrong-requirement |
| FND-437 | high | AP-201 fails its installed versioned schema, so its `review_selection.mode: require` cannot select the #414 review set. The existing workflow intake recorded that invalid selection without an owner choice. Its rendered documents may be reused as provisional analysis inputs, but the run cannot establish `/spec-review`; a fresh intake must record `base`, `all`, or an owner-named subset. | AP-201; workflow dd759f54-7dfd-40ff-b16b-b5947a42cf1e; ADR-0012 | wrong-requirement |

## Coverage

- ID and link integrity: ADR-0012, NFR-009, NFR-009-AC-1..4 and TC-330..332
  use valid, non-duplicate identifiers; both reviewed documents are
  grammar-clean under Quire 0.31.0.
- Coverage: AC-1..3 have matrix rows but no bound implementation evidence;
  AC-4 has no TC. Decision-specific obligations are absent (FND-428).
- Option permutation: the ADR compares six dependency strategies, but the
  accepted option needs only positive selected-package and negative deprecated,
  wrong-version, caret and wildcard cases.
- Constraint boundaries: exact, tilde, caret, wildcard, wrong package, wrong
  patch and incompatible-MSRV cases are not all named by TC-330/332.
- Error paths: the ADR lists parity, typed-parse, MSRV, license and advisory
  blockers, but they are prose rather than matrix obligations.
- State transitions: proposed to accepted and accepted to reopened are governed
  by the ADR; no implementation may start from the proposed state.
- Edge cases: duplicate keys, merge keys, implicit scalars, aliases and
  non-string mapping keys are present in the spike, but not yet retained as
  versioned test inputs.
- Validation gate: the two target specifications and all three review documents
  validate individually; the required repository-wide scoped command fails on
  the pre-existing assurance/assets set described by FND-436.

## Dispositions after specification amendment

| ID | Disposition |
| --- | --- |
| FND-427 | Fixed in the candidate specification: NFR-009 now names the `yaml_serde` package, exact 0.10.2 pin, complete production YAML surface, and ADR-0012 reopen rule. |
| FND-428 | Fixed in the candidate specification: NFR-009-AC-4..10 and TC-1820..TC-1825 now mint and map the decision-specific qualification obligations. All remain pending implementation. |
| FND-429 | Specification and matrix mismatch fixed; implementation remains deliberately open. The audit and manifest are unchanged until the repeated review is validated and owner-accepted. |
| FND-436 | Open. The repository-wide validation and governing assurance-schema compatibility must be repaired or explicitly pinned by their owner before this workflow advances to `validated`. |
| FND-437 | Open. The invalid profile selection is not treated as authority. Start a fresh review intake and record the owner-selected set before any review can be accepted or implementation can begin. |

The amended ADR, NFR, and matrix are 3/3 grammar-clean. This disposition is a
provisional repeat review of specification changes, not a valid selection,
owner acceptance, or permission to implement.
