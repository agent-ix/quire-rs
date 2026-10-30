---
id: FR-070
title: "Typed Properties extraction to FieldDecl[]"
type: FR
verification_method: test
evidence:
  - kind: test_case
    ref: tests/semantic_properties.rs
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-019"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-069"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-011"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-025"
    type: "requires"
    cardinality: "1:1"
---
# FR-070: Typed Properties extraction to FieldDecl[]

## Description

When the engine extracts an object artifact whose module carries a
`semantic` block, the engine SHALL extract the `## Properties` section into a
`FieldDecl[]` that validates against the embedded semantic-core
`FieldDecl.json`, from either the typed table or the `sysml` fence, per the
`agent-ix/quoin` FR-071 mapping.

The engine SHALL leave the existing `properties` string yielded by
`section_body` unchanged beside the new array.

The engine SHALL resolve a `Type` cell reading `Clock` to the native name
`Clock`, and a `Type` cell reading `Quantity<u>` to the bundle identity of the
unit artifact `u` (Type cells, below). The same token resolution serves the
dimension and unit tokens of [FR-075](./FR-075-model-feature-extraction.md)
"Dimension and unit tables", so a consumer classifies every one of these
tokens from the engine's verdict and never from the token's spelling.

## Inputs

- The parsed document and its `## Properties` section.
- A `SemanticContext`: the module's `SemanticModule` record
  ([FR-069](./FR-069-semantic-module-contract-at-load.md)), the corpus-relative
  `path` of the document, and a `BundleIndex`
  `{ objects: [{ id, names[] }], enumerations: [{ id, names[] }], imports: { <package>: exports[] }, importedArtifacts?: [{ package, id, names[], object }] }`.
  Every surface supplies the index explicitly: corpus extraction
  ([FR-025](./FR-025-spec-corpus-model.md)) builds it from the loaded corpus,
  where an object's `names` are its `id`, its `title`, and its frontmatter
  `name` when present, and `imports` come from the loaded modules' `exports`;
  `importedArtifacts` lists the artifacts of every package an `imports` entry
  names and a loaded corpus provides, each with that artifact's `package`
  (`<org>/<repo>`), `id`, `names` (as for `objects`) and frontmatter `object`
  type, and is empty or absent when the corpus loads none;
  the Filament API takes it as the optional `semanticBundle` input; a caller
  that has none passes an empty index, which is an explicit state, not a
  default.
- The golden fixtures vendored read-only from `agent-ix/quoin`
  `tests/fixtures/semantic-module/mapping/`, plus a quire-rs-authored `config-version.bundle.json` index
  (`{ objects: [{ id: "FR-006", names: ["ConfigVersion"] }, { id: "FR-005", names: ["ConfigOverlay"] }] }`)
  under which the golden cases run.

## Outputs

- `fields[]`: `FieldDecl` entries in authored order, normalized (canonical
  key order, optional keys omitted).
- `fieldsForm`: `table` or `fence`.
- A row locus (1-based line of the table row or fence line) on every
  diagnostic.
- Diagnostics with a `semantic.*` or `agent-ix.semantic-core.*` code, a
  severity in `advisory`, `warning`, `error`, and a locus.
- The `fields` availability state ([FR-072](./FR-072-semantic-extraction-surface.md)):
  `available` with the array, or `unavailable` with a reason.

## Behavior

Form recognition:

- The engine SHALL treat a table whose header row is exactly
  `Field | Type | Multiplicity | Constraints` (cells trimmed, optional
  backticks stripped) as the typed table, a fenced block tagged `sysml`
  directly under the heading as the fence, and any other table or any bullet
  list as a legacy form (`free-column-table`, `bullet-list`).
- If the section holds a typed table and a fence, or a typed form followed by
  a legacy block, then the engine SHALL emit the error
  `semantic.properties-both-forms` at the second form's start line and set
  `fields` `unavailable` with reason `both-forms`.
- If the section holds only legacy forms, then the engine SHALL emit
  `semantic.legacy-properties-form` with the form name, the first block's
  line, and `migration: typed-table`, at severity `warning`, or `error` when
  the module sets `legacy_forms: error`; `fields` is `unavailable` with
  reason `legacy-form`.
- If the document has two `## Properties` headings, then the engine SHALL
  emit `semantic.duplicate-section` at the second heading and set `fields`
  `unavailable` with reason `duplicate-section`.
