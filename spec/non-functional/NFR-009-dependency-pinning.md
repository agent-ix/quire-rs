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
| NFR-009-AC-2 | ADR-0001 records the validator choice and ADR-0012 records the YAML engine choice and compatibility boundary. | inspection |
| NFR-009-AC-6 | Before the YAML engine change is accepted, the old and selected engines produce identical outcomes and JSON-compatible values over the governed frontmatter population and focused semantic-risk cases; an injected difference proves the comparison fails closed. | integration-testing |
| NFR-009-AC-7 | Existing frontmatter parity and typed YAML consumer suites pass unchanged with the selected engine. | integration-testing |
| NFR-009-AC-8 | The selected locked graph passes on the repository's supported Rust version and passes license and advisory gates without a new waiver. | compile-time-check |
| NFR-009-AC-9 | Existing NFR-002 parse and validation performance gates remain within their unchanged thresholds. | performance-benchmarking |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| YAML differential outcome or value differences | 0 | 0 | one-time differential recorded in ADR-0012/SR-108 |
| Existing parity and typed-consumer suites | pass | unchanged expectations | integration tests |
| Parse and validation regression | none | existing NFR-002 limits | performance benchmarks |

## Verification

- ADR-0012 and SR-108 retain the aggregate one-time YAML differential result.
  The comparator itself is not permanent product tooling.
- Existing parser-parity and typed-consumer suites exercise the behavior-bearing
  YAML paths without maintaining a brittle call-site registry.
- Standard local build, test, license, advisory, and performance gates qualify
  a dependency change.

## Dependencies

- **Upstream**: ADR-0001 selects the validator; ADR-0012 selects the YAML engine.
- **Downstream**: dependency updates and release qualification consume this
  policy.
