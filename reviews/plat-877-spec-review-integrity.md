---
id: SR-16328
title: "Spec review (integrity) — PLAT-877 FR-051-AC-31 / CR-206 / TC-2082 (quire-rs PR 607)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md (AC-31, CR-206 note), spec/tests.md (FR-051 matrix row, TC-2082, audit row)"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references}
---

## Summary

Ticket: PLAT-877. PR head: `1e404e9`. This review covers spec integrity and EARS form for the new FR-051-AC-31, the CR-206 note, and TC-2082. CR-205 and TC-2081 are skipped on purpose because PR #606 claims them. The new ids do not collide with existing ones. The FR-051 matrix row range is updated to AC-1..31, and the audit mapping row is added.

## Verdict

**PASS with findings.**

- **Atomicity and testability.** AC-31 is atomic and testable. It names the trigger, the reason token and the payload, and it has an explicit negative clause.
- **Consistency.** The CR-206 note states the ruling (keep identity, report the collision) and is consistent with the code.
- **TC-2082.** The row's text matches its tests.

Both defects concern how a consumer and a reader are meant to treat the new diagnostic.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-31 states no severity, whereas AC-28 pins FR-057 `warning` for its finding. `SymbolDiagnostic` has no severity field. quire-cli prints every extraction diagnostic through `emit_diagnostic` (the error form, not `warning:`). As a result, benign same-id pairs show as errors on every `coverage`, `symbols` and `assurance` run. Exit codes are unchanged. Examples are quire-rs's own `From<A>`/`From<B>` impls (evidence.rs) and quire-cli's cfg-gated `libc_eloop`. State the severity, for example advisory or `warning`. | spec/functional/FR-051-source-symbol-extraction.md:179 |
| FND-002 | low | The AC-31 examples and the CR-206 note frame collisions as cfg variants and branch redefinitions. On measured trees the main non-cfg cause is distinct functions that the identity scheme merges. All of quire-rs `src` (2 of 2) is Rust trait-impl methods qualified under the type, not the trait. auth-fastapi has 5 groups of Python nested helpers qualified under the class, and ecaz's `psql.rs` has three `ConnectTarget` impls. A reader cannot tell from the spec that these are expected hits. | spec/functional/FR-051-source-symbol-extraction.md:181-187 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-051-AC-31 | examined | FND-001 |
| CR-206 note | examined | FND-002 |
| FR-051-AC-28 | context_only | Severity precedent |
| spec/tests.md FR-051 matrix row | examined | Range AC-1..31; TC-2082 listed |
| spec/tests.md TC-2082 row | examined | Matches the tests |
| spec/tests.md audit row FR-051-AC-31 | examined | Present |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 6b9c5af: AC-31 now states severity `warning` (as AC-28 does): advisory, carried on the symbol-table report, and never changes an exit code or refuses a result. The code adds `DiagnosticSeverity` to `SymbolDiagnostic` and to the report's `FileDiagnostic`, omitted when Error. That covers the report field; no further spec line is needed. quire-cli still prints extraction diagnostics through its error path, which is a quire-cli rendering choice outside this repo. |
| FND-002 | fixed | 6b9c5af: AC-31 and the CR-206 note now list the causes: cfg variants, trait-impl methods differing only by generics, Python branch and nested-helper repeats, and TypeScript duplicate titles. They also say the diagnostic reports shared identity and does not claim a defect. |
