---
id: US-011
title: "Python Library Parses a Repo of Markdown via quire-rs Bindings"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-005"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-023"
    type: "exercises"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-024"
    type: "exercises"
---

## Story

As a **Python orchestration layer** (`filament_parser`), I want to call `quire.load_repo(path)` and get back a list of parsed documents — each with frontmatter, section tree, and stable block ids — in a single FFI hop, so that I no longer maintain a Python directory walk + per-file frontmatter parse, and the per-document hot path runs at native Rust speed instead of in the interpreter.

## Context

`filament_parser/loader.py` today does, per file: `Path.glob("**/*.md")` → `read_text` → frontmatter split → `quire` (Python) parse → SHA-256 content hash → UUID5 id. Every step is interpreter-bound and sequential. The tier-2 extraction engine then walks each parsed tree with regex locators.

Under this story the Python layer keeps only orchestration: it calls into the Rust binding for the walk + parse (FR-024) and the binding returns owned, typed objects (FR-023). Tier-3 plugin discovery and dispatch stay in Python and operate on the already-parsed documents handed up from Rust (see StR-005).

The binding surface is feature-gated; the pure-Rust crate is unaffected (StR-001 boundary preserved).

## Acceptance

- **US-011-AC-1**: `quire.load_repo(path)` returns one parsed-document object per `.md` file under `path`, each exposing frontmatter, sections, and block ids — without the caller writing a Python walk loop.
- **US-011-AC-2**: A file that fails to parse does not abort the call; it is surfaced as a per-file diagnostic on the result, and the remaining documents load (mirrors FR-024).
- **US-011-AC-3**: The returned objects carry data constructed in Rust; no Python-side re-parse of the markdown body occurs (verified by the absence of a Python markdown/frontmatter parse on the binding path).
- **US-011-AC-4**: The binding exposes each document's durable frontmatter `uuid` (UUID7) and human `id` unchanged from the source — identity is *read*, not derived, so repeated calls are trivially stable and downstream consumers get the same durable catalog id (CR-002; see FR-024 Identity).
- **US-011-AC-5**: The binding call releases the GIL during the Rust walk/parse so a multi-threaded Python caller is not serialized on it (see NFR-016).

## Efficiency Analysis

**Round trips per repo load:** 1 FFI crossing for the whole tree, vs. today's N Python-level file operations plus N Python parse calls.

**Where the time goes today (Python):** per-file `read_text` + frontmatter regex + Python-object construction, all on one thread, plus the GIL preventing the obvious `ThreadPoolExecutor` win for the CPU-bound parse.

**Under the binding:** one crossing hands the directory to Rust; the walk + parse runs on a rayon pool with the GIL released (FR-024, NFR-016). The crossing cost is paid once, not per file. For a 500–1000 document spec repo this is the difference between seconds and tens of milliseconds.

**When NOT to use:** a single-document parse from already-in-memory bytes — there `quire.parse_document(text)` is the right call; `load_repo` is for trees on disk.

## Performance Criteria

- **US-011-PC-1**: `quire.load_repo` over a 1,000-document corpus completes in p50 < 200 ms on 8 threads on the canonical runner (the Rust leg is bounded by NFR-015; the binding adds one crossing, not per-file overhead). Bench: **TC-455**.
- **US-011-PC-2**: Per-FFI-crossing overhead for a single `parse_document` call is < 50 µs over the equivalent in-crate Rust call (NFR-016).
- **US-011-PC-3**: Wall-clock for the full corpus through the binding is ≥ 5× faster than the pure-Python `filament_parser` path on the same corpus (StR-005-AC-3). Bench: **TC-456**.
