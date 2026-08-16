# Changelog

All notable changes to `quire-rs` are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/) loosely. Version
numbers follow semver — pre-1.0, breaking changes may land in minor
bumps; once 1.0 ships, semver is strict.

## [Unreleased]

### Breaking

- **FR-049-AC-9 (CR-045)** — `validate_bundle` takes the two roots
  separately: `document_root` (locates the root `index.md`) and
  `reference_root` (the base for model-declared `document:`/`exclude:`
  paths, which modules author against the repository scope). A corpus
  walked from `<scope>/spec` with one conflated root silently un-minted
  every path-bound trace target — 123 spurious `dangling-trace-reference`
  findings on this repo's own spec. `validate_bundle_at(root, …)` is
  unchanged (both roots = `root`). TC-814.

## [0.23.0] — 2026-08-15

The document walk is bounded to a caller-supplied root, and the parse
splits into a cheap header tier and an expensive body tier.

### Added

- **FR-005-AC-5..6 (CR-046)** — `parse_document` splits into two tiers:
  `parse_header(md) -> Option<Header>` (one frontmatter extraction, no body
  work, no input copy; `None` = not a document per the CR-044 rule) and
  `parse_body(md, &Header) -> QuireDocument`, with `parse_document`
  composing them — signature and outputs unchanged, pinned by a composition
  proptest. `Header` carries `id`/`type_`/`uuid` **and the full frontmatter
  map**. `walk::parse_one` decides membership + identity via `parse_header`,
  retiring `read_identity` and the duplicate post-parse `extract_frontmatter`
  from CR-044 — a non-document now costs one read and one failed fence check.
  TC-812, TC-813. (agent-ix/quire-rs#92, umbrella #90)

- **FR-050-AC-17 (CR-045)** — the walk is bounded to a caller-supplied
  **document root**. `quire coverage` derives two distinct roots from its one
  `--scope`: document root `<scope>/spec` for `Spec::from_path`, code root
  `<scope>` for symbol extraction, excluding the document root — documents are
  not source. New `symbols::extract_tree_excluding(root, exclude)`;
  `extract_tree(root)` is unchanged and equal to an empty exclusion. The
  phantom `[--source <DIR>]` flag is withdrawn from FR-050 — both roots derive
  from `--scope`, no manifest key, no flag. A scope with no `spec/` is a named
  diagnostic in the CLI, never a silent fallback to the wider tree. The
  repo-root crawl this fixes is what produced the 9,172 frontmatter errors
  across 223 repos that CR-044 silenced at the membership layer; they are now
  gone because the files are never visited, not because they were classified
  away. TC-809; TC-810/TC-811 land with the `quire-cli` two-root derivation.
  (agent-ix/quire-rs#91, umbrella #90)

## [0.22.0] — 2026-08-15

Corpus membership is type-driven. The filename skip list is gone.

### Breaking

- **`WalkOptions::skip_names` is removed.** It is `pub` and re-exported from the
  crate root. Nothing in `src/` constructed a non-default `WalkOptions`; the
  only in-crate consumers were `corpus/spec.rs` and `corpus/glossary.rs`, both
  using `Default`. A downstream crate that set `skip_names` should delete the
  field — the behavior it configured no longer exists.

### Added

- **FR-024-AC-10 (CR-044)** — a markdown file is a corpus document iff it
  carries a **frontmatter block**. Filename plays no part. `DEFAULT_SKIP`,
  `WalkOptions::skip_names` and `is_skipped` are deleted; a file with no
  frontmatter is dropped **silently**, with no diagnostic, which is what
  actually retires the `README.md` entry and generalizes to every stray `.md` —
  a CHANGELOG, an AGENTS file, a design note — without the engine knowing any of
  their names. Frontmatter present but naming an unregistered type is still a
  document, triaged by validation as before (error under `Strict`, warning under
  `Okf`). TC-807.

  The constant was never a decision. It began in
  `filament-parser-lib/filament_parser/loader.py` as a **graph-ingestion**
  filter — commit `1d17b6f`: the listed files *"validate via quire as their own
  archetypes but are **not graph nodes**"* — and quire-rs `8dc32a5` copied it
  into `load_repo`, a **validation** loader, where *"not a graph node"* became
  *"not a document."* The engine could not load the canonical instance of
  `TestMatrix`, a type its own module registers.

  **[RAN]** `scripts/classify_matrices.py` over `~/dev`, worktrees and
  `-task<N>` copies deduped: of 184 matrices at a bound path, **0 carry no
  frontmatter block**; 170 are typed `TestMatrix` and are unaffected; 14 are
  mis-typed (10 declaring `type: index` — those documents saying they are not
  matrices), of which 6 mint rows today. Against that, **20 real matrices across
  9 repos become visible for the first time**, 12 of them minting, in filename
  conventions no enumeration covered — `spec/test-matrix.md`,
  `spec/test_matrix.md`, `spec/traceability_matrix.md`, `spec/*/matrix/tests.md`.

- **FR-024-AC-11 (CR-044)** — `glossary_terms_from_path` applies the same
  membership rule. It scans raw text rather than building a `Spec` and inherited
  the skip through `discover_files`, so its scope would have silently widened to
  every stray `.md`, letting a file that is not a document define a repository's
  ubiquitous language. The rule now lives once, in `walk::is_document`. TC-808.

### Changed

- **`NON_ARTIFACT_FILES` reduced to `{index.md, log.md}`.** The `README.md`
  entry is permanently dead under the frontmatter rule. The `tests.md` entry
  would have become a live suppression of a genuine index gap: a `TestMatrix` is
  an artifact and an index that omits it is incomplete. **[RAN]** 4 of 180 repos
  with a `spec/tests.md` already list it in `spec/index.md`, so **172 now report
  `index-incomplete`** — authoring debt the suppression was hiding. FR-038 and
  FR-038-AC-5 updated to match.
- `tests/spec_dogfood.rs::spec_documents` goes through `load_repo` instead of a
  hand-rolled `read_dir` recursion. It walked by hand precisely so TC-794 could
  reach `spec/tests.md` — the type-driven rule, implemented in a test, as a
  workaround for the engine not implementing it. Third independent markdown
  walker in the tree; now the second.

Closes agent-ix/quire-rs#63, #73, #76, #77. Review: `SR-005`
(`reviews/2026-08-15-cr044-type-driven-membership.md`).

## [0.21.0] — 2026-08-14

A legacy trace comment carrying a list binds every id it carries.

### Added

- **FR-051-AC-16 (CR-043)** — a legacy textual form mints one `verifies`
  relation per trace id its match carries. `marker_ids` already comma-split a
  canonical marker's argument list, so `#[trace("TC-001", "FR-007-AC-1")]` bound
  both ids; `legacy_id` returned capture group 1 whole, so
  `// Trace: FR-001-AC-1, FR-001-AC-2` bound the first and silently dropped the
  rest. **[RAN]** 98 such lines across `~/dev`, worktrees and `-task<N>` copies
  excluded: **205 ids binding to nothing across 17 repos**, spanning every
  declared legacy shape and all three languages.

  A form declaring `id_format` is unchanged — `TC-{1}` renders over a function
  name, which cannot carry a list. One match is one authored line, so a listed
  match yields one rewrite suggestion naming all its ids rather than N
  conflicting single-id rewrites.

  **The engine half alone converts nothing.** `Trace:\s*(ID)` matches once and
  stops at the comma, so capture group 1 is already a single id. The declared
  patterns must widen their id group to a list — that half lands in
  `spec-artifacts-process` and is what turns the 205 into coverage.

  Closes agent-ix/quire-rs#68.

## [0.20.0] — 2026-08-14

Phase D groundwork: the symbol adapters stop losing whole files, and the
traceability model learns two things it could not express.

### Added

- **FR-050-AC-15 (CR-038)** — declared **path scoping**. A trace target or
  document reference may carry `exclude:` globs, and may declare `archetype`
  and `document` together. Scanning `spec-artifacts-process` by archetype had
  been minting 67 test-case ids from deliberately malformed fixtures and reading
  50 of them as *backed*, because a fixture reusing `TC-017` collides with the
  real one.
- **FR-050-AC-16 (CR-041)** — module-declared `no_source_symbol` verification
  methods. A row verified by an agent-behaviour eval or by inspection cannot
  carry a trace tag, so reporting it as a status lie asserts something its own
  declared method makes impossible. The exemption changes the verdict and never
  the facts: the row stays in `unbacked_rows` and the counts are untouched.
- **FR-051-AC-14 (CR-039)** — one lexer pass per file in the TypeScript adapter,
  replacing three functions that each re-derived comment/string/template state.
- **FR-051-AC-15 (CR-040)** — the same for Rust, taught raw strings (`r#"…"#`),
  lifetimes (`&'a str`), character literals and **nesting** block comments.
- **FR-019, FR-020, FR-022 (CR-042)** — the v0.2 block model is authored as
  requirements for the first time. It shipped without documents, which is how
  10 matrix rows kept claiming `apply_block_patch`, an API the render removal
  deleted.

### Fixed

- 33 of this crate's own source files — every one holding a `r#"…"#` JSON
  fixture — were rejected as `unbalanced braces` and yielded **zero** symbols,
  so every trace tag in them bound to nothing. Skipped files: 33 → 1.
- `flatten_into_registry` merged `vocabularies` field by field, silently
  dropping any key added after `test_type`.

### Removed

- **FR-021** (block edit API) is retired. Its whole surface was render-dependent;
  US-006/US-007's acceptance criteria had already been retired for that reason.

### Measured

On this crate's own matrix, with no row edited to get there:

```
status lies   140 → 7
backed rows   144/907 → 358/926
```

The 7 remaining are untaggable rather than overclaiming — four verified in other
repos, a criterion bench, a fuzz target, and one asserting an absence.

Matrix at 488/488. TC-801..TC-805 added.

## [0.18.0] — 2026-08-08

Phase B of the acceptance-criteria property-testing program (#17, #20).

### Added

- **Acceptance-criteria property classification** (FR-052): a second,
  orthogonal shape axis over the same `ac` binding. A closed
  `PropertyShape` enum under one fixed precedence, `{domain, precondition,
  oracle}` spans that are statement-relative and carry both byte offsets
  and their own text, `row_id` and a `signals` audit trail on each record,
  and a module `property_idioms` registry demoted to a **booster** so
  CON-4 keeps extraction coverage independent of it. Never a
  `GrammarFinding`, never addressable by a `grammar_severity` key, so
  `--strict` immunity holds by construction (CON-1). New exports:
  `classify_document_properties`, `classify_document_criteria`,
  `AcClassification`, `PropertyShape`, `PropertyIdioms`, `AcPropertyCounts`.
- **`Extraction`, the three-valued outcome** (FR-052-AC-16/17, CR-033):
  `extractable | candidate | not-extractable`, derived from `(property,
  extractable)` and feeding back into neither. `candidate` names a
  metamorphic label the structural pass did not corroborate — a generator
  MAY emit and MUST mark the test as requiring review. Closes #46.
- **Coverage criteria counts** (FR-050-AC-13): `CoverageReport.criteria`
  plus two `CoverageTotals` counts, emitted as an all-or-nothing pair so a
  JSON consumer never sees one without the other.
- **Recall widening** (CR-030): the universal determiner is read at two
  further bounded subject positions. One of three candidate widenings
  cleared the ≥85% precision gate fixed in advance; the other two were
  deleted rather than narrowed.
- **PyO3 `classify_properties`** with full field parity.
- **Dogfood gate** (FR-048-AC-11): this repo's own `spec/` is judged under
  the severity promotion its published module ships.

### Fixed

- Four `ac` checker defects the CON-1 promotion sweep exposed
  (CR-024/025/026): the pair idiom tied to its separator, `Then` counted
  outside a Given/When/Then criterion, a vacuous predicate firing on a
  common noun, and a backtick run masking only to its first tick.

### Changed

- `ac:vacuous-outcome` and `ac:non-singular` are promoted to `error` in
  the `spec-artifacts-iso` manifest (CR-027). **`DEFAULT_SEVERITY` is
  unchanged** — the engine still ships every `ac` check advisory.

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