- A typed table with a header and no rows SHALL yield `fields` `available`
  and empty; a section with prose and no block yields `unavailable` with
  reason `no-block` and the warning `semantic.properties-no-block`.
- If any row or fence line carries an `error`, then the engine SHALL set
  `fields` `unavailable` with reason `row-errors` listing the loci and SHALL
  NOT emit a partial array; a consumer never receives a type missing a field.

Type cells, resolved in this precedence:

1. A kernel scalar name (`UUID`, `Boolean`, `Integer`, `Decimal`, `String`,
   `Timestamp`, `Duration`, `Bytes`, `JsonObject`) maps to that target;
   `Decimal(p,s)` adds `decimal: { precision, scale }`; bare `Decimal` is the
   error `agent-ix.semantic-core.MISSING_DECIMAL_POLICY`; `<Name> [<unit>]`
   adds `unit` and is `agent-ix.semantic-core.UNIT_NOT_ALLOWED` when the
   target is not `Integer`, `Decimal`, or `Duration`.
2. A bundle object whose `id` equals the token.
3. A bundle object one of whose `names` equals the token; two objects
   matching is the error `semantic.ambiguous-type` naming both.
4. A bundle enumeration whose `id` or `names` match the token.
5. A name exported by a package in `imports` that a loaded module provides.

Native names and quantities, resolved before steps 2..5 as step 1 is:

- The native name `Clock` maps to the target `Clock`, beside the kernel
  scalar names. It takes no `decimal` and no `unit`: `Clock [ms]` is
  `agent-ix.semantic-core.UNIT_NOT_ALLOWED`, and `Clock(1,2)` is
  `semantic.invalid-type-token`. A bundle object or enumeration named `Clock`
  never resolves from a `Type` cell, exactly as one named `UUID` never does.
  Because semantic-core's `TypeRef.target` admits a `SemanticId` or a kernel
  scalar, the engine validates a `Clock` entry against `FieldDecl.json` with
  its `target` read as the `SemanticId` `ix://quire/native/Clock`, and
  `semantic-v1.schema.json` admits the target `Clock`.
- A token of the form `Quantity<u>`, with no space inside the angle brackets,
  is a quantity token, and `u` is a *unit token*. The engine resolves a unit
  token (and a dimension token, [FR-075](./FR-075-model-feature-extraction.md)
  "Dimension and unit tables") by the rules of "Unit and dimension tokens"
  below and maps a match to `ix://<package>/type/<Name>` of the matched
  artifact (`<package>` the package that owns it, `<Name>` its `id`). A `u`
  that names no artifact of the required object type, including one that names
  an artifact of any other object type, maps to the placeholder
  `ix://<org>/<repo>/unresolved/<Token>` with the advisory
  `semantic.unresolved-type`. A bare `Quantity` with no angle brackets is an
  ordinary token resolved by steps 2..5.
- `Quantity<u>` takes no `decimal` and no `unit`: `Quantity<ms> [s]` is
  `agent-ix.semantic-core.UNIT_NOT_ALLOWED`, and `Quantity<ms>(1,2)` is
  `semantic.invalid-type-token`. A quantity token with an empty `u`, a `u` that
  is neither an `Identifier` nor a qualified token (below), a nested quantity,
  or text after the closing bracket is `semantic.invalid-type-token`.
- A `ref item` line (fence form) naming `Clock` or a quantity token is
  `semantic.sysml-outside-subset`, as for a kernel scalar; `attribute` lines
  admit both.

Unit and dimension tokens: a unit token or a dimension token is an
`Identifier` (a *bare* token) or `<package>::<Identifier>` (a *qualified*
token), `<package>` an `<org>/<repo>` package string spelled exactly as the
module's `package` or one of its `imports` entries spells it. Any other text,
including a second `::`, an empty side, or a qualifier that is not
`<org>/<repo>`, is `semantic.invalid-type-token`. The token kind fixes the
object type the matched artifact must carry: `unit` for a unit token,
`dimension` for a dimension token, and any object type or enumeration for a
`Type` cell. The engine resolves the token from the `BundleIndex` alone:

1. A bare token matches the bundle artifacts of that object type by `id`
   first, then by `names` (steps 2 and 3); two bundle artifacts matching at
   the same level are `semantic.ambiguous-type` naming both. A bundle match
   maps to `ix://<org>/<repo>/type/<Name>`.
