---
type: index
title: "Plan-001 — AC grammar + declarative traceability coverage"
description: "Contents of the Plan-001 bundle."
okf_version: "0.1"
---
# Plan-001 — AC grammar + declarative traceability coverage

## Contents

* [Plan-001: AC grammar + declarative traceability coverage](./plan.md) - Plan overview, dependency graph, tracks A/B/C, gates G1/G2, test plan.
* [Task-001: FR-048 — per-check grammar severity framework](./tasks/Task-001-fr048-severity-framework.md) - Severity registry, merge, routing, `off`.
* [Task-002: FR-047 — acceptance-criteria grammar (`ac`)](./tasks/Task-002-fr047-ac-grammar.md) - Shape classifier, five checks, bindings, PyO3.
* [Task-003: Grammar CLI-surface support](./tasks/Task-003-cli-surface-support.md) - Generic summary prefix + `--severity` helper (quire-cli wiring external).
* [Task-004: FR-050 — `traceability:` model loading](./tasks/Task-004-fr050-traceability-model.md) - Shared-dependency model loader + fixtures.
* [Task-005: FR-051 — language adapters + symbol identities](./tasks/Task-005-fr051-symbol-adapters.md) - Rust/Python/TS adapters, stable ids, degradation.
* [Task-006: FR-051 — trace binding and FR-045 records](./tasks/Task-006-fr051-trace-binding-records.md) - Canonical markers, legacy class, relations, records.
* [Task-007: FR-050 — coverage reconciliation + report](./tasks/Task-007-fr050-coverage-rollup.md) - Rollup, deterministic JSON, TC-756 boundary audit.
* [Task-008: FR-049 — verification-reference integrity](./tasks/Task-008-fr049-reference-integrity.md) - `dangling-trace-reference` in `validate_bundle`.
* [Task-009: AC-grammar baseline sweep](./tasks/Task-009-ac-grammar-baseline-sweep.md) - Track C cleanup; user-gated promotion (FR-047-CON-1).
* [Task-010: Legacy trace-tag migration](./tasks/Task-010-legacy-trace-tag-migration.md) - Track C cleanup; user-gated removal (FR-051-CON-3).
