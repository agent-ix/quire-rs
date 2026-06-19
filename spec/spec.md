---
type: master-requirements
name: quire-rs
org: agent-ix
component_type: rust-lib
tags:
  - rust
  - templating
  - markdown-parser
  - minijinja
  - serde
implementation_language: rust
depends_on: []
relationships:
  - target: "ix://agent-ix/quire"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/spec-artifacts-iso"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/spec-artifacts-app"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/filament-parser-lib"
    type: "replaces"
    cardinality: "1:1"
    scope:
      - parse_document
      - tier2_extract
      - harvest_edges
      - load_module
      - schema_validation

standards_alignment:
  - iso-iec-ieee-29148
  - ieee-828
title: "Master Requirements Specification"
---
# Master Requirements Specification
## quire-rs — Rust Templating + Parsing Engine for the Filament/Quire Ecosystem

---

## 1. Purpose

This document defines the **scope, intent, and governing requirements framework** for `quire-rs`, a Rust library crate that unifies two responsibilities in one engine:

1. **Schema-validated archetype rendering** — generate canonical markdown artifacts from typed data using MiniJinja templates.
2. **Markdown parsing** — port the existing `agent-ix/quire` (TypeScript) parser into pure Rust at byte-parity with the canonical TS fixtures.

It establishes:
- The problem space `quire-rs` addresses across rendering and parsing
- The boundaries of responsibility between layers (Edit API, Schema, Storage, Render, Parse, Query, Writeback)
- The authoritative structure for requirements, verification, and change control
- The relationship between user intent (typed edits, LLM-driven changes), system behavior (validation + render + parse + writeback), and test evidence (byte-parity with the reference implementation)

**Core invariant**: **markdown is canonical**. The on-disk `.md` is the source of truth. Blocks are *parsed from* markdown. Edits update one block's data → re-render that block via its template → splice new bytes back into the `.md` via writeback. Frontmatter and untouched blocks stay byte-identical.

This document is the **top-level requirements artifact** for the repository.

---

## 2. Scope

### 2.1 In Scope

This specification governs a **generic, archetype-agnostic engine** that processes data archetypes supplied by a caller. Local authoring/rendering tools may load those archetypes from the filesystem; service/parser runtimes may pass ObjectType rows directly in memory through the Python `ExtractionContext`. The engine itself knows nothing about specific archetypes (`FR`, `NFR`, `ADR`, etc.) — those are data shipped by Filament (or any other authoring source), synced to disk by ix-cli for local workflows, or fetched from `filament-core-service` by service consumers.

**Parse side:**
- Port of the existing `agent-ix/quire` markdown parser into pure Rust
- Markdown bytes → `QuireDocument` heading tree
- Query API: `section`, `sections`, `parse_table`, `parse_tables`, `table_from_section`, `parse_bullet_list`, `extract_diagrams`, `search`
- YAML frontmatter extraction with malformed-fallback semantics
- Fenced-code-block-aware heading parsing
- Byte-exact content slicing (no re-serialization)
- **Stable block IDs** via Pandoc heading attributes `## Heading {#blk-id}` — survive byte-exact through parse → edit → writeback round-trips

**Block model:**
- Each artifact is a typed list of **blocks** parsed *from* the canonical markdown
- `Block { id: String, type: String, schema_version: u32, data: serde_json::Value }`
- Per-block schema + per-block template (loaded from `blocks/<type>/{schema.json, template.md.j2}`)
- Frontmatter is itself a block (id = `frontmatter`, type = `frontmatter`)

**Edit side:**
- `apply_block_patch(doc, block_id, patch)` — merge patch onto block's data, validate against block-type schema, re-render block via block-type template, writeback into the markdown, return updated full-file string
- `replace_block(doc, block_id, new_data)` — full-replace variant
- LLM tool surface: `schema_for(block_type)` returns the JSON Schema for one block's patch shape, ready to wrap in any model-specific tool envelope

**Writeback side:**
- `update_section(doc, heading, new_content) → String` — section-level byte splice (port of TS `updateSection`)
- `update_block(doc, block_id, new_bytes) → String` — block-level byte splice (primitive used by `apply_block_patch`)
- Frontmatter and untouched blocks stay byte-identical

**Render side:**
- Two modes:
  - **Whole-artifact** render — assemble a new `.md` from scratch using per-block templates. Used for new-document creation.
  - **Per-block** render — re-render one block's bytes. Used by `apply_block_patch` + writeback.
- MiniJinja with `UndefinedBehavior::Strict`
- Byte-for-byte parity with the existing Python Jinja2 reference renderer

**Object-extraction side:**
- Body-extraction DSL evaluator supporting all six Locator primitives: `frontmatter_field`, `section_body`, `code_block`, `table_row`, `list_item`, `heading`
- Single-yield (`match`) and multi-yield (`iterate_over` + `per_match`) DSL patterns
- Secondary / fallback locators per field for author-variant tolerance

**Repository + corpus side:**
- `load_repo(path)` — parallel (rayon), ignore-file-aware directory walk that parses every `.md` into a `QuireDocument`, returning the collection + per-file diagnostics ([FR-024](./functional/FR-024-parallel-repo-walk.md)). Per-file parse failures are non-fatal.
- `Spec` corpus — a bounded, in-memory, immutable set of loaded documents indexed by stable artifact id ([FR-025](./functional/FR-025-spec-corpus-model.md)), with its **intra-spec** references resolved ([FR-026](./functional/FR-026-intra-spec-reference-resolution.md)) and read-only whole-spec queries (`by_id`, `by_type`, `referencing`, `outgoing`, `orphans`, `dangling`) over the resolved structure ([FR-027](./functional/FR-027-whole-spec-query-api.md)). Lifecycle is *load → examine → discard*; the corpus is a data structure, not a stateful engine.

**Python binding side:**
- Feature-gated (`--features python`) PyO3 + maturin bindings exposing parse / extract / validate / render / `load_repo` / corpus to Python as the `quire` wheel ([FR-023](./functional/FR-023-python-binding-surface.md)). With the feature off, the crate is unchanged and interpreter-free ([StR-001](./stakeholder/StR-001-single-rust-engine.md) boundary). Bindings invert the call direction (Python calls *into* Rust); the engine never shells *out*. This is the path by which `filament-parser-lib` consumes the engine at native speed ([StR-005](./stakeholder/StR-005-native-python-bindings.md)), superseding its Python hot paths.
- `ExtractionContext.from_object_types(...)` compiles caller-supplied ObjectType rows for service runtimes. This path performs no filesystem or network registry discovery; callers own registry sourcing.

**Cross-cutting:**
- Safety scaffolding inherited from `agent-ix/rust-lib-cookiecutter` (clippy MSRV, deny.toml, `// SAFETY:` enforcement)
- Hardening hygiene: compile-time `forbid(unsafe_code)` ([NFR-003](./non-functional/NFR-003-zero-unsafe.md)), fuzz (cargo-fuzz), mutation testing (cargo-mutants), advisory checking (cargo-audit) — all required, not opt-in. (The scheduled Miri job was retired — ADR 0006.)
- Public Rust API stable across parse, render, edit, writeback, and extract surfaces
- Engine is **offline by default** — works against the local filesystem with zero network dependencies

### 2.2 Out of Scope

This specification does not govern:

