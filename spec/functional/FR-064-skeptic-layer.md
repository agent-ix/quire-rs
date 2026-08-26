---
id: FR-064
title: "The skeptic layer"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/FR-051"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/StR-001"
    type: "traces_to"
---

# FR-064: The skeptic layer

## Description

Battletest pass 2's verdict on this toolchain was **good reporters, poor skeptics**. Every
conclusion-changing finding of that pass came from a human reading code. Two of them are
mechanizable, and this is those two.

**A property suite that asserted nothing.** TC-1596 was green throughout while checking a measured
**2.3%** of its samples — 4,000 samples split `Ok(Some)` 2.3%, `Ok(None)` 79.0%, `Err` 18.8%, with
the assertion inside the `Ok(Some)` arm. Nothing in any tool surface said so.

**An oracle that was a copy of the code under test.** TC-1598's oracle was character-for-character
the implementation, redundant branch included. It passed forever, because a copy computes the same
answer the same way and therefore asserts only that the code equals itself. Replacing it with a real
oracle immediately exposed a genuine Windows containment gap.

The engine SHALL report both shapes as **suspicions**.

### A suspicion is not a finding, and never a failure

A suspicion says *this looks like a known-bad shape* and carries the measurement that made it look
that way. Neither check can be certain: a guarded assertion is sometimes exactly right, and an
oracle legitimately resembles the code when the behaviour is a transformation with one obvious
spelling.

Advisory-first here is not only about blast radius. These fire on **test code**, and a check that
can fail somebody's build over a heuristic about their assertions will be switched off within a
week — at which point it detects nothing at all.

`evidence` is required for the same reason. A suspicion a reader cannot check in one look is one
they learn to scroll past.

### Absence of an assertion is not the finding

The first draft also reported a bound symbol containing **no assertion macro**. Measured on this
repository that was 57 of 65 suspicions, and **12 of 12 sampled were rule, 0 real**:

```rust
fn document_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}   // the assertion is at COMPILE time
    assert_send_sync::<QuireDocument>();
}

fn never_panics_on_arbitrary_utf8(s in "\PC*") {
    let _ = parse_document(&s);                // the oracle IS the absence of a panic
}
```

In Rust a test fails on panic, so absence of an assertion macro is not absence of an oracle. The
class was removed rather than tuned. **The finding is assertions that do not *run*, not assertions
that do not exist.**

**That reasoning is Rust's, and does not generalize (CR-102).** A `vitest` test whose body contains
no `expect()` passes silently, so in TypeScript absence of an assertion *is* absence of an oracle —
the shape this class was written to catch. The measurement above was taken on this crate's Rust
corpus only. The class stays removed for Rust; whether it should exist for TypeScript is a separate
question with its own corpus, and it is not answered here.

### What the static check can and cannot say

It reads shape, not sample distribution — that needs a run. So it reports *"every assertion sits
behind a narrowing guard"* and says that is what it measured, rather than claiming a pass rate it
did not observe.

Narrowing guards are `if let` / `while let` / a `match` arm. A plain `if` on a boolean is
deliberately excluded: it is also how every table-driven test is written, and including it made the
check fire across most of the corpus for reasons unrelated to vacuity.

### The vocabulary is per language, and empty where nothing was measured

Both lists — assertions and guards — belong to a language, not to the check. `=> {` is a `match` arm
in Rust and an **arrow function** in TypeScript; one shared list makes every `it("…", () => { … })`
body a guard.

Where a language has no measured equivalent of the shape, its guard list is **empty** and the check
reports nothing for that language. That is the honest state: neither TypeScript nor Python has a
binding-and-testing construct like `if let`, and what should stand in for one is an open question
that gets its own measurement before anything fires.

Comments are stripped before matching. A comment is prose, not an oracle, and prose that quotes code
is otherwise read as code.

### The oracle comparison has a production join

