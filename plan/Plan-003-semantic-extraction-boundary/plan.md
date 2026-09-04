---
id: Plan-003
title: "quire-rs — semantic extraction boundary"
type: Plan
status: active
relationships:
  - target: ix://agent-ix/quire-rs/US-019
    type: references
  - target: ix://agent-ix/quire-rs/FR-069
    type: references
  - target: ix://agent-ix/quire-rs/FR-070
    type: references
  - target: ix://agent-ix/quire-rs/FR-071
    type: references
  - target: ix://agent-ix/quire-rs/FR-072
    type: references
  - target: ix://agent-ix/quire-rs/NFR-021
    type: references
---
# Implementation Plan: Semantic extraction boundary (#388)

TDD plan for `agent-ix/quire-rs#388`, Track A step 4 after `agent-ix/quoin#293`. The plan owns the semantic module contract at load, the typed Properties / Invariants / Operations extraction into semantic-core declarations, the additive `semantic` record on every surface, and the boundary gates. IR lifting, clause typechecking, rendering, and the WASM binding crate stay downstream (`filament-core-data#36`, `quire-contract-ir#55`, `quire-wasm#3`).

## Requirements Summary

### User Stories

- [ ] **US-019**: A compiler frontend extracts an object artifact's declarations from one validated Quire record.

### Functional Requirements

- [ ] **FR-069**: `semantic` block and reference-form `data_schema` at load; offline `$ref` resolution; explicit refusal codes; cross-module checks; Filament snapshot context.
- [ ] **FR-070**: Typed table or `sysml` fence to normalized `FieldDecl[]`; cell grammars; `BundleIndex` resolution with precedence; legacy forms; no partial arrays.
- [ ] **FR-071**: `## Invariants` to `ClauseRef[]` with spans and verbatim `clauseText`; `## Operations` to `OperationDecl[]`; fence scanner reuse.
- [ ] **FR-072**: One `SemanticExtraction` record with per-kind availability and `lossy`; Filament API, `validate_document`, Python binding; hand-authored `semantic-v1` schema; fixture suite.

### Cross-cutting Requirements

- [ ] **NFR-021**: No clause parsing, no network/git/persistence, wasm-safe, byte-identical existing contracts, Rust/Python parity.

## Dependency Graph

- `FR-013 + FR-045 + FR-067 -> FR-069`
  Reason: the loader, the Filament snapshot, and the assurance digest tuple are the seams the semantic block attaches to.
- `FR-069 + FR-011 + FR-025 -> FR-070`
  Reason: field extraction needs the loaded `SemanticModule`, the section locators, and the corpus for the `BundleIndex`.
- `FR-070 + FR-005 -> FR-071`
  Reason: operations reuse the cell grammars; spans reuse the parser's fence recognition through the `code_block` locator.
- `FR-069 + FR-070 + FR-071 + FR-046 + FR-032 + FR-055 -> FR-072`
  Reason: the surface wraps the three extractions into one published, versioned record on the existing binding and validation paths.
- `NFR-021 -> every task`
  Reason: byte identity and the non-parsing boundary are part of the contract, so the baselines are minted before the first code change and the audits run from the first commit.

Seams: `loader::{manifest, compile, mod}` for the block and schema; `extract::locator` for section and fence recognition; `filament` for the snapshot path; `validate_document` for findings; `python` for the binding. New code lives behind one `semantic` module (`src/semantic/{contract,resolver,properties,clauses,surface}.rs`) plus `schemas/vendored/` and `schemas/output/semantic-v1.schema.json`.

## Design decisions fixed by the reviews (SR-068..SR-075)

- `$ref` resolution is an in-memory `$id → document` map (module siblings + embedded semantic-core bundle via `include_str!`), never the schema library's file/http resolvers; identical under `--features wasm`.
- `BundleIndex` and `sourceIdentity` are explicit `SemanticContext` inputs on every surface; an empty index is a state (`no-bundle-index`), not a default.
- A row or entry error makes the whole kind `unavailable`; no partial declaration arrays.
- Existing severity sets and keys are untouched; `advisory` lives inside the semantic record and maps to `warning` outside it.
- Baselines (`tests/fixtures/semantic/baseline/*.json`) are minted from `main` before any code change.

## Test Plan

### Contract and Unit Tests

