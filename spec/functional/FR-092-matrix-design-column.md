---
id: FR-092
title: "Design column on the coverage matrix"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-086"
    type: "requires"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-058"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-084"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "implements"
    cardinality: "1:1"
---
# FR-092: Design column on the coverage matrix

## Description

The coverage matrix ([FR-050](./FR-050-declarative-coverage-computation.md)-AC-47..51) answers
requirement to criterion to test to result. The design-element join ([FR-086](./FR-086-design-element-join.md))
answers requirement to design element. Neither shows the whole chain, so a reader cannot see in one
place which requirements have no design element and which criteria that design element realizes.

`quire-rs` SHALL provide `design_matrix`, an operation that returns the `coverage_matrix` with a
`design` column attached to each requirement entry, computed from the FR-086 join over the same corpus
and per-member inputs. The column is the FR-086 rows, not a second walk: `design_matrix` runs the join
once, after the matrix is computed (FR-084 reads the matrix, so the matrix cannot depend on the join),
and attaches each row to the entry of the document the row obliges. The engine holds no archetype name
and no verb. Which requirements the column applies to is the loaded modules' `design_relations` and
each relation's `from`, `where` and `exclude` ([FR-058](./FR-058-upward-trace-completeness.md)).

`design_matrix` is a separate operation. The `coverage` computation and its `coverage_matrix` are
unchanged: a module with a faulty `design_relations` never makes `coverage` fail.

### Entries of a multi-repository corpus

`design_matrix` returns the entries of every corpus member. An entry carries `repo` when its member
declares an identity, and entries are ordered by `repo` (absent first), then `document`. Two members
holding the same `document` path give two entries.

### The `design` key

A requirement entry gains `design`, an array with one block per design relation that obliges the
entry's document, in `design_relations` order. A block is
`{"relation", "check", "state", "elements", "dangling", "kind_mismatch"}`:

- `relation` and `check`: the relation's name and its `trace:<check>` token, as the FR-086 row carries them.
- `state`: `covered` or `uncovered`, the FR-086 row's `state`.
- `elements`: the FR-086 row's resolved `linked` entries, in the row's order. Each is
  `{"id", "kind", "path", "repo", "verb", "realized_criteria"}`. `kind` is the element's `archetype`
  as FR-086 carries it, so it is the module's own archetype name for the element (a part, a connection
  or an ADR in the architecture module). `kind` is omitted when the element has no `type`, `repo` is
  omitted when the member has no identity, and `realized_criteria` is FR-086's.
- `dangling` and `kind_mismatch`: the FR-086 row's lists, unchanged.

An entry that has `design` also carries the document `id`, and carries `criticality` as FR-050 and
[FR-082](./FR-082-coverage-matrix-criticality.md) state for any entry. Key order is `document`, `repo`, `id`,
`criticality`, `criteria`, `design`. A `linked` entry of a `to: []` relation that is itself dangling (FR-084) is not an element: it appears
in `dangling` only, and the block is `covered` as FR-084 states.

**A gap is a block, never an absence.** A requirement that a design relation obliges and that has no
design element has a block with `state: uncovered` and `elements: []`. **Out of scope is a missing key.**
An entry whose document no design relation obliges, because its archetype is outside every `from` or its
frontmatter is outside the `where` selector or it is excluded, has no `design` key. A reader tells a gap
from out of scope by the key. When no loaded module declares `design_relations`, no entry has `design`
and the output is byte-identical to the FR-050 `coverage_matrix`.

### Entries for requirements with no criteria

A design relation can oblige a requirement that has no acceptance criteria, and FR-050 emits no entry
for it. `design_matrix` SHALL add an entry `{"document", "criteria": [], "design": [...]}` for each such
document, so the gap is not hidden by a missing table. Entries stay ordered by `document`. This is the
only case in which `criteria` is empty, and `design_matrix` omits the whole array only when it has no entry, so it is present when design
relations oblige documents even though no `obligations:` source yields a criterion.

### Elements of a criterion

A design element realizes a requirement as a whole, or specific criteria of it
(FR-086 `realized_criteria`). In an entry that has `design`, each criterion carries `design_elements`,
a list of `{"id", "repo"}` references (`repo` omitted when absent) into the entry's blocks: the elements,
across all blocks, whose `realized_criteria` is empty or contains the criterion's id, in block then
element order, once each by (`repo`, `id`). Consumers read it verbatim, so a design element that names
AC-1 is not shown against AC-2, and an element that realizes the requirement as a whole is shown against
every criterion. The engine does not report which criteria have no element (FR-086): a criterion with an
empty `design_elements` under a covered entry is not a gap.

