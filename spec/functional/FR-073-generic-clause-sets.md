---
id: FR-073
title: "Generic rights-aware clause sets"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/FR-013"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/FR-055"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/StR-001"
    type: "traces_to"
---

# FR-073: Generic rights-aware clause sets

## Description

Quire shall load and evaluate domain-neutral clause sets supplied by modules.
The engine defines only the transport, applicability, rights, and comparison
contracts; it contains no built-in publication, clause inventory, or mapping.

A clause-set file identifies its authority, id, version, and rights posture. It may declare classification dimensions, expected
output types, clauses, and directed crosswalks. Clauses retain their force,
subjects, obligated actors, approval roles, authoring styles, applicability,
and expected outputs.

Applicability is a predicate, not a stored boolean. The expression vocabulary
is `all`, `any`, `not`, `eq`, `in`, and `at_least`. Evaluation returns one of
`binding`, `not_binding`, or `unresolved`. A missing context dimension, unknown
value, or incomparable value remains unresolved and never silently becomes
false.

Clause text is loadable only when the file declares `rights.text` as `original`,
`explicitly_cleared` or `licensed`. Explicit clearance carries a review
reference. `licensed` text may be loaded and used locally, needs no
clearance reference, and is restricted from publication under
[FR-083](./FR-083-clause-set-publishability.md). `text: none` prohibits clause
text. A citation-only or structure-only module can keep titles, identifiers, and
official links while omitting clause text.

Each `outputCatalog` entry (`ExpectedOutput`) carries `kind` and `description`
and may carry `producers`, the machine-checked binding of the output to what
produces it. `producers` is a list of retained record kind strings; a record
whose kind is in the list satisfies the output. Rules:

- `producers` is optional. An absent member means the output is not
  machine-produced (for example a customer-supplied record or a procedure);
  there is no default list and no other field stands in for it.
- When present, `producers` is a non-empty list of distinct, non-empty kind
  strings. Loading and `validate` reject an empty list, a duplicate kind, and a
  kind that is empty or whitespace-only, each naming the catalog entry.
- The set of kinds is open. Kinds such as `run`, `scan`, `mock-inspection`,
  `analysis-report`, and `scenario`, and the `subject.kind` values of
  experiment and operational records, are defined by the consumer's retained
  records, not by the engine. Quire validates the shape of `producers` only and
  never enumerates or checks the kinds against a vocabulary.
- `producers` is the only producer binding the catalog carries: it names record
  kinds, not commands or tools.
- `producers` serializes as the camelCase member `producers` on the catalog
  entry and round-trips unchanged, in order, through the `ExpectedOutput`
  struct and the clause-set JSON and YAML forms; an absent member is not
  serialized as an empty list.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-073-AC-1 | A module may reference clause-set JSON or YAML inside its root; loading rejects path escape, malformed content, invalid rights posture, duplicate clause ids, and unresolved local output or crosswalk references. | Test |
| FR-073-AC-2 | The public model preserves authority, id, version, force, subjects, obligated actors, approval roles, styles, expected outputs, and directed equivalent/partial/informative crosswalks without a domain-specific enum. | Test |
| FR-073-AC-3 | `all`, `any`, `not`, `eq`, `in`, and ordered `at_least` expressions evaluate to binding, not-binding, or unresolved; missing, unknown, and incomparable context never becomes not-binding by default. | Test |
| FR-073-AC-4 | Diff requires the same authority and id and reports added, removed, and changed clauses between exact versions. | Test |
| FR-073-AC-5 | Binding and diff JSON conform to the hand-authored `clause-binding-v1` and `clause-diff-v1` schemas. | Test |
| FR-073-AC-6 | Clause text is rejected unless declared original, explicitly cleared or licensed, and explicit clearance requires a review reference; the engine and fixtures contain no external publication text. | Test |
| FR-073-AC-7 | A catalog entry with `producers: ["run", "scan"]` loads, and serializing the clause set and loading it again yields the same entry with the same list in the same order, in both JSON and YAML. | Test |
| FR-073-AC-8 | A catalog entry without `producers` loads with no producers and serializes without a `producers` member; it is not treated as an empty list. | Test |
| FR-073-AC-9 | Loading rejects a catalog entry whose `producers` is an empty list, contains the same kind twice, or contains an empty or whitespace-only kind, and the error names the entry. | Test |
| FR-073-AC-10 | A catalog entry whose `producers` contains a kind that no fixture or engine list defines (for example `vendor-experiment`) loads without error. | Test |

## Dependencies

- **Upstream**: [FR-013](./FR-013-archetype-loader.md), [FR-055](./FR-055-published-output-contract.md)
- **Downstream**: `agent-ix/quire-cli` for JSON, TSV, and human evaluation and diff views; `agent-ix/quoin` for discharge accounting over binding results
