---
id: NFR-009
title: "Dependency Version Pinning Policy"
artifact_type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-001"
    type: "implements"
    cardinality: "1:1"
---

## Statement

`Cargo.toml` SHALL pin dependencies to versions whose behavior the spec relies on. Pinning policy:

1. **Caret versions** (`^x.y.z` — Cargo default) for non-load-bearing crates (`serde`, `serde_json`, `thiserror`, etc.).
2. **Tilde versions** (`~x.y`) for crates whose minor releases historically introduce API or behavior change (`minijinja`, `jsonschema`).
3. **Equals versions** (`=x.y.z`) only when a specific patch is required for a known bug fix or perf characteristic.

### Load-bearing dependencies

| Crate | Role | Pinning | Why |
|---|---|---|---|
| `minijinja` | template rendering (FR-004) | `~2.x` (or current major) | byte-parity with Python Jinja2 depends on whitespace/filter behavior; minor releases could shift |
| `jsonschema` (or alternative) | JSON Schema validation (FR-002) | TBD at Task 005 bench | load-bearing for NFR-001; choice is an ADR (see notes) |
| `serde_yaml` (or `serde_yml`) | manifest + frontmatter parse (FR-006, FR-013) | TBD — `serde_yaml` in maintenance mode | swap to `serde_yml` if upstream goes inactive |
| `serde_json` | core data type | `^1` | stable |
| `indexmap` | iteration-order-preserved maps (NFR-006) | `^2` | stable |

### ADR for validator choice (NFR-001 load-bearing)

Task 005 (archetype loader) SHALL include a bench-driven ADR comparing candidate validator crates:

- `jsonschema` (most popular; mature)
- `boon` (newer; claimed faster)
- Custom subset (last resort if neither hits NFR-001)

The choice is recorded in `spec/assets/adr/0001-validator-crate.md` with bench numbers.

### Version-update policy

Dependency bumps within the pinned range are allowed without a CR. Bumps that cross the pinned range (e.g. `~2.x` → `~3.x` for minijinja) require a CR + render-parity-suite re-run.

## Rationale

NFR-001 byte-parity and perf targets implicitly depend on specific crate behavior. Without pins, an unattended `cargo update` could silently break the parity suite or regress perf. Pinning makes the dependency a first-class spec artifact.

## Acceptance Criteria

- **NFR-009-AC-1**: `Cargo.toml` uses tilde or equals pins for `minijinja`, the chosen JSON Schema validator, and `serde_yaml`/`serde_yml`. Other deps may use caret.
- **NFR-009-AC-2**: `spec/assets/adr/0001-validator-crate.md` exists and records the choice + bench numbers.
- **NFR-009-AC-3**: A static check confirms no load-bearing dep uses an unbounded version (e.g. `*` or `>=` without upper bound).
- **NFR-009-AC-4**: When a load-bearing dep is bumped across its pin, the render-parity suite is re-run and the result documented in the bump PR.

## Verification

- `scripts/audits/check_dep_pins.sh` (TBD by Task 022) parses `Cargo.toml` and asserts the pinning policy.
- Cargo-deny config (already in `deny.toml`) provides the orthogonal license + advisory gates.
