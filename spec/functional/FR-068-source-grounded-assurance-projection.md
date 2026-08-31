---
id: FR-068
title: "Source-grounded assurance projection"
type: FR
verification_method: test
evidence:
  - kind: test_case
    ref: tests/assurance_export.rs
  - kind: inspection_checklist
    ref: tests/assurance_boundary.rs
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-018"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-067"
    type: "requires"
    cardinality: "1:1"
---
# FR-068: Source-grounded assurance projection

## Description

When `quire-rs` builds an assurance export, the engine SHALL project the
authoritative corpus, obligation, and source-symbol records into source-
grounded JSON without re-parsing documents, renaming relation kinds, or deriving
a second graph.

The projection supplies facts, not assurance verdicts. Quire identifies what
was present, missing, inapplicable, or unread; Quoin remains responsible for
deciding whether evidence is fresh, sufficient, or persuasive.

## Inputs

- `Spec::documents()` and `Spec::edges()` from the loaded bounded corpus.
- [FR-053](./FR-053-obligation-record.md) obligation records.
- [FR-051](./FR-051-source-symbol-extraction.md) symbol records and the distinct
  `verifies` and `implements` relations.
- Module-declared required-relation rules and their applicability domains.

## Outputs

- `artifacts`: authored artifact identities, optional durable UUIDs, authored
  types, and source locators.
- `obligations`: FR-053 records with their declaring row locator.
- `symbols`: FR-051 stable symbol identities and declaration locators.
- `relations`: corpus, `verifies`, and `implements` relations.
- `relation_observations`: module-declared relation checks with explicit
  availability and freshness states.

Every source locator SHALL contain a scope-relative path, a 1-based line, and a
lowercase SHA-256 digest. Artifact locators digest the exact document bytes;
obligation locators carry FR-053's normalized statement hash in addition to a
digest of the verbatim row statement; symbol locators retain the FR-051 symbol
identity digest.

An artifact locator points to line 1 of its document. An obligation locator
points to the first source line containing the exact FR-053 statement in the
declaring document. A symbol or binding locator uses the declaration line
already carried by FR-051. A corpus-relation locator uses the source document
and the first line containing its target id. The exporter SHALL fail rather
than emit an absolute path, a parent traversal, a zero line, or a locator whose
source bytes are unavailable.

## Behavior

The projection SHALL preserve these meanings:

- Corpus relations copy `(source, target, edge_type, resolution)` directly from
  `Spec::edges()`. `resolved` and `dangling` remain distinct.
- Test evidence copies `VerifiesRelation` as relation kind `verifies`, including
  its symbol identity, trace id, form, provenance, path, and declaration line.
- Production scope copies `ImplementsRelation` as relation kind `implements`.
  It SHALL NOT be counted or labelled as test evidence.
- The projection SHALL copy obligation identity and statement hash from FR-053
  exactly, without recomputing either under a different normalization.
- A required-relation observation uses the module declaration that owns its
  applicability. An applicable subject with no satisfying relation is
  `missing`; a satisfying relation is `available`; a declaration with no
  applicable subject in the bounded corpus is `not_applicable`; a declaration
  that could not be evaluated because its vocabulary or source was unread is
  `unknown` with a non-empty reason.
- `freshness` is one of `current`, `suspect`, `unknown`, or `not_applicable`.
  Quire SHALL emit `unknown` for evidence relations because it has no stored
  binding hash or run revision to compare, and `not_applicable` for relations
  that do not represent evidence.
- Quire SHALL NOT emit freshness `current` or `suspect`; those are Quoin auditor
  verdicts over the exported operands and its store.

The observation identity is `(declaration, subject)`, where `subject` is the
applicable artifact id. A declaration with zero applicable subjects emits one
`not_applicable` observation with no subject. A document the corpus walker
could not read emits one `unknown` observation with its path as the subject and
the loader diagnostic as its non-empty reason; the exporter SHALL NOT infer a
missing relationship for content it could not inspect. Declaration exclusions
apply exactly as they do in bundle validation.

> **CR-157** (`agent-ix/quire-rs#386`, 2026-08-31) makes previously implicit
> identity and failure semantics testable: it fixes locator lines and digests,
> defines observation identity and zero-subject behavior, and preserves unread
> input as `unknown` rather than manufacturing a `missing` result.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-068-CON-1 | The exporter SHALL read only public authoritative records from `Spec`, obligation derivation, and symbol binding; direct frontmatter or source-tag harvesting is prohibited. | Architecture | Inspection |
| FR-068-CON-2 | The exporter SHALL derive relation kinds and applicability only from module data, with no built-in list of artifact types, edge verbs, test forms, verification methods, or required-relation rules. | Architecture | Test |
| FR-068-CON-3 | The projection SHALL carry neither a supported/unsupported assurance conclusion nor a comparison of run revisions or stored binding hashes. | Responsibility | Inspection |
| FR-068-CON-4 | The exporter SHALL sort records by their stable identity tuple and encode every map with observable order deterministically. | Determinism | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-068-AC-1 | Every artifact record carries its authored id, authored type, optional UUID without substitution, and a source locator whose path exists at the pinned revision and whose digest matches the exact document bytes. | Test (TC-1091) |
| FR-068-AC-2 | Every obligation record preserves FR-053's id, document, statement, statement hash, method, parameters, criticality, and target ids exactly, and its row locator selects the source statement that reproduces the exported hashes. | Test (TC-1092) |
| FR-068-AC-3 | Every symbol preserves FR-051's stable identity, language, kind, qualified name, path, declaration line, and binding capabilities; duplicate names in different containers or paths remain distinct. | Test (TC-1093) |
| FR-068-AC-4 | The corpus-relation projection is a bijection with `Spec::edges()` over `(source, target, edge_type, resolution)`, including dangling edges, and a relation kind absent from the engine's built-ins survives verbatim when its module declares it. | Test (TC-1094) |
| FR-068-AC-5 | `verifies` and `implements` remain separate relation record variants: a test binding carries provenance and counts as evidence, while a production binding carries no invented provenance and never counts as evidence. | Test (TC-1095) |
| FR-068-AC-6 | For module-declared required relations, fixtures exercise `available`, `missing`, `not_applicable`, and `unknown`; each state remains distinct in JSON, and `unknown` carries a non-empty reason rather than defaulting to another state. | Test (TC-1096) |
| FR-068-AC-7 | Evidence relations export freshness `unknown`, non-evidence relations export `not_applicable`, and no Quire-produced export claims `current` or `suspect`. | Test (TC-1097) |
| FR-068-AC-8 | Two projections over identical authoritative inputs serialize byte-identically, and changing an unrelated document does not change any other artifact, obligation, symbol, or relation identity. | Test (TC-1098) |
| FR-068-AC-9 | A static boundary test fails if the exporter calls frontmatter-harvest, markdown-query, or source-tag parsing functions instead of consuming the authoritative records. | Inspection (TC-1099) |

## Dependencies

- **Upstream**: [FR-026](./FR-026-intra-spec-reference-resolution.md),
  [FR-051](./FR-051-source-symbol-extraction.md),
  [FR-053](./FR-053-obligation-record.md), and
  [FR-067](./FR-067-versioned-assurance-export.md).
- **Downstream**: Quoin's assurance-case view consumes the projection and keeps
  its existing auditor verdict as the sole freshness decision.
