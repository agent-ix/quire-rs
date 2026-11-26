---
id: SR-967175026302197768900474181941935084883
title: "Code review with Rust lane of PLAT-1904"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; PLAT-1904; spec/functional/FR-092-matrix-design-column.md, src/coverage.rs, tests/it/relation_report.rs"
review_set: subset
---

## Summary

Ticket PLAT-1904, PR #744. At original reviewed head `d28071553961781445709c912c048f5a793e3869`, the helper implementing FR-092-AC-20 lacked its implementation marker, and the real-engine test did not prove that non-relation drops remained fatal. Both findings were rechecked at `9595e7170ca96751167af73b55d6851063b60f46` and fixed in commits `83d8527e` and `9595e717`; no further code defect was found. Ordinary `compute` and `compute_with_evidence` remain on the strict path.

Repository conventions and Rust lane reviewed. No dependency version move or workflow change is in the diff. The lead's locked focused real-engine rerun passed 1/1 at the current source; no full suite was run here.

## Assurance Context

Applicable profile: `spec/assurance/AP-201-detection-minting.md` (active; scope includes coverage collection and row minting). The changed code gates coverage collection based on dropped declaration kinds. The branch adds a real-engine test, but there is no independent QA corpus implementation evidence specific to this selective path. No population or measurement definition changed. Per the owning minder's ticket-specific disposition, no ix-flow run or separate Peter acceptance is required for PLAT-1904.

## Verdict

PASS — both initial findings are fixed at the current head and the focused real-engine rerun passed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `compute_inner` implements the new FR-092-AC-20 exception but lacked the required bare `#[implements("FR-092-AC-20")]` marker. | src/coverage.rs:946 |
| FND-002 | medium | The FR-092-AC-20 integration case exercised a dropped required relation but did not assert that another dropped declaration kind still makes the selective API fail. | tests/it/relation_report.rs:930 |

## Examined scope

- `spec/functional/FR-092-matrix-design-column.md`, FR-092-AC-20: `compute_coverage_for_design_matrix` returns retained coverage only when required relations were dropped; other dropped kinds and ordinary coverage remain strict. Examined.
- `src/coverage.rs`, `compute`, `compute_coverage_for_design_matrix`, `compute_with_evidence`, and `compute_inner`: existing APIs pass the strict flag; the selective path rejects any dropped kind other than `RequiredRelation`; the two changed functions now carry `#[implements("FR-092-AC-20")]`. Examined.
- `tests/it/relation_report.rs`, `dropped_record_key_relation_is_reported_with_its_valid_sibling`: real fixture loads dropped relation and valid sibling, strict `compute` errors, selective coverage produces one entry, relation report preserves one valid row and the dropped relation error; second case loads a dropped `Obligation` and asserts the selective API errors. Examined.
- `src/corpus/relation_report.rs`, `dropped_relations` and `ensure_non_relation_declarations_complete`: context only; confirms relation report filters required-relation drops for partial output and refuses other drops.

## Validation and limitations

`quire trace --file src/coverage.rs` confirms `compute_coverage_for_design_matrix` and `compute_inner` implement FR-092-AC-20; the integration test carries `#[trace("FR-092-AC-20")]`. `cargo fmt --check` passed. The lead reports `cargo test --locked --test it relation_report::dropped_record_key_relation_is_reported_with_its_valid_sibling` passed 1/1 (1,002 filtered) using the locked-build target. No full suite or clippy run was performed by this reviewer.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Added the bare criterion marker in commit 83d8527e. |
| FND-002 | fixed | Added the dropped `Obligation` case in commit 9595e717. |

```yaml
dispositions:
  - fnd: FND-001
    outcome: fixed
    after_excerpt: |-
      #[implements("FR-092-AC-20")]
      fn compute_inner(
  - fnd: FND-002
    outcome: fixed
    after_excerpt: '    assert!(compute_coverage_for_design_matrix('
```