The comparison is useful only when the engine supplies real pairs. The production join is narrow:
an evidence symbol binds an expression to `expected` or `oracle`, directly compares a production
function call with that binding, and the called function has one directly extractable return (or
Rust tail) expression. The called function is resolved by language and name, preferring the same
file; an ambiguous name yields no suspicion. This compares expressions with expressions — the
population on which the similarity floor was calibrated — rather than lowering the floor until a
whole test body resembles a whole implementation body.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-064-AC-1 | An evidence symbol whose every assertion sits behind a narrowing guard yields a `vacuous-under-guard` suspicion carrying the guarded/total counts. A symbol with at least one unguarded assertion yields none, a symbol with **no** assertion macro yields none — in Rust a test fails on panic — and a production symbol is never judged. A guard that opens and closes on **one line** guards the assertion on that line. | Test (TC-997, TC-998, TC-1002) |
| FR-064-AC-5 | The assertion and guard vocabularies are selected by the symbol's language. A language with no measured narrowing construct — TypeScript and Python — has an empty guard list and yields no `vacuous-under-guard` suspicion, so a TypeScript arrow function is never read as a `match` arm. Comments are stripped before matching, in every language. | Test (TC-1003, TC-1004) |
| FR-064-AC-2 | An oracle whose token similarity to the implementation it judges meets the declared floor yields an `oracle-resembles-implementation` suspicion carrying the score and the floor; an oracle that judges the same subject independently yields none. | Test (TC-999) |
| FR-064-AC-3 | Similarity is computed over identifier tokens, so reformatting scores identical and shared keywords alone do not make two unrelated fragments similar; an empty side scores 0 rather than dividing by zero. | Test (TC-1000) |
| FR-064-AC-4 | Suspicions reach `CoverageReport.suspicions`, ordered deterministically by `(path, line, symbol)`, each carrying `kind`, `path`, `symbol`, `line`, `message` and a non-empty `evidence`. They affect no total, no diagnostic count and no exit code. The list is empty — and the key absent — for a corpus with none. | Test (TC-1001) |
| FR-064-AC-6 | In Rust, Python and TypeScript, an evidence symbol that explicitly binds `expected` or `oracle` and directly compares a production-function call with it is joined to that function's directly extractable return expression. A copied expression yields a located `oracle-resembles-implementation` suspicion naming the implementation and score; the same assertion with an independent expectation yields none. An ambiguous function name stands down, and the pattern inside a string or comment is not code. | Test (TC-1061; controlled corpus `skeptic/oracle-copy`) |

## Dependencies

- **Upstream**: [FR-051](./FR-051-source-symbol-extraction.md) (the symbol spans and source-language classification both checks read)
- **Downstream**: `agent-ix/quire-cli` — the per-criterion rendering surface; `agent-ix/quoin#204` (the mocked-confirmation audit, the third class from the same review)

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-064-CON-1 | A suspicion is advisory. It never affects `totals`, never gates `--strict`, and never changes an exit code. | Design | Test (TC-1001) |
| FR-064-CON-2 | Neither check executes, imports or builds the code it reads — the FR-051-CON-1 boundary, unchanged. | Design | Inspection — both read the same text the binder does |
| FR-064-CON-3 | The assertion vocabulary is a closed list. An open heuristic (`any call containing "assert"`) binds helpers that may themselves assert nothing. | Design | Inspection of `ASSERTIONS` |
| FR-064-CON-4 | The oracle producer SHALL compare expression to expression without lowering `ORACLE_SIMILARITY_FLOOR` to compensate for a broader extraction unit. | Design | Test (TC-1061) |
| FR-064-CON-5 | The oracle producer SHALL yield no pair for a call that cannot be resolved to exactly one production function. | Design | Test (TC-1061) |

> **CR-143 note (2026-08-26):** `agent-ix/quire-rs#236`, epic
> `agent-ix/quoin#264`. The oracle comparison now has a production caller. The
> earlier CR-100 split — comparison here, join left to an unspecified consumer
> — shipped a capability no command exercised. The join is deliberately the
> explicit expression-to-expression convention above. The earlier attempt
> scored the literal copy **0.214** whole-test-to-body and **0.429** isolated
> oracle-to-body against the **0.75** floor; that is a mismatch of extraction
> units rather than evidence for weakening the floor. Corpus controls in all
> three languages determine whether it fires. A 241-repository calibration
> found four oracle suspicions, all four correct seeded cases, and zero on
> project code; every emitted oracle suspicion was inspected.

