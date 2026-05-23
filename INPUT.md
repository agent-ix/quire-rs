# Block-Based Artifact Architecture

## Overview

This document describes the architecture for an artifact system that supports efficient runtime edits and structured rendering. Artifacts are composed of typed, addressable **blocks**. Edits target individual blocks, validation is schema-driven, and rendering is handled by MiniJinja templates with markdown post-processing.

The design separates four concerns that are often conflated: transport (how edits arrive), correctness (what valid data looks like), persistence (what we store), and presentation (how it renders).

## Layered Architecture

```
┌─────────────────────────────────────────────────┐
│  Edit API           (patches, full replaces)    │  ← transport
├─────────────────────────────────────────────────┤
│  Schema layer       (typed structs + validators)│  ← correctness
├─────────────────────────────────────────────────┤
│  Storage            (blocks as canonical data)  │  ← persistence
├─────────────────────────────────────────────────┤
│  Render layer       (MiniJinja per-block tpls)  │  ← presentation
├─────────────────────────────────────────────────┤
│  Post-process       (markdown → HTML via comrak)│  ← output format
└─────────────────────────────────────────────────┘
```

Each layer has one job and a narrow interface to the next. Layers do not reach across boundaries.

| Layer | Responsibility | Knows nothing about |
|---|---|---|
| Edit API | Accept patches and IDs | Templates, validation internals |
| Schema | Turn untrusted input into typed, valid data | Rendering, storage format |
| Storage | Hold blocks as canonical artifact | How blocks render or validate |
| Render | Produce markdown from valid data | Validation, HTML, storage |
| Post-process | Markdown → final output format | Blocks, templates |

## Core Concepts

### Blocks

An artifact is an ordered list of blocks. Each block is:

```
Block {
  id: String,         // stable identifier
  type: String,       // discriminator (e.g. "callout", "heading")
  schema_version: u32 // for migrations
  data: { ... }       // shape determined by type
}
```

The block list is the **canonical artifact**. Rendered output is a derived cache, never the source of truth.

### The Schema/Template Pair

Each block type has two parallel definitions:

- A **schema**: a Rust struct (with serde + validators) defining valid data shape
- A **template**: a MiniJinja file defining how that data renders

They share a name and field references, but neither generates the other. The schema handles correctness; the template handles presentation. They meet at exactly one point: the validated data handoff.

```
blocks/
  callout/
    schema.rs        // CalloutData struct + validators
    callout.md.j2    // markdown render
  heading/
    schema.rs
    heading.md.j2
  ...
```

## The Three Critical Contracts

Three interfaces hold the system together. Get these right and the layers compose cleanly.

### 1. Patch → Typed Block Data (Edit API → Schema)

Edits arrive as patches. The schema layer merges the patch onto current data and validates the *complete merged result* — never the patch in isolation, because cross-field rules require the full picture.

```rust
#[derive(Serialize, Deserialize, JsonSchema, Validate)]
#[serde(tag = "type", rename_all = "snake_case")]
enum BlockData {
    Callout(CalloutData),
    Heading(HeadingData),
    // ...
}

#[derive(Serialize, Deserialize, JsonSchema, Validate)]
struct CalloutData {
    variant: CalloutVariant,
    #[garde(length(max = 120))]
    title: Option<String>,
    #[garde(length(max = 500))]
    body: String,
}
```

Validation failures return field-keyed structured errors (`data.body: "exceeds 500 chars"`) usable by both UIs and LLM editors for retry.

### 2. Typed Block Data → Template Context (Schema → Render)

The validated struct is handed directly to MiniJinja. The template trusts its input — no defensive guards beyond what the schema's optionality dictates.

```jinja
{# callout.md.j2 #}
> **{{ data.variant | upper }}**{% if data.title %}: {{ data.title }}{% endif %}
>
> {{ data.body }}
```

`UndefinedBehavior::Strict` on the environment catches any template field that the schema doesn't provide. This is the drift detector — schema and template stay in sync because the strict environment refuses to render mismatches silently.

### 3. Block Type → Template Path (Render Dispatch)

A top-level template dispatches by type using a path convention:

```jinja
{% for block in blocks %}
  {% include "blocks/" ~ block.type ~ "/" ~ block.type ~ ".md.j2" %}
{% endfor %}
```

One lookup, zero magic. Missing template = clear error.

## The Edit → Render Loop