- [ ] **TC-1599..TC-1609, TC-1645, TC-1646**: semantic block, reference `data_schema`, `$ref` rules, provenance, cross-module checks, inline parts.
- [ ] **TC-1610..TC-1618, TC-1647**: golden table/fence, both forms, cell grammars, legacy forms, no-block baseline, row-error state.
- [ ] **TC-1622..TC-1626, TC-1648**: golden clauses/operations, language cases, structural cases, dangling refs, `sourceIdentity` default.
- [ ] **TC-1630..TC-1635, TC-1637, TC-1638, TC-1650**: fixture suite, availability states, Filament integration, refusal, `validate_document`, Python parity, determinism, schema + compatibility fixture, generator audit.

### Property Tests

- [ ] **TC-1621**: generated cells always yield a valid `FieldDecl` or a diagnosed row; never a partial array.
- [ ] **TC-1629**: generated fence bodies round-trip byte-for-byte with correct spans.

### Static, Compile, and Snapshot Gates

- [ ] **TC-1606, TC-1607, TC-1639, TC-1643**: provenance and byte-identity baselines.
- [ ] **TC-1619, TC-1620, TC-1627, TC-1628, TC-1640, TC-1641, TC-1642**: boundary audits.
- [ ] **TC-1649**: wasm32 check in `make ci`.
- [ ] **TC-1636, TC-1644 (WASM leg)**: external, `agent-ix/quire-wasm#3`.

## Remaining Work

### Track A: Critical Path (serial)

- **A1 = Task-015** baselines, vendored schemas, audit scaffold — Medium; exit: baselines checked in from `main`, `schemas/vendored/` hashes to provenance, audits run in `make ci`.
- **A2 = Task-016** FR-069 loader contract and resolver — Hard; exit: every refusal code fires at its named locus, the resolver passes under the `wasm` feature, no-block registries equal the baseline.
- **A3 = Task-018** FR-070 typed Properties extraction — Hard; exit: golden table and fence produce one normalized array; every `cell-cases.json` case passes; row errors yield no partial array.
- **A4 = Task-019** FR-071 clause and operation extraction — Medium; exit: `operations.expected.json` reproduced; spans agree with the `code_block` scanner; `parse_document` golden unchanged.
- **A5 = Task-020** FR-072 surface, schema, bindings — Hard; exit: `cases.json` passes through library, Filament API, `validate_document`, and Python; `semantic-v1` schema and compatibility fixture checked in.

### Track B: Parallel

- **B1 = Task-017** vendored golden fixtures and case suite skeleton — Small; runs beside Task-016; exit: quoin fixtures pinned with provenance, `config-version.bundle.json` and `cases.json` skeleton with `issue_ref` on every case.
- **B2 = Task-021** boundary and compatibility gates — Medium; runs beside Task-020; exit: NFR-021 audits, wasm check, byte-identity tests green.

### Gate

- **Task-022** review gate — `/code-review` + `/gap-analysis`, matrix flipped, "mergeable" comment, squash merge verified on the tree.

## Parallel Execution Summary

```text
Track A: Task-015 -> Task-016 -> Task-018 -> Task-019 -> Task-020 --\
Track B:            Task-017 ---------------------------> Task-021 --+-> Task-022
```

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
| --- | --- | --- | --- | --- |
| Task-015 | A | FR-069, NFR-021 | TC-1606, TC-1607, TC-1639, TC-1643 | completed |
| Task-016 | A | FR-069 | TC-1599..TC-1605, TC-1608, TC-1609, TC-1633, TC-1645, TC-1646 | completed |
| Task-017 | B | FR-070, FR-071, FR-072 | — | completed |
| Task-018 | A | FR-070 | TC-1610..TC-1618, TC-1621, TC-1647 | completed |
| Task-019 | A | FR-071 | TC-1622..TC-1626, TC-1629, TC-1648 | completed |
| Task-020 | A | FR-072 | TC-1630..TC-1632, TC-1634, TC-1635, TC-1637, TC-1638, TC-1644, TC-1650 | todo |
| Task-021 | B | NFR-021, FR-070, FR-071, FR-072 | TC-1619, TC-1620, TC-1627, TC-1628, TC-1636, TC-1640..TC-1642, TC-1649 | todo |
| Task-022 | Gate | US-019 | — | todo |

## Coordination Rules

- Mint the baselines from `main` in Task-015 before any `src/` change; a later task that needs a baseline change has found a defect, not a baseline to refresh.
- Treat `schemas/vendored/` as immutable except through the re-vendor script, which rewrites the provenance record with the new revision and digests.
- Treat `semantic-v1.schema.json` and its compatibility fixture as immutable once Task-020 passes; a breaking correction mints v2.
- No rendering, code generation, clause parsing, network, git, persistence, or corpus-repository edit anywhere in the slice.
- Any change under `src/python/` or `tests/python/` runs `make ci-python` before merge; the WASM leg is reported by reference to `agent-ix/quire-wasm#3`.
