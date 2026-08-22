# quire-rs Usage Guide

A guide for developers building a **documentation standard** on quire-rs: how to model
document types as archetypes, package them as a module, validate and extract Markdown
against them, and work with a whole corpus.

This guide assumes you have read the feature list in the [README](../README.md). It is
organized as: mental model → core concepts → authoring a module → document structure →
the body-extraction DSL → validation → lint → input contracts → corpus → per-surface usage
→ an end-to-end worked example.

> **Render is out of scope.** quire-rs is a *parse → validate → extract → edit* engine.
> Markdown is canonical and authored directly; there is no template rendering. Manifests
> that reference templates are not supported. Schema = correctness; presentation lives
> elsewhere.

---

## 1. Mental model

```
 module  ──load──▶  Registry  ──┬──▶  parse_document(md)  ──▶  QuireDocument
(manifest.yaml +                │
 schemas/*.json)                ├──▶  validate_document(archetype, md)  ──▶ ValidationResult
                                ├──▶  extract(doc, dsl)                 ──▶ ExtractionResult
                                └──▶  input_contract_for(reg, name)     ──▶ InputContract + skeleton

 directory ──load_repo──▶ RepoLoad ──Spec::from_repo──▶ Spec (corpus + resolved edges)
```

- A **module** declares **archetypes** (document types) and ships their **JSON Schemas**.
- A **Registry** is the compiled, immutable index of every archetype across loaded modules.
- A **document** is a Markdown file: YAML frontmatter + a heading tree.
- The frontmatter `type` field is the **discriminator** that routes a document to its
  archetype.
- Validation and extraction run a document against its archetype's schema + body-extraction
  DSL.

---

## 2. Core concepts

### Concept (the OKF base contract)

Every document carries frontmatter. The base "concept" contract (`base_concept_schema`)
requires a `type` string and types optional `description` / `tags`. `type` is what the host
uses to pick an archetype. (When validating with an explicit `--archetype` override, the
`type`-required rule is relaxed — see `validate_concept_shape`.)

### Archetype

An **archetype** is the compiled definition of one document type (`FR`, `NFR`, `US`, …). A
`CompiledArchetype` bundles:

- a **frontmatter schema** (validates the YAML header),
- an optional **data schema** (validates an extracted record),
- a **body-extraction DSL** (`body_extraction`), and
- **carry-over** metadata: `defaults.id_pattern`, `allowed_links`, `grammar_ref`,
  `has_plugin`, etc.

### Module

A **module** is the unit of distribution: a directory containing a `manifest.yaml` and a
`schemas/` directory. One module ships many archetypes (the `spec-artifacts-iso` module
ships `FR`, `NFR`, `StR`, `US`, `IT`, `TC`, `AC`, `CON`, and a `master-requirements`
archetype).

### Plugins — a clarification

quire-rs has **no code-plugin system**. The manifest can carry a `has_plugin: true` marker
on an archetype, but this is purely metadata: it signals to the *host application* (e.g. an
editor or service) that the archetype has custom host-side behavior. The engine itself reads
the flag and does nothing with it. Do not expect quire-rs to load or execute plugin code.

### Registry

A `Registry` is immutable, thread-safe (`Arc`-wrapped, cheap to clone), and built once at
load time. Construct it with:

- `Registry::from_env()` — search `IX_FILAMENT_MODULES_PATH` → `IX_SCHEMA_PATH` →
  `~/.ix/filament/modules/`.
- `Registry::load_from(&[paths])` — treat each path as a *search root* and load every module
  subdirectory found.
- `Registry::load_module(path)` — load a *single* module directory (must contain
  `manifest.yaml` directly).
- `Registry::from_inline_parts(manifest_yaml, &schemas)` — in-memory (WASM / tests), no
  filesystem.

Each has a strict variant (`load_strict`, `load_module_strict`, `from_inline_parts_strict`)
that promotes the first collision to a fatal error. The default (tolerant) variants record
collisions as diagnostics and keep going. Query the registry:

```rust
registry.archetype("FR");                 // Option<&CompiledArchetype>
registry.archetype_in_module("spec-artifacts-iso", "FR");
registry.archetype_names();               // iterator
registry.module_names();
registry.module_version("spec-artifacts-iso");
registry.diagnostics();                   // non-fatal load issues
registry.failures();                      // per-archetype load failures
registry.lint_rules();                    // advisory rules declared by modules
```

---

## 3. Authoring a module

### On-disk layout

