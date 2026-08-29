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

> **CR-029 note (2026-08-07) — the warning stands, its premise does not.** That
> sentence was written as a prediction about the corpus, and the corpus
> contradicts it. Measured by this classifier over 197 deduplicated repositories
> and 13,950 binding criteria
> (`~/dev/reports/2026-08-07-ac-property-shape-sweep.md`), **StR is the
> highest-scoring archetype, not the lowest**:
>
> | Archetype | Criteria | Extractable | Rate |
> |---|---|---|---|
> | FR | 12,052 | 2,316 | 19.2% |
> | NFR | 781 | 226 | 28.9% |
> | StR | 1,117 | 325 | **29.1%** |
>
> StR `Validation Criteria` are frequently written as determiner-initial
> capability statements, which the determiner rule catches, while FR criteria
> skew toward `When`/`Given` conditionals that it does not. What survives is the
> **rule, not the description**: a low property-extractability score on any
> archetype — StR included — is still not a quality failure, and this FR is
> still not a gate on the ratio (CON-1). What is retracted is reading the
> sentence above as a statement about what this corpus looks like.

### The property shape taxonomy

The engine SHALL classify each criterion into exactly one shape drawn from a
**closed** enum:

| Shape | Meaning |
|---|---|
| `RoundTrip` | two operations composed, with an identity back-reference in the outcome |
| `Idempotence` | applying an operation a second time yields the first result |
| `Ordering` | an ordering or stable-sort property over a collection |
| `Invariant` | a property asserted to hold unconditionally |
| `Universal` | universally quantified over a generator domain by a universal determiner at a **bounded subject position** |
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

### Where the universal determiner is read

`Universal` is earned by a **closed universal determiner set** (`a`, `an`,
`any`, `every`, `each`, `all`, `no`) standing at the head of a **subject**. The
determiner set is closed English and is not extensible by a module: this is a
grammatical test, not a vocabulary one.

The engine SHALL read that determiner at three bounded positions, first match
wins:

1. **sentence-initial** — `Every finding whose key is absent … defaults to
   warning`;
2. the **subject of a fronted subordinate clause**, immediately after a
   sentence-initial subordinator — `When` **`a command`** `exceeds its timeout,
   the result carries timed_out true`;
3. the **subject of the main clause**, immediately after the comma that closes
   fronted material — `In strict mode,` **`every finding`** `is promoted to an
   error`.

Positions 2 and 3 are pinned to a function word — a sentence-initial
subordinator or preposition, or the comma that closes what it fronts. This is
emphatically **not "a determiner anywhere"**: `returns a diagnostic` SHALL NOT
match, because `a diagnostic` is the outcome and nothing anchors it to a
subject. Where the subject position cannot be bounded — a fronted phrase with
no comma, a determiner buried in a trailing prepositional phrase — the engine
SHALL **refuse** rather than guess, the same discipline the weak-boundary span
guard applies.

The signals distinguish the three so a census can attribute them:
`universal:determiner`, `recall:subject-determiner:fronted-subject`,
`recall:subject-determiner:main-subject`.

> **CR-030 note (2026-08-07) — positions 2 and 3 are new, and are the whole of
> what the recall experiment adopted.** This FR originally anchored the rule at
> the start of the statement only, which is what the "determiner rule's reach"
> open question below was filed against. Three candidate widenings were built
> behind measurement-only cargo features and measured factorially; one passed
> its precision gate and became positions 2 and 3, and two were deleted. The
> full result is CR-030 in [the log](../log.md).

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

`ErrorCase`, `Lifecycle` and `Concurrency` SHALL likewise carry engine built-in
signals rather than being reachable through a declared registry alone. A shape
reachable only by declaration would make its share of a corpus census a
function of which modules a repo activated, and CON-4 forbids a declared idiom
from changing whether a criterion is extractable at all.

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
label, never removes a criterion from extraction.

