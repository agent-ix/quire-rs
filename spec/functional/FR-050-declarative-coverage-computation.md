---
id: FR-050
title: "Declarative Coverage Computation"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-017"
    type: "implements"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-051"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-027"
    type: "requires"
    cardinality: "1:1"
---
# FR-050: Declarative Coverage Computation

## Description

`quire-rs` SHALL compute a deterministic requirement-coverage rollup — the
FR→AC→TC→test reconciliation the `gap-analysis` workflow currently performs by
grepping — and `quire-cli` SHALL expose it as `quire coverage`.

The engine SHALL NOT hardcode FR/AC/TC semantics. Coverage is driven by a
**declarative traceability model** that a Filament module declares in its
`manifest.yaml` under a `traceability:` section — the same
spec-semantics-as-module-data pattern as `body_extraction`, `lint_rules`, and
`grammar_ref`. The model SHALL declare:

- **Trace targets** — which archetype + section + table + id column mints
  trace ids (e.g. the FR `Acceptance Criteria` `ID` column via its existing
  `id_pattern`), including targets minted by an auxiliary trace source outside
  the corpus walk (e.g. Test Matrix rows in `spec/tests.md`).
- **Document references** — which columns or annotations reference which
  target kinds (e.g. the `Verification` cell annotation `Test (TC-nnn)` → TC
  targets; the matrix `Traces To` column → requirement/AC targets). These
  declarations also drive [FR-049](./FR-049-verification-reference-integrity.md).
- **Status vocabulary** — the matrix status column and which values class as
  `complete`, `pending`, or `failed`.
- **Trace-tag grammar** — a reference to the source-tag patterns
  ([FR-051](./FR-051-source-symbol-extraction.md)) that bind source symbols to
  trace ids.

Given a loaded `Spec` corpus, a `Registry` with a declared model, and a symbol
graph from [FR-051](./FR-051-source-symbol-extraction.md), the engine SHALL
reconcile declared targets, declared references, and scanned source tags
generically, and SHALL emit a machine-readable report containing:

1. **Unbacked rows** — declared reference rows (e.g. matrix test cases) whose
   trace target has no backing `verifies` relation from any source symbol.
2. **Status lies** — rows whose status classes as `complete` while their
   target has no backing source symbol.
3. **Untracked symbols** — source symbols carrying a trace tag that resolves
   to no declared target or reference row.
4. **Per-target-group counts** — for each minting document (e.g. each FR),
   backed and total trace-target counts.

`quire coverage [PATHS] --scope <DIR> [--source <DIR>]...` SHALL print the
report as JSON on stdout; repeated runs over identical inputs SHALL emit
byte-identical output (NFR-006 ordering discipline). When the active modules
declare no traceability model, the command SHALL exit with a distinct
diagnostic instead of an empty report.

A module other than `spec-artifacts-iso` obtains coverage by declaring its own
model; the engine knows nothing of "AC" or "TC" as concepts.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-050-CON-1 | Verdict policy (PASS/CONDITIONAL/FAIL), review gating, and SpecReview authoring SHALL remain outside quire — the `gap-analysis` workflow consumes the report and owns judgment. | Architecture | Inspection |
| FR-050-CON-2 | The coverage computation SHALL perform no network or service I/O; inputs are the corpus, the registry, and local source trees. | Architecture | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-050-AC-1 | A manifest `traceability:` section declaring trace targets, document references, a status vocabulary, and a trace-tag grammar loads, and the `Registry` exposes the declared model. | Test (TC-732) |
| FR-050-AC-2 | A malformed `traceability:` section fails module load like any other manifest shape error; an absent section loads and marks the model undeclared. | Test (TC-733) |
| FR-050-AC-3 | A reference row whose trace target has no backing `verifies` relation appears in the report's unbacked rows with the row id and its target id. | Test (TC-734) |
| FR-050-AC-4 | A row whose status classes as `complete` with no backing source symbol appears in the report's status lies; the same row with a backing symbol does not. | Test (TC-735) |
| FR-050-AC-5 | A source symbol whose trace tag resolves to no declared target or row appears in the report's untracked symbols with its file and symbol name. | Test (TC-736) |
| FR-050-AC-6 | The report carries per-minting-document backed/total counts, and their sum equals the bundle-wide totals. | Test (TC-737) |
| FR-050-AC-7 | Repeated `quire coverage` runs over identical corpus, model, and source inputs emit byte-identical JSON. | Test (TC-738) |
| FR-050-AC-8 | A fixture module with a non-ISO vocabulary (different archetype, id pattern, and status values) obtains a correct rollup from its own declaration, with no engine change. | Test (TC-739) |
| FR-050-AC-9 | When no active module declares a traceability model, `quire coverage` exits non-zero with a diagnostic naming the missing declaration. | Test (TC-740) |

## Dependencies

- **Upstream**: [FR-051](./FR-051-source-symbol-extraction.md) (the symbol graph and trace tags), [FR-027](./FR-027-whole-spec-query-api.md) (corpus queries), [FR-014](./FR-014-module-activation.md) (manifest loading), [FR-010](./FR-010-query-api.md) (table extraction)
- **Downstream**: [FR-049](./FR-049-verification-reference-integrity.md) (reference declarations reused by bundle validation); `spec-artifacts-iso` declares the ISO model (follow-up change in that module); the `gap-analysis` workflow replaces its grep step with `quire coverage`
