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
| `code_block` | source of a fenced code block by language, **resolved from the `under_section` content slice** (section-owned) | `language: String`, `under_section: Option<String>` |
| `table_row` | rows from a markdown table, optionally within a section | `under_section: Option<String>`, `column: Option<String>` |
| `list_item` | items from a bulleted list, optionally within a section, optional list pattern | `under_section: Option<String>`, `pattern: Option<ListPattern>` |
| `heading` | heading text of a section by level or path | `level: Option<u8>`, `path: Option<Vec<String>>` |

### Section-owned `code_block` resolution — CR-003

The `code_block` locator is **section-owned**: it resolves fenced blocks from the
content slice of `under_section` (or, when `under_section` is absent, from the joined
section bodies of the document — or, under multi-yield, the iteration unit's local
scope), exactly like `table_row` and `list_item`. This guarantees containment and
determinism: a `code_block per_match` locator isolates per unit under `iterate_over`,
and a `code_block` assert under iteration checks only that unit's content.

> **CR-003 note:** The original implementation resolved `code_block` against a
> **document-wide diagram catalog** (`extract_diagrams(doc)` — every fenced block in
> `doc.raw`, each annotated with its nearest heading) and then filtered by
> `under_section == block.section`. That defeated per-unit scoping under multi-yield
> (`scope_to_section` carries the whole document's `raw`, so the locator saw the entire
> document) — a latent gap surfaced during the markdown-validation review. It was latent
> because the only `code_block` users (`process`, `state_machine`, `event` in
> `spec-objects-business`) are single-yield with a unique `after_heading`. The fix makes
> the locator read the section content slice (`diagrams_from_content(&content, …)`)
> while the document-wide `extract_diagrams` remains a **separate harvest query**
> (US-010 / RAG), no longer the locator substrate. No manifest/schema change: the three
> object types already declare `after_heading`.

### Fence-character parity for `code_block` — CR-004

The fenced-code-block scanner that backs `code_block` (and the document-wide
`extract_diagrams` harvest) SHALL recognize **both** ` ``` ` (backtick) and `~~~`
(tilde) fences, capturing the info-string (language word) after either fence, and
SHALL close a block ONLY on a fence line whose character **matches** the opening
fence. A `~~~` line inside a ` ``` ` block (and a ` ``` ` line inside a `~~~` block)
is **content**, not a close. This mirrors the parser's heading-walk fence model
(FR-007-AC-3 tilde support, FR-007-AC-4 matching-character independence), so the
heading walk and the code-block scanner agree on what opens and closes a block.

> **CR-004 note:** The original scanner (`src/query.rs`) recognized backtick fences
> only (`^```(\w*)`), matching the TS reference's `extractDiagrams`. The parser's
> heading walk already handled `~~~` (FR-007-AC-3/AC-4); the scanner did not, so a
> `~~~`-fenced diagram was invisible to `code_block`/`extract_diagrams` even though
> the same block correctly suppressed inner headings during the parse walk. The fix
> extends the scanner to the same two-fence, matching-character model. This is an
> intentional extension of the TS surface (recorded in
> `tests/parser_parity/divergences.md` §9), not a parity break: the TS suite has no
> `~~~` `extractDiagrams` fixture. No manifest/schema change.

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
- **FR-011-AC-13**: A `code_block` `per_match` locator under `iterate_over` is section-owned: against a document where each iteration unit owns its own fenced block, each yielded record receives **its own** unit's block (not unit #1's for all), and a `required: true` `code_block` locator returns `QuireError::MissingField` for the specific unit that lacks a block — proving containment, not a document-wide fallback. A single-yield `code_block under: X` returns only `X`'s block, excluding a same-language block in a different section.
- **FR-011-AC-14**: The fenced-code-block scanner recognizes both ` ``` ` and `~~~` fences and closes a block only on a matching-character fence line (parity with the FR-007 parser fence model): a `~~~mermaid` block is extracted with language `mermaid`; a `~~~` line inside a ` ``` ` block (and a ` ``` ` line inside a `~~~` block) is content, not a close; an unclosed `~~~` block is flushed as the final block; and a section-owned `code_block` locator resolves a `~~~` block under its heading.
