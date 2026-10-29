---
id: FR-053
title: "Obligation Record"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-052"
    type: "references"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-014"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-017"
    type: "implements"
---
# FR-053: Obligation Record

## Description

`quire-rs` SHALL derive an **obligation record** for every normative statement a
module declares as one, and SHALL surface those records on the existing JSON
interfaces. An obligation record carries the statement's id, a deterministic
the declared verification method, any declared
parameters, and the declared criticality.

ADR 0011 names the obligation record the contract between **quire**, which
states obligations, and **quoin**, which audits their discharge. Neither side
has such a record today. `quire properties --json` carries property *shapes*;
[FR-050](./FR-050-declarative-coverage-computation.md) coverage carries the
AC→TC *rollup*. Neither carries a verification method, so nothing can enforce **method
conformance**, where a requirement whose declared method is analysis is
discharged by a unit test.

An obligation is **derived, never authored**. There is no obligation document
and no `quire obligations` command. Choosing an acceptance criterion's
verification method *is* minting the obligation; the engine reads what the
author already wrote.

### Obligation sources are module data

The engine SHALL NOT know that `Verification` is a column, that `Test` is a
method, or that an acceptance criterion is the thing being obliged. A module
declares its obligation sources under the existing `traceability:` model:

```yaml
traceability:
  obligations:
  - name: acceptance-criterion
    target: acceptance-criterion       # reuses a declared trace_target
    statement_column: Criteria
    method_column: Verification
    criticality_column: Priority       # optional
  - name: nfr-metric
    archetype: NFR
    section: Measurement and Evaluation
    id_format: "{document}-M-{row}"
    statement_column: Metric
    method_column: Method
    parameters:
      target: Target
      threshold: Threshold
```

Each entry SHALL resolve its minting documents in exactly one of two ways, and
declaring both or neither SHALL be a load-time error:

- **`target:`** — names an existing
  [FR-050](./FR-050-declarative-coverage-computation.md) `trace_target`. The
  archetype, section and id column come from it, so an acceptance criterion is
  not declared twice and the obligation id is by construction the same id the
  coverage rollup and every trace tag already key on.
