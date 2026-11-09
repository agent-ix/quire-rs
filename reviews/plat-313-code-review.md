---
id: SR-16390
title: "Code review — PLAT-313 id_column as one name or a list (quire-rs PR 616)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; src/traceability.rs, src/corpus/declared_tables.rs, docs/USAGE.md, tests/it/id_column_lists.rs, tests/fixtures/corpus_cases/id-column-lists.json, tests/it/main.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references}
---

## Summary

Ticket: PLAT-313. PR: agent-ix/quire-rs#616, head `6166414`, branch `plat-313-id-column-code`. Diff scope: `git diff origin/main...HEAD`. This is code-review with the rust-review lane (rust-style loaded) folded in. The authority is the merged FR-050-AC-63, CR-210 and TC-2095..TC-2100.

The checks: each AC-63 clause against `IdColumns`, `ScannedRow::ids`, `scan` and `mint`, and the load validation in `TraceabilityModel::validate`. Every caller of `TraceTarget::id_column()` and `id_columns()`: `src/obligation.rs` (`resolve`), `src/obligation/inventory.rs` (through `resolve`), `src/coverage/reconcile.rs` (retirement), `src/coverage/row_binding.rs`, `src/evidence.rs` and `src/corpus/trace_refs.rs` (through `mint`), `tests/it/trace_target_sources.rs`, `src/registry.rs`, plus `examples/`, `tools/`, `crates/`, `benches/` and `schemas/`. The serde refactor shared with `SectionNames`. `quire trace --id` for TC-2095..TC-2100. Status convention in `spec/tests.md`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The USAGE note is on the wrong key. The edited row belongs to the locator `assert` facet table (FR-033). That `id_column` is still `Option<String>` (`src/extract/locator.rs:243`, `src/extract/assert_eval.rs`) and does not accept a list or `*`. The doc now says it does, so a manifest that follows it fails to load. The trace-target `id_column` that this PR widened is still documented nowhere in `docs/`. Fix: restore the row's original text. If a note is wanted, add one line about the `trace_targets` `id_column` where traceability is described. | docs/USAGE.md:309 |
| FND-002 | low | Misplaced doc comment. The new `declared_list` was inserted under `quoted_list`'s doc line. `declared_list` now opens with "capped, with the remainder counted rather than dropped", which contradicts its own "never truncated", and `quoted_list` has no doc. Fix: move the first doc line back onto `quoted_list`. | src/corpus/declared_tables.rs:1039 |
| FND-003 | low | No CHANGELOG entry. The PR changes the public Rust API: `TraceTargetSource::TableRow.id_column` goes from `String` to `IdColumns`, `TraceTarget::id_column()` now returns `None` for a list or wildcard target, and `id_columns()` and `IdColumns` are new. It also adds a manifest feature. The repo records both kinds of change under `[Unreleased]` (PLAT-316 `from`/`pattern` and `TraceTarget.source`; PLAT-877 "Breaking (Rust API)"). Fix: add an Added entry and a Breaking (Rust API) entry. | CHANGELOG.md:8 |
| FND-004 | low | TC-2095..TC-2100 are still 🚧 in spec/tests.md. The repo's code PRs flip their TC rows to ✅ when the code lands (PLAT-57 `e1da9c6`, PLAT-35/PLAT-55 `8dc8a1b`). The FR-050 traceability row already reads "Implemented except AC-62". No gate compares the cell: a backed 🚧 row is in the pending class, not a status lie, so this is a convention fix only. Fix: set the six rows to ✅. | spec/tests.md:1134 |

## Verdict

Mergeable after one small fix round: FND-001 (medium) and the three low findings.

What holds. Every AC-63 clause is implemented. `IdColumns::matches` lowercases and trims both sides and uses `wildcard_match` from `section:`, with `*` the only metacharacter and no `normalize_heading`. `ScannedRow::ids` walks headers in column order, reads each header once and dedups ids within the row. `scan` fires `IdColumnMatchesNothing` only when no header matches any entry, and `declared_list` names every entry without truncation. An empty list or a blank entry fails load. Serialization uses the scalar for one name. Load fails when an `obligations:` source binds a list or `*` target, and the message names the source and the target.