2. If a bare token matches no bundle artifact, the engine matches it against
   `importedArtifacts` entries of that object type by `id` first, then by
   `names`, across every imported package. One match maps to
   `ix://<package>/type/<Name>` of the entry's `package`; matches in two or
   more packages (or two entries of one package at the same level) are
   `semantic.ambiguous-type` naming each package and artifact. An own-package
   match therefore always wins over an imported one, and an imported match
   never changes when a different package is added unless both carry the
   token.
3. Under a non-empty index (rule 5), a qualified token consults only the
   artifacts of its `<package>`: the
   bundle's artifacts when `<package>` is the module's own package, otherwise
   the `importedArtifacts` entries whose `package` equals it, by `id` then
   `names` as above. Each of these maps to the placeholder: a package that
   the module's `imports` does not name (`reason` `unknown-token`); an
   imported package that no `importedArtifacts` entry carries, because no
   loaded corpus provides it (`import-unresolved`); and an imported package
   with entries but none of the object type with that name (`unknown-token`).
4. A match never crosses an owner: the identity names the package that owns
   the artifact, so a `Time` dimension of package `acme/a` and a `Time`
   dimension of package `acme/b` are the identities `ix://acme/a/type/Time`
   and `ix://acme/b/type/Time`, and the engine neither merges, aliases nor
   compares them. Whether two such dimensions are the same is decided by the
   consumer from the owner-qualified identities (QSpec FR-142 and FR-322
   "Model-owned members"), never by the engine.
5. The placeholder segment `<Token>` is the `Identifier` the token names
   (`u` for `Quantity<u>`, the name after `::` for a qualified token); the
   advisory's message carries the token as written. Each unresolved token has
   exactly one `reason`, chosen in this precedence: `no-bundle-index` when the
   index holds no object, enumeration or imported artifact (this applies to a
   qualified token too, whatever the module's `imports` name, so rule 3's
   `unknown-token` and `import-unresolved` never apply under an empty index);
   otherwise, for a qualified token, the one rule 3 gives; otherwise
   `unknown-token`. A bare token never has the reason `import-unresolved`.

A token that is not an `Identifier` or a qualified token is
`semantic.invalid-type-token` for every kind.

Matches at steps 2..4 map to `ix://<org>/<repo>/type/<Name>` where
`<org>/<repo>` is the bundle's package (the `SemanticContext` `package`
segments); step 5 maps to `ix://<package>/type/<Name>`. The engine
maps any other token to the placeholder `ix://<org>/<repo>/unresolved/<Token>`
with the advisory `semantic.unresolved-type` at the row carrying `reason`
`no-bundle-index` (empty index, which takes precedence), `unknown-token`, or
`import-unresolved` (rule 5);
a token that is not a semantic-core `Identifier` is instead the error
`semantic.invalid-type-token`. The engine SHALL NOT emit a bare string for an
unresolved token.

Multiplicity cells: `n` and empty map to `{ lower: n, upper: n }` (empty is
`1`); `n..m` to `{ n, m }`; `n..*` to `{ lower: n }`; `*` to `{ lower: 0 }`;
the flags `ordered` and `unique` are admitted only when `upper` is absent or
greater than 1; `lower` greater than `upper`, a non-integer bound, or a flag
on a single value is the error `semantic.invalid-multiplicity` at the row.

Constraints cells: entries separated by commas outside a `/…/` pattern;
`keyword: value` pairs and bare flags from the closed set `min`, `max`,
`exclusiveMin`, `exclusiveMax`, `minLength`, `maxLength`, `pattern /…/`
(the value runs from the first `/` to the last `/` of the entry, carried
opaque with `dialect: ecma-262`, never compiled), `enumValues a|b`
(`|`-separated, no spaces), `nonEmpty`, `unique`, `format ns:name`, and the
field flags `identity` and `nullable`; an unknown keyword is the error
`semantic.unknown-constraint-keyword` at the row; a value that is empty or
not a finite number where one is required is
`semantic.invalid-constraint-value`.