```
1. Edit arrives:    { block_id, patch }
2. Load block:      { id, type, schema_version, data }
3. Migrate if schema_version is behind
4. Merge patch onto data
5. Deserialize merged data into typed struct for block.type
6. Run field-level validators
7. On failure: return structured field errors
8. On success: persist
9. Render:        select template by type, pass typed data to MiniJinja
10. Post-process: pipe markdown through comrak → HTML (if HTML is needed)
```

Steps 1–8 are the schema's territory. Steps 9–10 are presentation. They meet only at the handoff of validated data.

## Schema Responsibilities

The schema does more work than just type checking. In order of when it fires during an edit:

1. **Shapes the edit before it's made** — for UIs, generate forms from the schema; for LLMs, derive the tool definition via `schemars`. The model cannot propose `variant: "scary"` because the tool input is constrained.
2. **Validates incoming edits** — serde catches type errors and unknown fields; `garde`/`validator` handles field rules and cross-field invariants.
3. **Constrains partial edits correctly** — always validates the *merged result*, never the patch in isolation.
4. **Authorizes edits** — schema metadata marks which fields are user-editable vs system-managed vs immutable.
5. **Drives migrations** — `schema_version` on the block lets old data be migrated forward before validation.
6. **Types the render context** — the same struct that validation produces is what the template receives, so strict-undefined catches drift.

## What MiniJinja Is and Isn't Doing

MiniJinja is the engine of the render layer. Nothing more.

- It does **not** validate
- It does **not** authorize
- It does **not** store
- It does **not** decide which block to edit

It receives `(template_name, typed_data)` and returns a string. That narrowness is the point: fast, swappable, and isolated from the rest of the system.

### Render-Layer Configuration

Two settings are non-negotiable:

- `UndefinedBehavior::Strict` — missing fields are errors, not silent empties
- A long-lived `Environment` with templates pre-loaded, shared across requests — rebuilding the environment per request is the single biggest performance footgun

## What Goes Where

| Concern | Lives in |
|---|---|
| Field required/optional | Schema |
| Field types and value constraints | Schema |
| Cross-field invariants | Schema |
| Which fields are editable | Schema (metadata) |
| Schema version + migration | Schema layer |
| How a block visually looks | Template |
| Computed/derived display values | Template |
| Conditional formatting | Template |
| CSS classes, icons, colors per variant | Template |
| Output format (markdown, HTML, plain) | Template variant + post-process |
| Block ordering and document structure | Storage / top-level template |

The line to remember: **presentation choices never live in the schema**. If `variant: "warning"` should produce a yellow box with a ⚠ icon, that mapping is in the template — the schema just constrains `variant` to a known enum. This keeps data portable across renderers.

## Edit Strategies

The system supports two edit operations:

1. **Patch** — surgical change to specific fields of a block. Cheap, reviewable, ideal for localized edits.
2. **Full replace** — replace a block's `data` entirely. Simple, always correct, used for major rewrites.

Both go through the same validation pipeline. The editor (human or LLM) chooses based on the size of the change.

Real-time multi-user editing (CRDT/OT) is out of scope. The block-addressable model plays nicely with it if needed later, but the current design assumes serialized edits.

## LLM-Driven Edits

The schema is the LLM's contract. Workflow:

1. LLM sees: *"edit block `intro-001` (type: callout)"*
2. System exposes tool `edit_callout(block_id, patch)` with patch schema derived from `CalloutData` via `schemars`
3. LLM emits a structurally valid patch
4. Server merges, validates, persists, re-renders
5. On failure, field-level errors go back to the LLM, which retries

Most "the LLM produced garbage" failure modes disappear when the schema is the tool contract rather than an afterthought.

## Performance Notes

With MiniJinja + pulldown-cmark/comrak, rendering an entire artifact is in the microsecond-to-low-millisecond range for typical sizes. Don't over-engineer caching here:

- The block list is the source of truth; rendered output is a cache
- Re-render on change is fine for the vast majority of cases
- Cache renders only if you're serving them at high QPS, and even then per-block, not per-document
- The one optimization that matters: long-lived `Environment` with templates pre-loaded

## Adding a New Block Type

1. Create `blocks/{type}/schema.rs` with the typed struct and validators
2. Add the variant to the `BlockData` enum
3. Create `blocks/{type}/{type}.md.j2`
4. (Optional) Add additional render targets like `{type}.html.j2`

No other layer changes. This is the payoff of the layering.

## Tradeoffs and Non-Goals

**What this design pays for:**

- Upfront complexity: schemas + templates + dispatch + strict environment
- A learning curve for contributors who need to understand the schema/template split