```
spec-artifacts-iso/
├── manifest.yaml
└── schemas/
    ├── fr-frontmatter.schema.json
    ├── nfr-frontmatter.schema.json
    └── … one per archetype
```

(See the real example under `tests/fixtures/modules/iso/`.)

### `manifest.yaml`

Top-level keys:

| Key                | Purpose                                                              |
| ------------------ | ------------------------------------------------------------------- |
| `manifest_version` | Manifest format version (e.g. `1.0.0`).                             |
| `name`             | Module name (e.g. `spec-artifacts-iso`).                            |
| `version`          | Module SemVer.                                                      |
| `description`      | Human description.                                                  |
| `archetypes`       | Container/doc archetypes (e.g. a `spec` container with `composition`). |
| `grammars`         | Named grammars the artifact types reference via `grammar_ref`.      |
| `artifact_types`   | The document-type archetypes (FR, NFR, …).                          |
| `object_types`     | Object archetypes (for composed `object:` validation).             |
| `lint_rules`       | Advisory lint rules (§7).                                           |
| `plain_language_profiles` | Named/versioned reader-language profiles (§7).              |

> **Composition** belongs on entries under `archetypes:` (the container), not on
> `artifact_types:`. A container declares `composition.expected_artifacts: [...]`.

Per-archetype fields (under `artifact_types`/`object_types`):

| Field                    | Purpose                                                       |
| ------------------------ | ------------------------------------------------------------- |
| `name`                   | **Required.** Archetype name, used as the routing key.        |
| `frontmatter_schema_ref` | Path (relative to the manifest) to the frontmatter JSON Schema. |
| `data_schema`            | Inline JSON Schema for an extracted record (optional).        |
| `body_extraction`        | The body-extraction DSL (§5).                                 |
| `defaults.id_pattern`    | ID allocation template, e.g. `FR-{next:03d}`.                 |
| `allowed_links`          | Relationship types this archetype may declare.               |
| `grammar_ref`            | Name of a grammar declared in `grammars:`.                    |
| `has_plugin`             | Host-side marker only (§2).                                   |

### Schemas

Schemas are **JSON Schema Draft 2020-12** documents in `schemas/`. They are read, parsed, and
compiled at load time into cached validators. **Cross-file `$ref` is not resolved** — keep
each schema self-contained.

### Discovery

At runtime, `Registry::from_env()` resolves modules from (in priority order)
`IX_FILAMENT_MODULES_PATH`, then `IX_SCHEMA_PATH`, then `~/.ix/filament/modules/`. Paths
support `~` expansion, are de-duplicated by canonical path, and are guarded against symlink
loops.

---

## 4. Document structure

A document is parsed by `parse_document` into a `QuireDocument`:

```rust
pub struct QuireDocument {
    pub preamble: Option<String>,         // text before the first heading
    pub sections: Vec<QuireSection>,      // top-level headings (tree)
    pub raw: String,                      // original input, unchanged
    pub frontmatter: Option<Map<String, Value>>,
}

pub struct QuireSection {
    pub id: String,                  // "<slug>-L<line>" — UNSTABLE across edits
    pub block_id: Option<String>,    // Pandoc "{#blk-id}" — STABLE
    pub heading: String,             // heading text (attributes stripped)
    pub level: u8,                   // 1–6
    pub content: String,             // byte-exact content to next heading
    pub children: Vec<QuireSection>,
    pub start_line: usize,
    pub end_line: usize,
}
```

**Frontmatter** extraction is forgiving (BOM-stripped, CRLF-tolerant; malformed YAML falls
back to treating the whole input as body). **Validation** of that frontmatter against a
schema is strict — the two are separate steps.

**Block IDs.** Author a stable handle with a Pandoc attribute:

```markdown
## Acceptance Criteria {#blk-ac-01}
```

→ `heading: "Acceptance Criteria"`, `block_id: Some("blk-ac-01")`. Use `block_id` for stable
addressing in writeback (`update_block`); the computed `id` shifts when lines move.

**Relationships** are declared in frontmatter and/or as `ix://` links in the body (see §9).

---

## 5. The body-extraction DSL

`body_extraction` tells the engine how to pull structured fields out of the Markdown body —
and, in `validate_document`, which fields are required. Its top-level shape is a
`yield_pattern` that is **exactly one** of:

- `match:` — **single-yield.** A map of field → locator; produces 0 or 1 record.
- `iterate_over:` + `per_match:` — **multi-yield.** Iterate a root (headings / list items /
  table rows) and produce one record per unit.

