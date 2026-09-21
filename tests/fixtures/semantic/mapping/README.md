# Semantic-module mapping fixtures (FR-070, FR-071, FR-072, FR-074, FR-076/FR-104)

These fixtures are first-party, owned and authored by `quire-rs`. They exercise the
Markdown → semantic-core mapping this crate implements directly; none of it is a
copy of another repository's test data.

| File | Requirement | Behavior |
|---|---|---|
| `overlay.table.md`, `overlay.fence.md`, `overlay.expected.json` (semantic-core `0.1.0`) | FR-071-AC-1, AC-2, FR-072-AC-7 | Extraction of both forms to the expected array (normalized), carrying the `ocl` clause with the listed advisory and lossy availability |
| `overlay-both-forms.md`, `overlay-both-forms.expected.json` | FR-071-AC-3 | Extraction fails at the second form's locus |
| `cell-cases.json` | FR-071-AC-4..AC-8 | Each cell/fence-line case, executed and validated against the embedded semantic-core schemas |
| `operations.md`, `operations.expected.json`, `operations-cases.json` (semantic-core `0.2.0`) | FR-072-AC-1..AC-7 | Clauses/operations extraction, the listed diagnostics, and the listed availability |
| `clause-language-0.1.0-cases.json` (semantic-core `0.1.0`) | FR-072-AC-8 | A `quire` fence is refused under `0.1.0` at the fence |
| `relationships.md`, `relationships.expected.json`, `relationships-cases.json` (semantic-core `0.2.0`) | FR-076-AC-1..AC-15, FR-104-AC-1..AC-12 | `RelationDecl[]` and `relationSources` extraction under the recorded `context`, the listed diagnostics, and the listed availability |
| `legacy-bullets.md`, `legacy-mixed.md`, `legacy.expected.json`, `../corpus/config-service/` | FR-074-AC-1, AC-2 | Legacy-properties-form detection agrees on form, line, and warning |

## `relationships-cases.json` fields

Each case in `cases[]` has these fields:

| Field | Meaning |
|---|---|
| `id` | Case name, cited by FR-076 acceptance criteria. |
| `relationships` | Body appended to `artifactHead`; body line 1 is artifact line 18. |
| `artifact` | A full artifact used verbatim, in place of `artifactHead` + `relationships`. |
| `mappings` | Replaces `context.mappings` for this case. |
| `withoutRelationVocabulary` | When `true`, the extraction runs with no relation vocabulary: no `edgeTypes`, `roles`, or `allowedLinks` (FR-104 `no-relation-vocabulary`). |
| `withoutBundleIndex` | When `true`, the extraction runs with no bundle index: no `bundle.artifacts`; `bundle.package` and `bundle.imports` stay (FR-104 `no-bundle-index`). An absent `bundle.artifacts` and an empty one are equivalent: both are the no-index state. |
| `withoutBundlePackage` | When `true`, the extraction runs with no bundle package: an empty `bundle.package` and no `sourceIdentity` (FR-104 `no-bundle-package`). |
| `diagnostics[]` | Expected diagnostics: `code`, `severity`, `locus`, `line`, `section`, `reason`, and optionally `messageContains`. |
| `diagnostics[].locus` | Where the diagnostic sits: `row`, `second-row`, `header`, `second-header`, `heading`, `second-heading`, or `list`. |
| `diagnostics[].messageContains` | Substrings the diagnostic `message` must contain. |
| `exactDiagnostics` | The exact number of diagnostics the extraction emits; a row that fails several checks yields one (FR-104 refusal order). |
| `relations`, `relationSources` | Expected outputs; `relations: null` means neither is emitted. |
| `availability` | Expected `availability`; `{}` means no `availability.relations` key. |

`context` records a fixed, published edge-type/role/allowed-links taxonomy and a
first-party bundle of test artifacts (`FR-005`/`ConfigOverlay`, `FR-006`/`ConfigVersion`,
`FR-007`/`ConfigEntry`, `FR-010`/an operation). It is not pinned to any other repository's
revision.

`../corpus/config-service/FR-005-config-overlay-entity.md` is a first-party document
modeling a "free-column-table" legacy Properties form, authored to exercise FR-074-AC-1;
it is not a copy of any real config-service document.
