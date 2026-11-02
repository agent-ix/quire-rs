---
id: SR-16302
title: "PLAT-1236 engine spec-review/scope-boundary review"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-rs; src/coverage*, src/traceability.rs, src/loader/mod.rs, schemas/output/coverage-v1.schema.json, tests/coverage*, tests/corpus_cases.rs, tests/output_contract.rs; PLAT-1236; PR 564"
review_set: subset
---

## Summary

Engine owns observation and generic module-classified retirement; spec-artifacts-process owns the concrete Withdrawn vocabulary and Verification lint; quire-cli owns strict exit and rendering. The engine adds no hardcoded Withdrawn classification. Declared IDs remain referenceable and actual binders remain factual. Changing active totals is accompanied by separate historical counts/schema and an actual CLI companion rather than a fake engine-only strict helper. QVC/QSL/Kani/pin-purge and credential changes are outside this PR.

Reviewed production engine head b18cc5e0c36cefcdbefd244cc744af19f5f410c9 plus the explicitly authorized tests-only attribution correction bc59d73545fd652a040d48daf2ab32f24a35d90e against merged spec ce542e302c2312ab4974aaeacc2a3a537499c444. No source edits or Cargo runs. Parent owns final hosted aggregate. Candidate command probes use CLI 1de79cff / engine de973da, not the frozen final engine binary; final-head defects were separately confirmed by source inspection. Probe receipts are in this scratch directory.

## Assurance Context

AP-201 (spec/assurance/AP-201-detection-minting.md) applies to coverage collection, row minting, diagnostics and published schemas, with material evidence-interpretation impact. Evidence and scope-boundary methods ran. Selected controls, baseline and schema checks were inspected; no enforced review-policy run, independent producer qualification or final hosted aggregate is claimed. The QA and engine graders share data and cannot establish complete technique independence. Source/runtime receipts retain their exact producer heads. No observation-stage metric was promoted into a gate.

## Examined Units

- **FR-050-AC-3** (examined): A live reference row whose trace target has no backing `verifies` relation appears in the report's unbacked rows with the row id and its target id.
- **FR-050-AC-4** (examined): A live row whose status classes as `complete` with no backing source symbol appears in the report's status lies; the same row with a backing symbol does not.
- **FR-050-AC-6** (examined): The report carries per-minting-document live backed/total and separate retired counts, and their sums equal the corresponding bundle-wide totals.
- **FR-050-AC-38** (examined): The report carries one `minted_targets` record per minted row with its id, target declaration, document, 1-based row line and backed state, deterministically ordered by target, document, id and line. The record count equals `totals.total + totals.retired` (an omitted retired count is zero); retired records carry `retired: true` and retain their actual backing. Live records whose state is backed equal `totals.backed`. An empty minted population omits the additive field under FR-055-CON-3 rather than inventing rows; a current CLI advertises the `minted_targets` capability so a census can refuse 
- **FR-050-AC-47** (examined): The report carries a `coverage_matrix` whose population is exactly the [FR-053](./FR-053-obligation-record.md) `Obligation` records the module's declared `obligations:` sources produce (`obligation::derive` on the legacy path; the supplied FR-053 selected inventory on the explicit FR-078 evidence path) — never a `MintedTargetRecord` by itself. Each obligation becomes one criterion, grouped by its own `document` field into a `requirements[]` entry per distinct document. Its criterion population is built only from those obligations; document-reference declarations and auxiliary trace-target sour
- **FR-050-AC-49** (examined): Each criterion carries a computed `status`, exactly one of `retired`, `tagged`, `untagged`, `tagged-by-ignored-test`, `method-without-symbol`, or (only in explicit FR-078 evidence mode) `qualified-record` — never read from an authored column. Explicit retirement yields `retired` first, preserving every actual binder and making no qualification claim. For live criteria, one or more accepted record binders yields `qualified-record` first, preserving every source binder. Without accepted record binders, `method-without-symbol` applies when the criterion's declared `method` is in the module's decl
- **FR-050-AC-53** (examined): A fixture with one declared `Withdrawn` verification row, one live Complete unbacked row and one Planned unbacked row reports only the two live rows as unbacked and only the Complete row as a status lie. The retired row retains its exact id, statement/reason, authored cells and locus in `retired_rows`, and its minted record remains unbacked with `retired: true`; live totals and separate retired totals reconcile with the minted records. The mixed scope fails `--strict` because of the live rows.
- **FR-050-AC-54** (examined): A nonempty all-retired fixture passes `--strict` when no other strict premise fails; the same scope with no declared targets still fails the empty-population premise. A fixture with only live unbacked rows continues to fail. An incomplete explicit evidence inventory still fails beside retired rows.
- **FR-050-AC-55** (examined): Only module-declared exact normalized retirement values and the existing selected `retired` status class retire a row. A misspelled value, a retirement word appearing only in Criteria prose, an undeclared `Withdrawn`, and a live row sharing another document's retired id remain live. A referenced target is not retired by a retiring row. Blank retirement vocabulary values or overlap with `no_source_symbol` fail module load.
- **FR-050-AC-56** (examined): Retired obligations remain in the computed matrix with their complete statement, authored method and actual binders, and computed status `retired`. Configured retirement markers produce no uncatalogued-verification-method warning; unknown methods still do. Retiring a row does not add a source binder or turn its backing fact true.
- **FR-050-AC-57** (examined): A fixture using the existing declared retired status marker receives the same retirement treatment as a declared verification-cell retirement. Repeated reports are byte-identical; a model declaring no retired rows emits no additive retired keys and preserves its prior live counts, unbacked rows and status lies.

## Attribution Guard Delta

The bc59d73 test-only change aligns the consuming issue_ref guard with the qa-corpus declared nonblank filing contract. Existing required-field mutation checks remain unchanged. Real Linear/GitHub controls and empty/whitespace negatives are present. The old GitHub-only punctuation requirement had no owner contract; no fabricated GitHub filing or source-coverage suppression is introduced.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for this method within the examined scope; final aggregate/release qualification remains a lead responsibility.