Reader rules carried from semantic-core: a duplicate field name is
`agent-ix.semantic-core.DUPLICATE_NAME` at the second row; `identity` on a
field whose multiplicity is not `1..1` is `IDENTITY_NOT_SINGLE`; `identity`
on `JsonObject` is `IDENTITY_ON_JSON_OBJECT`; `decimal` on a non-`Decimal`
target is `DECIMAL_ON_NON_DECIMAL`; flags on a non-collection are reported
as `semantic.invalid-multiplicity` (the multiplicity grammar refuses them
before the reader rule would).

Fence form: the engine SHALL read the fence line by line;
`attribute <name> : <Type>[<mult>]` and `ref item <name> : <Type>[<mult>]`,
each with an optional trailing `{ … }` whose content is taken as the
`Constraints` cell text and is otherwise opaque; any other non-blank line
(`item`, `part def`, `:>`, …) is the error `semantic.sysml-outside-subset` at
that line.

Normalization: the table and fence forms of the same content SHALL produce
identical normalized `FieldDecl[]`. Every produced entry SHALL validate
against the embedded `FieldDecl.json`; an entry that does not is an engine defect and SHALL fail extraction
with `semantic.internal-invalid-decl`, never be dropped.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-070-CON-1 | The engine SHALL recognize fences line by line and carry brace content and pattern text as opaque strings, never parsed or compiled. | Architecture | Test |
| FR-070-CON-2 | The engine SHALL resolve types from the `BundleIndex` and the loaded modules only, with no filesystem walk and no network read on this path. | Architecture | Test |
| FR-070-CON-3 | For an artifact of a module without a `semantic` block, the extracted record SHALL equal the checked-in pre-change baseline byte-for-byte and carry no `semantic` key. | Compatibility | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-070-AC-1 | Under `config-version.bundle.json`, the vendored `config-version.table.md` extracts to the normalized `fields` of `config-version.expected.json`, every entry validating against `FieldDecl.json`. | Test |
| FR-070-AC-2 | The vendored `config-version.fence.md` extracts to the identical normalized array with `fieldsForm: fence`. | Test |
| FR-070-AC-3 | `both-forms.md` yields `fields` `unavailable` (`both-forms`) and one `semantic.properties-both-forms` error at the fence start line recorded in `both-forms.expected.json`; a typed table followed by a bullet list yields the same code at the list's line; two `## Properties` headings yield `semantic.duplicate-section`. | Test |
| FR-070-AC-4 | Every `Type` case in `cell-cases.json` yields the expected target, `decimal`, `unit`, placeholder, or diagnostic at the row; a token matching two bundle names yields `semantic.ambiguous-type` naming both; an empty index yields `unresolved` with reason `no-bundle-index`; a non-`Identifier` token yields `semantic.invalid-type-token`. | Test |
| FR-070-AC-5 | Every `Multiplicity` case in `cell-cases.json`, plus `*`, `0..*`, `0..0`, `2..2 unique`, and `a..b`, yields the expected `Multiplicity` or `semantic.invalid-multiplicity` at the row. | Test |
| FR-070-AC-6 | Every `Constraints` case in `cell-cases.json`, plus `nullable`, `pattern: /a,b|c/`, and a duplicated field name, yields the expected constraints, flags, or diagnostic at the row, with the pattern text carried verbatim. | Test |
| FR-070-AC-7 | Every fence-line case in `cell-cases.json` yields `semantic.sysml-outside-subset` at the fence line, and a fence line with brace content of arbitrary UTF-8 text maps that text as the `Constraints` cell. | Test |
| FR-070-AC-8 | The pinned `FR-006-config-version-entity.md` copy, `legacy-bullets.md`, and `legacy-mixed.md` yield the form, line, and warning of `legacy.expected.json`, with `properties` unchanged and `fields` `unavailable` (`legacy-form`); under `legacy_forms: error` the same finding is an error. | Test |
| FR-070-AC-9 | An artifact of a module without a `semantic` block yields a record equal to the checked-in baseline byte-for-byte, with no `semantic` key and no semantic diagnostic. | Test |
| FR-070-AC-10 | A row error in any one row yields `fields` `unavailable` with reason `row-errors` naming every erroring locus and no partial array; a typed header with zero rows yields `available` and empty. | Test |
| FR-070-AC-11 | A `Type` cell `Clock` yields the `FieldDecl` target `Clock` with no `decimal` and no `unit`, in the typed table and as `attribute c : Clock` in the fence with identical normalized entries; the entry validates against `FieldDecl.json` with its target read as `ix://quire/native/Clock`, and the record validates against `semantic-v1.schema.json`; `Clock [ms]` yields `agent-ix.semantic-core.UNIT_NOT_ALLOWED`, `Clock(1,2)` yields `semantic.invalid-type-token`, and a bundle object titled `Clock` does not change the verdict. | Test |
| FR-070-AC-12 | Under a bundle whose artifact `ms` has `object: unit` and whose artifact `Span` has `object: entity`, a `Type` cell `Quantity<ms>` yields the target `ix://<org>/<repo>/type/ms`; a unit artifact whose title is `Millisecond` resolves from `Quantity<Millisecond>` to the same target; `Quantity<hour>` (no artifact), `Quantity<Span>` (an entity) and `Quantity<ms>` under an empty index yield the placeholder `ix://<org>/<repo>/unresolved/hour`, `.../Span` and `.../ms` with `semantic.unresolved-type` reasons `unknown-token`, `unknown-token` and `no-bundle-index`; two unit artifacts both named `Millisecond` yield `semantic.ambiguous-type` naming both; a bare `Quantity` cell resolves by steps 2..5. | Test |
| FR-070-AC-13 | Each of these `Type` cells yields exactly the stated verdict at the row, with `fields` `unavailable` (`row-errors`): `Quantity<>`, `Quantity< ms >`, `Quantity<m-s>`, `Quantity<Quantity<ms>>`, `Quantity<ms>x`, `Quantity<ms>(1,2)`, `Quantity<a::b::c>`, `Quantity<::ms>`, `Quantity<acme::ms>` and `Quantity<acme/rpc::>` yield `semantic.invalid-type-token`; `Quantity<ms> [s]` yields `agent-ix.semantic-core.UNIT_NOT_ALLOWED`; a `ref item r : Clock` and a `ref item r : Quantity<ms>` yield `semantic.sysml-outside-subset` at the fence line; the resolution reads only the `BundleIndex`, with no filesystem or network read. | Test |
| FR-070-AC-14 | Under a module whose `imports` name `acme/time-units` and `acme/other-units`, and a `BundleIndex` whose `importedArtifacts` hold unit `ms` of package `acme/time-units` (title `Millisecond`), unit `ms` of package `acme/other-units`, and dimension `Time` of `acme/time-units`, and whose own bundle holds unit `s`: `Quantity<s>` yields `ix://<org>/<repo>/type/s`; `Quantity<Millisecond>` yields `ix://acme/time-units/type/ms`; `Quantity<acme/time-units::ms>` yields `ix://acme/time-units/type/ms` and `Quantity<acme/other-units::ms>` yields `ix://acme/other-units/type/ms`; the bare `Quantity<ms>` yields `semantic.ambiguous-type` naming both packages; once the own bundle also holds a unit `ms`, bare `Quantity<ms>` yields the own identity; `Quantity<acme/time-units::Time>` (a dimension) and `Quantity<acme/none::ms>` (package not in `imports`) yield the placeholder with `unknown-token`, and a package in `imports` (for example `acme/missing`) that no corpus provides yields `import-unresolved`; a dimension token `Time` yields `ix://acme/time-units/type/Time`, and a dimension `Time` also held by the own bundle yields the own identity, distinct from the imported one in the same record; with `importedArtifacts` absent and a non-empty own bundle, `Quantity<acme/time-units::ms>` yields the placeholder with `import-unresolved`; under an entirely empty index every unresolved token, qualified or bare, has the single reason `no-bundle-index`: `Quantity<acme/time-units::ms>` (package in `imports`), `Quantity<acme/none::ms>` (package not in `imports`) and `Quantity<ms>` each yield the placeholder with `no-bundle-index` and never `unknown-token` or `import-unresolved`. | Test |

## Dependencies

- **Upstream**: [FR-069](./FR-069-semantic-module-contract-at-load.md), [FR-025](./FR-025-spec-corpus-model.md), [FR-011](./FR-011-body-extraction-dsl.md); `agent-ix/quoin` FR-071/FR-074
- **Downstream**: [FR-071](./FR-071-clause-and-operation-extraction.md), [FR-072](./FR-072-semantic-extraction-surface.md); `agent-ix/filament-core-data#36`; `agent-ix/filament-core-data` FR-092 and FR-146 (extraction frontend)
