---
id: NFR-009
title: "Dependency Version Pinning Policy"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-004"
    type: "traces_to"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-001"
    type: "traces_to"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/assets/adr/0012-yaml-engine-maintenance-and-parity"
    type: "references"
---

## Statement

`Cargo.toml` SHALL pin dependencies to versions whose behavior the spec relies on. Pinning policy:

1. **Caret versions** (`^x.y.z` — Cargo default) for non-load-bearing crates (`serde`, `serde_json`, `thiserror`, etc.).
2. **Tilde versions** (`~x.y`) for crates whose minor releases historically introduce API or behavior change (`minijinja`, `jsonschema`).
3. **Equals versions** (`=x.y.z`) only when a specific patch is required for a known bug fix or perf characteristic.

### Load-bearing dependencies

| Crate | Role | Pinning | Why |
|---|---|---|---|
| `minijinja` | template rendering ([FR-004](../functional/FR-004-minijinja-strict-environment.md)) | `~2.x` (or current major) | byte-parity with Python Jinja2 depends on whitespace/filter behavior; minor releases could shift |
| `jsonschema` (or alternative) | JSON Schema validation ([FR-002](../functional/FR-002-schema-validation-pipeline.md)) | TBD at Task 005 bench | load-bearing for [NFR-001](./NFR-001-render-latency.md); choice is an ADR (see notes) |
| `serde_yaml` import backed by package `yaml_serde` | frontmatter, manifest, clause-set, extraction-DSL, traceability-model, and lint-rule YAML ([FR-006](../functional/FR-006-frontmatter-with-fallback.md), [FR-013](../functional/FR-013-archetype-loader.md), [ADR-0012](../assets/adr/0012-yaml-engine-maintenance-and-parity.md)) | `=0.10.2` | exact package/version preserves Rust 1.75; any replacement or version change reopens ADR-0012 and its differential gate |
| `serde_json` | core data type | `^1` | stable |
| `indexmap` | iteration-order-preserved maps ([NFR-006](./NFR-006-determinism.md)) | `^2` | stable |

### ADR for validator choice ([NFR-001](./NFR-001-render-latency.md) load-bearing)

Task 005 (archetype loader) SHALL include a bench-driven ADR comparing candidate validator crates:

- `jsonschema` (most popular; mature)
- `boon` (newer; claimed faster)
- Custom subset (last resort if neither hits [NFR-001](./NFR-001-render-latency.md))

The choice is recorded in `spec/assets/adr/0001-validator-crate.md` with bench numbers.

### Version-update policy

Dependency bumps within the pinned range are allowed without a CR. Bumps that
cross a load-bearing dependency's pinned range require a CR and a repeat of the
behavioral, differential, compatibility, MSRV, license, and advisory gates that
protect that dependency. For the YAML engine, any package or version change is
outside the exact pin and reopens ADR-0012.

## Rationale

[NFR-001](./NFR-001-render-latency.md) byte-parity and perf targets implicitly depend on specific crate behavior. Without pins, an unattended `cargo update` could silently break the parity suite or regress perf. Pinning makes the dependency a first-class spec artifact.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-009-AC-1 | `Cargo.toml` conforms to every load-bearing pin in the policy table, including `serde_yaml = { package = "yaml_serde", version = "=0.10.2" }`. | static-quality |
| NFR-009-AC-2 | `spec/assets/adr/0001-validator-crate.md` exists and records the validator choice and benchmark results. | inspection |
| NFR-009-AC-3 | The dependency audit rejects wildcards, unbounded ranges, a wrong YAML package, a non-exact YAML version, and every load-bearing pin that is weaker than its policy-table entry. | static-quality |
| NFR-009-AC-4 | A load-bearing dependency bump across its pin records a passing rerun of every affected behavioral, differential, compatibility, MSRV, license, and advisory gate before merge. | integration-testing |
| NFR-009-AC-5 | `Cargo.lock` and `cargo tree` contain `yaml_serde 0.10.2` and its reviewed backend, and contain no `serde_yaml 0.9.34+deprecated` package. | static-quality |
| NFR-009-AC-6 | The old and selected YAML engines produce identical success/failure outcomes and identical JSON-compatible values for every governed frontmatter input and every focused semantic-risk fixture. | integration-testing |
| NFR-009-AC-7 | The unchanged TypeScript/Python frontmatter parity suite and the Rust manifest, clause-set, extraction-DSL, traceability-model, and lint-rule suites all pass at the migration revision. | integration-testing |
| NFR-009-AC-8 | The default crate and all targets changed by the migration compile with Rust 1.75 from the locked dependency graph. | compile-time-check |
| NFR-009-AC-9 | The license and advisory gates accept the selected locked graph with zero unwaived finding. | sca-sbom |
| NFR-009-AC-10 | The unsafe-surface and static dependency gates accept the selected locked graph with zero finding. | static-quality |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Load-bearing dependencies conforming to their policy-table pins | all | all | static-quality (`check_dep_pins.sh`) |
| Load-bearing deps with unbounded version (`*` / `>=` no upper bound) | 0 | 0 | static-quality |
| `spec/assets/adr/0001-validator-crate.md` exists with bench numbers | Pass | Pass | Inspection |
| Affected behavior/compatibility gates rerun on cross-pin bump | Pass | Pass | integration-testing |
| YAML differential outcome or value differences | 0 | 0 | integration-testing |
| Production YAML consumer classes omitted from the migration suite | 0 | 0 | static-quality |
| Unwaived license, advisory, unsafe-surface, or dependency-policy findings | 0 | 0 | static-quality |

## Verification

- `scripts/audits/check_dep_pins.sh` parses `Cargo.toml` and asserts the complete
  load-bearing pinning policy, including ADR-0012's exact package and version.
- A versioned two-engine differential records exact input revisions, manifests,
  population counts, focused cases, producer versions, raw output, and every
  difference.
- `tests/parser_parity.rs` provides the TypeScript/Python frontmatter reference
  comparison. Rust suites cover module manifests, clause sets, extraction DSL,
  traceability models, and lint rules without treating test-loader adaptation as
  production evidence.
- The locked Rust 1.75 build, full CI, cargo-deny, cargo-audit, unsafe-surface,
  and static-audit gates provide the remaining compile and dependency evidence.

## Dependencies

- **Upstream**: [ADR-0012](../assets/adr/0012-yaml-engine-maintenance-and-parity.md)
  selects the exact YAML package and defines its compatibility boundary.
- **Downstream**: dependency changes and release qualification consume this
  policy; no package migration may begin from a Proposed ADR or an unvalidated
  review.
