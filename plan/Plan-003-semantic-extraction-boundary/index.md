---
type: index
title: "Plan-003 — Semantic extraction boundary"
description: "Contents of the Plan-003 semantic extraction boundary bundle (#388)."
okf_version: "0.1"
---
# Plan-003 — Semantic extraction boundary

## Contents

* [Plan-003: Semantic extraction boundary](./plan.md) - Plan overview, dependency graph, test plan, tracks, and gates.
* [Task-015: baselines, vendored schemas, audit scaffold](./tasks/Task-015-baselines-vendoring-audits.md) - Freeze the pre-change contracts and vendor upstream schemas with provenance.
* [Task-016: loader semantic contract and resolver](./tasks/Task-016-loader-semantic-contract.md) - FR-069 block, reference data_schema, offline `$ref` map, cross-module checks.
* [Task-017: golden fixtures and case suite](./tasks/Task-017-golden-fixtures-case-suite.md) - Pin quoin mapping fixtures and lay out `cases.json`.
* [Task-018: typed Properties extraction](./tasks/Task-018-typed-properties-extraction.md) - FR-070 forms, cell grammars, BundleIndex resolution.
* [Task-019: clause and operation extraction](./tasks/Task-019-clause-operation-extraction.md) - FR-071 spans, clauseText, operations.
* [Task-020: surface, schema, bindings](./tasks/Task-020-surface-schema-bindings.md) - FR-072 record, semantic-v1 schema, Filament/validate/Python paths.
* [Task-021: boundary and compatibility gates](./tasks/Task-021-boundary-compat-gates.md) - NFR-021 audits, wasm check, parser golden.
* [Task-022: review gate](./tasks/Task-022-review-gap-analysis-pr.md) - Code review, gap analysis, PR, merge.
