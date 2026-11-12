---
id: SR-20261112034127
title: "spec-review (base) of PLAT-1476 located engine findings (quire-rs PR 653)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@cc37f57; spec/functional/FR-033-locator-assert-facet.md; spec/functional/FR-049-verification-reference-integrity.md; spec/functional/FR-069-semantic-module-contract-at-load.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-069
    type: references
  - target: ix://agent-ix/quire-rs/FR-049
    type: references
  - target: ix://agent-ix/quire-rs/FR-033
    type: references
---

## Summary

Ticket: PLAT-1476. PR head `cc37f57`, base `origin/main` `a224155`. Spec-only diff: FR-033-AC-18, FR-049-AC-19..21, FR-069-AC-21..27, TC-2300..TC-2310. Each mechanism claim was checked read-only against the engine at the PR base (`src/validate_document.rs`, `src/extract/mod.rs`, `src/extract/locator.rs`, `src/corpus/trace_refs.rs`, `src/corpus/declared_tables.rs`, `src/corpus/validate.rs`) and against the spec-artifacts-process module at `origin/main` `e4e1a21` (`manifest.yaml` Anomaly entry, `schemas/Anomaly.json`, `Section.json`, `Provenance.json`, `AssessmentRow.json`, `examples/Anomaly.record.json`). Review set is the base checklist plus the integrity lens (separate SR).

Verified true: `validate_document` validates only the declaration record, and only its first error (`validator.iter_errors(&declaration).next()`, validate_document.rs:470). FR-081 line 111 forbids reading a record key from frontmatter. `ScannedRow.line` exists as a 1-based, frontmatter-counted line (declared_tables.rs:25-28). The line-1 fallback matches the existing `FRONTMATTER_LINE`. Not true as stated: a yield record does exist. `extract::extract` builds it (`eval_match`, extract/mod.rs:87-121). Only the validate path never builds it.

## Verdict

**FAIL.** The FR-049 half is sound and close to mergeable. The FR-069 half specifies a mechanism that cannot do the job that motivates it. The record a `yield_pattern` builds is not the record `Anomaly.json` describes. Renaming match keys cannot close that gap, so the only consumer named (the Anomaly archetype) could not opt in without every valid document failing. AC-27 would still pass, because its fixture is only "Anomaly-shaped".

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The yield record is validated against the archetype's own `data_schema`, but for Anomaly that is the emitted-record schema and no yield record can satisfy it. `description` must be a `Section` object (`text`/`startLine`/`endLine`), but `section_body` yields a string. `provenance` needs a computed `path` and `digest` that no locator can produce, and it is required. `assessment` rows need lowercase `kind`/`impact`/`rationale` plus a computed `line`, under `unevaluatedProperties: false`. Renaming match keys fixes none of this. Fix: `record_validation` names its own schema for the yield record (for example `record_validation: {schema: schemas/AnomalyFields.json, locate_absence_at: status}`). Names stay verbatim, so the engine stays free of name mapping | spec/functional/FR-069-semantic-module-contract-at-load.md:273-281 |
| FND-002 | medium | "in addition to the declaration record" validates two different records against one schema. The FR-081 declaration record reads `status` and `provenance` from sections. The yield record reads them from frontmatter. A schema that requires a key of one fails the other, so one schema cannot describe both | spec/functional/FR-069-semantic-module-contract-at-load.md:273-276 |
| FND-003 | medium | The FR does not say which validation layer runs the yield record, or from which entry points. Today semantic and record checks run only for a resolved `object:` archetype inside `validate_in_registry_core` (validate_document.rs:596-607), and the type-only `validate_document` runs none. Anomaly is an `artifact_types` entry dispatched by frontmatter `type:`. The FR names only `validate_document`, yet AC-27 needs the type layer through the registry or bundle path | spec/functional/FR-069-semantic-module-contract-at-load.md:273 |
| FND-004 | medium | Locating "element i at the line of the ith row or item" by index is wrong whenever values are dropped before the array is built. A `regex` filter drops non-matching rows, a `column` locator skips rows without that cell, whole-value placeholders are dropped (FR-011-AC-17), and a fallback chain switches primitive. Each value must carry its own line through filtering. A `frontmatter_field` YAML list also has no per-item line under FR-033-AC-18 | spec/functional/FR-069-semantic-module-contract-at-load.md:293-299 |
| FND-005 | medium | FR-033-AC-18 is ambiguous on two locators. For `frontmatter_field` "the line of its key" does not say which key when `path` is nested (`[a, b]`), or what happens for flow-style YAML. For `code_block` "the line of the heading (or fence)" does not say which one applies when (`under_section` set or not) | spec/functional/FR-033-locator-assert-facet.md:87 |
| FND-006 | medium | One defect can be reported twice. A `required: true` match locator that is absent already gets a validate_body `missing` finding. If the yield schema also requires that key, a second `semantic.record-invalid` fires at a different line (the `locate_absence_at` line or line 1). The same holds for an `assert: choices` failure against a schema `enum` on the same value. The FR does not say whether both fire | spec/functional/FR-069-semantic-module-contract-at-load.md:282-292 |
| FND-007 | medium | FR-069-AC-27 is verified through `quire validate --module`. That command is quire-cli's (EXT-3), so this repository's tests cannot run it, and other FRs mark such ACs as awaiting quire-cli. "No Quoin involvement" is an unfalsifiable negative. Restate it against the library entry point (`validate_document_in_registry` / `validate_bundle`) with the same fixture | spec/functional/FR-069-semantic-module-contract-at-load.md:366 |
| FND-008 | low | "In schema order" (AC-23) is undefined. The order is the `jsonschema` validator's `iter_errors` evaluation order. Say "the validator's error order, identical across runs", which AC-26 already tests | spec/functional/FR-069-semantic-module-contract-at-load.md:362 |
| FND-009 | low | Two wording gaps. "An archetype with no `semantic` block" misdescribes the manifest, because `semantic` is a module-level block and no archetype carries one. Say "in a module without a `semantic` block". An archetype with `iterate_over` and no `match` is refused only implicitly, through "not a key of `yield_pattern.match`" | spec/functional/FR-069-semantic-module-contract-at-load.md:300-306 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-033-AC-18 | examined | FND-005, FND-004 (frontmatter list) |
| FR-049 Located findings prose + FR-049-AC-19 | examined | Sound; lines exist on ScannedRow; render clause in integrity SR |
| FR-049-AC-20 | examined | Clean |
| FR-049-AC-21 | examined | Clean: line is the last key, so the AC-8/AC-17 order is preserved |
| FR-069 Located record validation prose | examined | FND-001, FND-002, FND-003, FND-004, FND-006, FND-009 |
| FR-069-AC-21 | examined | Clean |
| FR-069-AC-22 | examined | FND-009 |
| FR-069-AC-23 | examined | FND-001, FND-006, FND-008 |
| FR-069-AC-24 | examined | Clean, given FND-001 fixed |
| FR-069-AC-25 | examined | FND-004 |
| FR-069-AC-26 | examined | Clean |
| FR-069-AC-27 | examined | FND-007 |
| spec/tests.md TC-2300..TC-2310 + audit rows | examined | Every AC has one TC row and one audit row |
| FR-081 (line 111) | context_only | Confirms the no-frontmatter rule |
| FR-011-AC-21 | context_only | See integrity SR |

## Coverage

All 11 new ACs have a TC row (TC-2300..TC-2310) and an audit-table row, and they are marked 🚧 Open in the summary rows. `quire matrix` reports all 11 as `untagged`, which is expected for a spec-only PR.
