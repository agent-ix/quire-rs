---
id: SR-174
title: "Code review of optional abstract TypeScript property recovery"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@c2863cda55b88dd5e4de3894c1293460cc247b89; src/symbols/typescript.rs, tests/coverage_matrix.rs; Ticket AGE-2236; PR agent-ix/quire-rs#531"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: reviews
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-051"
    type: reviews
---

## Summary

Ticket: AGE-2236. Reviewed the exact frozen PR head and both changed files using
the repository conventions plus the Rust lane of code-review. The change fixes
the reported valid `abstract?: boolean` extraction case and adds a traced
computed-matrix regression. The recovery predicate has one medium correctness
gap: it does not establish that the abstract-property diagnostic is the only
structural error, and its textual type-tail check admits malformed syntax.

## Assurance Context

The applicable profile is AP-201, Quire detection and minting assurance. The
change affects source parsing, trace binding, and coverage population. The
review could inspect the static code and traced regression, but no independent
controlled-corpus run or measurement evidence was available at the frozen
head. The focused Cargo test was intentionally not started because the shared
QSL-704 build lock was pending.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The recoverable exception trusts only the first parser diagnostic and validates the rest with a textual prefix. When a file contains `abstract?: boolean` before another structural error, `ParsedFile::diagnostic()` reports the abstract token first, `is_recoverable_optional_abstract_property` returns true, and the malformed file is walked as if it were valid; the same happens for malformed tails such as `abstract?: : boolean` because any non-empty text after the first colon is accepted. This can mint trace bindings from a syntactically invalid module and produce false coverage. The exception needs to prove the affected syntax node is the sole recoverable error and that the property has a valid type node, with negative controls for both cases. | src/symbols/typescript.rs:188-205, src/symbols/typescript.rs:245-275 |

## Verdict

**CONDITIONAL.** The reported valid one-line property is handled, the added
regression is behaviorally meaningful, and `cargo fmt --all -- --check` plus
`git diff --check` pass. FND-001 should be addressed before merge because a
malformed source file can otherwise contribute source symbols and coverage.

## Coverage

Examined FR-050-AC-48 (computed matrix binders), FR-051-AC-9 (per-file
diagnostic for an unparseable fixture), FR-051-AC-18 (TypeScript registration
recognition), FR-051-AC-23 (symbol extraction positions), FR-051-AC-25
(syntax-tree attachment), and AP-201 (detection/minting assurance). The
changed test exercises the valid one-line `abstract?: boolean` case; no test
exercises a valid case followed by a second structural error or a malformed
optional-property tail.

Gate results: `cargo fmt --all -- --check` PASS; `git diff --check` PASS;
focused Cargo test NOT RUN (shared QSL-704 build lock, per dispatch brief).

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 65154eca53bdfeee695e8ea122881ba07c29a85 |

`has_other_declaration_structure_error` now scans for additional structural
errors while allowing only the exact `abstract` error span, and the malformed
tail regression asserts zero symbols, one diagnostic, an untagged criterion,
and no binders.
