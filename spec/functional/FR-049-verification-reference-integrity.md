---
id: FR-049
title: "Verification-Reference Integrity"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-038"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-057"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-017"
    type: "implements"
---
# FR-049: Verification-Reference Integrity

## Description

Bundle validation ([FR-038](./FR-038-okf-bundle-validation.md)) SHALL check
that document references minted inside table cells — e.g. the ISO AC
`Verification` cell annotation `Test (TC-035)` — resolve to a real target, the
same way `ix://` edges are checked for dangling targets today
(`dangling-reference` in `src/corpus/validate.rs`).

The engine SHALL NOT hardcode the annotation pattern, the referencing column,
or the target archetype: all three come from the active module's declared
**traceability model** ([FR-050](./FR-050-declarative-coverage-computation.md)
`traceability:` manifest section). For each *document-reference declaration* in
that model (archetype + section + column + annotation pattern + target kinds),
`validate_bundle` SHALL extract every referenced id from the declared cells.
The engine SHALL resolve each extracted id against the bundle's **resolution
set** for the declaration's target kinds.

The resolution set for a target kind SHALL be the union of:

- ids of bundle documents whose archetype matches the declared target
  archetype; and
- trace ids minted by a declared auxiliary trace source — a file (e.g. the
  repo Test Matrix, `spec/tests.md`) that the corpus walk excludes as a
  non-artifact ([FR-038](./FR-038-okf-bundle-validation.md)) but that the
  model declares as minting trace ids. The engine SHALL harvest such files
  with a targeted scan, following the glossary-harvester pattern
  ([FR-044](./FR-044-project-glossary-lexicon.md)).

If a referenced id resolves to no member of the resolution set, then
`validate_bundle` SHALL report a `dangling-trace-reference` finding carrying
the referencing document path, the unresolved id, and the declaration that
minted the reference. The finding SHALL be posture-degradable exactly like
`dangling-reference`: an error under the `Strict` posture and a warning under
`Okf`.

When the active modules declare no traceability model, the check SHALL emit no
findings.

### Kind-typed references (PLAT-1471)

A cell whose allowed target set depends on a sibling cell of the same row — an
anomaly-assessment row whose `Kind` is `Requirement`, `Acceptance criterion` or
`Hazard` and whose `Target` must be an id of that kind — cannot be checked by
`targets` alone, which names one flat set. A document-reference declaration
MAY therefore carry two additional fields, both module data, neither
hardcoded in the engine:

- `kind_column` — the header of a sibling column in the same table as the
  declaration's `column`; and
- `kind_targets` — a map from a kind value (the text of that sibling cell) to
  a non-empty list of target names, each of which is also named in the
  declaration's `targets`.

The declaration's `targets` stays the **union** set. For a kind-typed
declaration the engine SHALL resolve each extracted id first against the union
(unchanged: an id absent from it is `dangling-trace-reference`), and then, for
an id that resolves in the union, against the **kind subset** — the resolution
set of the targets `kind_targets` lists for the row's kind value. An id that
resolves in the union but not in the kind subset SHALL yield one
`reference-kind-mismatch` finding carrying the referencing document path, the
id, the row's kind value, the target names the kind allows, and the
declaration. An id that is dangling never also yields a kind mismatch.

Kind matching SHALL trim the cell and compare ASCII case-insensitively. A row
whose kind cell is empty, or names a value absent from `kind_targets`, SHALL be
checked against the union only: it can dangle but cannot mismatch, because no
subset is declared for it. This check does not validate the Kind column's
values; that belongs to the declaring module's own table grammar or lint
rules, and a misspelt or multi-valued kind therefore surfaces there, not here.
A table that has the `column` header but lacks the `kind_column` header SHALL
likewise be checked against the union only, and the engine SHALL report one
warning-tier `kind-column-matches-nothing` finding per such table, naming the
document path and the declaration, so a misspelt `kind_column` cannot disable
kind checking silently. It is registrable as
`trace:kind-column-matches-nothing`, unconfigured a warning in both postures
(the `archetype-matches-nothing` tier). A cell that yields no pattern match —
a `-` placeholder, an empty cell — extracts no id and yields no reference and
no finding; whether a row must carry a target is not this check's concern.
Each id of a multi-id cell is classified independently. Findings are ordered
by the existing key extended with the reason — document path, id, reason,
message — so a declaration without kind fields orders exactly as before; each
id occurrence yields at most one finding.

