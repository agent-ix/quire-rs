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
  A declaration MAY additionally request two normalizations of the cell before
  ids are extracted, both off by default (CR-015): `expand_ranges` turns a
  same-prefix range (`FR-001..FR-006`) into its concrete ids, and
  `strip_annotations` drops parenthetical spans so a qualifier
  (`FR-022-AC-5 (superseded by FR-030)`) contributes one reference, not two.
- **Status vocabulary** — the matrix status column and which values class as
  `complete`, `pending`, `failed`, or `retired`. A cell SHALL class by its
  **leading marker**, so an authored qualifier (`✅ Complete`,
  `⚠️ scale evidence deferred`) classes correctly while keeping the note that
  carries why (CR-015).
- **Column vocabularies** — a `vocabularies:` block declaring the values a
  matrix column admits (first entry: `test_type`). The engine SHALL treat these
  as the single source for both validation and the rollup, so a module's
  contract and its coverage computation cannot disagree about what a column
  means (CR-015).
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
| FR-050-AC-10 | A status cell carrying a trailing note (`✅ Complete`) classes by its leading marker, and a value declared in the `retired` class classes as retired rather than unknown. | Test (TC-758) |
| FR-050-AC-11 | A declared `vocabularies.test_type` is exposed on the `Registry` as the core values plus the module's extensions, and is the same list a matrix contract validates against. | Test (TC-759) |
| FR-050-AC-12 | With `expand_ranges` declared, `FR-001..FR-003` resolves as three references; with `strip_annotations` declared, `FR-022-AC-5 (superseded by FR-030)` resolves as one. Both are off unless declared. | Test (TC-760) |
| FR-050-AC-13 | A report over a corpus whose documents carry criteria contains a `criteria` entry per contributing document and the two new totals; a corpus whose documents carry none contains an empty `criteria` list and serializes byte-identically to a report from an engine that predates the field. | Test (TC-788) |

> **CR-027 note (2026-08-07):** `CoverageReport` grows a `criteria` field and
> `CoverageTotals` grows two counts — a criteria count and a property-shaped
> count — carrying the classification
> [FR-052](./FR-052-acceptance-criteria-property-classification.md) computes.
> The rollup gains a grammar dependency and **no acceptance-criteria knowledge**:
> it walks the already-path-sorted corpus, asks FR-052 for per-document counts,
> and skips any document yielding none, so a non-requirement corpus produces an
> empty list and output identical to today's. The counts are declaration-free by
> necessity — the process module declares exactly one trace target and no
> document references over criteria rows, so there is no group to hang a
> declared column on, and a declaration-driven column would ship structurally
> present and permanently unpopulated.
>
> The FR-050-AC-7 byte-identical guarantee therefore now covers content this FR
> does not otherwise describe. That is the point of stating it here rather than
> in FR-052: the ordering discipline is this FR's, the new fields sort into the
> same determinism block as every other, and `#[serde(default)]` on all three
> keeps a report written by an older engine round-tripping.
>
> FR-050-CON-1 is unaffected. A count is not a verdict, and a low
> property-shaped count is not a failing corpus — CR-020 already recorded that
> StR criteria legitimately score low. Judgment stays in the consuming workflow.

> **CR-015 note:** The ecosystem sweep behind FR-003 (report:
> `spec-artifacts-process/reports/2026-08-04-tests-md-sweep.md`) found the
> matrix vocabularies declared in two places — a module's `column_choices` and
> this model's status vocabulary — and drifting. It also found the authored
> corpus using forms the model could not express: statuses carrying the reason
> they are partial (`⚠️ scale evidence deferred`), a retired class, and
> `Traces To` cells written as ranges or carrying parenthetical qualifiers. This
> amendment makes the model the single source for column vocabularies, classes
> statuses by leading marker, and moves range expansion and annotation stripping
> into declared, default-off normalizations so the engine gains no behaviour a
> module has not asked for.

## Dependencies

- **Upstream**: [FR-051](./FR-051-source-symbol-extraction.md) (the symbol graph and trace tags), [FR-027](./FR-027-whole-spec-query-api.md) (corpus queries), [FR-014](./FR-014-module-activation.md) (manifest loading), [FR-010](./FR-010-query-api.md) (table extraction)
- **Upstream (added CR-027)**: [FR-052](./FR-052-acceptance-criteria-property-classification.md) (the per-criterion property classification the `criteria` counts summarize)
- **Downstream**: [FR-049](./FR-049-verification-reference-integrity.md) (reference declarations reused by bundle validation); `spec-artifacts-iso` declares the ISO model (follow-up change in that module); the `gap-analysis` workflow replaces its grep step with `quire coverage`
