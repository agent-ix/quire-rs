---
id: FR-064
title: "The skeptic layer"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/FR-051"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/FR-052"
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

### What the static check can and cannot say

It reads shape, not sample distribution — that needs a run. So it reports *"every assertion sits
behind a narrowing guard"* and says that is what it measured, rather than claiming a pass rate it
did not observe.

Narrowing guards are `if let` / `while let` / a `match` arm. A plain `if` on a boolean is
deliberately excluded: it is also how every table-driven test is written, and including it made the
check fire across most of the corpus for reasons unrelated to vacuity.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-064-AC-1 | An evidence symbol whose every assertion sits behind a narrowing guard yields a `vacuous-under-guard` suspicion carrying the guarded/total counts. A symbol with at least one unguarded assertion yields none, a symbol with **no** assertion macro yields none — in Rust a test fails on panic — and a production symbol is never judged. | Test (TC-997, TC-998) |
| FR-064-AC-2 | An oracle whose token similarity to the implementation it judges meets the declared floor yields an `oracle-resembles-implementation` suspicion carrying the score and the floor; an oracle that judges the same subject independently yields none. | Test (TC-999) |
| FR-064-AC-3 | Similarity is computed over identifier tokens, so reformatting scores identical and shared keywords alone do not make two unrelated fragments similar; an empty side scores 0 rather than dividing by zero. | Test (TC-1000) |
| FR-064-AC-4 | Suspicions reach `CoverageReport.suspicions`, ordered deterministically by `(path, line, symbol)`, each carrying `kind`, `path`, `symbol`, `line`, `message` and a non-empty `evidence`. They affect no total, no diagnostic count and no exit code. The list is empty — and the key absent — for a corpus with none. | Test (TC-1001) |

## Dependencies

- **Upstream**: [FR-051](./FR-051-source-symbol-extraction.md) (the symbol spans both checks read), [FR-052](./FR-052-acceptance-criteria-property-classification.md) (the oracle spans the similarity check compares against — improved by CR-096, without which most specific-shape criteria carried none)
- **Downstream**: `agent-ix/quire-cli` — the per-criterion rendering surface; `agent-ix/quoin#204` (the mocked-confirmation audit, the third class from the same review)

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-064-CON-1 | A suspicion is advisory. It never affects `totals`, never gates `--strict`, and never changes an exit code. | Design | Test (TC-1001) |
| FR-064-CON-2 | Neither check executes, imports or builds the code it reads — the FR-051-CON-1 boundary, unchanged. | Design | Inspection — both read the same text the binder does |
| FR-064-CON-3 | The assertion vocabulary is a closed list. An open heuristic (`any call containing "assert"`) binds helpers that may themselves assert nothing. | Design | Inspection of `ASSERTIONS` |

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
> **The oracle check is library API taking explicit pairs**, deliberately: the
> join from a criterion's oracle span to the implementation it judges needs the
> `Registry` and the `implements` relation, which the binder does not have. The
> comparison is the part worth pinning now; wiring the join is the consumer's,
> and it is named in Dependencies rather than left implied.
