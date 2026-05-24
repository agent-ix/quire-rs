---
artifact_type: master-requirements
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
  - target: "ix://agent-ix/quire-py"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/spec-artifacts-iso"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/spec-artifacts-app"
    type: "consumes"
    cardinality: "1:1"
  - target: "ix://agent-ix/filament-parser-lib"
    type: "supersedes"
    cardinality: "1:1"

standards_alignment:
  - iso-iec-ieee-29148
  - ieee-828
---
# Master Requirements Specification
## quire-rs — Rust Templating + Parsing Engine for the Filament/Quire Ecosystem

---

## 1. Purpose

This document defines the **scope, intent, and governing requirements framework** for `quire-rs`, a Rust library crate that unifies two responsibilities in one engine:

1. **Schema-validated archetype rendering** — generate canonical markdown artifacts from typed data using MiniJinja templates.
2. **Markdown parsing** — port the existing `agent-ix/quire` (TypeScript) parser into pure Rust at byte-parity with the TS/Python references.

It establishes:
- The problem space `quire-rs` addresses across rendering and parsing
- The boundaries of responsibility between layers (Edit API, Schema, Storage, Render, Parse, Query, Writeback)
- The authoritative structure for requirements, verification, and change control
- The relationship between user intent (typed edits, LLM-driven changes), system behavior (validation + render + parse + writeback), and test evidence (byte-parity with reference implementations)

**Core invariant**: **markdown is canonical**. The on-disk `.md` is the source of truth. Blocks are *parsed from* markdown. Edits update one block's data → re-render that block via its template → splice new bytes back into the `.md` via writeback. Frontmatter and untouched blocks stay byte-identical.

This document is the **top-level requirements artifact** for the repository.

---

## 2. Scope

### 2.1 In Scope

This specification governs a **generic, archetype-agnostic engine** that processes data archetypes loaded from the local filesystem. The engine itself knows nothing about specific archetypes (`FR`, `NFR`, `ADR`, etc.) — those are data shipped by Filament (or any other authoring source) and synced to disk by ix-cli (or any other tool).

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

**Cross-cutting:**
- Safety scaffolding inherited from `agent-ix/rust-lib-cookiecutter` (clippy MSRV, deny.toml, `// SAFETY:` enforcement)
- Hardening hygiene: fuzz (cargo-fuzz), miri UB check, mutation testing (cargo-mutants), advisory checking (cargo-audit) — all required, not opt-in
- Public Rust API stable across parse, render, edit, writeback, and extract surfaces
- Engine is **offline by default** — works against the local filesystem with zero network dependencies

### 2.2 Out of Scope

This specification does not govern:

