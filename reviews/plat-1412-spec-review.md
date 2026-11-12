---
id: SR-20261103118155
title: "Base spec-review of FR-082 criticality trace column (quire-rs PR 645)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@326eb1bd; spec/functional/FR-082-criticality-trace-column.md, spec/functional/index.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-082
    type: references
---

# Base spec-review of FR-082 criticality trace column (quire-rs PR 645)

## Summary

Ticket: PLAT-1412. Base checklist over the PR diff: the new FR-082 and its index line. Examined the FR-082 statement, AC-1..AC-7 and CON-1, and checked them against FR-077 (the requirement FR-082 extends), `src/symbols/trace_search.rs` (`SearchResult`), quire-cli's `src/commands/trace.rs` on main (`TraceReport`), FR-050-AC-47/48 (`coverage_matrix`), FR-053 (the obligation `criticality` that already exists), and spec-artifacts-iso FR-008 (open PR spec-artifacts-iso#70; it is not on main). `quire validate` (quire 0.40.0, engine 0.56.0) passes on both changed files. No hash, digest, pin or tracking-record antipattern was found; AC-7 is a functional no-write-back property.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | FR-082 targets a "requirement row" that the trace model it extends does not have. FR-077's `SearchResult` and quire-cli's `TraceReport` are lists of claim and citation records keyed by `trace_id`, which is often an AC or TC id. `quire trace` extracts source with `spec/` excluded and never reads a requirement document or its frontmatter. To attach a frontmatter value the engine would need a trace-id to document join, plus an AC-to-owning-FR mapping, and FR-077-AC-6 ("engine knows ids, not hierarchy") rules that mapping out. The requirement-to-criterion model that does exist is FR-050's `coverage_matrix` (`requirements[]` per document, rendered by `quire matrix`). Fix: host the column on the `coverage_matrix` `requirements[]` entry (extend FR-050), or state the join explicitly: which document supplies the value, which id kinds carry it, and how AC/TC rows get it. | spec/functional/FR-082-criticality-trace-column.md:22-24, spec/functional/FR-077-trace-search-index.md:57-65, src/symbols/trace_search.rs:57-78 | wrong-requirement |
| FND-002 | medium | A second, unrelated criticality model. FR-053 already carries a per-criterion `criticality` from a module-declared `criticality_column`, absent when undeclared (FR-053-AC-7), and FR-068 preserves it into assurance records. FR-082 adds a frontmatter-level `criticality` but does not reference FR-053, gives no precedence rule when both exist, and encodes absence differently. Fix: relate FR-082 to FR-053, and state how a document-level value and a criterion-level value coexist or which one wins. | spec/functional/FR-082-criticality-trace-column.md:1-12, spec/functional/FR-053-obligation-record.md:180, src/obligation.rs:60-63 | missing-requirement |
| FND-003 | medium | `unassigned` is called "a reporting label, not a data value", yet AC-2 and CON-1 put that string into the engine row's `criticality` field. As a result, an authored `criticality: unassigned` is reported, per AC-5 ("as written"), identically to a missing field. Engine convention models absence as absent/`Option` (FR-053-AC-7). Fix: the engine field is absent/`None` when missing, and the renderer (quire-cli) prints `unassigned`. Alternatively, define how an authored `unassigned` is told apart. | spec/functional/FR-082-criticality-trace-column.md:26-28, FR-082-AC-2, FR-082-CON-1 | wrong-requirement |
| FND-004 | medium | AC-5 says "alongside the validation diagnostic", but `quire trace` runs no document validation, and the AC does not say which diagnostic, from which run, or on which channel. Under spec-artifacts-iso FR-008-AC-4 an out-of-enumeration value is a `quire validate` failure. Fix: say that trace reports the value as written and emits no validation, or name the diagnostic channel and its reason code. | FR-082-AC-5 | wrong-requirement |
| FND-005 | medium | AC-6 asserts a "table output" column header and a `criticality` key "on each requirement row" of the JSON. `quire trace` has `human`, `json` and `tsv` forms, no table, and its JSON has `claims.verifies[]`, `claims.implements[]` and `citations[]` records, no requirement rows. The AC names output shapes that do not exist, and a CLI snapshot cannot run in quire-rs (see the scope-boundary review). | FR-082-AC-6, quire-cli src/commands/trace.rs (TraceReport, render_tsv) | wrong-requirement |
| FND-006 | low | AC-4 tests non-inheritance only via `traces_to`/`satisfies`, but the statement forbids inheriting from any parent. Other edge types (`extends`, `refines`, `consumes`) go untested. Fix: generalise the AC to "any relationship", or add one case per edge type. | FR-082-AC-4 | wrong-requirement |

## Verdict

FAIL: not mergeable. FND-001 (high) means the requirement cannot be implemented as written against FR-077's model. FND-002..FND-005 (medium) are to be fixed in the same round. Clean units: FR-082-AC-1 and AC-3 are well-formed once the host row exists. FR-082-AC-7 and CON-1 are testable and carry no ceremony. The index line is correct.