**What it buys:**

- Safe, validated edits from any source (humans, LLMs, imports)
- Multiple output formats from one canonical representation
- New block types without touching existing code paths
- Clear errors at the right layer

**When it's overkill:** a system with one block type and three fields. A single template suffices. The layering earns its keep once block types multiply and edit sources diverge.

**Explicit non-goals:**

- Real-time collaborative editing (no CRDT/OT)
- Rendered output as source of truth (renders are always derived)
- Schema-driven rendering (templates are independent)
- Presentation logic in schemas (variants are enums; appearance is template-side)

## Dependencies

| Crate | Purpose |
|---|---|
| `minijinja` | Template rendering |
| `serde` | (De)serialization, type-level validation |
| `garde` or `validator` | Field-level and cross-field validators |
| `schemars` | JSON Schema generation for LLM tool definitions |
| `pulldown-cmark` or `comrak` | Markdown → HTML post-processing |

`comrak` if you want GitHub-flavored markdown (tables, task lists, footnotes); `pulldown-cmark` for pure-CommonMark and maximum speed.

---

# Quire Port Mandate (Parser Side)

The rendering architecture above is one half of `quire-rs`. The other half is a Rust port of the existing Quire markdown parser (`agent-ix/quire`, TypeScript) and its Python sibling (`agent-ix/quire-py`). Both halves ship as one crate so that consumers get parsing + rendering from a single dependency — the same engine that emits an artifact can also re-parse one.

## Parser scope (v1)

Port `quire@0.1` Layer 1 (pure parser) and Layer 2 (query API). Skip Layer 3 (React components) and Layer 4 (cross-document graph) — those are not Rust's lane.

## AST

Mirror the TypeScript types verbatim as Rust ADTs:

```rust
pub struct QuireSection {
    pub id: String,                // "<slug>-L<line>"
    pub heading: String,
    pub level: u8,                 // 1-6
    pub content: String,           // byte-exact slice, no re-serialization
    pub children: Vec<QuireSection>,
    pub start_line: usize,         // 0-based
    pub end_line: usize,
}

pub struct QuireDocument {
    pub preamble: Option<String>,
    pub sections: Vec<QuireSection>,
    pub raw: String,
    pub frontmatter: Option<serde_json::Map<String, serde_json::Value>>,
}
```

## Algorithm (must match TS/Py reference)

1. Extract YAML frontmatter (`---...---`); on parse failure, return `(None, entire input)` — frontmatter errors are NOT fatal.
2. Walk body line-by-line; toggle `in_fence` on each ``` boundary; collect heading positions while skipping fenced regions.
3. Preamble = text before first heading, or entire body if no headings.
4. Slice content between consecutive heading positions byte-exactly — no re-serialization, no whitespace normalization.
5. Build tree via level-aware stack: pop until parent.level < child.level, attach child to stack top.

## Query API (Layer 2 surface)

```rust
fn section(doc: &QuireDocument, heading: &str) -> Option<&QuireSection>;
fn sections(doc: &QuireDocument, level: Option<u8>) -> Vec<&QuireSection>;
fn parse_table(content: &str) -> TableResult;
fn parse_tables(content: &str) -> Vec<TableResult>;
fn table_from_section(doc: &QuireDocument, heading: &str) -> Option<TableResult>;
fn parse_bullet_list(content: &str, pattern: Option<ListPattern>) -> Vec<ListItem>;
fn extract_diagrams(doc: &QuireDocument, language: Option<&str>) -> Vec<DiagramBlock>;
fn search(doc: &QuireDocument, query: &str) -> Vec<SearchResult>;
```

## Edge cases to mirror (acceptance fixtures)

These are non-negotiable parity points with the TS/Py originals — any deviation is a bug:

- **Fenced-block heading awareness**: `#` lines inside ` ``` ` blocks are NOT headings.
- **Byte-exact slicing**: `section.content` is a slice of the original input; identical input produces identical bytes including trailing whitespace.
- **Slug ID format**: `<lowercase-slug>-L<0-based-line>`. E.g. `"2.1 In Scope"` at line 6 → `"2-1-in-scope-L6"`.
- **Malformed YAML fallback**: parse failure on frontmatter returns `frontmatter: None`, body = entire input. Not an error.
- **Unclosed fenced block**: trailing content is still part of the block.
- **Level-aware nesting**: `## A` followed by `#### B` (skipping `###`) — `B` becomes a child of `A`, not a sibling of phantom `###`.

