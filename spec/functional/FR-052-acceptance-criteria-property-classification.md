---
id: FR-052
title: "Acceptance-Criteria Property Classification"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-047"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: "references"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-014"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-017"
    type: "implements"
---
# FR-052: Acceptance-Criteria Property Classification

## Description

`quire-rs` SHALL classify every acceptance criterion the
[FR-047](./FR-047-acceptance-criteria-grammar.md) `ac` grammar binds to by its
**property shape** — whether the criterion quantifies over an input domain, and
if so which clauses carry its generator domain, its precondition and its oracle.
The classification is **metadata, never a finding**. It exists so a downstream
generator can pick a property-test template and fill its holes without
re-parsing prose.

The metadata surfaces in two places and nowhere else: the
[FR-050](./FR-050-declarative-coverage-computation.md) coverage rollup carries
per-document counts, and the classification records are returned by an engine
entry point and its PyO3 binding. Neither surface is a verdict.

### Why classification is not a check

A pass/fail gate on "is this criterion property-shaped" invites an author to
reword a criterion to satisfy a checker rather than to describe the system, and
a criterion that is not property-shaped is not a defect: a specific scenario
verified by Inspection or Demonstration is a legitimate criterion. The
classification is therefore a **gradient that reports**, not a gate that
dictates, and CON-1 makes that boundary structural rather than conventional.

CR-020 already recorded the consequence this FR must not misread:

