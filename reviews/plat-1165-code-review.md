---
id: SR-20261102118103
title: "PLAT-1165 native inventory code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/corpus/walk.rs; src/obligation.rs; src/obligation/inventory.rs; tests/obligations.rs; tests/fixtures/corpus_cases/selected-inventory*.json; PLAT-1165; PR551"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-053", type: references}
---

## Summary

Reviewed frozen f69068f7f00ac7321857905d8e22aede75a12054 against 33d82907a362784a6a0aca2e7dfb731da0189557. Ticket PLAT-1165, original PR551. Read-only review; no source edits, Cargo, backend rebuilds or CI runs. Code-review includes the Rust lane. No spec, manifest, lock, workflow or gitlink diff. No new scoped defect found. All three ticket acceptance items and all nine merged FR-053 criteria were examined against real public-consumer assertions.

## Assurance Context

AP-201 (spec/assurance/AP-201-detection-minting.md) applies to row minting, complete populations and diagnostics. Reviewed owner-only completion, original typed failure provenance, retained partial rows, deterministic identity and separation from evidence/freshness policy. Native integration controls share the implementation and are conformance evidence, not independent QA-corpus grading. Luna decision 7ed2e8d5 approves native controls, valid constants/degenerate zero-obligation Complete after exhaustion, and strict MissingSection. No FR-065 credit, QA extension or normative relaxation is claimed. Aggregate make ci, controlled corpus/qualification measurements, producer reliance records and broader architecture/impact/structural evidence were not supplied for this head; owner retains required final gates. No new spec-authoring sub-analyses apply to this implementation-only diff.

## Examined Units

