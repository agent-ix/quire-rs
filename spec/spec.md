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

1. **Schema-validated block rendering** — generate canonical markdown artifacts from typed data using MiniJinja templates.
2. **Markdown parsing** — port the existing `agent-ix/quire` (TypeScript) parser into pure Rust at byte-parity with the TS/Python references.

It establishes:
- The problem space `quire-rs` addresses across rendering and parsing
- The boundaries of responsibility between layers (Edit API, Schema, Storage, Render, Post-process, Parse, Query)
- The authoritative structure for requirements, verification, and change control
- The relationship between user intent (typed edits, LLM-driven changes), system behavior (validation + render + parse), and test evidence (byte-parity with reference implementations)

This document is the **top-level requirements artifact** for the repository.

---

## 2. Scope

### 2.1 In Scope

This specification governs a **generic, archetype-agnostic engine** that processes data archetypes loaded from the local filesystem. The engine itself knows nothing about specific archetypes (`FR`, `NFR`, `ADR`, etc.) — those are data shipped by Filament (or any other authoring source) and synced to disk by ix-cli (or any other tool).

**Render side:**
- Load archetype modules from the local filesystem at well-known search paths (FR-013)
- Compile each archetype's JSON Schema and MiniJinja template once at load time
- Validate input `serde_json::Value` data against the compiled schema
- Render via MiniJinja with `UndefinedBehavior::Strict`
- Byte-for-byte parity with the existing Python Jinja2 reference renderer across whatever archetype corpus is loaded (currently 17 archetypes: 8 ISO + 2 App + 7 Process)
- Markdown is the canonical output format

**Parse side:**
- Port of `agent-ix/quire` Layer 1 (markdown → `QuireDocument` heading tree)
- Port of `agent-ix/quire` Layer 2 (Query API: `section`, `tables`, `lists`, `diagrams`, `search`)
- YAML frontmatter extraction with malformed-fallback semantics
- Fenced-code-block-aware heading parsing
- Byte-exact content slicing (no re-serialization)
- Stable `<slug>-L<line>` ID generation

**Object-extraction side:**
- Body-extraction DSL evaluator supporting all six Locator primitives in current `filament-parser-lib` use: `frontmatter_field`, `section_body`, `code_block`, `table_row`, `list_item`, `heading`
- Single-yield (`match`) and multi-yield (`iterate_over` + `per_match`) DSL patterns
- Declarative `emit_edges` evaluation
- Relationship harvesting from frontmatter sugar fields + structured `relationships:` block with `(source, type, target)` deduplication
- Secondary / fallback locators per field for author-variant tolerance

**Cross-cutting:**
- Safety scaffolding inherited from `agent-ix/rust-lib-cookiecutter` (clippy MSRV, deny.toml, `// SAFETY:` enforcement)
- Public Rust API stable across render, parse, and extract surfaces
- Engine is **offline by default** — works against the local filesystem with zero network dependencies

### 2.2 Out of Scope

This specification does not govern:

- **Sync from Filament to disk.** The local schema directory (e.g. `~/.ix/schemas/`) is populated by external tools — `ix-cli` is the canonical syncer (handles Filament auth + transfer). `quire-rs` only reads from disk; it never calls Filament directly.
- **Authoring tooling.** Schema files, templates, and manifests are authored elsewhere (in Filament, by hand, by another tool). `quire-rs` does not write archetype data.
- Quire Layer 3 (React component bindings) — TypeScript-only.
- Quire Layer 4 (cross-document graph queries) — separate concern.
- HTML output via `comrak` — markdown is the canonical output; HTML post-processing is a future variant.
- CRDT or OT live-editing semantics.
- Schema-driven template generation (schemas validate; templates present; neither generates the other).
- Hardening suites (kani / loom / shuttle / sim-spire) — opt-in via future cookiecutter variant.
- Real-time multi-user editing.
- Generating Rust types from JSON Schemas. Downstream Rust consumers that want typed bindings author them by hand or use `schemars` themselves — `quire-rs` does not derive types.

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
│  Schema layer       (typed structs + validators)│  ← correctness
├─────────────────────────────────────────────────┤
│  Storage            (blocks as canonical data)  │  ← persistence
├─────────────────────────────────────────────────┤
│  Render layer       (MiniJinja per-block tpls)  │  ← presentation
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
- **LLM agents** — receive `schemars`-generated JSON Schemas as tool definitions, emit validated patches that the schema layer accepts and the renderer formats

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
- **Invalid block type** — explicit error when the type discriminator does not match a registered archetype

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

## 17. References

- ISO/IEC/IEEE 29148 — Requirements Engineering
- IEEE 828 — Configuration Management
- `agent-ix/quire` — TypeScript reference parser (Layer 1+2)
- `agent-ix/quire-py` — Python port of `agent-ix/quire`
- `agent-ix/spec-artifacts-iso` — reference Jinja2 renderer for 8 ISO archetypes
- `agent-ix/spec-artifacts-app` — reference Jinja2 renderer for 2 App archetypes
- `agent-ix/ecaz` — source of Rust safety scaffolding (backported via `agent-ix/rust-lib-cookiecutter`)
- `INPUT.md` — design input document combining block-rendering architecture with the Quire port mandate
