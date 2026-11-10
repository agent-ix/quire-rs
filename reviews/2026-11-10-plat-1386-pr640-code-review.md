---
id: SR-20261103118153
title: "Code review (with Rust lane) — PLAT-1386 strict exemption of planned, blocked and parked rows (quire-rs PR 640)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/coverage/reconcile.rs, src/coverage.rs, src/evidence.rs, src/coverage/binding_diagnostics.rs, schemas/output/coverage-v1.schema.json, tests/it/coverage_rollup.rs, tests/it/output_contract.rs, CHANGELOG.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references}
---

## Summary

Ticket: PLAT-1386. PR head `cb26344c5648d6f66cca8199739bec28ed7e39d2` (`git diff origin/main...origin/plat-1386-engine`). Code review with the `/rust-review` lane folded in. The review checks the exemption matcher, the declared-status derivation, the two report keys, the schema keys and the `strict_decision` change against FR-050 "Strict exemption of planned, blocked and parked rows" and AC-64..66.

Focused tests pass on the head (`cargo test --test it -- coverage_ output_contract`: 98 passed, including TC-2219..2224, TC-824 baseline, TC-855/856/859). Tool versions: quoin 0.29.4, quire 0.39.0 (engine 0.55.0).

Checked and correct:

- `is_strict_exempt_status` implements the matching rule as specified: the three words are fixed, matching is ASCII case-insensitive, surrounding whitespace is trimmed, leading non-ASCII-alphanumeric bytes are skipped (so a multibyte `🚧` is skipped), and a word must be followed by end of text or a non-ASCII-alphanumeric byte (so `plannedé` matches and `plannedly` does not). It never consults the vocabulary or a severity.
- The status column is selected through `selected_status_column`, so the reference `status_column` takes precedence over the global column.
- Declared-status rows are resolved after the full row pass, before `unbacked_rows` is sorted, so the stored indices are still valid.
- `totals.strict_exempt_unbacked` is computed from the resolved records. Both keys use `skip_serializing_if`, so the TC-824 baseline is byte-identical and unchanged.
- The schema keys are `minimum: 1` and `const: true` under `additionalProperties: false`, as the spec states.
- `strict_decision` changes only the unbacked-row premise. ZeroTargets, ContradictedStatus, evidence results and declared inputs are untouched.
- No FR/AC/TC id or id-prefix semantics are hardcoded. No existing assertion is weakened: the only existing-test edits add `strict_exempt: false` to struct literals.
- `is_none_or` is within the declared MSRV of 1.98.1.

## Verdict

**Changes requested (one medium).** The blank-cell handling deviates from the spec (FND-001). FND-002 is a simplification.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A blank status cell is treated as "no status cell". `DeclaredRow::cell` filters out empty values, so `own_exempt` is `None` for a row whose status cell exists but is empty. Two spec deviations follow. (a) A blank-status row is skipped when the engine derives the declared status of the ids it references. If AC-1 is referenced by `TC-001 🚧 planned` and by a backed `TC-002` with a blank status, AC-1 is marked exempt, but the spec says an empty cell is a status cell that does not match, so it should veto. (b) A blank-status unbacked row whose own id is referenced only by exempt rows is routed through declared-status derivation and can become exempt, while AC-64 says a blank status is not exempt. TC-2219's blank row passes only because nothing references `TC-010`. Fix: distinguish column-present-and-empty from column-absent (`row.cells` holds the header), and treat empty as a non-exempt own status. | src/coverage/reconcile.rs:253-262, src/coverage/reconcile.rs:297-299, src/corpus/declared_tables.rs:33-39 |
| FND-002 | low | `StatusRefs.rows` is dead state. An entry is only created by `or_default()` immediately followed by `rows += 1`, so `refs.rows > 0` is always true for a present entry. A `BTreeMap<String, bool>` (`*entry.or_insert(true) &= exempt`) says the same thing and needs no hand-written `Default` impl. | src/coverage/reconcile.rs:22-35, src/coverage/reconcile.rs:362 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-050-AC-64: "An unbacked row is strict-exempt when its status text matches `planned`, `blocked` or `parked` … a blank status and a row with no status column and no referencing row are not … A reference row with no status cell takes its exemption from the rows that reference it" | examined | Matcher correct; blank-cell conflation (FND-001) |
| FR-050-AC-65: "`totals.strict_exempt_unbacked` (integer, omitted when zero) and `strict_exempt: true` on exactly the exempt `unbacked_rows` records, both optional keys of `schemas/output/coverage-v1.schema.json`" | examined | Correct; byte-identical when absent |
| FR-050-AC-66: "The strict verdict and an `error`-promoted `coverage:unbacked-row` count only non-exempt unbacked rows … The exemption applies identically on the FR-078 evidence path." | examined | `strict_decision` and `non_exempt_unbacked()` correct; the shared `reconcile` serves the evidence path |
| CHANGELOG.md PLAT-1386 entry | examined | Present and accurate |
