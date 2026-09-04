---
id: NFR-021
title: "Semantic extraction stays an offline, non-parsing, additive boundary"
type: NFR
quality_attribute: maintainability
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-069"
    type: "constrains"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-070"
    type: "constrains"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-071"
    type: "constrains"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-072"
    type: "constrains"
---
# [NFR-021] Semantic extraction stays an offline, non-parsing, additive boundary

## Statement

The semantic extraction path SHALL add declarations to existing contracts
without parsing clause bodies, reaching the network, rendering, or changing
any byte of the outputs consumers already read.

## Scope

- Applies to: the semantic extraction module, its use inside the Filament
  extraction API and `validate_document`, the vendored schema bundles, and the
  Python and WASM bindings.
- Operational context: `quire validate` over a spec tree, IDE worker sync,
  and the compiler frontend consuming extraction output.

## Rationale

The compiler and the modules trust one engine only if its output is
deterministic, produced offline, and additive: a consumer pinned to the
pre-#388 contract must read identical bytes, and a clause typechecker added
later (`agent-ix/quire-contract-ir#55`) must find the clause text untouched.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Clause-language parser or evaluator dependencies in the crate graph | 0 | 0 | inspection (static) |
| Network, git, or persistence calls on the semantic extraction path | 0 | 0 | inspection (static) |
| Changed bytes versus the checked-in baselines (Filament graph cases, coverage-v1, properties-v1, assurance-v1) | 0 | 0 | unit-testing |
| Output mismatches across repeated runs and across Rust, Python, and WASM for the semantic fixture suite | 0 | 0 | integration-testing |

## Verification

A static boundary test scans the dependency graph and the semantic module
sources; contract-byte tests replay the existing fixture suites; a parity test
runs the semantic fixture cases through all three surfaces and compares JSON
values.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-021-AC-1 | A static audit of `cargo metadata` and the semantic module sources finds none of the denylisted crates (`ocl`, `sysml`, `fret`, `tera`, `handlebars`, `minijinja`, `askama`, `reqwest`, `ureq`, `hyper`, `git2`, `rusqlite`, `sled`) and no `eval`/`parse_expr`/`typecheck` symbol over clause text; clause text leaves the engine byte-identical to the fence body. | Test |
| NFR-021-AC-2 | A static audit finds no `std::net`, `std::process`, or filesystem write on the semantic path, and `cargo check --target wasm32-unknown-unknown --no-default-features --features wasm` passes under `make ci` with the vendored bundle embedded. | Test |
| NFR-021-AC-3 | Every Filament graph case output equals `tests/fixtures/semantic/baseline/filament-graph-cases.json`, and coverage-v1, properties-v1, and assurance-v1 outputs equal their checked-in fixtures byte-for-byte; no existing contract schema gains a required key. | Test |
| NFR-021-AC-4 | The semantic fixture suite yields identical JSON values across repeated runs and between the Rust and Python surfaces under `make ci-python`, including diagnostic order and loci; the WASM leg is verified by `agent-ix/quire-wasm#3`. | Test |

## Dependencies

- **Upstream**: [FR-069](../functional/FR-069-semantic-module-contract-at-load.md),
  [FR-070](../functional/FR-070-typed-properties-extraction.md),
  [FR-071](../functional/FR-071-clause-and-operation-extraction.md),
  [FR-072](../functional/FR-072-semantic-extraction-surface.md),
  [NFR-020](./NFR-020-filament-extraction-boundary.md)
- **Downstream**: `agent-ix/filament-core-data#36`, `agent-ix/quire-contract-ir#55`