### What the engine does not decide

Whether a gap blocks anything is policy for the consumer. The engine reports no verdict, holds no
notion of a live requirement beyond FR-058 selection, and writes nothing back to any document. A
retired criterion keeps its place in the entry (FR-050), and the design block attaches regardless of its
status.

### Errors

`design_matrix` SHALL fail with the FR-086 error naming the entry when a `design_relations` entry names
no declared relation, and SHALL return no matrix.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-092-AC-1 | Given a module naming one relation in `design_relations`, a corpus of two requirements with criteria, one realized by a design element and one not, `design_matrix` returns both entries with a one-block `design`: the first `covered` with the element in `elements`, the second `uncovered` with `elements: []`. | Test |
| FR-092-AC-2 | An `elements` entry carries `id`, `kind` equal to the element's archetype, `path`, `verb` and `realized_criteria`, and `repo` only when the member has an identity; an element with no `type` carries no `kind` key. | Test |
| FR-092-AC-3 | A requirement that a design relation obliges and that has no acceptance criteria yields an entry with `"criteria": []` and an `uncovered` block, positioned by `document` among the other entries. | Test |
| FR-092-AC-4 | A requirement outside the relation's `from`, outside its `where` selector or excluded by `exclude` has no `design` key, while a selected requirement beside it in the same document set has one; both entries are present. | Test |
| FR-092-AC-5 | With no `design_relations` declared, `design_matrix` output is byte-identical to `coverage_matrix`, and the `coverage` report is byte-identical whether or not `design_relations` is declared. | Test |
| FR-092-AC-6 | Two design relations obliging one document yield two blocks in `design_relations` order, each with its own `relation`, `check` and `state`. | Test |
| FR-092-AC-7 | For each block, `state`, `dangling` and `kind_mismatch` equal the FR-086 row for the same document, relation and inputs, and `elements` equal the row's resolved `linked` entries projected to `id`, `kind`, `path`, `repo`, `verb`, `realized_criteria`; a dangling edge of a relation with a non-empty `to` is listed in `dangling` and leaves the block `uncovered`. | Test |
| FR-092-AC-8 | In a two-member corpus, a design element in member `o/b` realizing a requirement in `o/a` appears in the requirement's `elements` with `repo` `o/b`. | Test |
| FR-092-AC-9 | Each criterion's `design_elements` holds an element with `realized_criteria: []` for every criterion of the entry, an element whose `realized_criteria` names only AC-1 under AC-1 and not AC-2, and an element named by two blocks once. | Test |
| FR-092-AC-10 | A `design_relations` entry naming no declared relation makes `design_matrix` fail with an error naming the entry and return no matrix, while `coverage` over the same corpus still succeeds. | Test |
| FR-092-AC-11 | A criterion with status `retired` still sits under an entry carrying its `design` block, and the block is computed as for a live criterion. | Test |
| FR-092-AC-12 | Over a fixture whose relation, archetype, verb, inverse and criterion-id spellings are unrelated to requirements or design, the column is complete and correct. | Test |
| FR-092-AC-13 | The engine source for `design_matrix` contains no requirement or design archetype name, no realizing verb and no verdict. | Inspection |
| FR-092-AC-15 | Two members each holding a requirement at the same `document` path yield two entries, each with its own `repo`, ordered by `repo`; a corpus whose `obligations:` source yields no criterion but whose design relation obliges a document still yields the entry array. | Test |
| FR-092-AC-14 | Two runs over one corpus give byte-identical output independent of member and document order ([NFR-006](../non-functional/NFR-006-determinism.md)), and running `design_matrix` leaves every document byte-identical. | Test |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-092-CON-1 | `design_matrix` SHALL run the FR-086 join once and add no field to a block beyond the FR-086 row fields named above, with `archetype` named `kind`. | Design | Inspection |
| FR-092-CON-2 | `design_matrix` SHALL NOT change the `coverage` report or the `coverage_matrix` of FR-050. | Design | Test |

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md), [FR-058](./FR-058-upward-trace-completeness.md), [FR-084](./FR-084-two-way-relation-coverage-report.md), [FR-086](./FR-086-design-element-join.md)
- **Downstream**: the `quire matrix` design column and `--strict` (quire-cli FR-043); `quoin matrix` (quoin FR-165)