The callers are safe. `id_column()` now returns `Some` only for a single plain name. `row_binding.rs:43` therefore never joins a list or wildcard target. `obligation::resolve`, `inventory.rs` and `reconcile.rs:883` reach a target only through an `obligations:` binding, which validation already refuses for a list or wildcard. `evidence.rs` and `trace_refs.rs` go through `mint`, which handles lists. No JSON schema in `schemas/` or under `quire schema` describes `TraceTarget`. `SelectionIdentity` serializes the target through the new `Serialize`, so lists round-trip. The one cross-module gap is recorded in SR-16391.

The serde refactor keeps `section:` parsing and its error text: the "a section heading, or a sequence of them" expecting string survives. Existing scalar manifests parse unchanged.

Each `#[trace]` sits on its own `#[test]` fn. `quire trace --id` gives TC-2095, TC-2096 and TC-2097 one claim each, TC-2098 four, TC-2099 three and TC-2100 two, all in `tests/it/id_column_lists.rs`, with no neighbour stealing another's tag.

The tests are real. They drive `Registry::load_module` and `coverage::compute` with no mocks. Python, TypeScript and Rust corpus cases each carry a list case, a scalar control and a one-column control.

No ceremony: no census, SHA, pin, manifest, count or compatibility layer.

Rust style is fine: public items are documented, there is no `unwrap` on input, borrowing is right, and `From<&str>` is a fitting trait. The `Vec<(String, bool)>` matcher tuple and the `flat_map(.. collect::<Vec<_>>())` allocation are acceptable at this size and are not findings.

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-050-AC-63 | examined | Implemented; docs and convention findings only |
| src/traceability.rs IdColumns, serde helpers, validate | examined | Clean |
| src/corpus/declared_tables.rs ids, scan, mint, scan_finding | examined | FND-002 |
| docs/USAGE.md | examined | FND-001 |
| id_column() callers (obligation, inventory, reconcile, row_binding, evidence, trace_refs) | examined | Clean |
| tests/it/id_column_lists.rs trace tags | examined | Clean |
| spec/tests.md TC-2095..TC-2100 status | examined | FND-004 |
| FR-050-AC-33, FR-050-AC-34 | context_only | Consistent |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | The fix round repeats FND-002's defect in a new place. `check_bound_target` was inserted under `validate`'s doc paragraph. `check_bound_target`'s rustdoc now opens with "Shape-validate the declaration. Returns a caller-facing reason on the first problem…", and `pub fn validate` has no doc at all. Fix: move those three lines back onto `validate`. | src/traceability.rs:1304 |
| FND-006 | low | The merged-model failure does not name the module whose target won the merge. It is recorded against the module that declares the `obligations:` source (mod-b), with the text "obligations entry 'o' references target 'x', whose `id_column` is a list or contains `*`". That module's own `x` is a single name, so the message describes a declaration the reader cannot find in their manifest. Fix: add which module's `x` shadowed theirs (the first-wins winner, available from `outcome.modules` order), e.g. "… target 'x' (declared first by module 'mod-a') …". | src/loader/mod.rs:1271 |
| FND-007 | low | SR id collision. Open PR #614 (`plat-362-spec`, PLAT-362) adds `spec/reviews/plat-362-spec-review-*.md` with `id: SR-16390`, `SR-16391`, `SR-16362`, `SR-16363` and `SR-16364`. #614 was pushed after this review allocated SR-16390/16361 and posted them to Linear. Whichever PR lands second has duplicate SpecReview ids. Fix: #614 renumbers (these ids were allocated and published first). If #616 lands second, renumber this file and `reviews/plat-313-gap-analysis.md` to the next free ids; the Linear comments are keyed by path. | reviews/plat-313-code-review.md:2 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `docs/USAGE.md` is no longer in the diff against `origin/main`. The FR-033 locator row is back to "Which column holds the id." and no wrong doc remains. |
| FND-002 | fixed | `declared_list` has its own doc ("Every declared entry as `'a', 'b'`, never truncated …"), and the "capped" line is back on `quoted_list` (declared_tables.rs:1039-1049). |
| FND-003 | fixed | CHANGELOG `[Unreleased]` has an Added entry (FR-050-AC-63) and a "Breaking (Rust API), PLAT-313" entry covering `IdColumns`, `id_column()` returning `None`, and `id_columns()`. |
| FND-004 | fixed | TC-2095..TC-2100 now read ✅ in spec/tests.md. The diff against `origin/main` changes only those six rows. |
