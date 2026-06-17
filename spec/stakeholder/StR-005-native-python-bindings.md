---
id: StR-005
title: "Native Python Bindings Replace the filament-parser-lib Hot Paths"
type: StR
relationships:
  - target: "ix://agent-ix/filament-parser-lib"
    type: "replaces"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "requires"
    cardinality: "1:1"
---

## Stakeholder Need

`agent-ix/filament-parser-lib` (Python) currently owns the per-document hot paths of the Filament pipeline: a sequential filesystem walk, frontmatter parsing, a regex-and-tree body-extraction engine (tier-2), and per-node schema validation. These are the measured cost centers, and `quire-rs` already implements equivalents in Rust (`extract/`, `loader/`, `validate.rs`, `parser/`). What is missing is a way for the Python layer to *call* the Rust engine in-process — today it would have to subprocess out or maintain a parallel Python implementation that drifts.

`quire-rs` SHALL expose its parse, extract, validate, and render surfaces to Python through **native in-process bindings** (PyO3 + maturin, published as the `quire` wheel on PyPI), so that `filament_parser` collapses to a thin orchestration layer over the Rust engine rather than re-implementing it.

This decision direction is the "optimal speed" choice: an in-process FFI call avoids both interpreter-loop overhead on the hot path and the per-call serialization + process-spawn cost of a CLI/subprocess bridge.

## Rationale

This need exists because `filament-parser-lib` (Python) owns the measured per-document cost centers of the Filament pipeline — the sequential filesystem walk, frontmatter parsing, the regex-and-tree tier-2 body-extraction engine, and per-node schema validation (the `parse_document`, `tier2_extract`, `harvest_edges`, `load_module`, and `schema_validation` paths) — while `quire-rs` already implements Rust equivalents. What is missing is a way for the Python layer to *call* the Rust engine in-process; without it, Python must either subprocess out or maintain a parallel implementation that drifts. Native in-process bindings are the "optimal speed" choice: an in-process FFI call avoids both interpreter-loop overhead on the hot path and the per-call serialization plus process-spawn cost of a CLI/subprocess bridge, letting `filament_parser` collapse to a thin orchestration layer over the Rust engine.

## Validation Criteria

This need is considered satisfied when `quire-rs` builds a Python wheel (`pip install quire`) exposing at least `parse_document`, `extract`, `validate`, `render_block`, and a repository loader, with the underlying work executing in Rust (no Python re-implementation), and when building the crate *without* the `python` feature produces an artifact with no Python/CPython linkage. Satisfaction is further judged by a benchmark over a representative corpus (≥ 500 markdown documents) showing the binding path is at least 5× faster wall-clock than the pure-Python `filament_parser` path, by the binding returning structured objects without a JSON round-trip through a subprocess boundary (no `subprocess`/`Popen`/socket call on the data path), and by an abi3 wheel built once importing on at least two CPython 3.x minor versions without rebuild.

### Boundary with StR-001

StR-001 requires the **core crate** to be filesystem-only and interpreter-free (StR-001-AC-2, StR-001-AC-3). This requirement does not weaken that: the Python binding surface is **feature-gated** (`--features python`). With the feature off, the crate builds and behaves exactly as StR-001 specifies — no Python linkage, no interpreter dependency. The bindings invert the call direction (Python calls *into* Rust); `quire-rs` still never shells *out* to an interpreter.

### What stays in Python (out of this need)

- Plugin discovery via `importlib.metadata` entry points (tier-3).
- Tier-routing orchestration (`dispatch.py`) — microsecond cost, not a hot path.
- Pydantic DSL/schema validation at registry load time — load-once, not per-document.

These remain Python because they depend on the Python plugin model or are not on a hot path; pushing them into Rust buys nothing.

## Priority

Must-Have

## Acceptance

- **StR-005-AC-1**: `quire-rs` builds a Python wheel (`pip install quire`) exposing, at minimum, `parse_document`, `extract`, `validate`, `render_block`, and a repository loader, with the underlying work executing in Rust (no Python re-implementation of these paths).
- **StR-005-AC-2**: Building `quire-rs` **without** the `python` feature produces an artifact with no Python/CPython linkage — verified by inspecting the default-feature build's linked libraries and `Cargo.lock` feature resolution.
- **StR-005-AC-3**: A benchmark parses a representative corpus (≥ 500 markdown documents) through the Python binding and through the pre-existing pure-Python `filament_parser` path; the binding path is at least 5× faster wall-clock on the canonical runner.
- **StR-005-AC-4**: The binding returns structured objects (parsed documents, extraction records, validation errors) to Python without a JSON round-trip through a subprocess boundary — confirmed by the absence of any `subprocess`/`Popen`/socket call on the binding's data path.
- **StR-005-AC-5**: A wheel built once for the abi3 (stable ABI) target imports successfully on at least two different CPython 3.x minor versions without rebuild (see NFR-016).
