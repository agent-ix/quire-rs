---
id: SR-20261103118113
title: "Code and Rust review — explicit composite edge metadata"
type: SpecReview
analysis: code-review
scope: "quire-rs: CHANGELOG.md, Cargo.toml, Cargo.lock, src/vocab.rs, src/semantic/relations.rs, tests/inverse_edges.rs, crates/quire-fixtures/fixtures/mapping/relationships.{md,expected.json,cases.json}, spec/functional/FR-040-object-edge-vocabulary.md, spec/functional/FR-076-relationships-extraction.md; Quoin: Cargo.toml, Cargo.lock, rust/crates/quoin-quire/Cargo.toml, rust/crates/quoin-core/src/ops/quire/mod.rs, rust/crates/quoin-core/src/ops/quire/tests.rs, rust/crates/quoin-semantic/tests/tc_452_sweep_criteria.rs, spec/functional/FR-104-relationships-mapping.md, tests/fixtures/semantic-module/mapping/relationships.{md,expected.json,cases.json}, .claude-plugin/plugin.json, .codex-plugin/plugin.json; Quire CLI: Cargo.toml, Cargo.lock, CHANGELOG.md; spec-artifacts-iso/spec_artifacts_iso/manifest.yaml; spec-objects-business/spec_objects_business/manifest.yaml"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/FR-040"
    type: references
  - target: "ix://agent-ix/quire-rs/FR-076"
    type: references
---

## Summary

Reviewed the Rust engine, Quoin consumer, CLI dependency changes, and both active module manifests for the duplicate inverse warning fix. The engine now reads explicit composite metadata with a false default; both contains and aggregates retain composite semantics while using unique inverse labels. No review findings remain after fixing Quoin's coverage mapping for the two new engine status variants.

## Verdict

**PASS** — no remaining code or Rust review findings; dependent consumers handle the engine API change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Dispositions

No findings required disposition.

## Coverage

Examined the complete changed file set across quire-rs, Quoin, Quire CLI, spec-artifacts-iso, and spec-objects-business, including the public EdgeTypeDef and EdgeVerb API, serialization defaults, semantic relation extraction, consumer status mapping, release metadata, fixtures, and tests. Checked for duplicate helpers, copied specification text, CI gate weakening, and changed behavior without a requirement. No new public behavior is unowned; FR-040 and FR-076 own the engine changes, and the Quoin status projection is covered by its existing coverage contract.

Verification: QRS targeted inverse_edges and semantic_relations tests passed; QRS formatting and deny/audit checks passed. Quoin's locked default-feature workspace test suite passed; its one offline-cache-dependent test passed after fetching the missing crate. The optional all-features run enabled the explicitly gated live-api experiment and three criterion-strength no-go tests failed their pre-registered quality bars; this is outside Quoin's default rust-test gate and unrelated to these changes. The focused coverage projection test passed. Quire CLI's locked all-target/all-feature workspace test suite passed with a temporary npm cache; the isolated packaging tests also passed. Formatting checks passed in all three Rust repositories. QRS make ci had four local macOS filesystem path-canonicalization failures in obligation tests (/private/var/... versus /var/...); the other exercised CI lanes passed. Hosted Ubuntu CI remains the merge/release gate.

## Examined Units

- quire-rs: changed vocabulary, relation extraction, FR-040/FR-076, inverse edge tests, relationship mapping fixtures.
- Quoin: relationship mapping FR-104, engine status bridge, relationship fixture, coverage projection tests, dependency metadata.
- Quire CLI: engine dependency and version/changelog metadata.
- spec-artifacts-iso and spec-objects-business: edge registry inverse and composite metadata.