- **`archetype:` + `section:` + `id_format:`** — for a table whose rows mint no
  id of their own. The NFR `Measurement and Evaluation` table is the motivating
  case: 19 of 19 NFRs in this repository carry one, every row is a quantified
  obligation (metric, threshold, method), and no row has an `ID` column. The
  `id_format` template SHALL support `{document}` (the owning document's id) and
  `{row}` (the 1-based row ordinal), so `NFR-006-M-2` is derived rather than
  authored.

### The second source is not an afterthought

An NFR measurement row states one number three times over: the spec threshold,
the benchmark assertion, and the evidence gate are the same quantity. Modelling
it as an obligation is what lets a performance budget be discharged, ratcheted
and reported by the same machinery as a functional criterion, rather than by a
parallel mechanism that drifts. `parameters:` exists for exactly this — a map of
result key to source column, so `{target: "< 5ms", threshold: "< 8ms"}` travels
with the obligation instead of being re-parsed downstream. It is equally the
carrier for a t-way strength or a mutation-score floor a later method needs.

### Surface

Records SHALL travel on the existing interfaces, adding no command:

- `classify_document_criteria` records gain an `obligation` field, so
  `quire properties --json` carries the obligation beside the property shape
  that criterion already has. A criterion its module declares no obligation
  source for carries `obligation: null` — classification is unchanged either
  way.

  This nested form carries `{source, method, parameters,
  criticality}` and **not** `id`, `statement` or `document`. Those three are
  already on the record or its enclosing object — `row_id`, `statement`, and
  `documents[].document` — and repeating them would put the same fact in two
  places on one payload, which is the shape that eventually disagrees with
  itself. It is also what keeps the classification API text-only: a criterion's
  method and criticality are all readable from the criterion's own row,
  where a document path is not.
- The [FR-050](./FR-050-declarative-coverage-computation.md) coverage report
  gains an `obligations` list — ordered by source **declaration** order, then
  document path, then row ordinal (AC-9) — so the rollup and the obligations are
  one read rather than two. The list is skipped when empty, so a module
  declaring no sources emits a byte-identical payload (AC-11).

  > **CR-063** (2026-08-17) corrected this paragraph. It said "ordered by id",
  > which no code ever did and which contradicts AC-9 in the same document; and
  > AC-11 said the empty list is "present rather than absent", which contradicts
  > its own test, the `skip_serializing_if` attribute and the published schema.
  > Three statements of one fact, two of them wrong, is what a single source
  > exists to prevent.

Neither payload SHALL gain a version key
([FR-008](./FR-008-byte-exact-slicing.md)-AC-5 stands); contract versioning lives in
the published schema artifacts.

## Inputs

- A loaded `Spec` corpus and a `Registry` whose merged traceability model
  declares one or more `obligations:` entries.
- For a `target:` entry, the resolved `TraceTarget` it names.

### Selected-inventory entry point (PLAT-1165)

The additive public Rust entry point is `obligation::inventory(root, model,
source_names, stop_limit)`. It accepts a filesystem document root, a borrowed
`TraceabilityModel`, a **nonempty** list of existing obligation-source names and
an optional nonnegative emitted-entry stop limit. It does not accept a `Spec`,
`RepoLoad`, caller-supplied rows or a caller-supplied completion value. The owner
loads the selected corpus through its own walk and parser. An absent limit
requests normal exhaustion; a limit requests the stopping policy below.

The existing `derive(&Spec, root, &TraceabilityModel)` entry point remains
unchanged and does not acquire a completion guarantee.

## Outputs

- `Obligation { source, id, document, statement, method,
  parameters, criticality }` — one per minting row, on
  `CoverageReport::obligations`.
- `CriterionObligation { source, method, parameters,
  criticality }` on `AcClassification::obligation` — the same facts minus the
  three the enclosing record already carries.

### Selected-inventory result (PLAT-1165)

The additive entry point returns an owner-issued `SelectedInventory`: the
requested inputs, a validated selection identity when validation succeeds,
borrowable origin-aware entries, observed entry count, typed causes retaining
partial rows, and terminal `Complete` or non-`Complete`. Its completion state is
not publicly constructible from a caller `Spec` or deserialized row list.
Borrowed entry and selection views remain tied to this immutable owner result.

## Behavior

For each declared obligation source, in declaration order, the engine SHALL walk
the minting documents the source resolves to, read each row of the declared
section's table, and emit one record per row:

1. `id` — from the target's `id_column`, or rendered from `id_format`.
2. `statement` — the `statement_column` cell, verbatim and untruncated.
3. `method` — the `method_column` cell with any trailing parenthetical
   annotation removed, so a `Verification` cell reading `Test (TC-707)` yields
   method `Test` while [FR-049](./FR-049-verification-reference-integrity.md)
   continues to read `TC-707` from the same cell. The engine SHALL NOT validate
   the method against any vocabulary here — that is the catalog's job
   ([FR-054](./FR-054-verification-method-catalog.md)) and the auditor's.
4. `parameters` — a `BTreeMap` of declared key to cell value; absent keys are
   omitted, never defaulted.
5. `criticality` — the `criticality_column` cell, or absent when the module
   declares no such column. Criticality is genuinely optional today: the ISO FR
   acceptance-criteria contract is `ID | Criteria | Verification` and carries no
   priority column, so declaring one is a module's choice rather than a
   precondition for obligations to exist at all.

Ordering SHALL be by `(source declaration order, document id, row ordinal)` and
every map SHALL be a `BTreeMap`, so two runs over identical inputs serialize
byte-identically ([NFR-006](../non-functional/NFR-006-determinism.md)).

A row whose `statement_column` is empty or whose id cannot be rendered SHALL be
skipped with a diagnostic naming the document and row, never emitted as a record
with an empty statement — an obligation nothing can state is not an obligation.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-053-CON-1 | The engine SHALL NOT name a verification method, a column name, or an archetype anywhere on this path. Every one is module data, so an external user's catalog and vocabulary drive their obligations exactly as this ecosystem's drive ours. | Architecture | Inspection |
| FR-053-CON-2 | An obligation record SHALL NOT carry a verdict, a threshold comparison, an evidence reference or a freshness judgement. The engine states obligations; reading evidence and judging discharge is quoin's Auditor role (ADR 0011 invariant 1). | Architecture | Inspection |
| FR-053-CON-4 | Obligation derivation SHALL NOT construct a `GrammarFinding` or carry a [FR-048](./FR-048-per-check-grammar-severity.md) `grammar_severity` key, so no obligation can fail a build. A missing method is a report, not an error, until a consuming workflow decides otherwise. | Architecture | Test |

### The join the record used to drop

A criteria row states two things about verification: the **method** and the
**test cases** that discharge it — `Test (TC-707)`, `Eval (TC-EV-054, TC-EV-055)`.
The record carried the first and discarded the second, even though `method_of`
finds that same `(` to know where the method name ends.

That cost a consumer the join its own spec states. quoin's evidence store binds
by matching a run's trace ids against **obligation** ids; an adapter reporting a
**test case** id — an agent-eval report keys on the scenario, `TC-EV-057` — bound
nothing, while `FR-038-AC-8 → TC-EV-057` sat in the table the engine had just
read. Recovering it meant re-parsing the criteria table, which is the
duplication FR-050 exists to prevent (agent-ix/quire-rs#180).

`target_ids` is **additive** on the published v1 contract: absent on payloads
from an older engine, omitted where the cell names none, and not in `required`.
An unclosed parenthetical reads nothing rather than to end-of-cell — reading on
would turn a typo into a plausible-looking id that binds to nothing and looks
like a real target.

### Selected criterion inventory — change request PLAT-1165

The selected-inventory API is an additive producer contract. Its implementation
and the acceptance checks below are **Planned** until the owning code and tests
land. The existing `derive(&Spec, root, &TraceabilityModel)` return value and its
existing consumers retain their current behavior.

The measured legacy tuple contains rows and skips, not proof that its corpus was
normally exhausted. A valid empty table, a missing root, a document missing its
selected section, an invalid unvalidated model and a caller-truncated `Spec` can
all yield no rows. The new API SHALL distinguish those outcomes without asking
an external consumer to re-read Markdown or interpret private scan diagnostics.

### Selection and captured identity

The inventory SHALL validate its document root, traceability model and source
selectors before issuing a valid selected-corpus identity. The selector list
must name at least one existing obligation-source declaration. An empty list,
unknown name or duplicate name is a typed invalid selection, not a valid empty
corpus. Selected declarations are resolved in their model declaration order,
regardless of the order in which the caller lists their names. The declarations
continue to own archetypes, sections, columns, ID rendering and exclusions. No
acceptance-criterion archetype, column name, verification method or QSL formal
clause identifier becomes an engine constant.

The inventory SHALL issue a selection identity whose equality basis consists
of the canonical document-root path, the resolved selected source declarations
in model order (including their target definitions and effective model/source
exclusions), and the selected-document snapshot. That snapshot is the ordered
set of scope-relative document paths and exact captured document bytes for
readable documents selected by at least one selected declaration after its
exclusions apply. A byte change anywhere in a selected document, including an
unrelated section, changes this identity even when its criterion rows are
unchanged. Adding, removing or renaming a selected document also changes it.
The same canonical root, resolved declarations and identical selected-document
snapshot yield equal identities on repeated loads; pointer addresses, clocks
and load order do not enter equality.

Readable, valid documents whose declared type matches no selected source are
**unselected nonmembers**. Their bytes do not enter the selected-document
snapshot, so changes confined to such a nonmember do not change the identity
while it remains a nonmember. A change that makes it selected does change the
snapshot. Documents excluded from every selected source and files outside the
document root likewise do not enter that snapshot; changes confined to those
files do not change identity. A changed selection or effective exclusion
contract changes identity even when the emitted records happen to be equal.

An external consumer SHALL be able to borrow the identity and check whether an
entry belongs to that exact selection. A different canonical root, resolved
source selection or selected-document snapshot SHALL produce a typed
source-selection mismatch. The identity is read-only owner data, not a
caller-authored revision token, file hash inventory or second release/version
ledger. Binding it to an IR/QSL/production source revision remains the downstream
source-author contract; the inventory does not judge evidence freshness or
verification success.

### Origin-aware entry identity and lookup

Each entry SHALL expose its source declaration, scope-relative document path
and a typed origin. An `AuthoredRow` origin retains the row's authored ID. A
`RenderedRow` origin retains the owner's rendered obligation ID and 1-based row
ordinal for an `archetype`/`id_format` source whose row authors no ID. A
`CombinatorialTable` origin retains the owner's rendered whole-table obligation
ID for the existing combinatorial source. Rendered origins SHALL NOT expose a
rendered ID as an authored row ID. Deleting an earlier rendered row may renumber
later rendered IDs under the existing `id_format` rule; this is not a promise
of a stable authored identity for those rows. Existing obligation IDs and their
statement/method/parameter/criticality/target data remain owner data.

An external consumer SHALL be able to borrow these origin-aware identities and
values and perform an exact lookup without parsing Markdown, reading private
diagnostics or substituting QSL `ClauseRef` for an authored criterion. Every
selected authored criterion remains an entry when it has no formal-clause,
source-symbol or test-case mapping. The presence of a declared target ID alone
is not verification evidence.

If two selected authored rows have the same `(source declaration, document,
authored row ID)`, then the inventory SHALL report a typed duplicate-identity
cause and a non-`Complete` terminal. It retains both conflicting rows, their
ordinals/loci and every otherwise emitted row as partial data. Exact lookup of
that identity SHALL return a typed ambiguity refusal rather than choosing the
first row; lookup of another unambiguous partial identity remains available.
The same authored row ID in different documents or declarations is not this
collision, because those identity members differ.

### Classification, partial results and causes

The inventory result SHALL retain the requested selection, emitted entries,
observed entry count, typed causes and terminal state. A validated selection also
retains its owner-issued identity; failed root/model/selector validation retains
the requested inputs and cause without issuing a valid selection identity.

The inventory SHALL apply the effective declared path exclusions before
classifying a discovered Markdown file. A path excluded from every selected
source contributes neither a selected entry nor a classification failure.
A non-excluded file that cannot be read or classified because its frontmatter
is absent or malformed is **unclassifiable**: the inventory cannot prove that it
is an unselected nonmember. The inventory SHALL retain a typed load or parse
cause for that path and return non-`Complete`. It SHALL NOT infer membership
from a filename or silently drop that file. A readable document with valid
frontmatter and a type matching no selected source is the healthy nonmember
control; it contributes no entry and does not by itself block completion.
These are additive inventory-boundary decisions, not changes to the existing
FR-006 parser or tolerant `load_repo` membership behavior.

A missing root, a non-directory root, a root or document read failure, an invalid
model or source selection, an unclassifiable non-excluded file, a selected
document missing its declared section/table, an absent required ID or statement
column, an unusable selected row, a duplicate authored identity or a walk error
SHALL produce a typed non-`Complete` result. The cause SHALL retain the original
underlying error or diagnostic and its known source, document and row repair
locus. Successfully emitted rows before or beside a failure SHALL remain
available as partial rows. The inventory SHALL NOT discard those rows or
present them as a complete population. Metadata advisories that do not prevent
classification or reading a selected criterion, such as an absent optional UUID,
remain visible without becoming evidence or a blocking criterion-read failure.

A valid root containing no selected documents, including one containing only
valid nonmembers, and a selected table with its required headers but no authored
rows are valid empty inventories. The inventory SHALL report count zero with
`Complete` for those inputs only after normal exhaustion. An empty selector
list or an absent required section/table/header is not a valid empty inventory.

### Normal exhaustion and stopping

When validation has succeeded and the walk and every selected source/document
scan have normally reached end-of-scan without a blocking cause, the inventory
SHALL issue terminal `Complete`. While a required end-of-scan has not been
observed or a blocking cause exists, the inventory SHALL issue non-`Complete`.
Neither a caller-created/truncated `Spec` nor empty row/skipped-row vectors
constitute that end observation.

An explicit stop limit counts successfully emitted entries after input
validation. When the limit is reached, the inventory SHALL stop before
requesting further scan progress, retain the accumulated entries, and return a
typed limit-stop cause with non-`Complete`. A limit of zero stops before scanning
even a genuinely empty corpus. For a selection with exactly N entries, limit N
returns non-`Complete` even when the Nth entry happens to be the last: reaching
the count is not observing the subsequent end-of-scan. A limit greater than N
permits normal end observation and `Complete` if no other blocking cause exists;
an absent limit does the same. An explicit interruption before end observation
also retains partial rows and its typed stop cause. No stop or limit relabels a
visited prefix as a smaller successful selection.

Normal enumeration retains declaration, relative document-path and row ordering.
Neither inventory creation nor failure reporting SHALL add a `GrammarFinding`,
change legacy `derive` behavior, or judge whether an obligation is discharged.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-053-AC-1 | A module declaring an obligation source with `target:` yields one record per row of the named trace target's table, each carrying the same id the coverage rollup mints for that row. | Test (TC-831) |
| FR-053-AC-2 | A module declaring an obligation source with `archetype:`, `section:` and `id_format:` yields one record per row of a table whose rows carry no id column, with ids rendered from `{document}` and the 1-based `{row}`. | Test (TC-832) |
| FR-053-AC-3 | An obligation source declaring both `target:` and `archetype:`, or neither, is rejected at manifest parse: the module contributes nothing, the load failure names the offending source, and no obligation is derived from the unexecutable declaration. | Test (TC-833) |
| FR-053-AC-5 | A `method_column` cell reading `Test (TC-707)` yields method `Test`, and the same cell continues to yield the reference `TC-707` to FR-049's integrity check, so the two readings of one cell do not interfere. | Test (TC-835) |
| FR-053-AC-6 | A source declaring `parameters:` yields a record whose parameter map carries one entry per declared key that has a non-empty cell, with absent keys omitted rather than present-and-empty. | Test (TC-836) |
| FR-053-AC-7 | A source declaring no `criticality_column` yields records whose criticality is absent, and the records are otherwise identical to those from the same source with a criticality column declared and empty. | Test (TC-837) |
| FR-053-AC-8 | A row whose statement cell is empty is skipped with a diagnostic naming the document and the row ordinal, and contributes no record. The diagnostic appears in the coverage report a consumer reads, not only in the derivation helper's return value. | Test (TC-838, TC-870) |
| FR-053-AC-9 | Two runs of obligation derivation over an identical corpus serialize byte-identically, and the record order is by source **declaration** order (never source name), then scope-relative document path, then row ordinal. | Test (TC-839, TC-872) |
| FR-053-AC-10 | A criterion whose module declares an obligation source over its archetype carries a populated `obligation` on its property classification record, matched to the criterion by row id; a criterion whose module declares none carries `obligation: null` and is otherwise field-for-field unchanged. | Test (TC-840) |
| FR-053-AC-11 | The record carries the test-case ids its method cell names, so a consumer binding evidence keyed on a test case can join without re-parsing the criteria table. A cell naming none carries none, and an unclosed parenthetical reads nothing. | Test (TC-935) |
| FR-053-AC-13 | The obligation nested on a classification record carries no `id`, `statement` or `document` key, because the record and its enclosing object already carry all three. | Test (TC-843) |
| FR-053-AC-11 | A coverage report over a corpus with declared obligation sources carries one `obligations` entry per minting row, and a corpus with no declared sources carries an **absent** `obligations` key — the empty list serializes away — so FR-050-AC-7 byte-identity holds for every module that has not adopted them. | Test (TC-841) |
| FR-053-AC-14 | An `exclude:` glob on an obligation source removes the matching document's rows from **both** obligation surfaces — the coverage rollup and the `obligation` nested on a property classification record — so the two never disagree about whether a row states an obligation. Content with no path (stdin) matches no glob. | Test (TC-873) |
| FR-053-AC-15 | A valid document root, model and nonempty existing source-name selection yield the public owner-issued SelectedInventory and borrowable selection identity; empty, unknown or duplicate selectors and an invalid model yield typed non-Complete results, and the entry point accepts no caller Spec, RepoLoad, rows or completion state. | Test (TC-1954) |
| FR-053-AC-16 | A selected valid corpus with two authored criteria emits both, including one with no clause, source-symbol or test-case mapping, reports count two and Complete; changing a statement changes its owner statement, and deleting either authored row removes exactly that entry and changes the count. | Test (TC-1955) |
| FR-053-AC-17 | An external Rust consumer borrows the selection and origin-aware identities and owner statement/method/target data and performs exact lookup without Markdown/private-diagnostic/QSL ClauseRef parsing; AuthoredRow retains its authored ID, RenderedRow retains rendered ID plus row ordinal, and CombinatorialTable retains its rendered table ID, so deleting an earlier rendered row can renumber rendered IDs but never manufactures authored IDs. | Test (TC-1956) |
| FR-053-AC-18 | Missing/file/unreadable roots, unreadable documents and unclassifiable non-excluded Markdown (absent or malformed frontmatter) yield typed non-Complete with original cause/locus and retained healthy partial rows; valid unselected nonmember documents and paths excluded from every selected source contribute no entries and do not themselves block Complete. | Test (TC-1957) |
| FR-053-AC-19 | Missing selected section/table/required ID or statement column and unusable rows yield typed non-Complete; duplicate authored IDs in one selected source/document retain both rows and their loci, yield typed non-Complete and make exact lookup return ambiguity rather than first-wins, while other unambiguous partial lookups remain available; a nonempty valid selection with no selected documents or a valid empty headed table yields Complete with count zero after exhaustion. | Test (TC-1958) |
| FR-053-AC-20 | A real walk error or explicit interruption retains its original typed cause and partial rows and is non-Complete; a limit stops immediately when its emitted-entry count is reached without observing subsequent end-of-scan, so zero limit and limit N on an N-entry selection are non-Complete, while a greater-than-N or absent limit permits Complete only after actual normal end observation and no blocking cause. | Test (TC-1959) |
| FR-053-AC-21 | Selection identity equality uses canonical root, resolved selected declarations/effective exclusions and the exact path/byte snapshot of selected documents: changes within a selected document, even outside its criterion table, or changes to selected membership reject cross-inventory lookup as a typed mismatch; repeated identical captures compare equal, changes confined to valid still-unselected nonmembers, wholly excluded documents or files outside the root leave identity equal, and caller-truncated Spec or deserialized rows cannot issue Complete. | Test (TC-1960) |
| FR-053-AC-22 | Repeated exhaustive enumeration yields equal origin-aware identities, records, counts and terminal state in model declaration, relative document-path and authored row order; explicit exclusions remove exactly their selected documents, preserve healthy unexcluded criteria, and a changed selection/exclusion contract changes identity even when emitted records remain equal. | Test (TC-1961) |
| FR-053-AC-23 | Existing derive, single-document obligation and coverage consumers retain legacy healthy/skipped/tolerant-bad-root outputs; tests check unchanged statements, absence of evidence/freshness/discharge verdicts and GrammarFinding creation, and a producer API needing neither an IR schema copy nor a manually authored revision/hash ledger. | Test (TC-1962) |

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md) (the traceability model this extends, and the trace targets `target:` names), [FR-052](./FR-052-acceptance-criteria-property-classification.md) (the classification record obligations ride on), [FR-014](./FR-014-module-activation.md) (manifest loading), [FR-010](./FR-010-query-api.md) (table extraction)
- **Downstream**: [FR-054](./FR-054-verification-method-catalog.md) (the catalog the method is conformant against); `spec-artifacts-process` declares the ecosystem's obligation sources and the evidence-layer archetypes (agent-ix/spec-artifacts-process#34); quoin's evidence store and auditor (agent-ix/quoin#79, agent-ix/quoin#80)