- **FR-053-AC-15** (examined), spec/functional/FR-053-obligation-record.md: A valid document root, model and nonempty existing source-name selection yield the public owner-issued SelectedInventory and borrowable selection identity; empty, unknown or duplicate selectors and an invalid model yield typed non-Complete results, and the entry point accepts no caller Spec, RepoLoad, rows or completion state.
- **FR-053-AC-16** (examined), spec/functional/FR-053-obligation-record.md: A selected valid corpus with two authored criteria emits both, including one with no clause, source-symbol or test-case mapping, reports count two and Complete; changing a statement changes its owner statement, and deleting either authored row removes exactly that entry and changes the count.
- **FR-053-AC-17** (examined), spec/functional/FR-053-obligation-record.md: An external Rust consumer borrows the selection and origin-aware identities and owner statement/method/target data and performs exact lookup without Markdown/private-diagnostic/QSL ClauseRef parsing; AuthoredRow retains its authored ID, RenderedRow retains rendered ID plus row ordinal, and CombinatorialTable retains its rendered table ID, so deleting an earlier rendered row can renumber rendered IDs but never manufactures authored IDs.
- **FR-053-AC-18** (examined), spec/functional/FR-053-obligation-record.md: Missing/file/unreadable roots, unreadable documents and unclassifiable non-excluded Markdown (absent or malformed frontmatter) yield typed non-Complete with original cause/locus and retained healthy partial rows; valid unselected nonmember documents and paths excluded from every selected source contribute no entries and do not themselves block Complete.
- **FR-053-AC-19** (examined), spec/functional/FR-053-obligation-record.md: Missing selected section/table/required ID or statement column and unusable rows yield typed non-Complete; duplicate authored IDs in one selected source/document retain both rows and their loci, yield typed non-Complete and make exact lookup return ambiguity rather than first-wins, while other unambiguous partial lookups remain available; a nonempty valid selection with no selected documents or a valid empty headed table yields Complete with count zero after exhaustion.
- **FR-053-AC-20** (examined), spec/functional/FR-053-obligation-record.md: A real walk error or explicit interruption retains its original typed cause and partial rows and is non-Complete; a limit stops immediately when its emitted-entry count is reached without observing subsequent end-of-scan, so zero limit and limit N on an N-entry selection are non-Complete, while a greater-than-N or absent limit permits Complete only after actual normal end observation and no blocking cause.
- **FR-053-AC-21** (examined), spec/functional/FR-053-obligation-record.md: Selection identity equality uses canonical root, resolved selected declarations/effective exclusions and the exact path/byte snapshot of selected documents: changes within a selected document, even outside its criterion table, or changes to selected membership reject cross-inventory lookup as a typed mismatch; repeated identical captures compare equal, changes confined to valid still-unselected nonmembers, wholly excluded documents or files outside the root leave identity equal, and caller-truncated Spec or deserialized rows cannot issue Complete.
- **FR-053-AC-22** (examined), spec/functional/FR-053-obligation-record.md: Repeated exhaustive enumeration yields equal origin-aware identities, records, counts and terminal state in model declaration, relative document-path and authored row order; explicit exclusions remove exactly their selected documents, preserve healthy unexcluded criteria, and a changed selection/exclusion contract changes identity even when emitted records remain equal.
- **FR-053-AC-23** (examined), spec/functional/FR-053-obligation-record.md: Existing derive, single-document obligation and coverage consumers retain legacy healthy/skipped/tolerant-bad-root outputs; tests check unchanged statements, absence of evidence/freshness/discharge verdicts and GrammarFinding creation, and a producer API needing neither an IR schema copy nor a manually authored revision/hash ledger.
- **src/corpus/walk.rs** (examined), src/corpus/walk.rs: //! Parallel repository walk + parse — `load_repo` (FR-024, NFR-015).
- **src/obligation.rs** (examined), src/obligation.rs: //! Obligation records — the quire↔quoin contract (FR-053).
- **src/obligation/inventory.rs** (examined), src/obligation/inventory.rs: //! Owner-issued selected obligation inventories (FR-053-AC-15..23).
- **tests/obligations.rs** (examined), tests/obligations.rs: //! FR-053 — obligation records (TC-831..TC-843, TC-870..TC-873).
- **tests/fixtures/corpus_cases/selected-inventory.json** (examined), tests/fixtures/corpus_cases/selected-inventory.json: {
- **tests/fixtures/corpus_cases/selected-inventory-configurations.json** (examined), tests/fixtures/corpus_cases/selected-inventory-configurations.json: {
- **PLAT-1165-AC-1** (examined), Linear:PLAT-1165: A valid selected corpus with two authored criteria, including one with no mappings, reports both and an owner-issued terminal Complete; changing/removing one authored row changes the inventory.
- **PLAT-1165-AC-2** (examined), Linear:PLAT-1165: Missing/bad root, unreadable file, missing required section/ID column, invalid model, and partial scan have typed non-Complete results with original cause and retained partial rows where available; empty valid corpus is distinct.
- **PLAT-1165-AC-3** (examined), Linear:PLAT-1165: An external IR consumer can borrow the public typed inventory and selection identity without parsing spec Markdown, reading private diagnostics, or treating QSL `ClauseRef` as an AC identity. Focused deletion and source-selection mismatch controls demonstrate the boundary.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the reviewed changed scope. Final aggregate/hosted gates remain pending and merge authority remains Luna. This is not a repository-wide assurance PASS.

## Coverage

Plan completion: not assessed

Reconciliation: installed quire 0.36.2 (engine 0.50.2), matrix --scope explicitly set to this worktree --format json; no run-evidence store read. All nine changed-scope criteria are tagged, none ignored. The full repository matrix has 1020 criteria: 634 tagged, 279 untagged, 107 method-without-symbol. These global counts include retired/unchanged requirements and are disclosed as existing repository context; this PR review does not certify repository-wide assurance. Scope is the changed selected-inventory API and nine ACs, as directed by the reviewer role.

Read actual assertions and production flow for all bindings below. No new stubs, copied upstream schema/corpus, manually maintained revision ledger, evidence verdict or GrammarFinding creation was found. Shared row_obligation and existing collect_combinatorial remain the single owner minters; new row validation calls existing split_values/parse_exclusion. No Tokio or new unsafe surface.

Owner-supplied focused logs inspected: /tmp/plat1165-final-focused-tests.log: 26 PASS (15 legacy + 11 new), /tmp/plat1165-final-focused-docs.log: 3 compile-fail PASS, /tmp/plat1165-final-focused-clippy.log: PASS. /tmp/plat1165-degenerate-red.log demonstrates actual behavioral RED with restored old degenerate-space behavior. These were not independently rerun. No full gate or benchmark result is claimed. Indexed own-selection lookup uses pointer equality and nested BTreeMap keys; foreign equal captures intentionally pay exact snapshot comparison. No wall-time performance assertion.

## Verified Bindings

| Test | Criterion | Trace |
| --- | --- | --- |
| selected_inventory_validates_the_real_request | FR-053-AC-15 | correct |
| selected_inventory_declarative_controls | FR-053-AC-16 | correct |
| selected_inventory_borrowed_identity_changes_with_authored_source | FR-053-AC-16 | correct |
| selected_inventory_borrowed_identity_changes_with_authored_source | FR-053-AC-17 | correct |
| selected_inventory_rendered_rows_never_claim_authored_ids | FR-053-AC-17 | correct |
| selected_inventory_configuration_rows_follow_the_real_minter | FR-053-AC-17 | correct |
| selected_inventory_declarative_controls | FR-053-AC-18 | correct |
| selected_inventory_filesystem_failures_retain_original_causes_and_rows | FR-053-AC-18 | correct |
| selected_inventory_real_permission_and_walk_failures_have_healthy_controls | FR-053-AC-18 | correct |
| selected_inventory_declarative_controls | FR-053-AC-19 | correct |
| selected_inventory_duplicate_lookup_retains_every_locus | FR-053-AC-19 | correct |
| selected_inventory_configuration_rows_follow_the_real_minter | FR-053-AC-19 | correct |
| selected_inventory_stops_without_mistaking_a_prefix_for_exhaustion | FR-053-AC-20 | correct |
| selected_inventory_filesystem_failures_retain_original_causes_and_rows | FR-053-AC-20 | correct |
| selected_inventory_real_permission_and_walk_failures_have_healthy_controls | FR-053-AC-20 | correct |
| selected_inventory_borrowed_identity_changes_with_authored_source | FR-053-AC-21 | correct |
| selected_inventory_identity_exclusions_and_order_are_owner_facts | FR-053-AC-21 | correct |
| selected_inventory_identity_exclusions_and_order_are_owner_facts | FR-053-AC-22 | correct |
| selected_inventory_preserves_legacy_consumers_without_evidence_verdicts | FR-053-AC-23 | correct |

## Historical Finding Reconciliation

Original SR-16297/SR-16298 findings in ticket comments cb56c05b/ba16b438 were read in full and remain unchanged. This takeover review independently checks their present state; this table does not rewrite their original findings or pretend to be their original reviewer.

| Artifact | Finding | Outcome | Reason |
| --- | --- | --- | --- |
| SR-16297 | FND-001 | fixed | Malformed combinatorial rows are checked independently and retain ordinal/line even when good dimensions still mint; valid constants are healthy per Luna/FR-061. |
| SR-16297 | FND-002 | fixed | Removed blanket empty-output UnusableRow: syntactically valid one-dimension/constants/empty spaces exhaust to zero Complete; declarative controls assert it. |
| SR-16297 | FND-003 | accepted-no-change | Luna 7ed2e8d5 explicitly retains strict AC-19 MissingSection. Optional source semantics require a separate spec-first decision. |
| SR-16297 | FND-004 | fixed | Duplicate causes use captured original LoadedDocument.path via document_index; exact canonical absolute path tested. |
| SR-16297 | FND-005 | fixed | Pointer fast path and emission-built exact source/document/ID/origin index replace repeated own-snapshot comparison and linear scan; ambiguous buckets remain ambiguous. |
| SR-16297 | FND-006 | fixed | thiserror source detail forwards original IO/ignore objects, meaningful Display; public integration tests downcast original errors. |
| SR-16298 | FND-001 | fixed | FR-053-AC-15 now has committed public integration bindings with concrete assertions; no external-probe or FR-065 grading substitution. |
| SR-16298 | FND-002 | fixed | FR-053-AC-16 now has committed public integration bindings with concrete assertions; no external-probe or FR-065 grading substitution. |
| SR-16298 | FND-003 | fixed | FR-053-AC-17 now has committed public integration bindings with concrete assertions; no external-probe or FR-065 grading substitution. |
| SR-16298 | FND-004 | fixed | FR-053-AC-18 now has committed public integration bindings with concrete assertions; no external-probe or FR-065 grading substitution. |
| SR-16298 | FND-005 | fixed | FR-053-AC-19 now has committed public integration bindings with concrete assertions; no external-probe or FR-065 grading substitution. |
| SR-16298 | FND-006 | fixed | FR-053-AC-20 now has committed public integration bindings with concrete assertions; no external-probe or FR-065 grading substitution. |
| SR-16298 | FND-007 | fixed | FR-053-AC-21 now has committed public integration bindings with concrete assertions; no external-probe or FR-065 grading substitution. |
| SR-16298 | FND-008 | fixed | FR-053-AC-22 now has committed public integration bindings with concrete assertions; no external-probe or FR-065 grading substitution. |
| SR-16298 | FND-009 | fixed | FR-053-AC-23 now has committed public integration bindings with concrete assertions; no external-probe or FR-065 grading substitution. |
| SR-16298 | FND-010 | fixed | InventoryOptions.interrupt polls emitted progress and retains the original boxed Error through the cause chain; tests stop at 0/1/2 emitted entries. |