- **Sync from Filament to disk.** The local module directory (e.g. `~/.ix/filament/modules/`) is populated by external tools — `ix-cli` / `quoin` are the canonical syncers (handle Filament auth + transfer / module install). `quire-rs` can consume a path when a caller asks it to load local archetypes, but it never owns `.ix` synchronization and never calls Filament directly.
- **Runtime ObjectType registry sourcing.** `filament-core-service` owns the dynamic ObjectType registry. Consumers such as `filament-analysis-worker` and `cloudmanager-local-sync` fetch registry snapshots from core and pass them through parser-lib into `ExtractionContext`. `quire-rs` does not discover those ObjectTypes itself.
- **Authoring tooling.** Schema files, templates, and manifests are authored elsewhere (in Filament, by hand, by another tool). `quire-rs` does not write archetype data.
- **Author-time schema validation.** `quire-rs` validates JSON Schema documents at archetype-load time ([FR-013](./functional/FR-013-archetype-loader.md)). Pre-publish validation (catching authoring errors before they reach disk) is Filament's concern.
- **Hot reload on filesystem change.** `quire-rs` does NOT watch the filesystem and does NOT automatically reload archetypes when files change on disk. Consumers refresh archetypes by calling `Registry::load_from(...)` again. The previous Registry stays alive for any outstanding references and is dropped when they release. There is no in-place update or change-event subscription.
- **Schema migration when archetypes evolve.** When an archetype's schema changes (e.g. Filament publishes a new version of `fr-frontmatter.schema.json`), `quire-rs` validates incoming data against the loaded version only. Migrating existing artifacts written against an older schema is Filament's responsibility (or downstream migration tooling).
- **LLM model-specific tool-call adapters.** `quire-rs::schema_for` returns a JSON Schema. Wrapping it into a model-specific tool-call envelope (OpenAI function-calling shape, Anthropic tool-use shape, etc.) is the consumer's concern.
- **ID generation (partially relaxed — CR-002).** `quire-rs` validates that *human* artifact IDs match the schema's `pattern` (e.g. `^[A-Z]{2,4}-[0-9]+$`); those are authored upstream (Filament UI, scripts). **However**, as of v0.3 `quire-rs` DOES generate a durable `uuid` (UUID7) when **creating a new artifact** (the whole-artifact render path), embedding it in the new document's frontmatter — going forward every quire-authored doc carries a `uuid`. `quire-rs` still does NOT backfill `uuid`s into pre-existing files on disk (no load-time mutation); `load_repo` reads the `uuid` and reports a non-fatal diagnostic when absent. Cross-repo catalog assignment beyond the per-doc `uuid` remains an upstream/service-layer concern.
- **Internationalized slug normalization.** [FR-009](./functional/FR-009-slug-line-id.md) implements ASCII-only slug normalization to match the TS/Py reference. Non-ASCII heading authoring works (the section parses correctly), but the slug collapses non-ASCII characters to `-`. Full Unicode slug support is deferred to a future version.
- **Windows path semantics.** `v1` supports macOS and Linux only. Filesystem-loader behavior on Windows (drive letters, `\` separator, symlink permissions) is undefined.
- **React UI bindings.** `agent-ix/quire` ships React components for browser-side rendering; those are TypeScript-only. `quire-rs` does not provide a UI layer.
- **Cross-document graph queries — *general/stateful*.** `agent-ix/quire` ships a React provider that indexes multiple parsed documents and exposes hooks for cross-doc queries. The *general, stateful* graph engine remains out of scope: no persistence of a resolved graph, no query/traversal DSL, no caching across calls, no incremental reparse on change, and no resolution of references that point into a **different** spec. These are service-layer concerns (see ADR-0002). **Carve-out:** the bounded, in-memory, ephemeral **per-spec corpus** ([FR-025](./functional/FR-025-spec-corpus-model.md)/026/027) *is* in scope — it loads one spec, resolves the references *within that loaded set*, and answers whole-spec read-only queries, then is discarded. The rule is: intra-spec resolution = `quire-rs`; inter-spec or stateful = service layer ([StR-006](./stakeholder/StR-006-whole-spec-corpus.md)).
- CRDT or OT live-editing semantics.
- Schema-driven template generation (schemas validate; templates present; neither generates the other).
- Heavy hardening tooling (kani formal verification, shuttle concurrency permutation, sim-spire) — opt-in via future cookiecutter variant. (Standard Rust safety hygiene — `forbid(unsafe_code)`, fuzz, mutants, advisory — is required and in scope. Scheduled Miri was retired per ADR 0006; loom ([NFR-017](./non-functional/NFR-017-concurrency-permutation.md)) is adopted for the concurrency surface.)
- Real-time multi-user editing.
- Generating Rust types from JSON Schemas. Downstream Rust consumers that want typed bindings author them by hand or use `schemars` themselves — `quire-rs` does not derive types.

---

## 2bis. Drift Audit

The v0.1 implementation drifted from `INPUT.md`. The v0.2 spec restores discovery alignment. Two reports are recorded here so future readers can trace every divergence to a deliberate decision.

### A. Discovery (`INPUT.md`) ↔ Current Spec

#### Dropped in v0.1, restored in v0.2

| Discovery feature | INPUT.md lines | v0.1 status | v0.2 status |
|---|---|---|---|
| Block data model `Block { id, type, schema_version, data }` | 38–48 | No `Block` type | First-class type |
| Per-block schema + template pairing | 52–69 | Schema/template per *archetype* | Schema/template per *block type* |
| Block-level edit API `{block_id, patch}` | 128–138 | Artifact-level `apply_patch` | `apply_block_patch(doc, block_id, patch)` |
| Two edit operations: patch + full-replace | 192–195 | Only deep-merge patch | Both `apply_block_patch` and `replace_block` |
| Writeback into canonical markdown | implied | No writeback at all | `update_section` + `update_block` primitives |
| Schema versioning + migration (`schema_version`) | 45, 151 | Not addressed | Convention defined; migrations supplied by consumer |
| Stable block IDs surviving round-trip | implied | `<slug>-L<line>`, unstable | `## Heading {#blk-id}` Pandoc attribute |
| LLM tool per block type (`edit_callout(block_id, patch)`) | 196–206 | Single `apply_patch` whole-frontmatter | `schema_for(block_type)` per block |

#### Added in v0.1 without discovery basis — stripped in v0.2

| v0.1 item | Disposition |
|---|---|
| FR-015 Relationship harvesting (`harvest_edges`) | Removed |
| FR-017 Diagnostic Collection API (public collector + by_kind filter) | Removed (the internal `Diagnostic` enum stays as payload for `QuireError` paths only) |
| FR-018 IxUriResolver + `RelationshipResolver` trait | Removed |
| NFR-008 Tracing instrumentation | Removed |
| Layer 1/2/3/4 terminology | Replaced with plain references ("parser", "Query API", "React UI", "cross-document graph") |
| [FR-014](./functional/FR-014-module-activation.md) expansion (`load_strict`, `archetype_in_module`, `module_version`) | Trimmed to minimal "multiple modules coexist; collisions diagnosed" |

#### Stays unchanged

- Parser FRs ([FR-005](./functional/FR-005-parse-document-api.md)..009): pure port of `agent-ix/quire` core
- [FR-010](./functional/FR-010-query-api.md) Query API: pure port of `agent-ix/quire` query
- [FR-003](./functional/FR-003-archetype-schema-surface.md) schema_for, [FR-004](./functional/FR-004-minijinja-strict-environment.md) strict MiniJinja env
- [FR-011](./functional/FR-011-body-extraction-dsl.md)/016 body-extraction DSL (reworked around block-level scope)
- [FR-013](./functional/FR-013-archetype-loader.md) archetype loader (reworked: `blocks/<type>/...` layout)
- [NFR-001](./non-functional/NFR-001-render-latency.md)..007, [NFR-009](./non-functional/NFR-009-dependency-pinning.md)..010, [NFR-011](./non-functional/NFR-011-fuzz-testing.md)/013/014 (perf, safety, error shape, determinism, dep pinning, API stability, fuzz, mutants, audit). [NFR-012](./non-functional/NFR-012-miri-ub-check.md) (Miri) retired — ADR 0006.
- HTML / comrak references: already purged in `08f5b00` (never asked for in discovery)

### B. quire-TS (`agent-ix/quire`) ↔ quire-rs

#### TS exports quire-rs implements

| TS export (file) | quire-rs equivalent |
|---|---|
| `parseDocument` (core/parser.ts) | `parse_document` (`src/parser/mod.rs`) |
| `extractFrontmatter` (core/frontmatter.ts) | `extract_frontmatter` (`src/parser/frontmatter.rs`) |
| `section`, `sections`, `parseTable`, `parseTables`, `tableFromSection`, `parseBulletList`, `extractDiagrams`, `search` (core/query.ts) | All present in `src/query.rs` |
| `updateSection` (core/writeback.ts) | **PORTED in v0.2** — `update_section` in `src/writeback.rs` |

#### TS exports skipped intentionally

| TS export | Reason |
|---|---|
| `findDiagramByTag` | Consumers can filter `extract_diagrams[…].tag` themselves |
| `parseDelegations` | Niche — no use case requested |
| `src/react/*` (Layer 3 React components) | Out of scope per § 2.2 |
| `src/graph/index.tsx` (Layer 4 cross-document graph) | Out of scope per § 2.2 |

#### quire-rs subsystems with no TS analog

| quire-rs subsystem | Origin |
|---|---|
| Render layer (MiniJinja env, loader, schema validation, render dispatch) | TS quire is parse-only; the render half lives in Python `spec-artifacts-iso`. quire-rs unifies both halves. |
| Block edit API (`apply_block_patch` / `replace_block`) | New ground; TS has only `updateSection`. |
| Body-extraction DSL evaluator | Formerly lived in Python `filament-parser-lib`; quire-rs is the canonical implementation. |

#### Behavioural differences kept

- `parse_table` returns `Option::None` on miss; TS returns `{headers: [], rows: []}`. Intentional Rust-idiomatic divergence.
- Heading matching: case-insensitive + section-number normalization. Matches TS exactly.
- Section content slicing: byte-exact in `quire-rs` ([FR-008](./functional/FR-008-byte-exact-slicing.md)); TS applies `.strip()`. Required for writeback fidelity.

### C. Render Removal (v0.4 — 2026-06-04)

The render/templating half of `quire-rs` is **removed** — **no backward-compatibility
layer**, no deprecated-but-kept field, no dual-read. `quire-rs` is now a
**parse / validate / extract / byte-splice** engine. Markdown is authored directly
(not generated from typed data) and checked structurally by `validate_document`
([FR-032](./functional/FR-032-validate-document.md)). This reverses the original "unify render + parse" mandate (§1, §3) for the
render half; that prose is retained above for history but is superseded by this
entry. The `validate` engine fn (data-schema validation, `src/validate.rs`, [FR-002](./functional/FR-002-schema-validation-pipeline.md))
**stays** — it backs `validate_document` even though the downstream CLI `--json`
context mode is removed.

#### Retired (kept for history, ACs dropped from the required-coverage tally)

| Artifact | Kind | Why |
|---|---|---|
| **[FR-001](./functional/FR-001-render-dispatch.md)** Render dispatch | FR | No `render`/`render_by_name`; generic render API removed |
| **[FR-004](./functional/FR-004-minijinja-strict-environment.md)** Strict MiniJinja env | FR | `minijinja` dependency + `Environment` removed |
| **[FR-012](./functional/FR-012-archetype-parity-suite.md)** Archetype render-parity suite | FR | No render path to compare against Python; `render_parity/` removed |
| **[NFR-001](./non-functional/NFR-001-render-latency.md)** Render latency | NFR | No render path to bench; perf gate now parse/validate/extract |
| **[US-001](./usecase/US-001-llm-emits-validated-patch.md)** LLM emits validated patch (→ render) | US | Render-centric |
| **[US-004](./usecase/US-004-filament-editor-rerender.md)** Editor merge-validate-render | US | Render-centric (block edits now byte-splice only) |
| **[US-005](./usecase/US-005-ci-detects-render-regression.md)** CI detects render regression | US | Render byte-parity suite (`render_parity/`) removed; missed in the first pass, retired here |
| **[US-006](./usecase/US-006-llm-patches-one-block.md)** LLM patches one block (render+splice) | US | Render-and-splice path retired |
| **[US-007](./usecase/US-007-llm-replaces-block.md)** LLM replaces a block (render+splice) | US | Render-and-splice path retired |
| **[US-009](./usecase/US-009-llm-creates-new-artifact.md)** LLM creates a new artifact (render) | US | Whole-artifact render retired; author markdown directly |
| **Gate G2** Render parity | Gate | Retired; G4 reframed to byte-splice-only round-trip |
| **Tasks 007/010/011/012/013** | Task | Render env / dispatch / parity harness / gate / sweep |
| **[FR-028-AC-1](./functional/FR-028-expanded-python-binding-surface.md)** render byte-parity; **[NFR-006-AC-1](./non-functional/NFR-006-determinism.md)** render determinism | AC | Render-specific ACs (ids retained, immutable; dropped from tally) |

#### Revised (CR-noted, kept and active)

| Artifact | Change |
|---|---|
| **[FR-013](./functional/FR-013-archetype-loader.md)** Archetype loader | Schema-only; no template parse/register; `template_ref` not read |
| **[FR-031](./functional/FR-031-unified-archetype-shape.md)** Unified archetype shape | Drops `template_ref` + `is_renderable()`; `template_ref` is a hard-rejected deprecated field. AC-1/AC-2 recast to validate/extract |
| **FR-021** Block edit (no FR doc — tests.md only) | Render-and-splice retired; block edit = byte-splice `update_block` (FR-022) |
| **[FR-023](./functional/FR-023-python-binding-surface.md) / [FR-028](./functional/FR-028-expanded-python-binding-surface.md)** Python bindings | Drop `render`/`render_by_name`/`render_block` + `QuireRenderError`; keep validate/validate_document/extract/parse/load_repo/harvest_edges |
| **[NFR-006](./non-functional/NFR-006-determinism.md)** Determinism | Names `parse_document`/`validate_document`/`extract` (render determinism retired) |
| **Task 014** Perf gates | parse/validate/extract/load only; render bench dropped |
| **D4** Block edit task | Byte-splice only |

#### Decisions (v0.4 — 2026-06-04)

- **Placeholder sentinel set reduced** ([FR-032-AC-7](./functional/FR-032-validate-document.md)/AC-8): bare `none` and `n/a` are
  **NOT** sentinels (they reject legitimate content like `Upstream: none`). The set is
  `TODO`/`TBD` (case-insensitive prefix), whole-value `{{…}}`, whole-value
  `placeholder`, whole-value `none specified`, and empty.
- **Empty/header-only tables and item-less lists** report reason **`empty`** (and a
  wholly-unresolved locator reports **`missing`**) — **not** `placeholder`
  ([FR-032-AC-9](./functional/FR-032-validate-document.md)). The [FR-030](./functional/FR-030-required-section-validation.md) / [FR-032](./functional/FR-032-validate-document.md) prose is corrected to match the code.

#### Gap back-fills (CR-noted ACs added)

[FR-033-AC-7](./functional/FR-033-locator-assert-facet.md) (assert-kind legality matrix), [FR-033-AC-8](./functional/FR-033-locator-assert-facet.md) (id-column precedence),
[FR-033-AC-9](./functional/FR-033-locator-assert-facet.md) (`id_pattern` on non-table locators); [FR-011-AC-15](./functional/FR-011-body-extraction-dsl.md) (`regex:` projection),
[FR-011-AC-16](./functional/FR-011-body-extraction-dsl.md) (`under_section:None` substrate), [FR-011-AC-17](./functional/FR-011-body-extraction-dsl.md) (whole-value `{{…}}`),
[FR-011-AC-18](./functional/FR-011-body-extraction-dsl.md) (unclosed-fence → final block, backtick + tilde), [FR-011-AC-19](./functional/FR-011-body-extraction-dsl.md)
(`emit_edges` record-derived edges); [FR-032-AC-7](./functional/FR-032-validate-document.md)..10 (placeholder set / `none`-`n/a` /
empty-table-list reason / asserts-on-resolved); [FR-032-AC-2](./functional/FR-032-validate-document.md) (`line` is `Option`,
`None` for a wholly-absent section); [NFR-002-AC-4](./non-functional/NFR-002-parse-latency.md) (`validate_document` latency),
[NFR-006-AC-4](./non-functional/NFR-006-determinism.md) (validate/extract determinism), and **[NFR-019](./non-functional/NFR-019-input-robustness.md)** (input robustness:
validate/extract/query never panic on arbitrary input).

---

## 3. System Overview

### 3.1 System Description

`quire-rs` is a single Rust crate that exposes three complementary APIs in one dependency:

1. A **renderer** that, given a loaded archetype (compiled schema + compiled template) and a `serde_json::Value`, validates the data and emits canonical markdown via a pre-loaded MiniJinja environment with strict undefined behavior.
2. A **parser** that takes raw markdown and produces a `QuireDocument` heading tree with O(1)-lookup query helpers.
3. An **extractor** that, given a parsed `QuireDocument` and a `body_extraction` DSL, returns typed extraction records + harvested relationship edges.

The three halves share a domain (the agent-ix knowledge ecosystem) but execute independently — the parser does not invoke the renderer, the renderer does not invoke the parser. They meet at the canonical markdown surface: a renderer output is a valid parser input.

**The engine knows nothing about specific archetypes.** `FR`, `NFR`, `ADR`, `Plan`, `domain`, `entity`, etc. are not Rust types compiled into `quire-rs` — they are data, authored as `(manifest.yaml, schemas/*.json, templates/*.j2)` triples, stored in Filament (or any other source-of-record), synced to the local filesystem by an external tool (`ix-cli` is the canonical syncer), and loaded into a `Registry` at engine startup. This decoupling means: (a) adding a new archetype requires no code change to `quire-rs`; (b) the engine has zero runtime dependency on Filament or any network service; (c) hand-authored or test-fixture archetype sets are first-class.

The crate is the Rust home for these responsibilities so that downstream consumers (the Filament editor stack via FFI, spec pipelines, CLI tools, batch extractors) get one performant binary dependency rather than coordinating TypeScript and Python toolchains.

### 3.2 Architecture (data flow)

```
┌──────────────┐
│   Filament   │   knowledge platform — authoritative archetype store
└──────┬───────┘
       │ (authenticated API)
       ▼
┌──────────────┐
│   ix-cli     │   auth + sync — pulls archetype modules to disk
└──────┬───────┘
       │ (writes files)
       ▼
┌────────────────────────┐
│ ~/.ix/filament/modules │   filesystem contract — manifest.yaml + schemas/ per module
└──────┬─────────────────┘   (or IX_FILAMENT_MODULES_PATH / IX_SCHEMA_PATH dirs)
       │ (reads at load time)
       ▼
┌──────────────┐
│   quire-rs   │   generic engine — Registry of CompiledArchetype values
└──────────────┘
       │
       └──> render(archetype, data) → markdown
       └──> parse_document(md) → QuireDocument
       └──> extract(doc, dsl) → ExtractionResult
```

Sync from Filament to disk is **explicitly outside `quire-rs`'s concern** (§2.2). The engine is a filesystem consumer.

### 3.3 Layered Architecture (per side)

The layered architecture for the **render side** is taken from the design described in `INPUT.md`, with the schema and template layers now sourced from filesystem-loaded data rather than compiled Rust types:

```
┌─────────────────────────────────────────────────┐
│  Edit API           (patches, full replaces)    │  ← transport
├─────────────────────────────────────────────────┤
│  Schema layer       (compiled JSON Schema       │  ← correctness
│                      validators from on-disk    │
│                      schema documents)          │
├─────────────────────────────────────────────────┤
│  Storage            (canonical markdown on disk;│  ← persistence
│                      blocks parsed from it)     │
├─────────────────────────────────────────────────┤
│  Render layer       (MiniJinja per-block-type   │  ← presentation
│                      templates)                 │
├─────────────────────────────────────────────────┤
│  Writeback          (byte-splice block bytes    │  ← persistence-out
│                      back into canonical .md)   │
└─────────────────────────────────────────────────┘
```

The **parse side** is a single-pass pipeline:

```
┌──────────────────────────────────────────────┐
│  Frontmatter extraction  (YAML or fallback)  │
├──────────────────────────────────────────────┤
│  Heading walk            (fence-aware)       │
├──────────────────────────────────────────────┤
│  Content slicing         (byte-exact)        │
├──────────────────────────────────────────────┤
│  Tree assembly           (level-aware stack) │
├──────────────────────────────────────────────┤
│  Query API               (lazy, on-demand)   │
└──────────────────────────────────────────────┘
```

Each layer has one job and a narrow interface to the next. The parser does not validate; the renderer does not parse.

### 3.4 Intended Users

- **Filament document editor** — needs schema-validated edits and re-render on patch
- **`agent-ix/filament-parser-lib`** — the Python orchestration layer; consumes `quire-rs` in-process via the feature-gated PyO3 bindings ([FR-023](./functional/FR-023-python-binding-surface.md)), superseding its own walk/parse/extract/validate hot paths ([StR-005](./stakeholder/StR-005-native-python-bindings.md)). Keeps tier-3 plugin discovery + dispatch in Python; parser/extractor/validator semantics remain in quire-rs.
- **`spec-artifacts-*` Python repos** — call `quire-rs` via the same PyO3 bindings for parity-rendered artifacts
- **`spec-analysis-*` / `spec-matrix` tooling and LLM agents auditing a spec** — load a `spec/` tree into a `Spec` corpus ([FR-025](./functional/FR-025-spec-corpus-model.md)) and run whole-spec traceability/coverage/reference queries ([FR-027](./functional/FR-027-whole-spec-query-api.md)) instead of re-walking + re-greps ([US-012](./usecase/US-012-agent-audits-whole-spec.md), [US-013](./usecase/US-013-agent-resolves-intra-spec-refs.md))
- **`spec-objects-business` extractors** — evaluate `body_extraction` DSL via the parser's Query API
- **CLI tools** — invoke the renderer to produce spec artifacts from typed YAML/JSON sources
- **LLM agents** — receive the on-disk JSON Schemas (surfaced unchanged via `schema_for`) as tool-call input contracts, emit validated patches that the schema layer accepts and the renderer formats

---

## 4. Requirements Architecture

Requirements are decomposed and managed using a **hierarchical structure** consistent with ISO/IEC/IEEE 29148.

```
spec/
├── spec.md                # This document (master specification)
├── stakeholder/           # Stakeholder requirements (StR-XXX)
├── usecase/               # User intent and usage scenarios (US-XXX)
├── functional/            # System / functional requirements (FR-XXX)
├── non-functional/        # Non-functional requirements (NFR-XXX)
├── tests.md               # Bidirectional requirements ↔ tests mapping
├── test_cases/            # Verification artifacts (TC-XXX)
└── assets/
    ├── diagrams/          # Sequence and architecture diagrams
    └── models/            # Reference AST shapes, schema models
```

---

## 5. Requirement Classes

### 5.1 Stakeholder Requirements

Stakeholder Requirements capture **authoritative needs and expectations**.

- Format: `StR-XXX`
- Location: `stakeholder/`
- Nature: Normative for intent
- Purpose: Drive system requirements

### 5.2 User Requirements

User Stories describe **intent, expectations, and usage outcomes**.

- Format: `US-XXX`
- Location: `usecase/`
- Nature: Informational, non-binding
- Purpose: Drive functional requirements

### 5.3 Functional Requirements

Functional Requirements define **authoritative, testable system behavior**.

- Format: `FR-XXX`
- Location: `functional/`
- Nature: Normative and binding
- Purpose: Define observable behavior

All functional requirements:
- Use deterministic language
- Are independently testable
- Trace back to one or more user requirements

### 5.4 Non-Functional Requirements

Non-Functional Requirements define **quality constraints** (performance, security, etc.).

- Format: `NFR-XXX`
- Location: `non-functional/`
- Nature: Normative and binding
- Purpose: Constrain system qualities

### 5.5 Acceptance Criteria

Acceptance criteria define **verifiable outcomes** for functional requirements.

- Format: `{FR-XXX}-AC-N`
- Location: Within each functional requirement file
- Purpose: Verification anchor

---

## 6. Requirement Identification

### 6.1 Identifier Schema

| Artifact | Format | Example |
|-------|-------|--------|
| Stakeholder Requirement | `StR-XXX` | [StR-001](./stakeholder/StR-001-single-rust-engine.md) |
| User Story | `US-XXX` | [US-002](./usecase/US-002-developer-parses-spec-doc.md) |
| Functional Requirement | `FR-XXX` | [FR-014](./functional/FR-014-module-activation.md) |
| Non-Functional Requirement | `NFR-XXX` | [NFR-003](./non-functional/NFR-003-zero-unsafe.md) |
| Acceptance Criteria | `{FR}-AC-N` | [FR-014-AC-1](./functional/FR-014-module-activation.md) |
| Test Case | `TC-XXX` | `TC-021` |
| Change Request | `CR-XXX` | `CR-009` |

Identifiers are immutable once assigned.

---

## 7. Requirement Quality Policy

All **functional requirements** SHALL:
- Define observable behavior
- Be unambiguous and atomic
- Avoid implementation details unless required
- Be testable through explicit criteria

Functional requirements SHALL NOT:
- Encode application-specific policy
- Contain compound behaviors
- Use subjective language

---

## 8. Archetype Model

### 8.1 Archetypes Are Data

An **archetype** is the named pairing of a JSON Schema document and a MiniJinja template that together describe one renderable kind (e.g. `FR`, `NFR`, `ADR`, `Plan`). Archetypes are authoring artifacts — they live as files on disk, version-controlled, hand-editable or agent-editable, NOT compiled into the engine.

A `quire-rs` Registry knows an archetype by:

- **Name** (e.g. `"fr"`) — bare string identifier from the module manifest
- **Module provenance** — which module the archetype was loaded from
- **Compiled JSON Schema validator** — built once at load time from the on-disk schema document
- **Pre-parsed MiniJinja template** — registered with the long-lived strict environment at load time
- **Manifest metadata** — `required_sections`, version, etc.

The Registry is populated by [FR-013](./functional/FR-013-archetype-loader.md) (filesystem loader) and [FR-014](./functional/FR-014-module-activation.md) (multi-module activation). No archetype names are hard-coded in Rust source.

### 8.2 The Schema/Template Pair (on disk)

Each archetype is two files under a module root:

```
<module-root>/
├── manifest.yaml                          # declares the archetype
├── schemas/<name>-frontmatter.schema.json # JSON Schema draft 2020-12
└── templates/<name>.md.j2                 # MiniJinja template
```

The manifest entry references both by relative path:

```yaml
- name: fr
  template_ref: templates/fr.md.j2
  frontmatter_schema_ref: schemas/fr-frontmatter.schema.json
  required_sections: [Description, Specification, Acceptance Criteria, Dependencies]
```

The schema and template share a name and field references, but neither generates the other. Their contract is the validated `serde_json::Value` handoff at render time.

### 8.3 v1 Baseline Corpus (informational)

At spec authoring time the local filesystem-synced corpus contains **17 archetypes** across three modules:

| Module | Count | Archetype names |
|---|---|---|
| `spec-artifacts-iso` | 8 | FR, NFR, StR, US, IT, TC, AC, CON |
| `spec-artifacts-app` | 2 | ApplicationSpec, MasterRequirements |
| `spec-artifacts-process` | 7 | ADR, Plan, Task, Review, Finding, TestMatrix, Standard |

This list is **informational, not normative.** The Registry contents are whatever the filesystem holds at load time. Adding a new archetype is a sync operation (new files in `~/.ix/filament/modules/`), not a code change.

The parity suite ([FR-012](./functional/FR-012-archetype-parity-suite.md)) enumerates archetypes from `tests/render_parity/corpus.yaml` and runs against every fixture pair on disk — that file is the byte-parity source of truth, not this table.

### 8.4 Object Archetypes vs. Artifact Archetypes

The same `manifest.yaml` mechanism that declares **artifact archetypes** (renderable kinds like FR/NFR) also declares **object archetypes** (extractable kinds with `body_extraction` DSL — e.g. `domain`, `entity`, `permission`). Object archetypes do not render; they extract.

The Registry tracks both kinds. Render operations target artifact archetypes; extract operations target object archetypes. The engine raises a typed error when the kinds are mismatched (e.g. asking to render an object archetype).

---

## 9. Parser Model

### 9.1 Document Shape

`quire-rs` SHALL expose a `QuireDocument` ADT mirroring `agent-ix/quire`:

- `preamble: Option<String>` — text before the first heading
- `sections: Vec<QuireSection>` — top-level tree
- `raw: String` — original input, preserved verbatim
- `frontmatter: Option<serde_json::Map<String, Value>>` — parsed YAML, or `None` if absent/malformed

`QuireSection` mirrors the TS interface:

- `id: String` — `<slug>-L<line>` format
- `heading: String` — raw heading text (no `#` prefix)
- `level: u8` — 1–6
- `content: String` — byte-exact slice
- `children: Vec<QuireSection>` — recursive
- `start_line: usize`, `end_line: usize` — 0-based line bounds

### 9.2 Parser Guarantees (parity points)

The parser SHALL match `agent-ix/quire` acceptance fixtures exactly:

- Headings inside fenced code blocks (` ``` ` or `~~~`) are NOT parsed as headings
- Content slicing is byte-exact — no whitespace normalization, no re-serialization
- Slug IDs are `<lowercase-slug>-L<0-based-line>` (e.g. `"2.1 In Scope"` at line 6 → `"2-1-in-scope-L6"`)
- Malformed YAML frontmatter returns `frontmatter: None` and `body = entire input` — not an error
- Unclosed fenced blocks: trailing content is part of the block
- Level-skipping headings (e.g. `## A` then `#### B`) — B becomes a child of A, not a phantom sibling

### 9.3 Query API

The Query API SHALL expose, at minimum:

- `section(doc, heading)` — first section by exact heading
- `sections(doc, level)` — flat list, optional level filter
- `parse_table(content)` / `parse_tables(content)` — markdown table extraction
- `table_from_section(doc, heading)` — convenience
- `parse_bullet_list(content, pattern)` — bullet items with optional pattern
- `extract_diagrams(doc, language)` — fenced code blocks of a given language
- `search(doc, query)` — substring or word search across sections

---

## 10. Body-Extraction DSL

`quire-rs` SHALL evaluate the YAML-encoded body extraction DSL used by `spec-objects-architecture` and `spec-objects-business`. Extractors keyed by:

- `frontmatter_field` — value at a JSONPath in the parsed frontmatter
- `section_body` — text of the section under a given heading (uses Quire `section()` internally)
- `code_block` — fenced code block of a given language, optionally constrained to a section

The DSL itself remains in YAML in the source repos; `quire-rs` supplies the evaluator. Output is a typed map keyed by the DSL's `match` keys.

---

## 10bis. Spec Corpus Model

A **Spec** is not one document — it is a bounded set of related artifacts (a `spec/` tree: StR, US, FR, NFR, test cases, `spec.md`) whose references point at each other. `quire-rs` SHALL be able to load such a set as an in-memory **corpus**, resolve the references among its members, and answer whole-spec questions — without becoming a graph engine.

### 10bis.1 Lifecycle

```
load_repo(path) ──► RepoLoad ──► Spec::from_repo ──► queries ──► drop
   (FR-024)                         (FR-025+026)        (FR-027)
```

*Load → resolve → examine → discard.* The corpus holds no state across calls, persists nothing, watches nothing, and reaches nothing outside the loaded set.

### 10bis.2 Data structure, not a stateful engine

The boundary that keeps this in `quire-rs` scope (and out of the territory of the previously-removed graph engine) is **data structure vs. stateful engine**:

| In scope (`quire-rs`) | Out of scope (service layer) |
|---|---|
| Load a directory into a corpus ([FR-024](./functional/FR-024-parallel-repo-walk.md)/025) | Persist the resolved graph |
| Resolve references **within** the loaded set ([FR-026](./functional/FR-026-intra-spec-reference-resolution.md)) | Resolve references into a **different** spec |
| Read-only by-id / by-type / reverse-edge / orphan queries ([FR-027](./functional/FR-027-whole-spec-query-api.md)) | A query / traversal DSL; transitive-closure precompute |
| Immutable, `Send + Sync`, rebuild-to-refresh | Incremental reparse, change subscription, caching |

The rule: **intra-spec resolution = `quire-rs`; inter-spec or stateful = service layer.** A reference whose target is absent from the loaded set is reported as *dangling*, never resolved outside the corpus.

### 10bis.3 Edges

Edge stubs are harvested from two already-parsed sources and unified into one resolved edge set ([FR-026](./functional/FR-026-intra-spec-reference-resolution.md)): frontmatter `relationships` entries (`{target, type, cardinality}`) and `ix://` body links. Each edge is classified `Resolved` (target present in the set) or `Dangling` (target absent). Resolution is O(edges) — one hash lookup per stub against the corpus id index — and deterministic.

### 10bis.4 Consumers

The corpus is the substrate the `spec-analysis-*` and `spec-matrix` skills need: traceability gaps (FRs with no `implements` edge to a StR), coverage gaps (user stories with no test), and reference navigation (everything that references a given artifact). These run today by re-walking and re-greps; against a corpus they query an already-resolved structure ([US-012](./usecase/US-012-agent-audits-whole-spec.md), [US-013](./usecase/US-013-agent-resolves-intra-spec-refs.md)).

---

## 11. Error and Failure Model

### 11.1 Error Classification

- **Schema violations** — typed field-keyed errors usable by both UIs and LLM editors for retry
- **Template errors** — missing field references caught by `UndefinedBehavior::Strict`
- **Parser tolerance** — malformed YAML and unclosed fences degrade gracefully; never panic
- **Invalid archetype type** — explicit error when the type discriminator does not match a registered archetype

### 11.2 Failure Handling Guarantees

- The library SHALL NOT panic on malformed input
- Errors propagate as typed `Result<_, QuireError>` values
- Schema violations carry the violating field path and a human-readable message
- Template errors carry the template name and missing field name

---

## 12. Traceability

Bidirectional traceability SHALL be maintained between:
- Stakeholder Requirements → User Stories / Functional Requirements
- User Requirements → Functional Requirements
- Functional Requirements → Acceptance Criteria
- Acceptance Criteria → Test Cases

Traceability is recorded in `tests.md` (produced via `/spec-matrix`).

---

## 13. Verification Strategy

Functional requirements SHALL be verified using one or more of:

- **Unit tests** — per-module behavior in `src/`
- **Integration tests** — end-to-end in `tests/`
- **Byte-parity tests** — for render archetypes, output is compared byte-for-byte against fixtures produced by the Python Jinja2 reference renderer
- **Acceptance ports** — for the parser, the TS/Py test fixtures are transliterated into Rust and SHALL all pass
- **Property tests** — `proptest` roundtrips, especially for the parser (parse → render → parse equivalence where applicable)
- **Criterion benchmarks** — for NFR latency targets
- **Python binding tests (`pytest`)** — the `python`-feature wheel is verified from Python via a `pytest` harness (built with `maturin develop`): parse/validate/render parity vs the Rust API, exception-mapping, GIL-release concurrency, and abi3 cross-version import ([FR-023](./functional/FR-023-python-binding-surface.md) / [NFR-016](./non-functional/NFR-016-binding-overhead.md)). The Rust test suite cannot exercise the FFI boundary; pytest is the verification method for the binding layer.
- **Concurrency + FFI hardening** — `loom` exhaustive interleaving on the parallel-walk path ([NFR-017](./non-functional/NFR-017-concurrency-permutation.md)) and scheduled `TSAN`/`ASAN` lanes on the built extension ([NFR-018](./non-functional/NFR-018-ffi-sanitizer-lanes.md)) cover the concurrency and FFI surfaces. (First-party `unsafe`/UB is compile-impossible via `forbid(unsafe_code)`, [NFR-003](./non-functional/NFR-003-zero-unsafe.md); the Miri job was retired — ADR 0006.)

Verification evidence SHALL reference test cases in `test_cases/`.

---

## 14. Change Management

All requirements artifacts are **configuration-controlled items**.

- Changes are proposed via change requests (`CR-XXX`)
- Changes require impact analysis
- Approved changes update affected requirements, tests, and traceability
- Historical versions are preserved
- Parser parity changes that diverge from `agent-ix/quire` require coordinated update of the upstream TS reference (or explicit documentation of intentional divergence)

---

## 15. Lifecycle Status

Functional requirements MAY declare a lifecycle status:
- DRAFT
- APPROVED
- IMPLEMENTED
- VERIFIED
- DEPRECATED

---

## 16. Governance Notes

- This document defines **system intent**, not implementation
- Functional requirements SHALL precede code changes
- Proof-of-concept code SHALL only exist when explicitly requested
- Deprecated requirements SHALL be archived, not removed
- Render parity with the Python reference is non-negotiable for v1; divergences require a CR

---

## 17. Inter-Tool Contract with ix-cli (Appendix A)

`quire-rs` reads archetype data from the local filesystem and does NOT communicate with Filament directly. The bridge between Filament and disk is owned by `ix-cli` (or any equivalent syncer). For `quire-rs` to operate correctly, the syncer SHALL honor the following filesystem contract:

### Atomicity

- File writes SHALL be atomic — write to a temp file in the same directory, then rename over the target. `quire-rs` does NOT acquire file locks; partial reads during in-place writes will produce parse errors for the affected archetype.
- Directory restructures (renaming a module directory, replacing a manifest) SHALL be atomic at the directory level when possible (rename of a sibling staging directory). If not possible, the syncer SHALL accept transient `quire-rs` load errors during the window.

### Validity

- `manifest.yaml` SHALL be valid YAML, conformant with the structural shape declared in [FR-013](./functional/FR-013-archetype-loader.md) (artifact_types and/or object_types arrays, each entry referencing `schema_ref` and `template_ref` by relative path).
- Each `schema_ref` target SHALL exist on disk and be valid JSON Schema (draft 2020-12, no cross-file `$ref` — see [FR-002](./functional/FR-002-schema-validation-pipeline.md)).
- Each `template_ref` target SHALL exist on disk and be valid MiniJinja (no `{% include %}` at v1 — see [FR-004](./functional/FR-004-minijinja-strict-environment.md)).
- **Filament SHOULD pre-validate** JSON Schema documents and MiniJinja templates before publishing — catching authoring errors at the authoring layer is preferable to surfacing them as `quire-rs` load errors. This is a SHOULD, not a SHALL: `quire-rs` does NOT depend on Filament's pre-validation for correctness; it always validates at load time.
- `quire-rs` does NOT validate the syncer's outputs proactively at startup; validation happens lazily at `Registry::load_from(...)` time and surfaces as `QuireError::ArchetypeLoadError` per archetype.

### Naming

- Module directories SHALL contain a `manifest.yaml` at the module root.
- The manifest's `name` field, if declared, SHALL be globally unique across all modules a given `Registry` will load (per [FR-014](./functional/FR-014-module-activation.md)).
- Archetype names within a module SHALL be unique within that module's manifest.

### Versioning

- Module-level `version` field in `manifest.yaml` is informational at v1 (per [FR-014](./functional/FR-014-module-activation.md)). The syncer MAY use semver to gate which version of an archetype set is synced; `quire-rs` does not enforce.

### Tool ownership

| Concern | Owner | Notes |
|---|---|---|
| Authenticate to Filament | `ix-cli` | API keys, OAuth, etc. |
| Discover available modules in Filament | `ix-cli` | |
| Download module contents to disk | `ix-cli` | Atomic writes per above |
| Resolve `~/.ix/filament/modules/` location | `ix-cli` / `quoin` for writes; `quire-rs` for reads ([FR-013](./functional/FR-013-archetype-loader.md)) | Both honor `IX_FILAMENT_MODULES_PATH` (and the legacy `IX_SCHEMA_PATH` alias) |
| Module versioning policy | `ix-cli` | quire-rs is version-blind |
| Conflict resolution between local edits and remote | `ix-cli` | quire-rs sees only the post-resolution state |
| Load archetypes into runtime registry | `quire-rs` | Pure filesystem reads |
| Validate / compile schemas + templates | `quire-rs` | At load time |
| Render / parse / extract / harvest edges | `quire-rs` | At call time |

If the syncer violates this contract (non-atomic writes, malformed YAML, etc.), the failure mode is bounded: per-archetype `ArchetypeLoadError`. The Registry construction itself does not abort; non-affected archetypes load normally. The consumer can inspect the diagnostic list to decide whether to proceed.

---

## 18. References

- ISO/IEC/IEEE 29148 — Requirements Engineering
- IEEE 828 — Configuration Management
- `agent-ix/quire` — TypeScript reference parser (Layer 1+2)
- `agent-ix/spec-artifacts-iso` — reference Jinja2 renderer for 8 ISO archetypes
- `agent-ix/spec-artifacts-app` — reference Jinja2 renderer for 2 App archetypes
- `agent-ix/ecaz` — source of Rust safety scaffolding (backported via `agent-ix/rust-lib-cookiecutter`)
- `INPUT.md` — design input document combining block-rendering architecture with the Quire port mandate
- `ix-cli` (agent-ix/ix-cli) — canonical syncer between Filament and the local filesystem; counterparty to the inter-tool contract in §17

---

## 19. Hardening Posture

`quire-rs` inherits Rust safety scaffolding from `rust-lib-cookiecutter` ([StR-004](./stakeholder/StR-004-safety-scaffolding-inheritance.md), itself backported from `agent-ix/ecaz`). This section records which ECAZ-grade hardening tools `quire-rs` adopts and which it skips, with rationale. Decisions are pinned to v1; tools marked "skip" may be revisited in v1.1+.

### Adopted (specified by NFRs)

| Tool | Purpose | Specified in |
|---|---|---|
| `cargo fmt --check` | Formatting drift | [StR-004](./stakeholder/StR-004-safety-scaffolding-inheritance.md) (inherited) |
| `cargo clippy -- -D warnings` | Lint discipline | [StR-004](./stakeholder/StR-004-safety-scaffolding-inheritance.md) (inherited) |
| `#![forbid(unsafe_code)]` (default build) | Zero first-party `unsafe` — compile-time impossible (scoped off for `python`) | [NFR-003](./non-functional/NFR-003-zero-unsafe.md) |
| `// SAFETY:` comment enforcement | Unsafe-comment baseline (covers the `python` build where forbid is off) | [NFR-003](./non-functional/NFR-003-zero-unsafe.md) |
| Zero `unsafe` blocks | Memory-safety surface = empty | [NFR-003](./non-functional/NFR-003-zero-unsafe.md) |
| `cargo deny check licenses` | License hygiene | [NFR-004](./non-functional/NFR-004-license-hygiene.md) |
| `proptest` (determinism + roundtrip) | Property testing | [NFR-006](./non-functional/NFR-006-determinism.md) |
| Dependency version pinning | Load-bearing crates pinned | [NFR-009](./non-functional/NFR-009-dependency-pinning.md) |
| Public API stability (semver) | Consumer contract | [NFR-010](./non-functional/NFR-010-api-stability.md) |
| **`cargo-fuzz` on untrusted-input surfaces** | Coverage-guided fuzzing (incl. v0.3 `load_repo` + resolution) | [NFR-011](./non-functional/NFR-011-fuzz-testing.md) |
| ~~`cargo miri test --lib`~~ **RETIRED** (ADR 0006) | Removed — zero first-party `unsafe` (now compile-enforced by `forbid`); dependency UB → cargo-audit; Miri false-positived on rayon | [NFR-012](./non-functional/NFR-012-miri-ub-check.md) (retired) |
| **`cargo-mutants` on high-value paths** | Test-quality validation | [NFR-013](./non-functional/NFR-013-mutation-testing.md) |
| **`cargo-audit` daily + on PR** | RustSec advisory check | [NFR-014](./non-functional/NFR-014-advisory-checking.md) |
| **`loom` on the parallel-walk path** *(v0.3)* | Exhaustive interleaving for the rayon fan-out ([FR-024](./functional/FR-024-parallel-repo-walk.md)) | [NFR-017](./non-functional/NFR-017-concurrency-permutation.md) |
| **`TSAN` on the Python extension** *(v0.3)* | Data races in the GIL-release window ([FR-023](./functional/FR-023-python-binding-surface.md)) | [NFR-018](./non-functional/NFR-018-ffi-sanitizer-lanes.md) |
| **`ASAN` on the Python extension** *(v0.3)* | Memory errors in FFI object handoff ([FR-023](./functional/FR-023-python-binding-surface.md)) | [NFR-018](./non-functional/NFR-018-ffi-sanitizer-lanes.md) |

### v0.3 re-review (corpus + bindings)

The v0.3 surface — rayon data-parallelism ([FR-024](./functional/FR-024-parallel-repo-walk.md)), a CPython C-ABI boundary ([FR-023](./functional/FR-023-python-binding-surface.md)), and untrusted on-disk trees — invalidated two v1 skip rationales. **loom** (was skipped: "no synchronization primitives") and the **address/thread sanitizers** (were skipped: "marginal for safe Rust above the unsafe-comment audit") are now adopted, scoped to the new surfaces. The remaining skips below were re-examined against v0.3 and still hold, with refreshed rationale.

### Skipped (with rationale)

| Tool | Skipped because |
|---|---|
| **kani** (model checker) | Best for algorithm kernels with complex invariants and bounded state. `quire-rs` parser is a linear walk; slug normalization is straightforward. v0.3 reference resolution ([FR-026](./functional/FR-026-intra-spec-reference-resolution.md)) is a hash-join with no complex invariant. `proptest` already covers the relevant invariants at lower operational cost. |
| **shuttle** (randomized scheduler) | `loom` ([NFR-017](./non-functional/NFR-017-concurrency-permutation.md)) adopted instead for the one concurrent path (the [FR-024](./functional/FR-024-parallel-repo-walk.md) data-parallel collect). At v0.3's concurrency size, loom's exhaustive small-scope checking is sufficient and stronger than shuttle's randomized scheduling. Reconsider shuttle only if a future version adds shared-mutable concurrency (a cache/pool). |
| **cargo-careful** (std-with-debug-asserts) | Marginal for a `forbid(unsafe_code)` pure-Rust core (no first-party `unsafe`); dependency advisories are covered by cargo-audit. (The FFI boundary cargo-careful cannot reach is covered by [NFR-018](./non-functional/NFR-018-ffi-sanitizer-lanes.md) sanitizers.) |
| **cargo-vet** (supply-chain attestation) | High org-wide operational lift (audits, vetted versions, maintained `audits.toml`). v0.3 expands the dependency surface (`ignore`, `rayon`, `sha2`, `uuid`; `pyo3`/`maturin` feature-gated), which raises the *priority* for org-level adoption — but it remains better adopted org-wide than crate-by-crate. `cargo-audit` ([NFR-014](./non-functional/NFR-014-advisory-checking.md)) covers advisories meanwhile. Defer to ix-org policy. |
| **Big-endian qemu tests** | `quire-rs` is text-in / text-out. v0.3 id derivation (SHA-256 → UUID5) is byte-deterministic regardless of host endianness; no endian-sensitive binary serialization. |
| **SIMD differential tests** | `quire-rs` does not use SIMD. rayon data-parallelism is task-level, not SIMD. |
| **pgrx multi-version test lanes** | N/A — `quire-rs` is not a PostgreSQL extension. |

**Moved to Adopted in v0.3:** `loom` (now [NFR-017](./non-functional/NFR-017-concurrency-permutation.md)), `-Z sanitizer=thread` + `-Z sanitizer=address` (now [NFR-018](./non-functional/NFR-018-ffi-sanitizer-lanes.md)). See the Adopted table above.

### Implementation notes

- Fuzz / mutants / **loom ([NFR-017](./non-functional/NFR-017-concurrency-permutation.md))** / **TSAN + ASAN ([NFR-018](./non-functional/NFR-018-ffi-sanitizer-lanes.md))** run on weekly schedule + workflow_dispatch + tag push — NOT per-PR. Per-PR jobs are the cookiecutter floor (fmt/clippy/test/deny/audit-unsafe/audit-static) plus `cargo-audit`. Heavy hardening lanes are scheduled to keep PR latency low.
- `make ci` runs the per-PR set locally. `make hardening` runs the scheduled set locally for pre-tag verification (now incl. `make loom` + `make sanitize`).
- **Miri retired (ADR 0006):** first-party `unsafe` is compile-impossible via `#![forbid(unsafe_code)]` ([NFR-003-AC-5](./non-functional/NFR-003-zero-unsafe.md)), so there is no first-party UB surface; dependency advisories are covered by `cargo-audit` ([NFR-014](./non-functional/NFR-014-advisory-checking.md)). The `python`-feature binding layer is covered by the pytest harness ([FR-023](./functional/FR-023-python-binding-surface.md)) + the sanitizer lanes ([NFR-018](./non-functional/NFR-018-ffi-sanitizer-lanes.md)).
- Discovered crashes / UB / advisory hits are P0 — fix or contain before next release.

---

## 20. Glossary

Canonical definitions for terms used throughout the spec. When in doubt, this section governs.

| Term | Definition |
|---|---|
| **Archetype** | The named pairing of a JSON Schema document and a MiniJinja template that together describe one renderable or extractable kind (e.g. `fr`, `nfr`, `domain`, `entity`). Authoring artifacts that live as files on disk; loaded into a `Registry` at runtime. NOT a Rust type. |
| **Artifact archetype** | An archetype whose pairing is `(JSON Schema + MiniJinja template)` — renderable. Example: `fr`, `adr`. Owned by `spec-artifacts-*` modules. |
| **Object archetype** | An archetype whose definition includes a `body_extraction` DSL — extractable rather than renderable. Example: `domain`, `entity`. Owned by `spec-objects-*` modules. |
| **CompiledArchetype** | The runtime representation of a single archetype after loading: a compiled JSON Schema validator + a pre-parsed MiniJinja template + manifest metadata. `Send + Sync`. Held inside a `Registry`. |
| **Module** | A directory containing `manifest.yaml` + `schemas/` + `templates/` (and/or `object_types/` for object archetypes). Identified by its manifest's `name:` field (or parent dir if unset). Multiple modules coexist in one `Registry`. |
| **Registry** | The runtime container holding all `CompiledArchetype` instances loaded from one or more search paths. `Send + Sync`. Immutable after construction; reload = construct a new `Registry`. |
| **Locator** | A DSL primitive that describes how to find a value in a parsed document. One of: `frontmatter_field`, `section_body`, `code_block`, `table_row`, `list_item`, `heading`. May be wrapped in a `Fallback(Vec<Primitive>)` chain ([FR-016](./functional/FR-016-secondary-locators.md)). |
| **Yield pattern** | A DSL construct under `body_extraction.yield_pattern` that determines whether extraction emits one record per document (`match`) or one record per iteration unit (`iterate_over` + `per_match`). |
| **Diagnostic** | A non-error informational message emitted by the engine (e.g. `DuplicateArchetype`, `FallbackLocatorUsed`). Surfaced in result types alongside the primary value; non-fatal. |
| **QuireError** | The crate's typed error enum returned in `Result<_, QuireError>` for all fallible operations. Variants are non-exhaustive. Each variant carries enough context (field path, file path, archetype name) for actionable handling. |
| **QuireDocument** / **QuireSection** | The parsed representation of a markdown document: frontmatter + preamble + nested section tree. Mirrors the TS reference `agent-ix/quire` shape verbatim. |
| **Search path** | The ordered list of filesystem directories the loader walks to discover modules. Resolved from explicit constructor arg → `IX_FILAMENT_MODULES_PATH` (then legacy `IX_SCHEMA_PATH`) env var → `~/.ix/filament/modules/` default. |
| **Filament** | The knowledge platform that authors and stores archetypes as data. Out-of-process from `quire-rs`. Sync to disk is owned by `ix-cli`. |
| **ix-cli** | The CLI tool that authenticates to Filament and syncs archetypes to the local filesystem. Counterparty to the inter-tool contract in §17. Not invoked by `quire-rs`. |
| **Parity / byte-parity** | The property that `quire-rs::render(archetype, data)` produces byte-identical output to the Python Jinja2 reference renderer (spec-artifacts-*) given the same input. Verified by `tests/render_parity/`. |
| **Baseline corpus** | The set of archetype modules present at `~/.ix/filament/modules/` (or under the v1 informational list in §8.3) used as the parity-test ground truth. Data, not code. |
