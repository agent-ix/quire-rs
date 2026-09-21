---
id: FR-077
title: "Trace search: structural forward/inverse lookup over the symbol graph"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-051"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-062"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "traces_to"
---

# FR-077: Trace search: structural forward/inverse lookup over the symbol graph

## Description

Agents answering "where is FR-047 implemented and tested?" or "what does this file verify?" today
grep the tree and read every hit to judge whether it is a real binding or an incidental mention — a
hand-rolled pipeline, repeated per question, that cannot make the one distinction that matters: a
tracking tag inside a test declaration is a **claim** the coverage rollup already trusts; the same id
in a docstring, a comment, or a string literal is not. `src/symbols/trace.rs` (FR-051, FR-062) already
computes the claims — `verifies` and `implements` — from the module-declared trace-tag grammar. Nothing
indexes them for lookup, and nothing at all accounts for the non-claim mentions a reader still needs to
see (PLAT-844).

This requirement adds a query surface over the existing `SymbolGraph`, and one new, additive
scan alongside it: every id-shaped token that is not part of a `verifies`/`implements` claim is
classified into `SymbolGraph.mentions`, so a mention is reported rather than silently absent from an
answer that only ever looked at the claim channels. **The forward direction** — given a trace id, every
symbol that claims it and every symbol that merely cites it. **The inverse direction** — given a file or
a symbol, every id it claims and every id it cites. Both read the same underlying graph; neither
reimplements the trace-tag grammar the binder already owns.

### What is new versus what is reused

Reused, unmodified: `VerifiesRelation`, `ImplementsRelation`, the canonical-marker and legacy-form
matching in `bind()`, `normalized_trace_id()`, `Symbol::attached_source()`. This requirement adds no
second copy of any of that.

New: a `Mention`/`MentionBucket` pair (`SymbolGraph.mentions`) recording every generic id-shaped token
found across a symbol's full attached source that is not itself a claim, whether that is a near-miss
tag on a test, an orphaned legacy tag on production code, or a plain-language citation in a comment,
docstring, or string literal that matches no declared form at all. And a query layer
(`src/symbols/trace_search.rs`) that indexes `verifies`/`implements`/`mentions` by trace id and by
`path`/`qualified_name` — pure lookup over already-computed data, no regex.

### The claim-vs-citation split is structural, not a flag

A `SearchResult` carries `verifies: Vec<VerifiesRelation>`, `implements: Vec<ImplementsRelation>` and
`citations: Vec<Mention>` as three separate typed fields — never one list with a discriminator a caller
can fail to check. (Not wrapped in a `claims` struct: three distinct fields already deliver the
property this section states, and a wrapper added only to match this document's own earlier prose would
be spec chasing its own tail rather than describing the shipped surface. A JSON encoding of this shape,
if one groups `verifies`/`implements` under a `claims` key, is PLAT-879's to design.) This is the same
discipline
FR-062 already applies to keeping `verifies` and `implements` apart (CR-061): the failure mode is a
typo, or a reader skimming past a flag, silently promoting a citation into evidence.

### Engine knows ids, not hierarchy

`traceability.rs` states plainly that quire knows nothing of "FR", "AC", or "TC" — the vocabulary is
module data. So this requirement's matching is **exact**, after the same normalization
`normalized_trace_id()` already applies (case/punctuation-insensitive), with no implicit
parent-to-child expansion: a query for `FR-047` matches only symbols claiming or citing the literal
id `FR-047`, never `FR-047-AC-2` by inference. A caller wanting the wider family opts in explicitly —
that is a CLI-level, user-requested string-prefix match on a separator boundary (so `FR-047` cannot
match `FR-0470`), not engine-side structural knowledge, and is out of this requirement's scope (a
`quire-cli` concern, tracked on the follow-up ticket).

### Cross-language honesty

Only Rust runs on the tree-sitter AST (PLAT-843); `python.rs` and `typescript.rs` stay line-structural
until PLAT-851. Every record this requirement emits carries the language it came from and a
`confidence` of `structural` (Rust) or `line_heuristic` (Python, TypeScript) — the tool never presents
a Python hit with the same evidentiary weight as a Rust one.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|---------------|
| FR-077-AC-1 | A forward query by exact trace id returns every `verifies` and `implements` relation bound to that id on `SearchResult`'s own `verifies`/`implements` fields, and every non-claim mention on a separate `citations` field — three distinct typed fields, never one list with a flag. (A JSON encoding of this shape is PLAT-879's, not this requirement's.) | Test (TC-1887) |
| FR-077-AC-2 | Every generic id-shaped token found in a scanned symbol's attached source that is not part of a `verifies`/`implements` claim on that same symbol is recorded in `SymbolGraph.mentions`, classified into exactly one of `EvidenceNearMiss`, `ProductionOrphanTag`, or `Mention` — no id-shaped token is silently dropped from all three channels. | Test (TC-1888) |
| FR-077-AC-3 | A query id matching nothing in `verifies`, `implements`, or `mentions` returns a fully-shaped result with `resolved: false` and empty arrays, not an omitted or bare-empty payload. | Test (TC-1889) |
| FR-077-AC-4 | An inverse query names a symbol either by its exact `path#qualified_name` ref (returning exactly that symbol's claims and citations) or by a bare unqualified name; a bare name matching more than one symbol returns every match under `ambiguous_matches` rather than selecting one. | Test (TC-1890, TC-1891) |
| FR-077-AC-5 | `Mention` carries the `language` it was extracted from directly; `VerifiesRelation`/`ImplementsRelation` carry none, and `symbol_language()` joins one back from `extraction` by `symbol_id`. Either way, one documented function (`language_confidence`) maps `language` to a confidence label (`structural` for Rust — AST-grounded, PLAT-843 — `line_heuristic` for Python/TypeScript, pre-PLAT-851), so a caller never presents a result set as uniformly grounded across languages. | Test (TC-1892) |
| FR-077-AC-6 | Trace-id matching is exact after `normalized_trace_id()` normalization; the engine performs no implicit FR/AC/TC hierarchy expansion. | Test (TC-1893) |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-077-CON-1 | The query/index layer (`src/symbols/trace_search.rs`) SHALL NOT declare or match any trace-tag-shaped regex; it reads only the data `src/symbols/trace.rs` already computed. No trace-form regex lives outside `trace.rs`. | Design | Inspection |
| FR-077-CON-2 | This requirement SHALL NOT execute, build, or type-resolve the extracted code, inheriting FR-051-CON-1. | Design | Inspection |
| FR-077-CON-3 | The `mentions` scan SHALL NOT mask string-literal content — an id-shaped token inside a string literal is reported as a citation, not hidden. This is deliberately narrower than the existing legacy-form masking on the `verifies`/`non_binding_tags` claim paths, which stays unchanged: hiding an id the caller asked about is the failure mode this scan exists to avoid. | Test (TC-1894) |

## Dependencies

- **Upstream**: [FR-051](./FR-051-source-symbol-extraction.md) (the symbol graph and trace-tag grammar this indexes), [FR-062](./FR-062-implements-relation.md) (the `implements` relation this reads unmodified)
- **Downstream**: a `quire-cli` `trace` subcommand and its accompanying agent skill (tracked as a follow-up ticket, blocked on this one) — the CLI surface and package distribution are a separate repo's decision, not this requirement's.