`reference-kind-mismatch` is posture-degradable exactly like
`dangling-trace-reference`: an error under `Strict`, a warning under `Okf`, and
registrable as `trace:reference-kind-mismatch`
([FR-057](./FR-057-corpus-check-severity.md)).

`kind_column` and `kind_targets` SHALL be declared together or not at all.
Module load SHALL reject, naming the declaration and the field at fault, a
`kind_column` without `kind_targets` or the reverse; an empty `kind_column`; a
`kind_column` equal to `column`; an empty `kind_targets` map; a kind key that is
empty after trimming or that duplicates another key under the matching rule; a
kind whose target list is empty; and a listed target name that is not in the
declaration's `targets`. A rejected declaration is never partially applied.

### Several tables under one declared section (PLAT-1491)

A declared section may hold more than one table (a prose line, a note or a
second grid between them). The declaration names the section, not a table, so
`validate_bundle` SHALL read **every** table under each section the declaration
matches, in document order, and extract every referenced id from the declared
column of each. Reading only the first table would leave later rows
unchecked and report a clean bundle: a `dangling-trace-reference`,
`reference-kind-mismatch` or `kind-column-matches-nothing` finding that a
row-for-row identical table under a later section produces would be missing,
and no diagnostic would say a table was skipped. Reporting the extra table as
an error, as [FR-076](./FR-076-relationships-extraction.md) does for a
`## Relationships` section, is not the rule here: that section is a lowering
whose output is one declaration, whereas a reference declaration is a read of
cells, so a second table is data to read, not malformed input.

Each table is read on its own: the declaration's `column`, `row_id_column`
and `kind_column` headers are matched against that table's headers
(FR-049-AC-18), a table lacking the `column` header contributes no rows and no
finding (a legend or summary grid is legitimate), and each finding carries its
own row's line (FR-049-AC-19..22). A table inside a fenced code block is an
example, not data, and is not read; this applies to every table the reference
scan reads, so a document whose only table sits in a fence is no longer scanned.

The rule is not this requirement's bundle check alone: a trace target's
`id_column` mint, an obligation source and the
[FR-050](./FR-050-declarative-coverage-computation.md) rollup (its
`unbacked_rows`, row binding and status-column check) read **every** table
under a declared section, outside fenced code blocks, by the same rule, so the
rollup's row population equals the ids that are minted and the obligations
that are derived, and a reference to an id that appears only in a later table
resolves. A table lacking the declared id column contributes no rows. The
`id-column-matches-nothing` and status-column findings describe the section,
not a table: they are raised on the section's first table, and only when no
table under that heading carries the column, so a legend or summary grid after
the matrix raises none (FR-049-AC-29, AC-30).

### Located findings (PLAT-1476)

The reference scan already reads each referencing row together with its
1-based document line (frontmatter included, the numbering `validate` findings
use). `BundleFinding` SHALL gain an optional `line` (`Option<usize>`): `Some`
for a `dangling-trace-reference` and a `reference-kind-mismatch` finding,
holding the line of the referencing table row in the referencing document, and
a document-level result bridged into the bundle report (schema, assert and record errors and warnings, [FR-057](./FR-057-corpus-check-severity.md)-CON-1) carries the line of the document error or warning it bridges (its own `Option<usize>`, unchanged), and every other pack finding has `None`. The bundle path therefore drops no line a document-level check produced, including the located `semantic.record-invalid` findings of [FR-069](./FR-069-semantic-module-contract-at-load.md). The row is the unit the scan knows; it
records no cell offset, so the finding names a row line and no column. Every id
of a multi-id cell takes the row's one line. How a surface renders the line is
that surface's concern, not this requirement's.