Declaring both, or neither, is rejected at load time.

### Single-yield example (from the FR archetype)

```yaml
body_extraction:
  yield_pattern:
    match:
      description:
        from: section_body
        after_heading: Description
        required: true
      specification:
        from: section_body
        after_heading: Specification
        required: true
      acceptance_criteria:
        from: section_body
        after_heading: Acceptance Criteria
        required: true
      dependencies:
        from: section_body
        after_heading: Dependencies
        required: true
```

### Multi-yield example

```yaml
body_extraction:
  yield_pattern:
    iterate_over:
      section_path: [Algorithms]
      kind: heading        # heading | list_item | table_row
      depth: 1
    per_match:
      name:
        from: heading
        level: 2
      description:
        from: section_body
        after_heading: Description
```

### Locator primitives

A locator is discriminated by its `from:` key. The six primitives and their fields:

| `from:`            | Selecting fields                                    |
| ------------------ | --------------------------------------------------- |
| `frontmatter_field`| `path: [key, …]` (nested keys)                      |
| `section_body`     | `after_heading: <heading text>`                     |
| `code_block`       | `language:`, `under_section:` (alias `after_heading`) |
| `table_row`        | `under_section:`, `column:` (name or index)         |
| `list_item`        | `under_section:`, `pattern:` (`bullet`/`dash`/`plus`) |
| `heading`          | `level:` (alias `depth`), `path:`, `regex:`         |

Every locator also accepts:

- `required: bool` — defaults to **`true`**; a required locator that resolves to nothing
  (or to placeholder/empty content) is a validation error.
- `multiple: bool` — return all matches vs. the first.
- `regex: <pattern>` — filter matched content.
- `assert: { … }` — constraints checked by `validate_document` (below). Ignored by `extract`.

### Fallback chains (FR-016)

Give a field a **list** of locators; the first non-empty result wins:

```yaml
specification:
  - { from: section_body, after_heading: Specification, required: true }
  - { from: section_body, after_heading: Spec, required: false }
```

### The `assert` facet (FR-033) and `{field}` interpolation (FR-034)

`assert` validates the *shape* of an extracted value during `validate_document`. Keys are
optional and only some apply per locator kind (unknown keys are rejected at load time):

| Assert key       | Applies to                  | Meaning                                  |
| ---------------- | --------------------------- | ---------------------------------------- |
| `level`          | `heading`, `section_body`   | Heading must be at this ATX level.       |
| `matches`        | most kinds                  | Content must match this regex.           |
| `id_pattern`     | most kinds                  | IDs in the content must match.           |
| `columns`        | `table_row`                 | Table must have these columns.           |
| `min_rows`       | `table_row`                 | Minimum row count.                       |
| `min_items`      | `list_item`                 | Minimum item count.                      |
| `id_column`      | `table_row`                 | Which column holds the id.               |

`{field}` tokens in an assert pattern interpolate a previously extracted frontmatter field;
an unresolved token raises `UnresolvedField`.

```yaml
title:
  from: heading
  level: 1
  regex: ^Master Requirements Specification$
  required: true
  assert:
    level: 1
```

### Edge emission

A DSL may declare `emit_edges` to produce relationship edges as a side effect of extraction
(see `ExtractionResult.edges`).

### Running extraction

```rust
let doc = quire::parse_document(&md);
let dsl = registry.archetype("FR").unwrap().body_extraction().unwrap();
let result = quire::extract(&doc, dsl)?;     // ExtractionResult { records, edges, diagnostics }
```

---

## 6. Validation flow

Two entry points:

- **Record validation** — `validate(archetype, &json)` / `validate_all(...)` validate a JSON
  data record against the archetype's schema. `apply_patch(archetype, &current, &patch)`
  deep-merges then validates (useful for incremental edits).
- **Document validation** — `validate_document(archetype, doc_text)` validates an authored
  Markdown file end-to-end:
  1. parse,
  2. concept-shape check (typed `description`/`tags`),
  3. frontmatter schema,
  4. each `required` body-extraction locator must resolve to non-empty, non-placeholder
     content, and its `assert` must hold,
  5. per-level heading uniqueness.

`validate_document_in_registry(registry, archetype, doc_text)` adds **composed `object:`
validation**: if the frontmatter carries an `object:` key naming another archetype the
registry knows, that archetype's body asserts also run (errors merged). If `object:` names an
unknown archetype, a **warning** is emitted — never an error. The `type` archetype always
produces hard errors.

