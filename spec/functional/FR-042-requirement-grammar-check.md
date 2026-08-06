---
id: FR-042
title: "Requirement-Grammar Check (EARS)"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-010"
    type: "requires"
    cardinality: "1:1"
---

## Description

`quire-rs` SHALL provide a **grammar-check framework** that evaluates the
natural-language requirement statements inside requirement-bearing artifacts
against a registered **grammar**, and SHALL ship **EARS** (Easy Approach to
Requirements Syntax) as its first grammar. Grammar checking is a posture
distinct from declarative lint ([FR-036](./FR-036-declarative-lint-rules.md)) and
structural validation ([FR-032](./FR-032-validate-document.md)): a grammar
classifies and checks the *prose of individual normative statements*, whereas
lint checks declarative content patterns and validation checks document
structure.

The framework SHALL register each grammar against an `(archetype, section)`
binding, so a grammar runs only on the sections it governs. When a
requirement-bearing document is validated, the framework SHALL segment each
governed section into **normative statements** — a sentence, list item, or
table cell bearing a modal verb (`shall`/`shall not`) — and SHALL skip text
inside fenced code blocks, blockquotes, and reference lines.

The framework SHALL route every grammar finding into the validation result by
**severity**: when a finding's severity is `warning`, the framework SHALL record
it in `ValidationResult.warnings` without setting `is_valid` false; when the
severity is `error`, the framework SHALL record it in `ValidationResult.errors`
and SHALL fail validation. Severity SHALL be sourced from configuration so a
deployment can promote a grammar from advisory to enforcing without a code
change.

### EARS grammar

The EARS grammar SHALL classify each normative statement into exactly one
pattern — `ubiquitous`, `event` (`When …`), `state` (`While …`), `unwanted`
(`If … then …`), `optional` (`Where …`), or `complex` (a combination) — or SHALL
mark it `unclassifiable` when no pattern matches.

For each statement, the EARS grammar SHALL emit a finding when the statement
violates a clause rule:

1. **non-singular** — the statement contains more than one `shall`.
2. **missing-subject** — the statement names no system/actor subject.
3. **vague-response** — the response verb is vague (`support`, `handle`,
   `manage`, `process`, `deal with`, `provide`, `enable`, `be able to`). The
   check is **object-aware**: a verb is flagged only when its object is abstract
   or absent — a concrete object surface (`provide an endpoint`, `process push
   events`), a **backticked code identifier** (`provide \`CodeBlockEditor\``), or
   a mechanism/quantitative qualifier (`handle X by Y`, `process within 16 ms`)
   states a verifiable response and is not flagged. `be able to`
   is the one verb-intrinsic case (capability-not-behavior) and is always flagged.
4. **non-canonical-trigger** — the statement leads with a non-EARS trigger
   (`On …`, `Upon`, `After`, `Before`, `Once`, `During`) instead of `When`/
   `While`.

The EARS grammar SHALL apply per-archetype dialects: an enumerated
`The <system> SHALL:` stem followed by a numbered response list SHALL count as a
single statement (not non-singular); a `StR` statement SHALL accept a stakeholder
or product subject (not only "the system"); and an `NFR` statement absent any
trigger SHALL NOT be reported as a defect. The EARS grammar SHALL NOT flag
passive voice.

### Non-goals (v1)

- The framework SHALL NOT judge whether the author chose the *semantically*
  correct keyword (e.g. `When` used for a continuous `While` state); that
  judgment is left to the agent review lens.
- No grammar other than EARS ships in v1; later grammars register onto the
  same framework. (Since realized by the `ac` acceptance-criteria grammar,
  [FR-047](./FR-047-acceptance-criteria-grammar.md). EARS stays the grammar for
  *normative statements* — obligations — while `ac` grades acceptance criteria
  as verification statements, whose canonical shape is the assertion, not an
  EARS obligation (CR-013). A `US` story grammar remains future work.)
- **`ears` does NOT adopt the CR-017 mention/use distinction.** The `ac` grammar
  reads a masked copy of a statement, in which each closed inline code span's
  contents are neutralized, when classifying shape and counting obligations, so
  a *quoted* keyword is example data rather than a use. `ears` deliberately does
  not, and this is a measured decision rather than an open question — see the
  CR-021 note below.

