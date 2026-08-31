---
id: SR-066
title: "Code review: source-grounded assurance export"
type: SpecReview
analysis: code-review
scope: "Plan-002 implementation for FR-067, FR-068, and TC-1084..TC-1099"
review_set: all
---

## Summary

Reviewed the complete #386 implementation against the Golden Path test, mock,
boundary, and code-to-test alignment checks. The implementation is a pure
projection over `Registry`, `Spec`, obligation, and symbol records; it adds no
Git, network, persistence, Markdown-query, frontmatter, or source-tag reader.
All three review findings were corrected before this report was finalized.

## Verdict

**PASS after remediation.** No open implementation finding remains. The focused
contract suite passes 12/12, formatting and clippy pass, and the boundary audit
pins both purity and legacy-output compatibility.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Resolved: the reader originally accepted duplicate module names or duplicate archetype names with conflicting premise values; it now rejects both before returning a typed export, with mutation coverage. | `src/assurance.rs`, `tests/assurance_export.rs`, TC-1087 |
| FND-002 | high | Resolved: an unreadable document outside the corpus root could have fallen back to an absolute diagnostic subject; observation projection now returns `PathOutsideRoot` and never serializes the path. | `src/assurance.rs`, FR-068-CON-2, TC-1085 |
| FND-003 | medium | Resolved: normalized relative-path corpus edges can lose their authored target spelling, so a mandatory target-text line was not always derivable; CR-157 now defines line 1 for that case while direct authored identities retain their first matching line. | `src/assurance.rs`, `spec/functional/FR-068-source-grounded-assurance-projection.md` |

## Test and Boundary Review

- TC-1084..TC-1099 each occur in executable tests; no skipped or placeholder
  assurance tests exist.
- Tests use real temporary repositories, registries, schemas, corpus loading,
  symbol extraction, and relation graphs. There are no mocks of the unit under
  test and no production-only test hooks.
- Negative cases cover source identity, module name/version, loader failure,
  root escape, schema mutation, format/version mismatch, premise mismatch, and
  duplicate premise identity.
- Collection order and byte serialization are asserted directly. Schema digest
  tests distinguish insignificant JSON formatting/key order from semantic
  changes.
- Static inspection found no `TODO`, `FIXME`, stub, ignored test, hidden
  network/process boundary, or assurance field added to the existing
  coverage/properties payloads.

## Gate Evidence

- `cargo test --locked --test assurance_export --test assurance_boundary`:
  12 passed.
- `make fmt-check`, `make lint`, `make check-python`, `make deny`,
  `make audit-unsafe`, `make audit-property`, and `make audit-static`: pass.
- Script suite in a temporary environment using the pinned versions:
  136 passed, 3 skipped.
- `make validate` against the exact locked Process and ISO revisions:
  145 documents, 0 failed.
- `make check-engine QUIRE_CLI=/Users/peter/dev/quire-cli`: pass.
- Full `make test` reaches the pre-existing `quality_lints::tc868` failure
  because that test hard-codes `/home/peter/dev/spec-artifacts-iso` on macOS;
  every suite before it, including both new assurance binaries, passes.

## Review Note

The code-review workflow's referenced `implementation-gap-analysis` skill is
not installed in this environment. Its reverse-gap checks were performed
directly: every new public surface maps to FR-067/FR-068 and a tagged test, and
no unrelated production behavior was found.