Test fixtures live in `~/dev/quire/tests/` and `~/dev/quire-py/tests/`. Both pass the same acceptance set; the Rust port MUST pass the same set.

---

# Concrete Archetypes (Render Parity, v1)

The renderer must reproduce the existing Jinja2 archetypes used in `agent-ix/spec-artifacts-iso` and `agent-ix/spec-artifacts-app`. These are the ground-truth fixtures.

## ISO spec artifacts (8)

| Type | Schema | Template | Required sections |
|---|---|---|---|
| `FR` | `fr-frontmatter.schema.json` | `fr.md.j2` | Description, Specification, Acceptance Criteria, Dependencies |
| `NFR` | `nfr-frontmatter.schema.json` | `nfr.md.j2` | Quality Attribute, Statement, Scope, Measurement and Evaluation, Verification, Dependencies |
| `StR` | `str-frontmatter.schema.json` | `str.md.j2` | Description, Dependencies |
| `US` | `us-frontmatter.schema.json` | `us.md.j2` | Description, Dependencies |
| `IT` | `it-frontmatter.schema.json` | `it.md.j2` | Description, Dependencies |
| `TC` | `tc-frontmatter.schema.json` | `tc.md.j2` | Description, Dependencies |
| `AC` | `ac-frontmatter.schema.json` | `ac.md.j2` | Description, Dependencies |
| `CON` | `con-frontmatter.schema.json` | `con.md.j2` | Description, Dependencies |

## App spec artifacts (2)

| Type | Schema | Template |
|---|---|---|
| `ApplicationSpec` | `applicationspec-frontmatter.schema.json` | `applicationspec.md.j2` |
| `MasterRequirements` | `masterrequirements-frontmatter.schema.json` | `masterrequirements.md.j2` |

## Acceptance: bit-for-bit parity

For each archetype, feed the existing Python Jinja2 input + schema-validated frontmatter through the Rust MiniJinja renderer; output MUST match the Python reference byte-for-byte (or differ only in normalizable whitespace explicitly documented).

---

# Parse Parity (Object Extraction)

Beyond the renderer's schema/template pairs, `quire-rs` must support the body-extraction DSL used in `spec-objects-architecture` (7 types) and `ix-spec-objects` (31 types). Extractors keyed by:

- `frontmatter_field` — read from parsed YAML at a JSONPath
- `section_body` — get the text under a heading (uses Quire `section()`)
- `code_block` — extract a fenced code block of a given language, optionally under a heading

The DSL itself stays in YAML; quire-rs supplies the engine that interprets it.

---

# Non-Functional Targets

| Concern | Target | Verification |
|---|---|---|
| Render latency | < 1 ms per archetype on a typical artifact (median, m1) | criterion bench |
| Parse latency | < 500 ms for a 5 MB markdown document | criterion bench |
| Memory safety | zero `unsafe` blocks in v1 | `make audit-unsafe` (clean baseline) |
| Schema validation | reject malformed input with field-keyed error before render | unit tests per archetype |
| Determinism | identical input → byte-identical output across runs | proptest roundtrip |
| License hygiene | all transitive deps within `deny.toml` allowlist | `make deny` in CI |
| Lint discipline | clippy with `-D warnings`, no per-file allows without comment | `make lint` in CI |

---

# Out of scope (v1)

- React component layer (Quire Layer 3) — TypeScript-only.
- Cross-document graph queries (Quire Layer 4) — out of scope.
- CRDT/OT live editing.
- Schema-driven template generation. Schemas validate; templates present; they do not generate each other.
- HTML output via comrak. Markdown is the canonical output format; HTML is a future post-process.
- Hardening suite (kani/loom/shuttle) — opt-in later via cookiecutter variable.

---

# Workspace shape (v1)

Single crate `quire-rs`, library only. Modules:

```
src/
  ast.rs        — QuireSection, QuireDocument, supporting types
  parser.rs     — parseDocument equivalent
  frontmatter.rs — YAML extraction with malformed-fallback semantics
  query.rs      — section, tables, lists, diagrams, search
  schema.rs    — JSON Schema draft 2020-12 validation (via schemars + serde)
  render.rs     — MiniJinja environment + dispatch
  archetypes/   — schemas + templates, mirrored from spec-artifacts-iso/app
    fr/
      schema.rs
      fr.md.j2
    nfr/
      ...
```

Promote to a Cargo workspace if parsing and rendering diverge enough to warrant separate crates.
