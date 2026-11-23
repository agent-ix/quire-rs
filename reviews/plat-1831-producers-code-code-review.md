---
id: SR-20278
title: "Code review of PLAT-1831 outputCatalog producers and catalog diff (quire-rs PR 714)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; PR 714 diff only (head 1b095e7f714ffbcb0827bff82c84f4c7a987b2e3): src/clauses.rs, schemas/output/clause-diff-v1.schema.json, tests/it/hardening_properties.rs"
review_set: subset
date: 2026-11-23
---

## Summary

Ticket: PLAT-1831. Code review with the Rust lane (rust-review, rust-style) of the `ExpectedOutput.producers` field, its load validation, the clause-set output-catalog diff, and the `clause-diff-v1` schema extension. Downstream callers checked: quire-cli origin/main (`src/commands/clauses.rs`, tests) and quoin origin/main (`quoin-quire`). Gates run in a separate review checkout and target: `cargo test --lib clauses` 10/10 pass, `cargo test --test it output_contract` 12/12 pass, `cargo fmt --check` clean, `cargo clippy -p quire-rs --lib --profile=test -D warnings` clean (`--tests` fails only on pre-existing `tests/it/assurance_export.rs:485,487`, outside this diff).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The `producers` doc comment says absent "means unspecified" and calls the entries "kinds of the producers", while FR-073 says an absent member means the output is not machine-produced and that entries are retained record kinds, not commands or tools. Fix: "Retained record kinds that satisfy this output (open vocabulary). Absent means the output is not machine-produced; never an empty list." | src/clauses.rs:85-86 |
| FND-002 | low | The schema's `producers` items carry only `minLength: 1`, so a diff document with `" run"` or `"  "` conforms although load validation rejects it. The rule is expressible in JSON Schema. Fix: add `"pattern": "^\\S(?:[\\s\\S]*\\S)?$"` to `$defs.producers.items` (covers whitespace-only and padded kinds; `minItems` and `uniqueItems` already mirror the rest). | schemas/output/clause-diff-v1.schema.json:45-50 |
| FND-003 | low | Adding the public field `ExpectedOutput.producers` and the member `ClauseSetDiff.output_catalog` is a breaking API change (struct literals stop compiling: quire-cli `tests/cli_clauses.rs:56` and `:158` build `ExpectedOutput { kind, description }` and will hit E0063 when quire-cli moves off `v0.62.2`), and `clause-diff-v1` now requires a new member, but `CHANGELOG.md` `[Unreleased]` has no entry. The repo records such changes as `**Breaking, ...**` entries (see 0.62.0). Fix: add a Breaking entry naming both struct changes and the new required `outputCatalog` diff member. | CHANGELOG.md:8-16; src/clauses.rs:88; src/clauses.rs:506 |

## Verdict

Approve after the low fixes; nothing blocks merge. Behaviour matches FR-073: `validate_producers` enforces non-empty, distinct, non-empty, non-padded kinds with an error naming `outputCatalog` and the entry key; no vocabulary check; `Option` plus `skip_serializing_if` keeps absent distinct from empty; `diff_output_catalogs` keys on catalog name and compares whole entries. No compatibility layer, no ceremony, no ticket citations in code comments. Downstream: quire-cli `src/commands/clauses.rs` only reads `ClauseSetDiff` fields and serializes it, so it compiles and its JSON diff gains `outputCatalog`, which its `assert_conforms(..., "clause-diff-v1.schema.json")` test will accept against the new schema; quoin only passes `ClauseSetDiff` through and constructs no `ExpectedOutput`. The `hardening_properties.rs` change is a rustfmt import reorder.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | Round 1 (head 2fdddf29): the comment now says absent means not machine-produced, but still calls the entries "kinds of the producers that emit this output", and the new parenthetical "(the entry is a retained record kind)" attaches to the absent case, where there is no entry. Fix: "Retained record kinds that satisfy this output (open vocabulary). Absent means the output is not machine-produced, which is distinct from an empty list." |
| FND-002 | fixed | Round 1: `$defs.producers.items` gains `"pattern": "^\\S(?:[\\s\\S]*\\S)?$"`; `diff_schema_rejects_padded_producer_kinds` checks `" run"`, `"run "`, `"  "`, `"run\n"` and passes. Without the pattern the schema accepts `" run"`, `"run "` and `"  "` (checked by evaluating the schema with the pattern removed), so the test fails on the old schema. |
| FND-003 | fixed | Round 1: CHANGELOG `[Unreleased]` gains "**Breaking, PLAT-1831:** `ExpectedOutput` gains an optional `producers` list (validated at load: non-empty, unique, trimmed kinds), and `ClauseSetDiff` gains an `output_catalog` member (`OutputCatalogDiff`) that `clause-diff-v1` now requires as `outputCatalog`." |
