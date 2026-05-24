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

### Required vs optional locators

Each locator has an implicit or explicit `required: bool`. When `required: true` and the locator fails to find a value, the evaluator returns `QuireError::MissingField { key, locator }`. When `required: false`, the key is omitted (single-yield) or the iteration unit is skipped (multi-yield, only if the iteration locator itself fails).

### DSL structural validation (load-time)

When a manifest entry's `body_extraction` is loaded, the loader SHALL validate the DSL's structural integrity before storing it:

- `yield_pattern.match` and `yield_pattern.iterate_over` are **mutually exclusive (XOR)**. A DSL setting both is `QuireError::ArchetypeLoadError { reason: "match and iterate_over are mutually exclusive" }`. A DSL setting neither is also a load error.
- **Unknown keys** under `body_extraction`, `yield_pattern`, or any `Locator` produce `QuireError::ArchetypeLoadError { reason: "unknown DSL key <k>" }`. The loader is strict to surface typos at author time.
- A `Locator` with no `from:` field, or a `from:` value not in {`frontmatter_field`, `section_body`, `code_block`, `table_row`, `list_item`, `heading`}: load error.

### Evaluation edge cases

- **Iterate root not found**: `iterate_over.section_path` referencing a section that does not exist in the document → returns zero records and emits `Diagnostic::IterateRootMissing { path }`.
- **Iteration unit yields no per_match values**: emit zero records for that unit, no diagnostic (this is the normal "no match" case for `per_match` evaluation).
- **Empty body**: a document with frontmatter but `sections: []` against a `section_body` locator returns `MissingField` if `required`, `None` otherwise.

### Purity

`extract` SHALL be pure (no I/O, deterministic). Given identical inputs it produces identical `ExtractionResult` across runs and threads.

### Bounded output

Per-call output is bounded by document size: `records.len() <= max(headings, list_items, table_rows)` for multi-yield; `records.len() <= 1` for single-yield. The engine does NOT impose an additional cap. Pathological documents (millions of iteration units) consume memory proportional to output size; consumers SHOULD bound input via NFR-002's 5 MB envelope.

### Public API

```rust
pub fn extract(doc: &QuireDocument, dsl: &ExtractionDsl)
    -> Result<ExtractionResult, QuireError>;

pub struct ExtractionResult {
    pub records: Vec<serde_json::Map<String, Value>>,
    pub diagnostics: Vec<Diagnostic>,
}
```

Single-yield DSLs return `records.len() <= 1`; multi-yield returns one record per iteration unit.

## Acceptance

- **FR-011-AC-1**: Each of the 6 Locator primitives is exercised by at least one unit test against a small fixture document; result matches the filament-parser-lib reference for the same input.
- **FR-011-AC-2**: A multi-yield DSL with `iterate_over: { section_path: [Algorithms], kind: heading, depth: 1 }` against a document with 3 sub-headings under "Algorithms" emits exactly 3 records.
- **FR-011-AC-4**: A DSL with `required: true` against a missing value returns `QuireError::MissingField` naming the DSL key.
- **FR-011-AC-5**: A parity sweep evaluates every `body_extraction` DSL from the six object-source repos (`spec-objects-{architecture,business,enterprise,operational,security}` + `ix-spec-objects`, total 87+ object types) against fixture documents and asserts each `ExtractionResult.records` matches the filament-parser-lib Python reference for the same input.
- **FR-011-AC-6**: A DSL with both `match` and `iterate_over` set under `yield_pattern` produces `QuireError::ArchetypeLoadError` at load time, NOT at evaluation time.
- **FR-011-AC-7**: A DSL with an unknown key (e.g. `from: section_bodyy` typo) produces `QuireError::ArchetypeLoadError` at load time.
- **FR-011-AC-8**: A DSL with `iterate_over.section_path: [Nonexistent]` against a document missing that section returns `ExtractionResult { records: [], diagnostics: [IterateRootMissing] }`.