```rust
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,     // { message, line: Option<usize>, reason }
    pub warnings: Vec<ValidationWarning>,
}
```

`ValidationReason` is a machine-readable enum: `Missing`, `Empty`, `Placeholder`, `Assert`,
`Frontmatter`, `DuplicateHeading`, `UnresolvedField`, `UnknownObjectType` (warnings only).

---

## 7. Lint rules

Lint is **advisory** — it never blocks validation or extraction. Rules are declared in the
manifest under `lint_rules` and discriminated by `type:`. Three rule types ship:

```yaml
lint_rules:
  - type: table_column_values
    id: ac-status-values
    archetypes: [FR]          # empty/omitted = applies to all
    section: Acceptance Criteria
    column: Status
    allowed: [Pass, Fail, Pending]
    annotation_pattern: '\(TC-[0-9]+\)'   # optional trailing annotation
    severity: warning
  - type: section_body_pattern
    id: spec-mentions-api
    section: Specification
    pattern: 'API'
    message: 'Specification should reference the API'   # optional
    severity: warning
```

Apply them with `lint_document(&doc, registry.lint_rules())`, which returns
`LintFinding { rule, severity, message }`.

### Reader-visible plain-language profiles

Plain-language analysis uses the same Rust Markdown boundary and never asks a
caller to reconstruct visible prose. A module may declare named profiles:

```yaml
plain_language_profiles:
  engineering-docs:
    version: 1.0.0
    document_types: [FR, NFR, StR, US]
    sentence_word_limit: 35
    max_heading_level_step: 1
    known_acronyms:
      API: application programming interface
    ignored_uppercase_terms: [SHALL, MUST, MAY]
```

```rust
let profile = registry
    .plain_language_profile("engineering-docs")
    .expect("profile selected explicitly");
let report = quire::check_plain_language_at(
    std::path::Path::new("spec"),
    "engineering-docs",
    profile,
);
```

`PlainLanguageReport` records the profile id/version, a fingerprint of every
effective setting, readable document/block counts, source-located warning
findings and skipped inputs. `readable_blocks == 0` is therefore distinct from
a clean run. The feature makes no external-standard conformance claim and has
no implicit profile or promotion path.

---

## 8. Input contracts & authoring skeletons

`input_contract_for(&registry, "FR")` derives, from the archetype's schemas + body-extraction
asserts, an `InputContract`:

- `.to_json()` — a deterministic JSON contract for tooling/agents, and
- `.skeleton()` — a Markdown scaffold (heading structure with required sections marked) an
  author or agent can fill in.

From Python: `quire.input_contract(name, module_root)` and
`quire.input_skeleton(name, module_root)`.

---

## 9. Working with a corpus

Load a whole directory tree:

```rust
let load = quire::load_repo(std::path::Path::new("spec/"));   // RepoLoad { documents, diagnostics }
let spec = quire::Spec::from_repo(load);                      // immutable corpus
```

`load_repo` walks in parallel (rayon), honors `.gitignore`, skips dotfiles and
`README.md`/`tests.md` by default, and parses every `.md` into a `LoadedDocument`
(`{ path, id, uuid, doc }`). Tune behavior with `load_repo_with(root, &WalkOptions)`.

> **Identity is read, never derived.** `id` is the human artifact id (the resolution key);
> `uuid` is the durable UUID7 from frontmatter. The loader never hashes content or mutates
> files.

The `Spec` (Rust) exposes `by_id(id)`, `len`, `is_empty`, and `diagnostics`. Richer
relationship queries are available on the **Python** `Spec` class: `by_type`, `referencing`,
`orphans`, `dangling` (see §10).

Reference handling:

- `harvest_edges(&loaded_document)` extracts relationship stubs (frontmatter `relationships`
  + body `ix://` links).
- `validate_bundle(&spec, posture)` validates corpus-wide archetype conformance —
  `BundlePosture::Strict` (every document must conform to a loaded archetype) vs.
  `Permissive` (foreign documents tolerated). `validate_bundle_at` scopes to a sub-path.
- `unlinked_references(&spec)` finds dangling edges and returns `UnlinkedReference`s with
  autofix `suggestions` (candidate target ids + scores).

---

## 10. Using the engine from each surface

### Rust

Key entry points: `Registry::from_env` / `load_from` / `load_module`, `parse_document`,
`validate_document` / `validate_document_in_registry`, `validate` / `apply_patch`,
`extract`, `update_section` / `update_block`, `input_contract_for`, `load_repo` / `Spec`.

