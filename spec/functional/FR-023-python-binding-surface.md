---
id: FR-023
title: "Python Binding Surface (Feature-Gated PyO3)"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-005"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-024"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-005"
    type: "requires"
    cardinality: "1:1"
---

> **CR note (render removal — 2026-06-04):** The render/templating feature is
> **removed** from `quire-rs` (no backward-compatibility layer). The binding surface
> drops `render`/`render_block` (and `QuireRenderError`). It exposes
> parse / extract / validate / `validate_document` / `load_repo` / corpus only. The
> Behavior below is updated accordingly. See `spec.md` §2bis.

## Description

`quire-rs` SHALL expose its parse, extract, validate, and repository-load surfaces to Python through a **feature-gated** PyO3 binding. The binding is built behind the `python` Cargo feature and packaged as a maturin wheel (`quire` on PyPI). When the `python` feature is **off**, the crate SHALL build and behave exactly as it does today: no PyO3 dependency is compiled, no CPython symbols are linked, and the default-feature dependency graph is unchanged (StR-001 boundary, StR-005-AC-2).

The bindings invert the call direction relative to StR-001: Python calls *into* Rust. `quire-rs` still never shells *out* to an interpreter.

### Exposed surface (minimum)

The `python` feature SHALL expose, as a Python module `quire`:

- `parse_document(text: str) -> Document` — wraps `parse_document` (FR-005). `Document` exposes frontmatter (as a Python `dict`), sections (heading, level, content, block id, line bounds), and preamble.
- `extract(doc, dsl) -> list[dict]` — wraps the body-extraction evaluator (FR-011/016).
- `validate(data, archetype_or_block_type) -> list[Violation]` — wraps schema validation (FR-002); violations carry the dotted field path and message (NFR-005).
- `validate_document(archetype, text) -> ValidationResult` — wraps markdown document validation (FR-032).
- `Registry` — wraps `Registry::load_from` / `from_env` (FR-013), exposing `archetype_names()` and lookup.
- `load_repo(path) -> RepoLoad` — wraps the parallel repository walk (FR-024).
- `Spec` / corpus constructor — wraps the corpus (FR-025) and its queries (FR-027), where the corpus feature is built.

### Object exchange

- The binding SHALL return **structured Python objects** (or `dict`/`list` of primitives), not serialized JSON strings the caller must re-parse. The data path SHALL NOT cross a subprocess or socket boundary (StR-005-AC-4).
- Rust `QuireError` variants SHALL map to a `quire` Python exception hierarchy (a base `QuireError` with subclasses for schema-violation, parse, and load failures) carrying the same context fields (field path, file path, archetype name) the Rust error carries.

### GIL and threading

- Binding entry points that perform non-trivial Rust work (`load_repo`, `parse_document` on large input, corpus construction) SHALL release the GIL for the duration of the Rust computation, so a multi-threaded Python caller is not serialized (NFR-016, US-011-AC-5).
- Returned objects SHALL be safe to hold and read from any Python thread after the call returns.

### ABI and packaging

- Wheels SHALL be built against the **abi3** (stable ABI) target so one wheel imports across multiple CPython 3.x minor versions without rebuild (NFR-016, StR-005-AC-5).
- The maturin build configuration lives alongside `Cargo.toml`; the `python` feature gates the `pyo3`/`pyo3-build-config` dependencies so they never enter the default crates.io build.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-023-AC-1 | `cargo build` (no features) and `cargo build --features python` both succeed; the no-feature build's `Cargo.lock`/feature resolution shows no `pyo3` linkage (StR-005-AC-2). | Inspection |
| FR-023-AC-2 | After `maturin build --features python` and install, `import quire; quire.parse_document(text)` returns an object whose frontmatter, section headings, and block ids match the Rust `parse_document` output for the same input. | Test |
| FR-023-AC-3 | `quire.validate(bad_data, "fr")` raises (or returns) a violation carrying the same dotted field path that `quire_rs::validate` produces for the same input (NFR-005 parity across the boundary). | Test |
| FR-023-AC-4 | A test confirms `quire.load_repo(path)` returns one document object per `.md` under `path` and surfaces per-file parse failures as diagnostics (delegates to FR-024). | Test |
| FR-023-AC-5 | A test runs two concurrent Python threads each calling `quire.load_repo` and confirms they execute concurrently (wall-clock < 2× single-call), demonstrating GIL release (NFR-016). | Test |
| FR-023-AC-6 | A single abi3 wheel built once imports successfully under two different CPython 3.x minor versions in CI (StR-005-AC-5). | Test |
| FR-023-AC-7 | A test asserts no `subprocess`, `Popen`, or socket usage exists on the binding's data path (StR-005-AC-4). | Inspection |

## Dependencies

- **Upstream**: StR-005; requires FR-024, FR-005
- **Downstream**: none