The line joins the finding's sort and de-duplication key, which becomes
(document path, id, reason, message, line). A declaration without
`row_id_column` previously collapsed the same id dangling on two rows of one
document into one finding; it now yields one finding per row, so corpus counts
of `dangling-trace-reference` may rise without any new defect. A declaration
without such repeats orders as before.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-049-AC-1 | A bundle whose AC `Verification` cell references a `TC` id present in the bundle (a TC document or a declared trace-source row) validates with no `dangling-trace-reference` finding. | Test |
| FR-049-AC-2 | A `Verification` cell referencing a TC id absent from the resolution set yields one `dangling-trace-reference` finding carrying the document path and the unresolved id. | Test |
| FR-049-AC-3 | The finding is posture-degradable: the same dangling reference is an error under `Strict` and a warning under `Okf`. | Test |
| FR-049-AC-4 | The annotation pattern and referencing column come from the declared model: a fixture module declaring a different pattern/column resolves references by its own declaration, with no ISO-specific behavior in the engine. | Test |
| FR-049-AC-5 | A declared auxiliary trace source outside the corpus walk (a `tests.md`-style matrix) contributes its minted trace ids to the resolution set via a targeted scan. | Test |
| FR-049-AC-6 | With no traceability model declared by any active module, `validate_bundle` emits zero `dangling-trace-reference` findings for any input. | Test |
| FR-049-AC-7 | A cell bearing multiple annotations (`Test (TC-035, TC-036)`) resolves each id independently and reports only the unresolved ones. | Test |
| FR-049-AC-8 | Findings are deterministic: repeated validation of the same bundle yields the same findings in the same order. | Test |
| FR-049-AC-9 | With the corpus walked from a document root nested under the scope, model-declared `document:` paths and `exclude:` globs resolve against the **reference root** (the scope), not the document root: a `document: spec/tests.md` target mints when `validate_bundle` receives the two roots separately, and un-mints — every reference to it dangling — when the roots are conflated (CR-045). | Test |
| FR-049-AC-10 | A declaration with `kind_column` and `kind_targets` and a row whose id resolves inside the row's kind subset yields no finding; the same id on a row whose kind maps to a different target set yields exactly one `reference-kind-mismatch` finding carrying the document path, the id, the kind value, the allowed target names and the declaration. | Test |
| FR-049-AC-11 | An id absent from the union of targets yields `dangling-trace-reference` only, never also `reference-kind-mismatch`, in a kind-typed declaration. | Test |
| FR-049-AC-12 | `reference-kind-mismatch` is posture-degradable like FR-049-AC-3: the same mismatch is an error under `Strict` and a warning under `Okf`, and `trace:reference-kind-mismatch` is registrable per FR-057. | Test |
| FR-049-AC-13 | A row whose kind cell is empty or names a value absent from `kind_targets`, and a table lacking the `kind_column` header, is checked against the union only: a resolvable id yields no finding and an unresolvable id yields `dangling-trace-reference`. Kind values match after trimming and ASCII case folding. | Test |
| FR-049-AC-18 | A table with the declaration's `column` header but no `kind_column` header yields exactly one `kind-column-matches-nothing` finding naming the document path and declaration, a warning under both `Strict` and `Okf` when unconfigured, mappable as `trace:kind-column-matches-nothing`; a table with both headers, and a table with neither, yields none. | Test |
| FR-049-AC-19 | `BundleFinding` has an optional `line`; a `dangling-trace-reference` finding carries `Some(line)` with the 1-based document line of the referencing table row (frontmatter lines counted); every unresolved id of a multi-id cell carries the same row line; the same id dangling on two rows of one document yields two findings at the two rows' lines (one finding before this change); every pack finding other than these two has `line` `None`. | Test |
| FR-049-AC-20 | A `reference-kind-mismatch` finding carries the line of the referencing row exactly as AC-19, for a row whose kind cell and target cell sit on the row's line; a row below another table or section of the same document reports its own line, not the table header's or the section heading's. | Test |
| FR-049-AC-21 | Located findings remain deterministic: repeated validation yields the same findings in the order (path, id, reason, message, line), and a fixture module declaring different column names and a different table position yields lines by its own declaration, with no archetype or column name in the engine. | Test |
| FR-049-AC-22 | A document-level error or warning bridged into the bundle report keeps its line: a bundle over a document whose `validate_document_in_registry` result carries an error at line N yields a bridged finding with `line` `Some(N)`, and one whose result has no line yields `None`; a `semantic.record-invalid` finding of FR-069 reaches a `validate_bundle` caller at the same line it has from `validate_document_in_registry`. | Test |
| FR-049-AC-23 | Rows of a second table under the same declared section are scanned like the first: with table 1 holding `K-1 \| north \| L-1` (resolving), a prose line, and table 2 holding `K-2 \| north \| R-1` (target outside the kind's targets) and `K-3 \| north \| L-7` (resolving nowhere), the bundle yields one `reference-kind-mismatch` and one `dangling-trace-reference` finding for the table-2 rows, and the same rows under a later section yield the same two findings. | Test |
| FR-049-AC-24 | Each finding from a later table carries the 1-based document line of its own row (frontmatter lines counted), not that of the first table or the heading, and a bundle whose rows sit in one table, with or without an unrelated further table, reports exactly the findings and lines it reported with the first table alone (a single dangling row and a single kind-mismatch row yield those two findings at their own lines). | Test |
| FR-049-AC-25 | The `column` and `kind_column` headers are matched per table: a second table with the `column` header and no `kind_column` header yields one `kind-column-matches-nothing` finding for that table only, a second table with neither header yields no finding and contributes no rows, and a first table with both headers is unaffected. | Test |
| FR-049-AC-26 | The FR-050 rollup reads every table of a declared section: a row of a second table adds an `unbacked_rows` entry exactly as the same row in the first table does, and a second table lacking the status column (a legend grid) raises no status-column finding. | Test |
| FR-049-AC-27 | A table inside a fenced code block (CommonMark rules: a fence closes only with the same character and at least the opening length, and a backtick fence's info string holds no backtick) under a declared section, after the real table or alone, yields no row and no finding, and the real table is unaffected. | Test |
| FR-049-AC-28 | An id that appears only in a second table of a trace target's section (outside a fenced code block) is minted: a reference to it from a declared cell yields no `dangling-trace-reference`, while the same id in a fenced table still dangles. | Test |
| FR-049-AC-29 | An obligation source derives obligations from every table under its declared section (outside fenced code blocks): a row of a second table yields its obligation, with the same id and statement as the same row in the first table. | Test |
| FR-049-AC-30 | `id-column-matches-nothing` and the status-column finding are raised per section, on its first table: a section whose first table lacks the column while a later table carries it raises none, a section where no table carries it raises exactly one, and a later table lacking the column raises none. | Test |
| FR-049-AC-14 | A target cell that matches no pattern (`-`, empty) extracts no id and yields no reference and no finding, whatever the row's kind; a multi-id cell classifies each id independently and reports only the mismatched ones. | Test |
| FR-049-AC-15 | Module load rejects, naming the declaration and the field at fault and applying no part of it, each malformed kind declaration: `kind_column` without `kind_targets` or the reverse, an empty `kind_column`, `kind_column` equal to `column`, an empty map, an empty or duplicate kind key, an empty target list, and a listed target not in `targets`. | Test |
| FR-049-AC-16 | A declaration without `kind_column` and `kind_targets` behaves exactly as before: the engine contains no kind name, and a fixture module declaring different kind values and column names is resolved by its own declaration. | Test |
| FR-049-AC-17 | `reference-kind-mismatch` findings are deterministic: repeated validation of the same bundle yields the same findings, interleaved with `dangling-trace-reference` findings, in the order (path, id, reason, message). | Test |

> **CR-045 note (2026-08-15):** `validate_bundle` gains the same two-root
> split `compute_coverage` has always had via its `root` parameter: a
> `document_root` (locates the root `index.md`) and a `reference_root` (the
> base for the model's `document:`/`exclude:` paths, which modules author
> against the repository scope — `spec/tests.md`). Found while landing the
> #91 CLI derivation: bounding the OKF bundle root to `<scope>/spec` with a
> single conflated root silently un-minted every path-bound trace target,
> surfacing as 123 new `dangling-trace-reference` findings on this repo's
> own spec. `validate_bundle_at` keeps single-root semantics for
> self-contained bundles (agent-ix/quire-rs#91, umbrella #90).

> **CR-419 note (PLAT-1500):** FR-049-AC-23..28 left minting, obligation
> sources and the FR-050 rollup reading a declared section's first table only,
> so an id appearing only in a later table was never minted and a reference to
> it dangled. All three now read every table outside fenced code blocks, the
> same rule as the reference scan. AC-26 and AC-28 are reversed, AC-29 and
> AC-30 are new. The section-level findings stay per section so a legend grid
> raises none.

## Dependencies

- **Upstream**: [FR-057](./FR-057-corpus-check-severity.md) (the severity registry AC-12 relies on), [FR-050](./FR-050-declarative-coverage-computation.md) (the declared traceability model), [FR-038](./FR-038-okf-bundle-validation.md) (bundle validation and postures), [FR-025](./FR-025-spec-corpus-model.md) (the corpus)
- **Downstream**: `spec-artifacts-iso` declares the ISO `Verification`-cell model (follow-up change in that module); the `gap-analysis` workflow consumes the findings
