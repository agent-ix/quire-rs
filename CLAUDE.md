# quire-rs

High-performance Rust templating + parsing engine for the Filament/Quire ecosystem.

## Commands

```bash
make fmt            # format with rustfmt
make fmt-check      # verify formatting (CI gate)
make lint           # clippy with -D warnings
make test           # cargo test
make build          # release build
make clean          # cargo clean
make deny           # cargo deny check licenses
make audit-unsafe   # check that every unsafe block has a // SAFETY: comment
make ci             # fmt-check + lint + test + deny + audit-unsafe
```

## Safety scaffolding

Backported from `agent-ix/rust-lib-cookiecutter` (originally from `agent-ix/ecaz`).
When upstream tightens MSRV, clippy lints, license allowlist, or audit scripts,
backport changes here via the `backport-code` skill (StR-004-AC-3). Drift is
detected on each `make ci` run via `scripts/audits/verify_cookiecutter_inheritance.sh`.

- `clippy.toml` pins MSRV to `1.75` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt`. Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses 100-char width and `StdExternalCrate` import grouping. CI fails on drift.
- `rust-toolchain.toml` pins to stable + rustfmt + clippy.

## Design taste

Write idiomatic Rust. Lean on the type system to encode invariants rather
than re-checking them at runtime: prefer enums + exhaustive `match` over
booleans + flags, `Option`/`Result` over sentinel values, newtypes over
raw strings/ints when the meaning matters, and `&str`/borrowed slices over
allocating clones on hot paths. Where the TS/Py references rely on dynamic
shapes (e.g. JSON `Value` blobs), the Rust port should look for a stronger
typed representation when it doesn't break parity. "Stringly-typed" code
and `unwrap()`s outside of tests are smells — surface errors via
`thiserror`-derived enums per NFR rather than panicking.

## Code quality & references

Write code that is **safe, idiomatic, performant, and well-spec'd**, and reach for the
authoritative references rather than guessing.

**Performant.** Latency/throughput are normative (NFR-001/002/007/015/016), CI-gated by
`scripts/check_perf_regression.sh` (10% band over a stored criterion baseline). Determinism
(NFR-006) is enforced by `scripts/audits/check_hashmap_audit.sh`, which **bans `HashMap`** in
`src/parser`, `src/merge.rs`, `src/extract`, `src/validate_document.rs` — use `BTreeMap`/`IndexMap`
anywhere iteration order is observable. Prefer borrowed slices over clones on hot paths; do work
once at load time (compile-once, parse/validate-fast). The render/templating feature is removed.

**Well-spec'd.** FRs precede code. `spec/tests.md` must stay at 100% AC→TC coverage (a `grep`
integrity check is wired into that file). Honor the gates in `plan/tasks/README.md` — don't start
gated work early. **Never silently edit `spec/` from an implementation branch**: a spec error or
change requires a CR note (see CR-002 in FR-024 for the pattern).

**v0.3 invariants (corpus + bindings).**
- `load_repo`'s parallel parse is **data-parallel with no shared mutable state** (no
  `Mutex`/`RwLock`/`Atomic` in first-party code) — `par_iter().map().collect()` of owned results,
  diagnostics gathered after the parallel region (FR-024-AC-9; this is what the NFR-017 loom check
  proves).
- The corpus (`Spec`) is immutable, `Arc<Inner>`-wrapped, `Send + Sync` — **mirror `Registry`**
  (`src/registry.rs`), don't invent a new lifecycle.
- Document identity is **read, never derived**: `id` = human artifact id (resolution key), `uuid` =
  durable UUID7 from frontmatter (catalog id). No path/content derivation, no file mutation at load.
- First-party `src/python/` stays **`unsafe`-free**; PyO3 macro-generated unsafe is upstream and
  doesn't count (NFR-003-AC-4). The `python` feature is gated so the default build stays
  interpreter-free.

**References.**
- `spec/spec.md` §19 — hardening posture (which ECAZ tools are adopted/skipped and why).
- `spec/assets/adr/` — 0001 (validator crate: `jsonschema ~0.18`), 0002 (three-layer pipeline:
  quire-rs per-doc/per-spec ← filament-parser-lib orchestration ← service layer graph).
- Upstream parity: `agent-ix/quire` (TS parser), `agent-ix/quire-py`; the Python reference renderer
  in `spec-artifacts-*`. Crates: [MiniJinja](https://docs.rs/minijinja),
  [`jsonschema`](https://docs.rs/jsonschema), [`rayon`](https://docs.rs/rayon),
  [`ignore`](https://docs.rs/ignore), [`uuid`](https://docs.rs/uuid) (v7),
  [`pyo3`](https://pyo3.rs).

## Layout

```
src/lib.rs             # crate root
src/parser/            # parse_document → QuireDocument (TS/Py parity)
src/loader/, registry.rs  # archetype Registry (compiled schema only)
src/validate.rs, validate_document.rs, writeback.rs  # schema/markdown validate + byte-splice edit
src/extract/           # body-extraction DSL evaluator
src/corpus/            # v0.3: load_repo + Spec corpus + resolution + query
src/python/            # v0.3: PyO3 bindings (feature = "python")
tests/                 # integration + parity + determinism + dogfood
benches/               # criterion benchmarks (harness = false)
fuzz/                  # cargo-fuzz targets
spec/, plan/           # requirements artifacts + execution plan
scripts/               # local tooling + CI audits
```
