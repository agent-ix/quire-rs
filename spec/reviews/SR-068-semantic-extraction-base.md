---
id: SR-068
title: "Base review of the semantic extraction boundary (#388)"
type: SpecReview
analysis: base
scope: "US-019, FR-069, FR-070, FR-071, FR-072, NFR-021, TC-1599..TC-1644"
review_set: all
---

## Summary

Reviewed the `agent-ix/quire-rs#388` chain (US-019 → FR-069..FR-072, NFR-021, TC-1599..TC-1644) against the checklist and the six matrix rules. Identifiers, links, and AC→TC coverage are complete (32 AC + 12 CON + 4 NFR AC → 46 TC rows); the substantive gaps are two unallocated inputs (the `sourceIdentity` of a span when Quire runs without an org/repo, and the classification of a table whose header is neither the typed header nor a free-column form), one verification locus that lives in another repository, and nine engine grammar warnings on the new statements.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-280 | high | FR-071 fixes `sourceSpan.sourceIdentity` as `ix://<org>/<repo>/spec`, but only the Filament API carries `org`/`repo_name`; `quire validate` and the library path over a corpus have no source of either, so the requirement is unimplementable there and no AC says what the span carries when they are absent. | spec/functional/FR-071-clause-and-operation-extraction.md (Outputs); FR-072-AC-5 |
| FND-281 | medium | FR-070 recognizes the typed table by an exact header and names free-column tables and bullet lists as legacy, but says nothing about a table with the four typed columns in another order, a fifth column, or a typed header with zero rows; the mapping of those inputs (legacy, error, or empty `fields`) is undefined and untested. | spec/functional/FR-070-typed-properties-extraction.md (Behavior, first bullet); TC-1617 |
| FND-282 | medium | FR-072-AC-6 / TC-1636 assert WASM parity, but the WASM binding is `agent-ix/quire-wasm`, which pins quire-rs by git branch; the matrix does not state that TC-1636 executes in that repository, so the row cannot be marked ✅ from `make ci` here and the gate that runs it is unnamed. | spec/functional/FR-072-semantic-extraction-surface.md (Behavior, bindings bullet); spec/tests.md TC-1636 |
| FND-283 | medium | Nine engine grammar warnings on the new statements: `ears:non-singular` (FR-069:101), `ears:unclassifiable`/`missing-subject` (FR-070:22, FR-071:22), `quality:agentless-passive` (FR-069:66, FR-070:51, FR-071:84, FR-072:70); the matrix must not be marked complete while non-singular or unclassifiable statements remain. | spec/functional/FR-069..FR-072 (lines cited) |
| FND-284 | low | `Multiplicity` boundaries beyond the vendored `cell-cases.json` (`0..0`, `*` alone, `0..*`, `1..1 unique`, non-numeric bounds) have no TC; TC-1621 is property-shaped but its generator domain is unstated. | spec/functional/FR-070-typed-properties-extraction.md (Multiplicity bullet); TC-1614, TC-1621 |
| FND-285 | low | US-019 has no Options section and no explicit priority field beyond prose; the checklist asks for both. The story is otherwise INVEST-shaped and traces to StR-001. | spec/usecase/US-019-extract-semantic-declarations.md |
| FND-286 | low | FR-072 introduces the severity token `advisory` into `CoreExtractionResult.diagnostics` as an additive value; the description says consumers keep working, but no TC feeds the pre-change consumer contract (`filament-core-service` reader, `filament-parser-lib` shim) a result containing `advisory` to show it is ignored rather than rejected. | spec/functional/FR-072-semantic-extraction-surface.md (severities bullet); TC-1632 |

## Coverage

- Trace chain: StR-001 → US-019 → FR-069 (requires FR-013, FR-045) → FR-070 → FR-071 → FR-072 (requires FR-046, FR-032); NFR-021 constrains all four.
- Coverage: 48/48 formal AC/CON obligations have at least one TC (FR-069 13, FR-070 12, FR-071 8, FR-072 11, NFR-021 4); US-019 EX-1..4 map to TC-1610/1612/1613/1600.
- Option permutation: table × fence × legacy forms × `legacy_forms: warning|error` × with/without `semantic` block are each a row (TC-1610, 1611, 1617, 1618); snapshot with/without `semantic` context (TC-1632).
- Boundaries and errors: every `semantic.*` code named in FR-069..FR-072 has a row; multiplicity and constraint boundaries follow the vendored `cell-cases.json` (FND-284 for the rest).
- State transitions: not applicable; extraction is a pure function over document and context.
- Edge cases: both forms, mixed legacy sections, self-`$id` fragment, two-file `$ref` cycle, arbitrary fence bytes and CRLF (TC-1629), unrelated-section edits (TC-1637).
- ID formats: US/FR/NFR/TC/AC/CON all conform; TC-1599..1644 are unused on every remote branch (scanned 149 refs); SR-068 follows SR-067.

## Dispositions (applied 2026-09-03, same branch, before Plan-003)

| ID | Disposition |
| --- | --- |
| FND-280 | Fixed — `sourceIdentity` is a `SemanticContext` input; absent → `ix://local/<scope>/spec` + `semantic.source-identity-defaulted` (FR-071 Inputs, AC-7, TC-1648). |
| FND-281 | Fixed — any other table is `free-column-table`; typed-then-legacy is both-forms; zero rows is empty `available`; two headings is `semantic.duplicate-section` (FR-070 form recognition, AC-3, AC-10). |
| FND-282 | Fixed — TC-1636 marked external, runner is `agent-ix/quire-wasm#3` (filed). |
| FND-283 | Fixed — statements rewritten with a named subject and one SHALL each; re-measured below. |
| FND-284 | Fixed — `*`, `0..*`, `0..0`, `2..2 unique`, `a..b` added to AC-5/TC-1614; TC-1621 domain named. |
| FND-285 | Fixed — Options and a Priority line added to US-019. |
| FND-286 | Fixed — `advisory` stays inside the semantic record; `CoreExtractionResult` severities stay `info|warning|error` (FR-072 Outputs, AC-9). |