> **CR-029 note (2026-08-07) — right conclusion, wrong mechanism.** This section
> originally said a missed idiom degrades a label *to a less specific shape*,
> naming `Universal` as the shape it falls to — i.e. degradation, not loss,
> because the degraded cell stays quantified and therefore stays extractable.
> The corpus does not behave that way. Sweeping 13,950 binding criteria twice
> over the same wheel and corpus, once with the module registry active and once
> with it stripped, **61 of the 70 registry-attributable labels (87%) fall to
> `Example`, not `Universal`**; only 5 (7%) fall to `Universal` as the text
> assumed. The conclusion survives intact for a different reason than the one
> given: `extractable` is derived **structurally** under CON-4, so the registry
> cannot move it, and the sweep confirms this at corpus scale — `extractable`
> differs on **0 of 13,950 criteria** between the two runs. Extraction coverage
> is protected by CON-4's derivation rule, not by a fallthrough path. The
> registry's total reach is those 70 cells, **0.5% of the corpus**, and 0
> extraction decisions.

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

The metamorphic shapes carry **no spans in v1**. A criterion carrying no strong
marker anywhere — no modal, no copula, no irregular past — also carries no
spans: the weak inflected-verb marker is `\b\w+(s|ed|ing)\b`, which cannot
distinguish a verb from a plural noun (CR-019 measured 23.4% of cells
satisfying the predicate test through that branch alone), so with nothing
strong to anchor the clause split the first match is as likely to land on a
plural noun as on the predicate. A confidently wrong span is worse for a
generator than an absent one.

The refusal is evaluated **per statement, not per marker**. A statement
carrying a strong marker is anchored, and a weak marker after that anchor is
read as the oracle boundary — which is why AC-1's own example
(`A finding whose key is absent from the merged map defaults to warning`)
yields all three spans: the copula in `is absent` anchors the filter clause, so
the weak `defaults` that follows it is a boundary rather than a guess. A
per-marker reading would refuse spans there and contradict AC-1.

Refusing spans does not change the criterion's shape or its `extractable`
value.

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
- `extractable` — derived in exactly one place from the **structural**
  (pre-boost) property shape and the presence of a predicate marker, so the
  coverage column, any summary line and any downstream consumer cannot disagree
  about the ratio. It is derived from neither the boosted label nor span
  emission: deriving it from the boosted label would let a declared idiom flip
  it, violating CON-4, and deriving it from span emission would make every
  span-less metamorphic criterion unextractable.
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

### Open questions this FR does not settle

Two questions the 2026-08-07 corpus measurement raised are genuine design
decisions rather than defects, and they are filed rather than answered here:

- **A metamorphic label may coexist with `extractable: false`** — 63 criteria,
  13.2% of the metamorphic set, carry a shape such as `RoundTrip` while their
  structure reads `Example`. This FR does not state what a consumer does with a
  `{property: <metamorphic>, extractable: false}` record: honouring the label
  strains CON-4's intent, ignoring it drops real metamorphic properties. Open as
  [agent-ix/quire-rs#46](https://github.com/agent-ix/quire-rs/issues/46).
- ~~**The determiner rule's reach** — the rule is anchored at the start of the
  statement, so quantification expressed by a leading `When`/`Given`/`If` or by
  a mid-sentence `any`/`every` is not seen. Sampling puts recall of
  `extractable` at roughly 30% against ~90% precision, and the loss class is a
  closed English pattern rather than an open vocabulary. Whether to extend the
  anchor is a spec decision, open as
  [agent-ix/quire-rs#45](https://github.com/agent-ix/quire-rs/issues/45).~~
  **Answered by CR-030 (2026-08-07)**, and answered smaller than this bullet
  expected. The anchor was extended to two further bounded subject positions
  ("Where the universal determiner is read", above), which took `extractable`
  from 20.8% to 25.3% of the corpus and recall from 25.2% to 31.3%. It did not
  produce the ~3× the bullet speculated: of the three widenings measured, the
  two with the wide reach failed the precision gate and were deleted. **The
  ~30% recall ceiling stands.** #45 remains open for the ceiling itself, not
  for the anchor.

### The recall ratio and its denominator

Any figure this FR quotes as *recall* SHALL name its denominator, because the
population of criteria the classifier **missed** is unobservable and has to be
estimated from a hand-labelled sample, and two defensible denominators disagree
by several points.

The engine's own surfaces report **`extractable` as a share of all binding
criteria** — that ratio has no estimated term in it and is the only one the
engine computes. Recall figures are measurement-report quantities, and the
2026-08-07 experiment states two:

- **unrestricted** — every binding criterion is in the denominator;
- **`Verification`=Test** — only criteria whose `Verification` cell names a
  test, on the reasoning that a criterion verified by Inspection, Demonstration
  or Analysis is legitimately not a property.

The test/non-test call SHALL be a **word-boundary match on `test`/`tests`**, not
a fixed vocabulary: the corpus carries **1,478 distinct `Verification` values
over 11,933 cells**, and `Test`/`Unit Test`/`Integration Test` together are only
57% of them, so any fixed set silently reclassifies the tail.

A criterion whose `Verification` cell cannot be joined on `row_id` SHALL be
reported as **`unknown`**, never assumed either way — **13.5% of criteria
(1,868 cells)** are in that class. A criterion with no `Verification` cell has
not been declared non-testable; it has simply not been declared.

> **CR-033 note (2026-08-08) — what `{property: <metamorphic>, extractable:
> false}` means to a consumer.** The Phase B corpus re-measurement found **63
> criteria — 13.2% of the metamorphic set — carrying a metamorphic property
> label together with `extractable: false`**, some of them textbook round-trip
> claims (agent-ix/quire-rs#46). This FR did not say which of the two a
> downstream generator honours, and both single-valued answers cost something
> real: honouring the label makes generation depend on module data, which is
> CON-4's spirit violated even where its letter holds; ignoring the label drops
> 63 genuine metamorphic properties.
>
> The record therefore gains a **third, explicit outcome** rather than forcing
> the question onto a boolean. `extraction` is a closed three-valued field:
>
> - **`extractable`** — the structural pass found both a quantified shape and an
>   oracle. A generator emits a test unattended.
> - **`candidate`** — the record carries a metamorphic label the structural pass
>   did not corroborate into extractability: either the shape came from a
>   declared idiom alone, or the structural shape landed but no predicate marker
>   supplied an oracle. A generator MAY emit a test, and MUST mark it as
>   requiring review.
> - **`not-extractable`** — neither. `Example` and `Unclassified` live here, and
>   neither is a defect (AC-5 is unchanged).
>
> **`extractable` is untouched**, and CON-4 is untouched with it: the boolean is
> still derived in exactly one place from the structural shape alone, and TC-790
> still pins that it is identical with and without a declared registry. What is
> new is that `extraction` **may** differ between those two runs — a criterion
> whose metamorphic label came only from a declared idiom reports `candidate`
> with the registry and `not-extractable` without it. That is deliberate and it
> is the reason the third state exists rather than a widening of the boolean:
> `candidate` is **review-gated by construction**, so module data can raise a
> criterion for human attention without ever silently entering an unattended
> generation set. CON-4 constrains what ships unattended; it was never a rule
> that module data may not be read at all.
>
> This adds no shape and no severity key, so CON-1 and CON-3 are unaffected, and
> the `ac` finding stream stays byte-identical (AC-7).

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-052-CON-1 | Property classification SHALL NOT construct a `GrammarFinding` or carry a [FR-048](./FR-048-per-check-grammar-severity.md) `grammar_severity` key. With no severity key there is no promotion path, so `--strict` immunity holds by construction rather than by convention. | Architecture | Test |
| FR-052-CON-2 | The engine SHALL NOT name a test framework, harness or generator library anywhere on this path. The shape-to-harness mapping belongs to the downstream generator, not to `quire-rs`. | Architecture | Inspection |
| FR-052-CON-3 | The property shape enum SHALL be closed in the engine. Extension happens by declaring idiom phrases, never by adding a shape: a shape with no downstream generator silently drops criteria. | Architecture | Inspection |
| FR-052-CON-4 | A criterion's `extractable` value SHALL NOT depend on any module-declared idiom. Extraction coverage rests on the closed structural signals; the idiom registry refines labels only. | Architecture | Test |


> **CR-096 note (2026-08-22):** AC-19 is new — decomposition stops keying on the
> winning label. `agent-ix/quire-rs#228`, epic `agent-ix/quoin#197`.
>
> **The cause was one condition, and it was neither of the two the filing
> proposed.** `#228` asked whether the shape classifier and the span extractor
> run on different normalizations, or whether specific-shape statements are
> routed past the extractor. Neither: both read the same masked statement, and
> `classify_property` carried an explicit gate —
>
> ```rust
> let spans = if structural == PropertyShape::Universal { … } else { None };
> ```
>
> — with the comment *"the metamorphic shapes carry no spans in v1"*. So
> `quantification` had already succeeded on those statements and `decompose`
> would have run; the result was computed and discarded because a
> higher-precedence structural signal had claimed the **label**. Shape and
> decomposition are orthogonal axes — one says what kind of property, the other
> says what to generate over — and a criterion can carry both. `decompose` was
> already general: it reads the masked statement, the determiner end and the
> predicate markers, none of which are `Universal`-specific.
>
> **Measured on this repository's own `spec/`** — 608 criteria, engine at this
> branch, module `spec-artifacts-iso`, counting records that carry **all three**
> spans:
>
> | shape | records | grounded before | grounded after |
> |---|---|---|---|
> | invariant | 39 | 0 | **13** |
> | round-trip | 8 | 0 | **3** |
> | ordering | 18 | 0 | **1** |
> | idempotence | 3 | 1 | 1 |
> | concurrency | 5 | 0 | 0 |
> | lifecycle | 1 | 0 | 0 |
> | **specific total** | **74** | **1** | **18** |
> | universal | 283 | 96 | **96 — unchanged** |
>
> `universal` moving not at all is the check that this widened the gate and
> changed nothing else.
>
> **What this does NOT claim, and the sample that says so.** `grounding` counts
> spans **present**, never spans **correct**. Seven newly-spanned records were
> read by hand: **2 were well-segmented and 5 were not** — domains that swallow
> the predicate (`FR-032-AC-10`, `FR-036-AC-6`), and one precondition/oracle
> boundary falling inside a hyphenated word (`FR-024-AC-10`, `frontmatter-` /
> `less draft`).
>
> **That is a pre-existing extractor defect, not one this introduces.** Seven
> `universal` records — the shipped path — were sampled the same way: **4 good,
> 3 poor**, including the identical mid-word split on `FR-002-AC-2`
> (`merged-` / `validated`) and a domain of `of the 6 Locator` on
> `FR-011-AC-1`. The boundary heuristic degrades on long clause-heavy
> statements, and specific-shape criteria in this corpus are longer, which is
> why their sample reads worse. Filed separately rather than fixed here, so this
> change stays one thing: `agent-ix/quire-rs#241`.
>
> **The v1 assertion was inverted, not deleted.** TC-780 asserted
> `spans.is_none()` for a quantified round-trip. It now asserts `is_some()`
> *and* adds the opposite direction — an unquantified `idempotence` statement
> still carries no spans — so nothing is invented for a shape that genuinely
> has no domain.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-052-AC-1 | A criterion reading `A finding whose key is absent from the merged map defaults to warning` classifies as `Universal` and carries a populated `domain`, `precondition` and `oracle` span. | Test (TC-779) |
| FR-052-AC-2 | A criterion carrying a round-trip, idempotence, ordering or invariant signal classifies as that metamorphic shape, and a universally quantified round-trip criterion classifies as `RoundTrip` rather than `Universal`, because the fixed precedence ranks structure above quantification. | Test (TC-780) |
| FR-052-AC-3 | A criterion whose only idiom phrase sits inside an inline code span fires no signal from that phrase, while the same criterion with the phrase unquoted fires it. | Test (TC-781) |
| FR-052-AC-4 | A criterion carrying no strong marker to anchor its clause split classifies as `Universal` with all three spans absent, so a consumer never receives a confidently wrong span, while a criterion whose weak boundary marker follows a strong anchor carries its spans. | Test (TC-782) |
| FR-052-AC-5 | A criterion describing one specific scenario classifies as `Example` with `extractable` false, and that same criterion contributes no finding to any `ac` check. | Test (TC-783) |
| FR-052-AC-6 | Every span on a classification record satisfies `statement[span.start..span.end] == span.text`, and the spans of one record are in bounds, non-overlapping, and ascending by start offset. | Test (TC-784) |
| FR-052-AC-7 | A fixture corpus checked with property classification reachable yields the same `ac` findings, in the same order and with the same fields, as the same corpus checked before classification existed. | Test (TC-785) |
| FR-052-AC-8 | A module declaring a `property_idioms` registry merges its phrases first-wins over the engine built-in idioms, and a module declaring none classifies against the built-ins alone. | Test (TC-786) |
| FR-052-AC-9 | A document classified for property shape yields exactly the criteria the `ac` grammar binds to — FR and NFR `Acceptance Criteria` rows, StR `Validation Criteria` rows, and their supplement subsections — while a US or IT document yields no records and no finding. | Test (TC-787) |
| FR-052-AC-10 | A coverage report over a corpus carrying criteria contains one `CriteriaCounts` entry per minting document plus the two new totals, a corpus carrying no criteria contains an empty entry list, and two runs over identical inputs serialize byte-identically. | Test (TC-788) |
| FR-052-AC-11 | A fixture document classified through the PyO3 `classify_properties` binding carries the same records, field for field, as the in-process Rust call over that document. | Test (TC-789) |
| FR-052-AC-12 | Every criterion in a fixture corpus carries the same `extractable` value with a `property_idioms` registry declared and with none declared, so only the `property` label differs between the two runs (CON-4). | Test (TC-790) |
| FR-052-AC-13 | A criterion composing two operations with an identity back-reference in its outcome classifies as `RoundTrip` with no registry declared, a criterion pairing a repetition adverb with an equality verb classifies as `Idempotence`, and a criterion whose only ordering-adjacent word is a bare `deterministic`, `before` or `order` classifies as neither. | Test (TC-791) |
| FR-052-AC-14 | A criterion fronting its condition with a subordinator (`When a command exceeds its timeout, …`) or fronting material closed by a comma before a determiner-headed main subject (`In strict mode, every finding …`) classifies as `Universal` and extractable, carrying the subject-position signal that names which of the two fired; while a definite determiner, an unbounded fronted phrase with no comma, and a determiner standing in the outcome rather than the subject each classify exactly as they did before the position widened. | Test (TC-792) |
| FR-052-AC-15 | Widening the determiner's position moves no `ac` finding — the finding stream over a fixture corpus is unchanged, field for field and order for order — and every criterion the widened positions decline keeps its exact prior `property`, `extractable` and `signals` values. | Test (TC-793) |
| FR-052-AC-16 | A criterion whose `extractable` is true reports `extraction: extractable`; a criterion whose `extractable` is false and whose `property` is one of the four metamorphic shapes reports `extraction: candidate`; every other criterion reports `extraction: not-extractable`, so an `Example` or `Unclassified` criterion is never a candidate. | Test (TC-795) |
| FR-052-AC-17 | `extraction` is derived from `property` and `extractable` alone and feeds back into neither: every criterion in a fixture corpus carries the same `extractable` value with a `property_idioms` registry declared and with none (CON-4 unchanged), while a criterion whose metamorphic label came only from a declared idiom reports `candidate` with the registry declared and `not-extractable` without it. | Test (TC-796) |
| FR-052-AC-18 | A property shape is **specific** when it names what property to write — every shape but the `universal` catch-all, `example` and `unclassified`. The per-document tally reports `specific_shaped` (extractable **and** specific) alongside `property_shaped`, and a `grounding` tally giving, per shape label, how many of that shape's records carry `domain` / `precondition` / `oracle` and how many carry all three. The specific set is a reading-list distinction, not a quality ranking, and gates nothing. | Test (TC-989) |
| FR-052-AC-19 | Decomposition keys on quantification, not on the winning structural label: a statement that is universally quantified carries `domain` / `precondition` / `oracle` spans whichever shape the precedence order assigns it, and a statement that states no domain carries none whatever its shape. A declared idiom can still neither add nor remove spans (CON-4 unchanged). | Test (TC-780, TC-990) |
| FR-052-AC-20 | Clause boundaries are whole-word and subject-grounded: neither half of a hyphenated compound becomes a predicate marker, a filter marker after the first predicate does not widen the domain retroactively, and a partitive quantifier excludes its counting frame from the domain. When no defensible strong boundary exists, the classifier emits no spans and retains the stable refusal signal. | Test (TC-1078) |

## Dependencies

- **Upstream**: [FR-047](./FR-047-acceptance-criteria-grammar.md) (the `ac` binding, the criteria-cell collection, the CR-017 mask and the shape axis carried through), [FR-042](./FR-042-requirement-grammar-check.md) (the grammar framework and its vocabulary assembly), [FR-014](./FR-014-module-activation.md) (manifest loading for the `property_idioms` registry), [FR-010](./FR-010-query-api.md) (table extraction)
- **Downstream**: [FR-050](./FR-050-declarative-coverage-computation.md) (the rollup carries the per-document counts); the `spec-correctness` workflow consumes the records to generate property tests; `spec-artifacts-iso` declares the ISO `property_idioms` registry (follow-up change in that module)