- **Sync from Filament to disk.** The local schema directory (e.g. `~/.ix/schemas/`) is populated by external tools — `ix-cli` is the canonical syncer (handles Filament auth + transfer). `quire-rs` only reads from disk; it never calls Filament directly.
- **Authoring tooling.** Schema files, templates, and manifests are authored elsewhere (in Filament, by hand, by another tool). `quire-rs` does not write archetype data.
- **Author-time schema validation.** `quire-rs` validates JSON Schema documents at archetype-load time (FR-013). Pre-publish validation (catching authoring errors before they reach disk) is Filament's concern.
- **Hot reload on filesystem change.** `quire-rs` does NOT watch the filesystem and does NOT automatically reload archetypes when files change on disk. Consumers refresh archetypes by calling `Registry::load_from(...)` again. The previous Registry stays alive for any outstanding references and is dropped when they release. There is no in-place update or change-event subscription.
- **Schema migration when archetypes evolve.** When an archetype's schema changes (e.g. Filament publishes a new version of `fr-frontmatter.schema.json`), `quire-rs` validates incoming data against the loaded version only. Migrating existing artifacts written against an older schema is Filament's responsibility (or downstream migration tooling).
- **LLM model-specific tool-call adapters.** `quire-rs::schema_for` returns a JSON Schema. Wrapping it into a model-specific tool-call envelope (OpenAI function-calling shape, Anthropic tool-use shape, etc.) is the consumer's concern.
- **ID generation.** `quire-rs` validates that artifact IDs match the schema's `pattern` (e.g. `^[A-Z]{2,4}-[0-9]+$`). Authoring tools (Filament UI, scripts) generate the IDs.
- **Internationalized slug normalization.** FR-009 implements ASCII-only slug normalization to match the TS/Py reference. Non-ASCII heading authoring works (the section parses correctly), but the slug collapses non-ASCII characters to `-`. Full Unicode slug support is deferred to a future version.
- **Windows path semantics.** `v1` supports macOS and Linux only. Filesystem-loader behavior on Windows (drive letters, `\` separator, symlink permissions) is undefined.
- **React UI bindings.** `agent-ix/quire` ships React components for browser-side rendering; those are TypeScript-only. `quire-rs` does not provide a UI layer.
- **Cross-document graph queries.** `agent-ix/quire` ships a React provider that indexes multiple parsed documents and exposes hooks for cross-doc queries. Out of scope for the Rust crate — the parser primitives that would underpin it (block-stable IDs, edge data) exist, but no in-process graph or query layer ships with v1.
- CRDT or OT live-editing semantics.
- Schema-driven template generation (schemas validate; templates present; neither generates the other).
- Heavy hardening tooling (kani formal verification, loom / shuttle concurrency permutation, sim-spire) — opt-in via future cookiecutter variant. (Standard Rust safety hygiene — fuzz, miri, mutants, advisory — is required and in scope.)
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
| FR-014 expansion (`load_strict`, `archetype_in_module`, `module_version`) | Trimmed to minimal "multiple modules coexist; collisions diagnosed" |

#### Stays unchanged

- Parser FRs (FR-005..009): pure port of `agent-ix/quire` core
- FR-010 Query API: pure port of `agent-ix/quire` query
- FR-003 schema_for, FR-004 strict MiniJinja env
- FR-011/016 body-extraction DSL (reworked around block-level scope)
- FR-013 archetype loader (reworked: `blocks/<type>/...` layout)
- NFR-001..007, NFR-009..010, NFR-011..014 (perf, safety, error shape, determinism, dep pinning, API stability, fuzz, miri, mutants, audit)
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
| Body-extraction DSL evaluator | Lives in Python `filament-parser-lib`. |

#### Behavioural differences kept

- `parse_table` returns `Option::None` on miss; TS returns `{headers: [], rows: []}`. Intentional Rust-idiomatic divergence.
- Heading matching: case-insensitive + section-number normalization. Matches TS exactly.
- Section content slicing: byte-exact in `quire-rs` (FR-008); TS applies `.strip()`. Required for writeback fidelity.

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
┌──────────────┐
│ ~/.ix/schemas│   filesystem contract — manifest.yaml + schemas/ + templates/
│  (or IX_SCHEMA_PATH dirs)         per module                                   │
└──────┬───────┘
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
- **`spec-artifacts-*` Python repos** — eventually call `quire-rs` via Python bindings or subprocess for parity-rendered artifacts
- **`ix-spec-objects` extractors** — evaluate `body_extraction` DSL via the parser's Query API
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
| Stakeholder Requirement | `StR-XXX` | `StR-001` |
| User Story | `US-XXX` | `US-002` |
| Functional Requirement | `FR-XXX` | `FR-014` |
| Non-Functional Requirement | `NFR-XXX` | `NFR-003` |
| Acceptance Criteria | `{FR}-AC-N` | `FR-014-AC-1` |
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

The Registry is populated by FR-013 (filesystem loader) and FR-014 (multi-module activation). No archetype names are hard-coded in Rust source.

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

This list is **informational, not normative.** The Registry contents are whatever the filesystem holds at load time. Adding a new archetype is a sync operation (new files in `~/.ix/schemas/`), not a code change.

The parity suite (FR-012) enumerates archetypes from `tests/render_parity/corpus.yaml` and runs against every fixture pair on disk — that file is the byte-parity source of truth, not this table.

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

The parser SHALL match `agent-ix/quire` and `agent-ix/quire-py` acceptance fixtures exactly:

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

`quire-rs` SHALL evaluate the YAML-encoded body extraction DSL used by `spec-objects-architecture` and `ix-spec-objects`. Extractors keyed by:

- `frontmatter_field` — value at a JSONPath in the parsed frontmatter
- `section_body` — text of the section under a given heading (uses Quire `section()` internally)
- `code_block` — fenced code block of a given language, optionally constrained to a section

The DSL itself remains in YAML in the source repos; `quire-rs` supplies the evaluator. Output is a typed map keyed by the DSL's `match` keys.

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

- `manifest.yaml` SHALL be valid YAML, conformant with the structural shape declared in FR-013 (artifact_types and/or object_types arrays, each entry referencing `schema_ref` and `template_ref` by relative path).
- Each `schema_ref` target SHALL exist on disk and be valid JSON Schema (draft 2020-12, no cross-file `$ref` — see FR-002).
- Each `template_ref` target SHALL exist on disk and be valid MiniJinja (no `{% include %}` at v1 — see FR-004).
- **Filament SHOULD pre-validate** JSON Schema documents and MiniJinja templates before publishing — catching authoring errors at the authoring layer is preferable to surfacing them as `quire-rs` load errors. This is a SHOULD, not a SHALL: `quire-rs` does NOT depend on Filament's pre-validation for correctness; it always validates at load time.
- `quire-rs` does NOT validate the syncer's outputs proactively at startup; validation happens lazily at `Registry::load_from(...)` time and surfaces as `QuireError::ArchetypeLoadError` per archetype.

### Naming

- Module directories SHALL contain a `manifest.yaml` at the module root.
- The manifest's `name` field, if declared, SHALL be globally unique across all modules a given `Registry` will load (per FR-014).
- Archetype names within a module SHALL be unique within that module's manifest.

### Versioning

- Module-level `version` field in `manifest.yaml` is informational at v1 (per FR-014). The syncer MAY use semver to gate which version of an archetype set is synced; `quire-rs` does not enforce.

### Tool ownership

| Concern | Owner | Notes |
|---|---|---|
| Authenticate to Filament | `ix-cli` | API keys, OAuth, etc. |
| Discover available modules in Filament | `ix-cli` | |
| Download module contents to disk | `ix-cli` | Atomic writes per above |
| Resolve `~/.ix/schemas/` location | `ix-cli` for writes; `quire-rs` for reads (FR-013) | Both honor `IX_SCHEMA_PATH` env var |
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
- `agent-ix/quire-py` — Python port of `agent-ix/quire`
- `agent-ix/spec-artifacts-iso` — reference Jinja2 renderer for 8 ISO archetypes
- `agent-ix/spec-artifacts-app` — reference Jinja2 renderer for 2 App archetypes
- `agent-ix/ecaz` — source of Rust safety scaffolding (backported via `agent-ix/rust-lib-cookiecutter`)
- `INPUT.md` — design input document combining block-rendering architecture with the Quire port mandate
- `ix-cli` (agent-ix/ix-cli) — canonical syncer between Filament and the local filesystem; counterparty to the inter-tool contract in §17

---

## 19. Hardening Posture

`quire-rs` inherits Rust safety scaffolding from `rust-lib-cookiecutter` (StR-004, itself backported from `agent-ix/ecaz`). This section records which ECAZ-grade hardening tools `quire-rs` adopts and which it skips, with rationale. Decisions are pinned to v1; tools marked "skip" may be revisited in v1.1+.

### Adopted (specified by NFRs)

| Tool | Purpose | Specified in |
|---|---|---|
| `cargo fmt --check` | Formatting drift | StR-004 (inherited) |
| `cargo clippy -- -D warnings` | Lint discipline | StR-004 (inherited) |
| `// SAFETY:` comment enforcement | Unsafe-comment baseline | NFR-003 |
| Zero `unsafe` blocks | Memory-safety surface = empty | NFR-003 |
| `cargo deny check licenses` | License hygiene | NFR-004 |
| `proptest` (determinism + roundtrip) | Property testing | NFR-006 |
| Dependency version pinning | Load-bearing crates pinned | NFR-009 |
| Public API stability (semver) | Consumer contract | NFR-010 |
| **`cargo-fuzz` on untrusted-input surfaces** | Coverage-guided fuzzing | NFR-011 |
| **`cargo miri test --lib`** | UB detection (incl. in deps) | NFR-012 |
| **`cargo-mutants` on high-value paths** | Test-quality validation | NFR-013 |
| **`cargo-audit` daily + on PR** | RustSec advisory check | NFR-014 |

### Skipped (with rationale)

| Tool | Skipped because |
|---|---|
| **kani** (model checker) | Best for algorithm kernels with complex invariants and bounded state. `quire-rs` parser is a linear walk; slug normalization is straightforward. `proptest` already covers the relevant invariants at lower operational cost. |
| **loom** (thread-schedule permutation) | Finds races in code that uses synchronization primitives (`Mutex`, atomics, etc.). `quire-rs` has none — `Registry` is immutable after construction; only `Arc` clones share state. Loom has nothing to permute. NFR-006 cross-thread proptest covers the relevant claim. |
| **shuttle** (randomized scheduler) | Same as loom. |
| **cargo-careful** (std-with-debug-asserts) | Belt-and-suspenders with miri; redundant signal. |
| **cargo-vet** (supply-chain attestation) | High org-wide operational lift (audits, vetted versions, maintained `audits.toml`). Better adopted org-wide than crate-by-crate. Defer to ix-org policy. |
| **Big-endian qemu tests** | `quire-rs` is text-in / text-out; no binary serialization with endian sensitivity. |
| **SIMD differential tests** | `quire-rs` does not use SIMD. |
| **`-Z sanitizer=address\|thread`** | Marginal value for safe Rust above what miri provides. |
| **pgrx multi-version test lanes** | N/A — `quire-rs` is not a PostgreSQL extension. |

### Implementation notes

- Fuzz / miri / mutants run on weekly schedule + workflow_dispatch + tag push — NOT per-PR. Per-PR jobs are the cookiecutter floor (fmt/clippy/test/deny/audit-unsafe/audit-static) plus `cargo-audit`. Heavy hardening lanes are scheduled to keep PR latency low.
- `make ci` runs the per-PR set locally. `make hardening` runs the scheduled set locally for pre-tag verification.
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
| **Locator** | A DSL primitive that describes how to find a value in a parsed document. One of: `frontmatter_field`, `section_body`, `code_block`, `table_row`, `list_item`, `heading`. May be wrapped in a `Fallback(Vec<Primitive>)` chain (FR-016). |
| **Yield pattern** | A DSL construct under `body_extraction.yield_pattern` that determines whether extraction emits one record per document (`match`) or one record per iteration unit (`iterate_over` + `per_match`). |
| **Diagnostic** | A non-error informational message emitted by the engine (e.g. `DuplicateArchetype`, `FallbackLocatorUsed`). Surfaced in result types alongside the primary value; non-fatal. |
| **QuireError** | The crate's typed error enum returned in `Result<_, QuireError>` for all fallible operations. Variants are non-exhaustive. Each variant carries enough context (field path, file path, archetype name) for actionable handling. |
| **QuireDocument** / **QuireSection** | The parsed representation of a markdown document: frontmatter + preamble + nested section tree. Mirrors the TS reference `agent-ix/quire` shape verbatim. |
| **Search path** | The ordered list of filesystem directories the loader walks to discover modules. Resolved from explicit constructor arg → `IX_SCHEMA_PATH` env var → `~/.ix/schemas/` default. |
| **Filament** | The knowledge platform that authors and stores archetypes as data. Out-of-process from `quire-rs`. Sync to disk is owned by `ix-cli`. |
| **ix-cli** | The CLI tool that authenticates to Filament and syncs archetypes to the local filesystem. Counterparty to the inter-tool contract in §17. Not invoked by `quire-rs`. |
| **Parity / byte-parity** | The property that `quire-rs::render(archetype, data)` produces byte-identical output to the Python Jinja2 reference renderer (spec-artifacts-*) given the same input. Verified by `tests/render_parity/`. |
| **Baseline corpus** | The set of archetype modules present at `~/.ix/schemas/` (or under the v1 informational list in §8.3) used as the parity-test ground truth. Data, not code. |