> StR criteria are validated by demonstration rather than quantified over an
> input domain, so they will legitimately score low on property-extractability
> — the property-shape classifier (#20) must not read that as a quality
> failure.

### The property shape taxonomy

The engine SHALL classify each criterion into exactly one shape drawn from a
**closed** enum:

| Shape | Meaning |
|---|---|
| `RoundTrip` | two operations composed, with an identity back-reference in the outcome |
| `Idempotence` | applying an operation a second time yields the first result |
| `Ordering` | an ordering or stable-sort property over a collection |
| `Invariant` | a property asserted to hold unconditionally |
| `Universal` | universally quantified over a generator domain by its determiner |
| `ErrorCase` | a criterion whose outcome is a named failure over a class of bad inputs |
| `Lifecycle` | a criterion quantifying over an ordered sequence of state transitions |
| `Concurrency` | a criterion quantifying over interleavings rather than over values |
| `Example` | one specific scenario, not quantified |
| `Unclassified` | no signal fired |

`Example` and `Unclassified` are first-class outcomes, not error states.
`Unclassified` is what the engine returns when nothing matched, and it is
never reported as a defect.

### Fixed precedence

Shape selection SHALL follow one **total, fixed precedence**, first match wins:

1. `RoundTrip`
2. `Idempotence`
3. `Ordering`
4. `Invariant`
5. `ErrorCase`
6. `Lifecycle`
7. `Concurrency`
8. `Universal`
9. `Example`
10. `Unclassified`

Structure therefore outranks quantification: a universally quantified
round-trip criterion classifies `RoundTrip`. The precedence is pinned here so
it is a specification decision rather than an implementation accident, and so
two engine versions cannot disagree about a corpus census.

### Closed structural signals, and the idiom registry as a booster

Signals are detected on the CR-017 masked copy of the statement, so a
backticked phrase is a mention rather than a use, and spans are emitted against
the unmasked original — the same byte-length-preserving trick `outcome_clause`
already relies on.

The metamorphic shapes SHALL be reachable from **engine-owned closed signal
sets** alone:

- `RoundTrip` — a composition of two operations together with an identity
  back-reference in the outcome (`yields the input`, `returns the original`,
  `byte-identical`, `equals the source`, `losslessly`, `round-trips`). The
  back-reference set is closed English, not domain vocabulary.
- `Idempotence` — a repetition adverb (`twice`, `again`, `a second time`,
  `repeatedly`, `re-run`, `re-applied`) in the same clause as an equality verb
  or `the same`.
- `Ordering` — a small ordering lexicon (`sorted`, `sort order`, `precedes`,
  `stable order`, `first-wins`, `in <corpus|path|input|document|declaration>
  order`, `order is preserved`). A bare `deterministic`, `before` or `order` is
  deliberately **not** an ordering signal — each swallows a large slice of
  `Example`.
- `Invariant` — `always`, `never`, `invariant`, `regardless of`, `under any`,
  `for any`, `in every case`, `no matter`.

A Filament module MAY additionally declare a `property_idioms:` registry in its
`manifest.yaml`, merged first-wins over the engine built-ins — the same pattern
as the FR-043 `lexicon`, `observable_verbs` and `vacuous_predicates`
registries. The registry is applied **after** the structural signals, and its
role is to refine the label of a criterion the structure already caught and to
catch the residual.

The registry is therefore a **booster, never a prerequisite**. Verb pairs
(`parse`/`serialize`, `encode`/`decode`, `marshal`/`unmarshal`,
`compress`/`inflate`) are an unbounded space, and CR-014's rule is that an open
set whose membership is *required* to earn a label is unsafe — that is what
retired `no-observable-outcome`. CON-4 keeps the exposure bounded by making
extraction coverage independent of the registry: a missed idiom degrades a
label to a less specific shape, never removes a criterion from extraction.

### Span model

For a `Universal` criterion the engine SHALL decompose the statement into three
spans:

- **domain** — the quantified noun phrase after the determiner, i.e. what a
  generator produces.
- **precondition** — the restrictive filter clause (`whose …`, `that …`,
  `with …`, `when …`), i.e. which generated values are in scope.
- **oracle** — the predicate and outcome, i.e. what must hold of them.

Spans are **statement-relative, never file-relative**. A supplement statement is
synthesised by joining wrapped source lines, so an offset does not index the
file; only the record's `line` field is file-relative. Each span SHALL carry
both a byte offset pair into the untruncated statement and the span text
itself: offsets so the engine can assert the spans partition the statement and
a viewer can render highlights, and text because a UTF-16 consumer handed raw
UTF-8 byte offsets is a defect generator. The redundancy is checkable, and
AC-6 checks it.

The metamorphic shapes carry **no spans in v1**. A criterion whose oracle
boundary rests only on the weak inflected-verb marker also carries no spans:
that marker cannot distinguish a verb from a plural noun (CR-019 measured 23.4%
of cells satisfying the predicate test through that branch alone), and a
confidently wrong span is worse for a generator than an absent one. Refusing
spans does not change the criterion's shape or its `extractable` value.

### The classification record

Each record SHALL carry:

- `row_id` — the criterion's own id (`FR-047-AC-3`), read from the `ID` column
  of the criteria table or from the `### <doc-id>-<kind>-N` supplement heading.
  Without it a downstream generator cannot emit the tracking tag that coverage
  reconciliation keys on, so the trace chain breaks silently.
- `statement` — the **untruncated** statement. `GrammarFinding.statement` is
  excerpted at 160 chars, so classification cannot ride on a finding-shaped
  record; span offsets index this field.
- `line` — the 1-based file-relative line of the criterion.
- `shape` — the FR-047 shape axis, carried through unchanged.
- `property` — the FR-052 shape axis.
- `extractable` — derived in exactly one place from the property shape and the
  presence of an oracle, so the coverage column, any summary line and any
  downstream consumer cannot disagree about the ratio.
- `domain`, `precondition`, `oracle` — the optional spans above.
- `signals` — the stable ids of the signals that fired, in the order they were
  evaluated.

There is **no confidence float**. A float is unreviewable and invites threshold
tuning, which is the failure mode this program has already hit; `Unclassified`
is the explicit outcome instead. `signals` is the audit trail that makes every
record re-derivable from its own output — the direct answer to CR-019's finding
that the original sweep script was never saved and the decision could not be
re-derived.

A `SchemaShaped` shape is **deliberately excluded**. Its signal would be "an
OpenAPI schema exists in the repo" — repo state rather than statement text —
which would break determinism (NFR-006) and make two corpora incomparable. A
downstream consumer may upgrade a record on its own evidence.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-052-CON-1 | Property classification SHALL NOT construct a `GrammarFinding` or carry a [FR-048](./FR-048-per-check-grammar-severity.md) `grammar_severity` key. With no severity key there is no promotion path, so `--strict` immunity holds by construction rather than by convention. | Architecture | Test |
| FR-052-CON-2 | The engine SHALL NOT name a test framework, harness or generator library anywhere on this path. The shape-to-harness mapping belongs to the downstream generator, not to `quire-rs`. | Architecture | Inspection |
| FR-052-CON-3 | The property shape enum SHALL be closed in the engine. Extension happens by declaring idiom phrases, never by adding a shape: a shape with no downstream generator silently drops criteria. | Architecture | Inspection |
| FR-052-CON-4 | A criterion's `extractable` value SHALL NOT depend on any module-declared idiom. Extraction coverage rests on the closed structural signals; the idiom registry refines labels only. | Architecture | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-052-AC-1 | A criterion reading `A finding whose key is absent from the merged map defaults to warning` classifies as `Universal` and carries a populated `domain`, `precondition` and `oracle` span. | Test (TC-779) |
| FR-052-AC-2 | A criterion carrying a round-trip, idempotence, ordering or invariant signal classifies as that metamorphic shape, and a universally quantified round-trip criterion classifies as `RoundTrip` rather than `Universal`, because the fixed precedence ranks structure above quantification. | Test (TC-780) |
| FR-052-AC-3 | A criterion whose only idiom phrase sits inside an inline code span fires no signal from that phrase, while the same criterion with the phrase unquoted fires it. | Test (TC-781) |
| FR-052-AC-4 | A criterion whose oracle boundary rests only on the weak inflected-verb marker classifies as `Universal` with all three spans absent, so a consumer never receives a confidently wrong span. | Test (TC-782) |
| FR-052-AC-5 | A criterion describing one specific scenario classifies as `Example` with `extractable` false, and that same criterion contributes no finding to any `ac` check. | Test (TC-783) |
| FR-052-AC-6 | Every span on a classification record satisfies `statement[span.start..span.end] == span.text`, and the spans of one record are in bounds, non-overlapping, and ascending by start offset. | Test (TC-784) |
| FR-052-AC-7 | A fixture corpus checked with property classification reachable yields the same `ac` findings, in the same order and with the same fields, as the same corpus checked before classification existed. | Test (TC-785) |
| FR-052-AC-8 | A module declaring a `property_idioms` registry merges its phrases first-wins over the engine built-in idioms, and a module declaring none classifies against the built-ins alone. | Test (TC-786) |
| FR-052-AC-9 | A document classified for property shape yields exactly the criteria the `ac` grammar binds to — FR and NFR `Acceptance Criteria` rows, StR `Validation Criteria` rows, and their supplement subsections — while a US or IT document yields no records and no finding. | Test (TC-787) |
| FR-052-AC-10 | A coverage report over a corpus carrying criteria contains one `CriteriaCounts` entry per minting document plus the two new totals, a corpus carrying no criteria contains an empty entry list, and two runs over identical inputs serialize byte-identically. | Test (TC-788) |
| FR-052-AC-11 | A fixture document classified through the PyO3 `classify_properties` binding carries the same records, field for field, as the in-process Rust call over that document. | Test (TC-789) |
| FR-052-AC-12 | Every criterion in a fixture corpus carries the same `extractable` value with a `property_idioms` registry declared and with none declared, so only the `property` label differs between the two runs (CON-4). | Test (TC-790) |
| FR-052-AC-13 | A criterion composing two operations with an identity back-reference in its outcome classifies as `RoundTrip` with no registry declared, a criterion pairing a repetition adverb with an equality verb classifies as `Idempotence`, and a criterion whose only ordering-adjacent word is a bare `deterministic`, `before` or `order` classifies as neither. | Test (TC-791) |

## Dependencies

- **Upstream**: [FR-047](./FR-047-acceptance-criteria-grammar.md) (the `ac` binding, the criteria-cell collection, the CR-017 mask and the shape axis carried through), [FR-042](./FR-042-requirement-grammar-check.md) (the grammar framework and its vocabulary assembly), [FR-014](./FR-014-module-activation.md) (manifest loading for the `property_idioms` registry), [FR-010](./FR-010-query-api.md) (table extraction)
- **Downstream**: [FR-050](./FR-050-declarative-coverage-computation.md) (the rollup carries the per-document counts); the `spec-correctness` workflow consumes the records to generate property tests; `spec-artifacts-iso` declares the ISO `property_idioms` registry (follow-up change in that module)
