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
    type: "depends_on"
---

## Statement

`Cargo.toml` SHALL pin dependencies to versions whose behavior the
specification relies on:

1. Caret versions are acceptable for non-load-bearing crates.
2. Tilde versions are used where minor releases may alter relied-on behavior.
3. Exact versions are used where a particular release is required by a known
   defect, performance result, or accepted compatibility decision.

### Load-bearing dependencies

| Crate | Role | Pinning | Why |
|---|---|---|---|
| `minijinja` | template rendering ([FR-004](../functional/FR-004-minijinja-strict-environment.md)) | `~2.x` (or current major) | byte parity depends on whitespace and filter behavior |
| `jsonschema` | JSON Schema validation ([FR-002](../functional/FR-002-schema-validation-pipeline.md)) | `~0.18` | selected by ADR-0001 as a behavior-bearing validation boundary |
| `serde_yaml` import backed by `yaml_serde` | frontmatter, manifests, clause sets, extraction DSL, traceability, and lint YAML | `=0.10.7` | selected by [ADR-0012](../assets/adr/0012-yaml-engine-maintenance-and-parity.md) after a zero-difference compatibility run |
| `serde_json` | core data type | `^1` | stable public data model |
| `indexmap` | observable deterministic iteration | `^2` | stable public ordering behavior |

### Version-update policy

A load-bearing dependency change outside its accepted range requires a reviewed
decision and reruns of the behavior, compatibility, supported-Rust, license,
and advisory gates affected by that dependency. Any YAML package or version
change reopens ADR-0012 and requires a fresh differential run.

## Rationale

Dependency releases can change behavior that Quire treats as part of its
contract. Governed pins make those choices visible and prevent unattended
updates from silently changing parsing, validation, or rendering semantics.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-009-AC-1 | `Cargo.toml` conforms to every load-bearing pin in the policy table, including the exact `serde_yaml` alias to `yaml_serde 0.10.7`. | static-quality |
| NFR-009-AC-2 | ADR-0001 records the validator choice and ADR-0012 records the YAML engine choice and compatibility boundary. | inspection |
| NFR-009-AC-3 | The dependency audit rejects wildcards, a wrong YAML package, a non-exact YAML version, and reintroduction of the deprecated YAML package or backend. | static-quality |
| NFR-009-AC-4 | A load-bearing dependency change outside its pin records passing results for every affected qualification gate before merge. | integration-testing |
| NFR-009-AC-5 | The locked graph contains `yaml_serde 0.10.7` and `libyaml-rs 0.3.0` and contains neither `serde_yaml 0.9.34+deprecated` nor `unsafe-libyaml`. | static-quality |
| NFR-009-AC-6 | Before the YAML engine change is accepted, the old and selected engines produce identical outcomes and JSON-compatible values over the governed frontmatter population and focused semantic-risk cases; an injected difference proves the comparison fails closed. | integration-testing |
| NFR-009-AC-7 | Existing frontmatter parity and typed YAML consumer suites pass unchanged with the selected engine. | integration-testing |
| NFR-009-AC-8 | The selected locked graph passes on the repository's supported Rust version and passes license and advisory gates without a new waiver. | compile-time-check |
| NFR-009-AC-9 | Existing NFR-002 parse and validation performance gates remain within their unchanged thresholds. | performance-benchmarking |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Load-bearing dependencies conforming to policy | all | all | `scripts/audits/check_dep_pins.sh` |
| Deprecated YAML packages in the root lock | 0 | 0 | dependency audit |
| YAML differential outcome or value differences | 0 | 0 | one-time differential recorded in ADR-0012/SR-108 |
| Existing parity and typed-consumer suites | pass | unchanged expectations | integration tests |
| Parse and validation regression | none | existing NFR-002 limits | performance benchmarks |

## Verification

- `scripts/audits/check_dep_pins.sh` enforces the manifest pins and selected or
  forbidden locked package identities.
- ADR-0012 and SR-108 retain the aggregate, revision-bound one-time YAML
  differential result. The comparator itself is not permanent product tooling.
- Existing parser-parity and typed-consumer suites exercise the behavior-bearing
  YAML paths without maintaining a brittle call-site registry.
- NFR-022 independently governs the supported Rust version; standard local
  build, test, license, advisory, and performance gates qualify the dependency
  change.

## Dependencies

- **Upstream**: ADR-0001 selects the validator; ADR-0012 selects the YAML engine;
  NFR-022 selects the supported Rust version.
- **Downstream**: dependency updates and release qualification consume this
  policy.
