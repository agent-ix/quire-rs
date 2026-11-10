---
id: SR-20263
title: integrity review of FR-069 import_types and import diagnostics
type: SpecReview
analysis: integrity
scope: agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md; spec/tests.md
review_set: subset
date: '2026-11-10'
---

# integrity review of FR-069 import_types and import diagnostics

## Summary

Ticket: PLAT-212 (agent-ix/quire-rs PR #637, head e2b6bc42). Checked the structure of the edit: where each new rule sits in FR-069's evaluation phases, AC numbering, TC id uniqueness, the coverage summary row, the AC-to-TC table, and status values. AC-16..20 follow AC-15 with no gap or reuse; TC-2214..TC-2218 are new and unique (highest existing id was TC-2213); each AC maps to exactly one TC and the TC rows name the right AC; the 🚧 statuses follow the file's existing usage. Three defects.

## Scope

- `FR-069#declared-import-ref` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069#import-types` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-14` (context_only) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-16` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-17` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-19` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `tests.md#FR-069-coverage-row` (examined) — spec/tests.md
- `TC-2214` (examined) — spec/tests.md
- `TC-2215` (examined) — spec/tests.md
- `TC-2216` (examined) — spec/tests.md
- `TC-2217` (examined) — spec/tests.md
- `TC-2218` (examined) — spec/tests.md

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The declared-import `$ref` refusal (`semantic.unknown-import-type` when the provider is loaded) sits in the per-module "Refusals, in evaluation order" list, but whether a provider is loaded is only known in the "Cross-module checks, after every module of a load has been read" phase; the rule cannot be evaluated where it is placed. | spec/functional/FR-069-semantic-module-contract-at-load.md:127 |
| FND-002 | low | The edited FR-069 coverage row lists `AC-10..14, AC-16..20` and omits AC-15 and its TC-2190, so the summary row skips a criterion that the AC-to-TC table covers. | spec/tests.md:109 |
| FND-003 | low | FR-069-AC-14 fixes the `<code>: <path>: <message>` form, but the new refusals (`semantic.unknown-import-type` from an `import_types` entry or from a `$ref`, and the malformed-list refusal) do not name their manifest locus. | spec/functional/FR-069-semantic-module-contract-at-load.md:184 |

## Details

- **FND-001** (FR-069#declared-import-ref): Fix: keep the admission (no follow, no fetch) in the per-module resolver bullet, and move the "provider loaded and type not exported" check to the cross-module list next to the `import_types` check, so both `unknown-import-type` sources run in one phase.
- **FND-002** (tests.md#FR-069-coverage-row): Fix: `AC-1..8, AC-10..20` and add TC-2190 to the TC column.
- **FND-003** (FR-069#import-types): Fix: name the loci, for example `semantic.import_types.<pkg>` for the list case and `<section>[<type>].data_schema.schema` for the `$ref` case.

## Verdict

Mergeable after FND-001 is fixed. Clean: AC numbering, TC uniqueness and mapping, status values.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | The new `semantic.invalid-value` shape rule for `import_types` sits under "Cross-module checks", but every case it lists is a per-module block check, step 2 of the refusal order. Also, the loader's existing schema-error mapping derives the locus from the JSON pointer, which gives `semantic.import_types.<org>~1<repo>.<index>` for an entry-level error, not the `semantic.import_types.<package>` that AC-16 requires. | spec/functional/FR-069-semantic-module-contract-at-load.md:192 |

- **FND-004** (FR-069#import-types-shape): `src/semantic/contract.rs:163-198` builds `at` from `error.instance_path()` with `/` replaced by `.`, so a package key `agent-ix/x` comes out as `agent-ix~1x`. Fix: move the bullet into the refusal list beside the block-schema step, and state that the locus is the unescaped package key, with no index, for every list-level and entry-level case.

## Dispositions

Round 1, against head 35ef824e (fix commit 35ef824).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
