# Record-sections fixtures (quoin FR-121, FR-122, FR-123)

Golden cases for the record-sections mapping that `quire-rs` FR-081 implements.
Quoin does not execute the extraction and holds no copy of these files; it reads
them from this crate.

| File | Requirement |
|---|---|
| `schemas/*.json` | Synthetic record roots the cases extract against (`Lifecycle.json` carries the lifecycle keys: an accepted status requires a provenance). No module schema is copied here: the architecture cases run against the published `spec-objects-architecture` schemas, fetched by `spec-objects-architecture-fixture`. |
| `cases.json` | The FR-081 cases (quoin FR-121-AC-1..9, FR-122-AC-1..6, FR-123-AC-1..7 and AC-9); each case names its criterion in `ac`. |

## `cases.json` fields

| Field | Meaning |
|---|---|
| `id`, `ac` | Case name and the FR-081 criterion it backs. |
| `schema` | The root schema file: under `schemas/`, or in the architecture module when `module` is `architecture`. |
| `objectType`, `constructReferences` | The extracting object type and its construct `references`. |
| `validates` | Key to module schema (`Name`, `Name[]`, `core:Name[]`) each extracted key must validate against. |
| `availabilitySubset` | Compare only the listed `availability` keys. |
| `artifact` | The full artifact. |
| `package` | The module package; default `acme/safety`. The architecture forms apply to `agent-ix/spec-objects-architecture`. |
| `mappings` | Replaces the default `["record-sections"]`. |
| `bundle` | The bundle index; default package `acme/safety` with no artifacts (the no-index state). |
| `relationVocabulary`, `bodyExtraction`, `withoutSourceIdentity` | Extraction inputs; absent means none. |
| `record`, `recordSources`, `availability` | Expected `record` (`null`: no key), the `recordSources` keys in order, and the expected `availability.recordKeys` (`null` or `{}`: no key). |
| `availability.<key>.reasonAt` | Source snippets whose lines the `entry-errors` reason lists; `@2` after a snippet selects its second line. |
| `diagnostics[]` | Expected diagnostics: `code`, `severity`, `reason`, and `at`, the snippet naming the line. The case emits exactly these. |
| `recordValid` | Whether the declaration record validates against the case schema. |
