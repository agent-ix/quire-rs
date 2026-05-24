# Changelog

All notable changes to `quire-rs` are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/) loosely. Version
numbers follow semver — pre-1.0, breaking changes may land in minor
bumps; once 1.0 ships, semver is strict.

## [0.1.0] — unreleased

The initial implementation pass. Lays down every layer of the v1 spec
(parse → query → load → render → extract → harvest) plus the hardening
scaffolding (fuzz, miri, mutants, audit, perf bands, parity).

### Added

- **Parser** (FR-005/006/007/008/009): `parse_document`,
  `extract_frontmatter`, `QuireDocument` / `QuireSection`. BOM-strip,
  CRLF-tolerant frontmatter, fence-aware ATX heading walk, byte-exact
  section slicing (no `.strip()`), ASCII-only slug-line IDs.
- **Query** (FR-010): `section`, `sections`, `parse_table`,
  `parse_tables`, `table_from_section`, `parse_bullet_list`,
  `extract_diagrams`, `search`. Regex driver compiles once via
  `OnceLock`. TS parity for case-insensitive heading match +
  section-number prefix stripping.
- **Loader** (FR-013/014): filesystem-first archetype loader.
  `IX_SCHEMA_PATH` env-var resolution, tilde expansion, canonical-path
  dedup, symlink-loop guard, file-not-dir / permission-denied
  diagnostics. `Registry::{load_from, load_strict, from_env,
  from_default}`. First-wins archetype + module collisions with
  shadow-queryable `archetype_in_module`. Per-archetype failures
  aggregate without aborting the load.
- **Schema validation** (FR-002): `apply_patch(archetype, current,
  patch)` deep-merges then validates the merged result. JSON
  Pointer → dotted field path conversion for NFR-005 error shape.
- **Render** (FR-001/004/017): strict `minijinja::Environment`,
  `{% include %}` / `{% extends %}` rejected at load time, `render`
  returns `RenderOutput { markdown, diagnostics }`, `render_by_name`
  + `render_with_env` entry points.
- **Schema surface** (FR-003): `Registry::schema_for(name)` returns
  the loaded JSON Schema verbatim. No `schemars` dep.
- **Extract / DSL** (FR-011/016/018): six Locator primitives,
  fallback chains via `Locator::Fallback`, single-yield + multi-yield
  evaluators, `emit_edges`. `per_match` Locators evaluate against
  the iteration unit's local scope. DSL structural validation at
  load time (mutually-exclusive `match`/`iterate_over`, unknown keys,
  missing `from:`).
- **Edge harvesting** (FR-015): `harvest_edges` walks structured
  `relationships:` block + 6 sugar fields in canonical order + DSL
  `emit_edges`. Targets normalized via `RelationshipResolver`
  (Identity / Mock / Ix-Uri reference impls). Dedup by
  `(source, type, target)` first-wins.
- **Error shape** (NFR-005): `QuireError` with 13 variants. Display
  strings carry variant name + load-bearing identifier; never leak
  serde / validator internal debug forms. `format_violation` truncates
  the observed preview at 80 chars on a char boundary.
- **Diagnostics** (FR-017): non-fatal `Diagnostic` enum + collector;
  `Diagnostics::by_kind` filter. Surfaced from `Registry`,
  `ExtractionResult`, `EdgeHarvest`, `RenderOutput`.
- **Tracing** (NFR-008): feature-gated `tracing` spans at every hot
  entry (`parse`, `render`, `apply_patch`, `extract`, `harvest_edges`,
  `load`). Zero cost when the feature is off.
- **Determinism** (NFR-006): proptest no-panic harness for
  `parse_document` (10 000 cases) and `apply_patch` (10 000 cases);
  byte-exact slice round-trip proptest (10 000 cases); 64-thread
  cross-thread `render` / `parse` / `harvest_edges` determinism tests.
- **Parity** (FR-012, StR-002, StR-003):
  - Parser parity: 88 tests transliterated from the TS + Py
    reference suites.
  - Render parity: 10 cases (8 ISO archetypes + 2 demo) compared
    byte-exact against Python+Jinja2 reference via the regen pipeline
    in `scripts/regenerate_parity_fixtures.sh`. CI fails on drift
    between regen and committed expecteds.
  - Real-document parser sweep: 61 markdown files across 4 corpora,
    asserting no panic + byte-exact stitch + well-formed slug IDs.
- **Perf gates** (NFR-001/002/007): criterion benches for render,
  parse, load, validator; `scripts/check_perf_regression.sh`
  enforces a 10 % band against the stored baseline (CI caches the
  baseline across runs).
- **Hardening** (NFR-011/012/013/014): 6 cargo-fuzz targets covering
  parse, frontmatter, apply_patch, DSL, manifest, schema. Weekly
  miri, mutants, fuzz; daily cargo-audit + on every PR. 6 static
  audit scripts (`check_no_net_deps`, `check_no_schemars`,
  `check_no_shellout`, `check_dep_pins`, `check_hashmap_audit`,
  `verify_cookiecutter_inheritance`).
- **Validator-crate ADR** (NFR-009): `spec/assets/adr/0001-validator-crate.md`
  decides `jsonschema ~0.18` with rationale and bench-baseline pointer.

### Notes for downstream

- Registry-shared clones: `Registry::clone()` is `Arc<Inner>`-cheap.
- `RelationshipResolver` is a trait object friendly bound; consumers
  can ship their own (e.g. an ix-cli-aware) impl.
- The crate ships zero `unsafe` blocks (NFR-003); the `audit-unsafe`
  baseline is empty.
- No network deps in `Cargo.lock` (NFR-013 / FR-013-AC-6, enforced by
  `check_no_net_deps.sh`).
