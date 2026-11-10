---
id: SR-20264
title: scope-boundary review of FR-069 import_types
type: SpecReview
analysis: scope-boundary
scope: agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md
review_set: subset
date: '2026-11-10'
---

# scope-boundary review of FR-069 import_types

## Summary

Ticket: PLAT-212 (agent-ix/quire-rs PR #637, head e2b6bc42). Checked who owns each input the edit relies on. FR-069 Inputs says the `semantic` block's shape, now including `import_types`, comes from `agent-ix/filament-core-data`'s published `module-semantic-block.schema.json`. That schema on filament-core-data `main` (read through `gh api`) has no `import_types` property and sets `additionalProperties: false`, and so does every local cargo checkout of it. The Filament snapshot field `importTypes?` names its producer correctly. One defect.

## Scope

- `FR-069#inputs-block-shape` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069#inputs-filament-snapshot` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069#dependencies` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-CON-1` (context_only) — spec/functional/FR-069-semantic-module-contract-at-load.md

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Inputs attribute the `import_types` shape to filament-core-data's `module-semantic-block.schema.json`, but that schema has no `import_types` and sets `additionalProperties: false`, so the loader's block validation refuses any manifest using it with `semantic.unknown-key`; the needed upstream schema change is not named in Dependencies. | spec/functional/FR-069-semantic-module-contract-at-load.md:49 |

## Details

- **FND-001** (FR-069#inputs-block-shape): Fix: add the filament-core-data schema change (and its ticket) to the Upstream dependencies, so the implementation knows it waits on, or ships with, that schema.

## Verdict

Mergeable after FND-001 is fixed. Clean: the FR-045 snapshot field and its producer, and CON-1 (admitted references are not fetched).

## Dispositions

Round 1, against head 35ef824e (fix commit 35ef824).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
