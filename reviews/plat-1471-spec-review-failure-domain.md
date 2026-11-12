---
id: SR-20261103118160
title: "Failure-domain analysis of FR-049 kind-typed references (quire-rs PR 649)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-rs@a2e2b4c0; spec/functional/FR-049-verification-reference-integrity.md (Kind-typed references, AC-10..17)"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-049
    type: references
---

# Failure-domain analysis of FR-049 kind-typed references (quire-rs PR 649)

## Summary

Ticket: PLAT-1471. Examined: the kind-matching rule, the fallback rules (empty kind cell, unknown kind value, table lacking `kind_column`), placeholder target cells, multi-id cells, the load-time rejections (FR-049-AC-13, AC-14, AC-15), and how this feature differs from FR-050-AC-19/33/43 and CR-054, which are never silent. Identity overlap (an id minted by two targets) is safe, because the subset is a union of resolution sets. A dangling id never also mismatches.

Author point (2), union-only fallback against Strict rejection: union-only is the right per-table behaviour. A reference declaration's section is legitimately optional (FR-050 CR-117 note), and the missing header is a document fact, not a load-time fact. Strict rejection would also fire on tables the declaration was never meant to type. The defect is that the fallback is silent (FND-001).

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | A table that lacks the `kind_column` header is checked against the union only, with no diagnostic. A misspelt or renamed `kind_column` in a manifest, or a renamed header in a document, therefore turns off kind checking for the whole corpus, under `Strict` too, and nothing says so. This is the silent no-op shape that FR-050-AC-19, AC-33 (`id-column-matches-nothing`), AC-43 (`status-column-matches-nothing`) and CR-054 exist to forbid. Fix: keep the union-only fallback, and when a kind-typed declaration reads a table whose headers lack `kind_column`, emit a warning-tier trace-pack diagnostic in both postures (for example `kind-column-matches-nothing`), naming the declaration, the document, the table-header line, the configured `kind_column` and the observed headers. Add an AC and a TC for it. | spec/functional/FR-049-verification-reference-integrity.md:83-84,118 | missing-requirement |
| FND-002 | low | A kind value absent from `kind_targets` also falls back to union only, with no diagnostic. A typo (`Hazrd`) or a multi-valued cell (`Requirement, Hazard`) therefore bypasses the check for that row. The cost may be acceptable if another check validates the Kind column's vocabulary, but FR-049 does not say which check does. Fix: either name the owner of kind-value validity (archetype schema enum or a FR-050 column vocabulary in the consuming module), or emit an advisory for an unmatched non-empty kind value. | spec/functional/FR-049-verification-reference-integrity.md:80-83 | missing-requirement |

## Verdict

FND-001 (medium) should be fixed before merge: a check that an assurance consumer relies on (spec-artifacts-process FR-015-AC-6) must not turn off silently because of a header typo. FND-002 is a judgement call; recording the owner is enough.