> **CR-021 note (2026-08-06):** Whether `ears` should adopt CR-017's mention/use
> mask was left open by CR-017 and is now closed: **it does not, yet.** The mask
> was implemented at the four `ears` sites that read keywords — pattern
> classification, `shall` counting, subject detection and trigger detection —
> and measured over the same corpus as CR-020 with
> `scripts/ac_corpus_sweep.py` (36,584 cells, 197 repos).
>
> Applying it to those four sites alone moved `ears` findings by 4: `non-singular`
> 1,562 → 1,560 and **`unclassifiable` 448 → 450**. The rise is the tell. When
> classification is masked but *segmentation* is not, a sentence can be selected
> as a normative statement because it contains a `shall`, then judged
> unclassifiable because that `shall` was quoted and does not count. Masking half
> the pipeline trades one false positive for a different one.
>
> Masking the segmentation gate too, and reusing `ac`'s code-span-aware
> `split_sentences` — the pair CR-017 needed in `ac` — removes the regression
> (`unclassifiable` back to 448) and leaves `non-singular` at 1,559, a net −3.
> But reading those three shows **only one is a mention/use fix**:
>
> | Repo | Statement | Verdict |
> |---|---|---|
> | quire-rs | ``table cell bearing a modal verb (`shall`/`shall not`) — and SHALL skip text`` | correct fix — two of the three modals are quoted |
> | ix-flow | ``The item SHALL carry a string `id` and SHALL be persisted under its …`` | **regression** — two genuine obligations, silenced |
> | quoin | ``… `plugin install` SHALL record …, `plugin list` SHALL print …`` | **regression** — two genuine obligations, silenced |
>
> The cause is not the mask. It is that `ears` segmentation is **line-based**
> (recorded above as a v1 limitation). When a code span wraps across source
> lines, the following line *begins inside* an unterminated span; code-span-aware
> splitting then treats the remainder of that line as quoted, and the mask
> neutralizes real modals in it. `ac` does not hit this: its statements are table
> cells and supplement bodies, not arbitrary wrapped prose lines.
>
> One true fix against two regressions is a net loss, so the mask is declined for
> now. **This is not a decision that the distinction is wrong for `ears`** — it is
> the same distinction, and the two grammars disagreeing is a real inconsistency
> that will keep biting hardest in the repos that document grammars. It is a
> decision that the mask cannot land safely on line-based segmentation. Adopting
> it is therefore conditional on the paragraph-joining segmentation refinement
> already recorded as planned: once a statement is a whole sentence rather than a
> line fragment, re-run this measurement and expect the two regressions to
> disappear. Until then the divergence is intentional and documented.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-042-AC-1 | Each of the six EARS patterns (`ubiquitous`, `event`, `state`, `unwanted`, `optional`, `complex`) is classified from a representative statement, and a statement matching none is reported `unclassifiable`. | Test |
| FR-042-AC-2 | A statement with two `shall` clauses yields exactly one `non-singular` finding; an enumerated `The X SHALL:` stem followed by a numbered list yields none. | Test |
| FR-042-AC-3 | A statement with no system subject yields a `missing-subject` finding; a `StR` statement with a stakeholder subject (`The operator …`) does not. | Test |
| FR-042-AC-4 | A statement using a vague response verb (`shall support`) yields a `vague-response` finding; a passive-voice statement (`shall be included`) yields none. | Test |
| FR-042-AC-5 | A statement leading with `On startup, … shall …` yields a `non-canonical-trigger` finding; an `NFR` statement with no trigger yields none. | Test |
| FR-042-AC-6 | A grammar runs only against its bound `(archetype, section)` pairs: an EARS rule bound to FR `Description` produces no findings against an FR `Dependencies` section or an `IT` document. | Test |
| FR-042-AC-7 | A `warning`-severity finding is recorded in `ValidationResult.warnings` and leaves `is_valid` true; promoting the same finding to `error` records it in `errors` and sets `is_valid` false. | Test |
| FR-042-AC-8 | Each finding carries the offending statement excerpt, a 1-based line number, the matched pattern, and a severity. | Test |
| FR-042-AC-9 | Statements inside fenced code blocks, blockquotes, and reference lines are not segmented as normative statements and yield no findings. | Test |
| FR-042-AC-10 | The framework entry point is exposed through the Python (PyO3) binding and returns the same findings as the in-process Rust call for a fixture document. | Test |

## Dependencies

- **Upstream**: [FR-032](./FR-032-validate-document.md) (requires — findings merge into `ValidationResult`), [FR-010](./FR-010-query-api.md) (requires — section/table/list extraction)
- **Downstream**: the authoring (`/specify`), coverage (`/spec-matrix`), and review (`/spec-review`) workflows consume grammar findings; the `US-014` author-validates-markdown story is extended by this gate.
