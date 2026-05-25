# ADR 0002: Three-Layer Document Pipeline (quire-rs / filament-parser-lib / service)

**Status**: draft
**Date**: 2026-05-24
**Decision authority**: TBD

## Context

The Filament/Quire document pipeline currently spans two repos with
overlapping responsibilities:

- `quire-rs` (Rust): per-document parse, extract, validate, render.
  Public API is single-document scoped per `spec/spec.md`.
- `filament-parser-lib` (Python): filesystem walk, tiered extraction
  (tier1 frontmatter, tier2 DSL, tier3 plugin), edge/link/relationship
  normalization, dispatch.

Three forces are reshaping this split:

1. **Performance.** `filament-parser-lib` currently does per-file Python
   `Path` ops, regex-based AST walks (`tier2/engine.py`,
   `tier2/locators.py`), and per-node schema validation. These are the
   measured hot paths. `quire-rs` already implements ~75% of tier2 in
   `extract/`, all of validate in `validate.rs`, and a per-doc loader in
   `loader/` — but ships no Python bindings, so the Python layer can't
   call into it.
2. **Scope creep risk.** A graph engine was previously built into
   `filament-parser-lib` and torn out. Cross-document concerns
   (corpus assembly, edge resolution, traversal, persistence) keep
   trying to migrate back into the parser layer.
3. **Plugin extensibility.** `tier3.py` discovers plugins via Python
   entry points (`importlib.metadata`) — a capability that cannot move
   to Rust without re-implementing Python's plugin model.

Without an explicit layering decision, the two repos will keep
absorbing responsibilities at the boundary that's most convenient for
each task, and the graph engine will get re-invented in whichever
layer touches a multi-doc feature first.

## Decision

Adopt a **three-layer pipeline** with strict scope boundaries:

| Layer | Language | Owns | Does NOT own |
|---|---|---|---|
| `quire-rs` | Rust | Per-doc parse, extract (tier2 DSL), schema validation, render, block-level edits. PyO3 bindings published as `quire` on PyPI. | Filesystem orchestration above a single repo root, plugin discovery, cross-doc state. |
| `filament-parser-lib` | Python | Orchestration over `quire-rs`. Plugin discovery (tier3 entry points), dispatch, edge/link/relationship stub emission. Returns `[Document { edges: [EdgeStub] }]`. | Edge resolution across documents, indexes, traversal, persistence. |
| Service layer | (TBD — Python or other) | Corpus assembly: joining edge stubs across documents, indexing by UUID/type, traversal/query, persistence, caching. | Single-doc parse semantics. |

**Concretely, `quire-rs` grows:**

- `pyo3` + `maturin` build target behind a feature flag.
- A `python/` module exposing typed wrappers around `QuireDocument`,
  `Registry`, `extract()`, `render_block()`, plus a new
  `load_repo(path, registry) -> [Document]` that does the parallel
  directory walk (currently `loader.py`) in Rust via
  `ignore::WalkBuilder` + `rayon`.

**`filament-parser-lib` collapses to:**

- A thin Python shim that calls `quire.load_repo(...)`.
- `tier3.py` plugin discovery + invocation against already-parsed
  `Document` objects handed up from Rust.
- `dispatch.py` orchestration (microsecond cost, not a hot path).

**The service layer is new** (or absorbs work that today lives in
ad-hoc consumers). Its API surface, persistence choice, and query
shape are out of scope for this ADR.

## Options considered

1. **Push everything into `quire-rs`, including the graph engine.**
   Rejected: re-creates the scope creep that justified tearing the
   previous graph engine out. Binds the parser to graph semantics
   that change on a different cadence than markdown parsing.
2. **Keep the graph engine in `filament-parser-lib`.**
   Rejected for the same reason it was torn out previously — graph
   concerns (persistence, query, caching) don't belong next to a
   per-doc parser, and mixing them re-creates the original maintenance
   pain.
3. **Keep `quire-rs` Rust-only; have `filament-parser-lib` shell out
   to a CLI with JSON I/O.**
   Rejected on performance grounds. The user explicitly chose
   "optimal speed" — per-call subprocess overhead and JSON round-trip
   serialization dominate at corpus scale.
4. **Selected: three-layer split with PyO3 bindings.**
   Each layer scales independently. `quire-rs` gets rayon parallelism
   without a Python plugin model. The service can swap persistence
   (Postgres, in-memory, sqlite) without touching the parser. The
   graph engine has one obvious home.

## Migration sequence

1. Add `pyo3`/`maturin` scaffolding to `quire-rs` (feature-flagged so
   the pure-Rust build is unaffected). Publish `quire` to PyPI,
   replacing the current Python `quire` package that
   `filament-parser-lib` imports.
2. Port the loader walk: Rust `ignore::WalkBuilder` + rayon parallel
   parse, returning `Vec<ParsedDoc>` to Python in a single FFI hop.
3. Finish wiring tier2 through `extract/` (the DSL evaluator gaps
   identified in the survey) so `tier2/engine.py` collapses to a
   single Rust call.
4. Move tier1 + `schema_validation.py` into `validate.rs` for per-doc
   validation. Pydantic stays at the registry boundary for plugin
   schema validation (load-time, not per-doc).
5. Port `edges.py` / `links.py` / `relationships.py` — small but
   per-doc; avoids an FFI round-trip per document.

What stays in Python: `tier3.py` (entry-point plugin discovery,
`importlib.metadata`), `dispatch.py` (orchestration), and the
Pydantic DSL schema validation at registry load.

## Consequences

- `quire-rs` grows a second public API surface (PyO3) and a second
  release pipeline (PyPI in addition to crates.io). ABI3 wheels keep
  this manageable but cost a few % on hot calls; accepted as the
  right default.
- `filament-parser-lib`'s public API contract shifts: callers that
  previously got Python objects from a Python parser now get objects
  whose underlying state was constructed in Rust. Equality, hashing,
  and serialization semantics must be re-verified.
- The "where does this feature go?" question for any new cross-doc
  capability now has a structural answer: **the service layer**.
  Pushback on graph-engine-shaped work landing in the lower two
  layers is now an ADR-backed default, not a per-PR judgment call.
- A future revision is needed once the service layer's language and
  shape are picked — that decision will land as ADR 0003.
- This ADR does not commit `quire-rs` to ever owning multi-doc state.
  Any future proposal to add a `Corpus` type or cross-doc index to
  `quire-rs` must amend or supersede this ADR.

## Open questions

- Service layer language: Python (continuity, same plugin model) or
  Rust/Go (consistency with `quire-rs`)? Out of scope here.
- Persistence shape for resolved edges: Postgres, in-memory, sqlite?
- Plugin invocation cost across the FFI boundary — if plugins
  themselves become a hot path, tier3 may need to move to a worker
  pool rather than per-doc invocation.
