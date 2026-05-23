---
id: FR-011
title: "Body-Extraction DSL Evaluator: Six Locators + Single/Multi-Yield Patterns"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-003"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/filament-parser-lib"
    type: "implements"
    cardinality: "1:1"
---

## Behavior

`quire-rs` SHALL evaluate the body-extraction DSL used by `spec-objects-*` and `ix-spec-objects` manifests. The DSL is YAML, loaded into a strongly-typed `ExtractionDsl` Rust struct by the consumer; the engine consumes the deserialized values.

### Locator primitives (6)

A `Locator` describes how to find one or more values in a parsed `QuireDocument`. The engine SHALL support all six primitives currently defined in `filament-parser-lib`:

| Primitive | Reads | Parameters |
|---|---|---|
| `frontmatter_field` | value at a JSONPath in `doc.frontmatter` | `path: Vec<String>` |
| `section_body` | text content of a section by exact heading | `after_heading: String` |
| `code_block` | source of a fenced code block by language, optionally constrained to a section | `language: String`, `under_section: Option<String>` |
| `table_row` | rows from a markdown table, optionally within a section | `under_section: Option<String>`, `column: Option<String>` |
| `list_item` | items from a bulleted list, optionally within a section, optional list pattern | `under_section: Option<String>`, `pattern: Option<ListPattern>` |
| `heading` | heading text of a section by level or path | `level: Option<u8>`, `path: Option<Vec<String>>` |

### Yield patterns (single XOR multi)

Each `body_extraction.yield_pattern` is one of:

- **Single-yield (`match`)** — emits exactly one record (or zero if any required locator fails). `match: HashMap<String, Locator>` (insertion-ordered for determinism).
- **Multi-yield (`iterate_over` + `per_match`)** — emits one record per iteration unit:
  - `iterate_over: { section_path: Vec<String>, kind: heading|list_item|table_row, depth: Option<u8> }`
  - `per_match: HashMap<String, Locator>` evaluated against each iteration unit's local scope

### Edge emission (`emit_edges`)

A list of declarative edge emissions. Each entry: `{ type: String, target: Locator | String, metadata: HashMap<String, Locator> }`. For each iteration record (or the single record in single-yield), the evaluator emits an edge per entry where the target resolves successfully. Static target strings are normalized via the relationship resolver (see FR-015).

### Required vs optional locators

Each locator has an implicit or explicit `required: bool`. When `required: true` and the locator fails to find a value, the evaluator returns `QuireError::MissingField { key, locator }`. When `required: false`, the key is omitted (single-yield) or the iteration unit is skipped (multi-yield, only if the iteration locator itself fails).

### Public API

```rust
pub fn extract(doc: &QuireDocument, dsl: &ExtractionDsl)
    -> Result<ExtractionResult, QuireError>;

pub struct ExtractionResult {
    pub records: Vec<serde_json::Map<String, Value>>,
    pub edges: Vec<HarvestedEdge>,
    pub diagnostics: Vec<Diagnostic>,
}
```

Single-yield DSLs return `records.len() <= 1`; multi-yield returns one record per iteration unit.

## Acceptance

- **FR-011-AC-1**: Each of the 6 Locator primitives is exercised by at least one unit test against a small fixture document; result matches the filament-parser-lib reference for the same input.
- **FR-011-AC-2**: A multi-yield DSL with `iterate_over: { section_path: [Algorithms], kind: heading, depth: 1 }` against a document with 3 sub-headings under "Algorithms" emits exactly 3 records.
- **FR-011-AC-3**: A DSL with `emit_edges: [{ type: depends_on, target: { from: frontmatter_field, path: [depends_on] } }]` against a doc with a frontmatter `depends_on` list emits one edge per list item.
- **FR-011-AC-4**: A DSL with `required: true` against a missing value returns `QuireError::MissingField` naming the DSL key.
- **FR-011-AC-5**: A parity sweep evaluates every `body_extraction` DSL from the six object-source repos (`spec-objects-{architecture,business,enterprise,operational,security}` + `ix-spec-objects`, total 87+ object types) against fixture documents and asserts each `ExtractionResult.records` matches the filament-parser-lib Python reference for the same input.
