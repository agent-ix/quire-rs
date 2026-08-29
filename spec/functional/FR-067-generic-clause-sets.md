---
id: FR-067
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

# FR-067: Generic rights-aware clause sets

## Description

Quire shall load and evaluate domain-neutral clause sets supplied by modules.
The engine defines only the transport, applicability, rights, and comparison
contracts; it contains no built-in publication, clause inventory, or mapping.

A clause-set file identifies its authority, id, version, canonical content
digest, and rights posture. It may declare classification dimensions, expected
output types, clauses, and directed crosswalks. Clauses retain their force,
subjects, obligated actors, approval roles, authoring styles, applicability,
and expected outputs.

Applicability is a predicate, not a stored boolean. The expression vocabulary
is `all`, `any`, `not`, `eq`, `in`, and `at_least`. Evaluation returns one of
`binding`, `not_binding`, or `unresolved`. A missing context dimension, unknown
value, or incomparable value remains unresolved and never silently becomes
false.

Clause text is loadable only when the file declares it original or explicitly
cleared. Explicit clearance carries a review reference. A citation-only or
structure-only module can keep titles, identifiers, and official links while
omitting clause text.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-067-AC-1 | A module may reference clause-set JSON or YAML inside its root; loading rejects path escape, malformed content, a mismatched canonical digest, invalid rights posture, duplicate clause ids, and unresolved local output or crosswalk references. | Test (TC-1085, TC-1087) |
| FR-067-AC-2 | The public model preserves authority, id, version, force, subjects, obligated actors, approval roles, styles, expected outputs, and directed equivalent/partial/informative crosswalks without a domain-specific enum. | Test (TC-1084) |
| FR-067-AC-3 | `all`, `any`, `not`, `eq`, `in`, and ordered `at_least` expressions evaluate to binding, not-binding, or unresolved; missing, unknown, and incomparable context never becomes not-binding by default. | Test (TC-1084) |
| FR-067-AC-4 | Diff requires the same authority and id and reports added, removed, and changed clauses between exact versions with both set digests. | Test (TC-1086) |
| FR-067-AC-5 | Binding and diff JSON conform to the hand-authored `clause-binding-v1` and `clause-diff-v1` schemas. | Test (TC-1087) |
| FR-067-AC-6 | Clause text is rejected unless declared original or explicitly cleared, and explicit clearance requires a review reference; the engine and fixtures contain no external publication text. | Test (TC-1085) |

## Dependencies

- **Upstream**: [FR-013](./FR-013-archetype-loader.md), [FR-055](./FR-055-published-output-contract.md)
- **Downstream**: `agent-ix/quire-cli` for JSON, TSV, and human evaluation and diff views; `agent-ix/quoin` for discharge accounting over binding results