```rust
let registry = quire::Registry::from_env()?;
let fr = registry.archetype("FR").expect("FR loaded");
let updated = quire::update_section(&quire::parse_document(&md), "Description", "New body.")?;
```

### Python (`quire` wheel, `--features python`)

Module-level functions: `parse_document(text)`, `load_repo(root)`,
`validate(archetype_name, module_root, data)`,
`validate_document(archetype_name, module_root, text)`,
`extract(archetype_name, module_root, text)`, `extract_frontmatter(text)`,
`harvest_edges(doc)`, `input_contract(name, module_root)`,
`input_skeleton(name, module_root)`, `validate_manifest(payload, schema_path)`.

Classes: `Registry` (`load_from`, `from_env`, `archetype_names`, `validate`), `Spec`
(`from_path`, `by_id`, `by_type`, `referencing`, `orphans`, `dangling`, `diagnostics`),
`ExtractionContext`.

Exceptions: `QuireBaseError` → `QuireValidationError`, `QuireSchemaError`, `QuireParseError`.
The GIL is released on heavy Rust work.

```python
import quire

result = quire.validate_document("FR", "~/.ix/filament/modules/spec-artifacts-iso", text)
if result["errors"]:
    raise SystemExit(result["errors"])
```

### WASM (`--features wasm`)

The `wasm32` build has no filesystem `$ref` resolution (`resolve-file` is disabled). Build
a registry from in-memory parts:

```rust
use std::collections::BTreeMap;

let mut schemas = BTreeMap::new();
schemas.insert("schemas/fr-frontmatter.schema.json".to_string(), schema_text);
let registry = quire::Registry::from_inline_parts(manifest_yaml_bytes, &schemas)?;
```

JS consumers normally use the [`quire-wasm`](../README.md#related-projects) package rather
than building this directly.

---

## 11. End-to-end worked example

**Manifest** (`spec-artifacts-iso/manifest.yaml`, excerpt):

```yaml
manifest_version: 1.0.0
name: spec-artifacts-iso
version: 0.1.0
artifact_types:
  - name: FR
    grammar_ref: iso-spec-core
    frontmatter_schema_ref: schemas/fr-frontmatter.schema.json
    defaults:
      id_pattern: FR-{next:03d}
    allowed_links: [implements, depends_on, references]
    body_extraction:
      yield_pattern:
        match:
          description:        { from: section_body, after_heading: Description, required: true }
          specification:      { from: section_body, after_heading: Specification, required: true }
          acceptance_criteria:{ from: section_body, after_heading: Acceptance Criteria, required: true }
          dependencies:       { from: section_body, after_heading: Dependencies, required: true }
```

**Schema** (`schemas/fr-frontmatter.schema.json`, excerpt):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["id", "title", "type"],
  "properties": {
    "id":    { "type": "string", "pattern": "^[A-Z]{2,4}-[0-9]+$" },
    "title": { "type": "string", "minLength": 1 },
    "type":  { "const": "FR" }
  }
}
```

**Document** (`spec/FR-001.md`):

```markdown
---
id: FR-001
title: User Authentication
type: FR
relationships:
  - target: ix://US-001
    type: implements
---

## Description
Users must be able to log in with email and password.

## Specification
1. Accept email + password.
2. Hash with bcrypt and compare against the stored hash.

## Acceptance Criteria
- Valid credentials succeed.
- Invalid password is rejected.

## Dependencies
Requires FR-002 (session management).
```

**What `validate_document_in_registry` does:**

1. Parse → sections `[Description, Specification, Acceptance Criteria, Dependencies]`.
2. Concept shape OK; frontmatter validates against `fr-frontmatter.schema.json`
   (`id` matches the pattern, `title` non-empty, `type == "FR"`).
3. Each required `section_body` locator resolves to non-empty content.
4. Heading uniqueness holds.
5. → `ValidationResult { is_valid: true, errors: [], warnings: [] }`.

If, say, the `Specification` section were missing, you'd get one
`ValidationError { reason: Missing, message: "[FR] specification …", line: … }` and
`is_valid: false`.

---

## 12. References

- `spec/spec.md` — the normative requirements (FR/NFR), including §19 hardening posture.
- `spec/assets/adr/` — ADR 0001 (validator crate), 0002 (three-layer pipeline), 0003
  (unified archetype shape), 0007 (internal relative-path links).
- `CLAUDE.md` — design taste, safety scaffolding, and the v0.3 corpus/bindings invariants.
- `tests/fixtures/modules/iso/` — a real, validating module to copy from.
