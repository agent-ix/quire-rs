---
id: SR-20274
title: "Code review — PLAT-1472 kind-typed cell references (quire-rs PR 650)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/traceability.rs (DocumentReference kind fields, targets_for_kind, check_kind_declaration); src/corpus/trace_refs.rs (validate_trace_references, referencing_rows); tests/it/trace_reference_kinds.rs; tests/fixtures/traceability/kinds/manifest.yaml; spec/tests.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-049", type: references}
---

## Summary

Ticket: PLAT-1472. PR head: `3e7083b` (merge of origin/main into `e89a4c2`). Code review with the rust-review lane (rust-style loaded) folded in, scoped to `git diff origin/main...HEAD` (6 files, +610/-25).

The change adds optional `kind_column` / `kind_targets` to `DocumentReference` (`skip_serializing_if` keeps untyped declarations byte-identical), validates them at module load in `check_kind_declaration`, reads the kind cell per row in `referencing_rows`, emits `reference-kind-mismatch` through the posture tier and `kind-column-matches-nothing` at warning tier, and sorts findings by `(path, id, reason, message)`.

Reviewer-run evidence: focused `cargo test --test it -- trace_reference_kinds trace_references` (19 tests) pass; `cargo fmt --check` clean; `examples/spec_validate.rs` reports 204 documents, 0 failed. Reviewer probes, not committed, reproduced FND-001 and FND-002.

## Verdict

Not mergeable as is: one high finding (FND-001) and one medium finding (FND-002). Everything else checks out. Untyped declarations keep the old sort key, because `"dangling-trace-reference"` is constant in the new tuple slot, and they serialize as before. No kind name appears in the engine. Kind matching trims and ASCII case-folds, both for keys and for cells. Unknown or empty kinds and a missing kind header fall back to the union only. A dangling id never also yields a mismatch. The `-` or empty cell case is handled by the pattern. Every other malformed-declaration form is rejected at load with an error that names the declaration and the field. Rust style matches the repo: `Result<(), String>` loader checks like the neighbouring `check_field`, `deny_unknown_fields` on the struct, `BTreeMap`/`BTreeSet` for determinism, and no panics on input.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | An exact duplicate kind key (`north: [lefty]` then `north: [righty]`) loads without error. yaml_serde deserializes into `BTreeMap<String, Vec<String>>` and keeps one entry silently, so `check_kind_declaration`'s duplicate check only ever sees case-variant duplicates (`north`/`North`). FR-049-AC-15 requires a duplicate kind key to be rejected at load, naming the declaration. Reproduced with a probe: `Registry::load_module` returned `Ok`. TC-2244 tests only the case-variant form. | src/traceability.rs:979, src/traceability.rs:1912-1917, tests/it/trace_reference_kinds.rs:215-218 |
| FND-002 | medium | `kind-column-matches-nothing` is per document and row-derived, but the spec says one finding per table. The missing-header check runs over `ScannedRow`s in `referencing_rows` and collects into a `BTreeSet<(PathBuf, String)>`. As a result (a) a table that has the `column` header but no kind header and **no data rows** yields no warning (probe: 0 warnings), and (b) two such tables in one document collapse into one warning. FR-049 says "one warning-tier `kind-column-matches-nothing` finding per such table". `declared_tables::scan`'s own comment explains that header checks belong there because callers iterating rows cannot see zero-row tables. | src/corpus/trace_refs.rs:100-110, src/corpus/trace_refs.rs:284-288, src/corpus/declared_tables.rs:526-535 |
| FND-003 | low | The `reference-kind-mismatch` message omits the row owner (`(row 'K-n')`) that the sibling dangling message carries. Two rows in one document that reference the same mismatched id then produce identical tuples, which `dedup` folds into one finding, and the author cannot tell which row is wrong. This is permitted by "at most one finding per occurrence", but it is poorer than the dangling path. | src/corpus/trace_refs.rs:131-143, src/corpus/trace_refs.rs:147-160 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| DocumentReference kind fields + serde | examined | Clean; skip_serializing_if keeps untyped output identical |
| targets_for_kind / fold_kind | examined | Clean; trim + ASCII lowercase on both sides |
| check_kind_declaration | examined | FND-001 (exact duplicate key) |
| validate_trace_references mismatch/dangling split | examined | Clean; dangling-only for ids outside the union |
| finding sort (path, id, reason, message) | examined | Clean; untyped order unchanged |
| referencing_rows missing-kind-column detection | examined | FND-002 |
| kind-column-matches-nothing routing | examined | Clean; warning tier, registry-tunable |
| tests/it/trace_reference_kinds.rs TC-2240..2247 | examined | Pass; trace tags correct; TC-2244 misses exact duplicate |

## Verification

Reviewer-only; no branch edits. The probes ran in a throwaway module (`tests/it/zz_probe_qrs650.rs`), which was removed afterwards; the worktree is clean.