> **CR-102 note (2026-08-22):** `agent-ix/quire-rs#235`, reopened. Three
> false-positive classes in the shipped `v0.44.0` check, all found by running it
> against corpora other than this crate (SR-054).
>
> **The guard list was Rust's, applied to every language.** `=> {` is a `match`
> arm here and an arrow function in TypeScript, so every `vitest` body opened a
> guard: **549 suspicions from 551 candidates** on `agent-ix/quoin`, against 2
> of 883 here. Sampled three, **3 rule, 0 real**. This is a narrowing, and the
> justification does not rest on the count — `=> {` in TypeScript is a
> different construct. TypeScript and Python now carry an empty guard list
> rather than a guessed one; inventing an unmeasured equivalent to keep the
> check firing is the mistake one level over.
>
> **A single-line guard was invisible.** `assertion_positions` tested the
> assertion before the guard and pushed only when `opens > closes`, so
> `if let Some(x) = y { assert!(x) }` reported **0 suspicions** while the
> multi-line spelling reported one — and the doc comment claimed the opposite.
> The corpus that motivated this work writes single-line guards.
>
> **Comments were read as code.** TC-1003's own comment quotes the TypeScript
> token it describes, and the check reported that test as vacuous — the
> wrong-language misread one level up. Comments are now stripped before
> matching, and braces counted on the stripped text.
>
> Measured after: `quoin` 549 → **0**, this crate 3 → **2** (the two genuine
> TC-1596-shaped positives, unchanged), `quire-cli` and
> `spec-artifacts-process` 0 → 0.
>
> **CR-102 note (2026-08-22):** `agent-ix/quire-rs#235`, reopened. Three
> false-positive classes in the shipped `v0.44.0` check, all found by running it
> against corpora other than this crate (SR-054).
>
> **The guard list was Rust's, applied to every language.** `=> {` is a `match`
> arm here and an arrow function in TypeScript, so every `vitest` body opened a
> guard: **549 suspicions from 551 candidates** on `agent-ix/quoin`, against 2
> of 883 here. Sampled three, **3 rule, 0 real**. The justification for
> narrowing does not rest on the count — `=> {` in TypeScript is a different
> construct. TypeScript and Python now carry an empty guard list rather than a
> guessed one.
>
> **A single-line guard was invisible.** The assertion was tested before the
> guard, and the guard pushed only when `opens > closes`, so
> `if let Some(x) = y { assert!(x) }` reported **0 suspicions** while the
> multi-line spelling reported one — and the doc comment claimed the opposite.
>
> **Comments were read as code.** TC-1003's own comment quotes the TypeScript
> token it describes, and the check reported that test as vacuous.
>
> Measured after: `quoin` 549 → **0**, this crate 3 → **2** (the two genuine
> TC-1596-shaped positives), `quire-cli` and `spec-artifacts-process` 0 → 0.
>
> **CR-100 note (2026-08-22):** FR-064 is new. `agent-ix/quire-rs#235` and
> `agent-ix/quire-rs#236`; epic `agent-ix/quoin#197`.
>
> **Measured before shipping, and the first rule did not survive it.** Over this
> repository's own `src/` and `tests/` — 921 evidence symbols — the first draft
> produced **65 suspicions**, 57 of them the `no-assertion` class. Reading
> twelve of those found **12 rule, 0 real**, and the class was removed.
>
> The shipped rule produces **8**, of which 6 are in fixture trees the real
> model excludes via `source_exclude`. The remaining **2 are true positives, and
> both are the TC-1596 shape** — in this crate's own parser suite:
>
> ```rust
> fn tc819_parse_body_never_panics_on_a_foreign_header(a in "\PC*", b in "\PC*") {
>     if let Some(h) = parse_header(&a) {          // random \PC* rarely parses
>         let doc = parse_body(&b, &h);
>         prop_assert_eq!(doc.raw.as_str(), b.as_str());
>     }
> }
> ```
>
> Its own comment says *"whenever the input is a document at all"* — the guard
> was known and its cost was not.
>
> **Wired through the graph, not left as library API.** `bind` is where the
> extraction and the symbol kinds are both in hand, and minting a fact then
> exposing it one release later is the CR-076 / CR-080 / CR-081 shape this
> repository has paid for four times.
>
> **Superseded by CR-143:** the oracle check originally shipped only as library
> API taking explicit pairs. No consumer supplied them, so no command could
> report the class; the production join now lives beside the symbol extraction
> it reads.
