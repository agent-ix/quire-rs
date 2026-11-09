---
id: SR-16350
title: "Base spec review of PLAT-333 FR-051-AC-32/33 (quire-rs PR 612)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md, spec/tests.md (PR 612 diff only, head 8186938)"
review_set: subset
---

## Summary

Ticket: PLAT-333. PR head `8186938`. Base checklist over the spec-only diff: FR-051-AC-32, FR-051-AC-33, the CR-211 note, the FR-051 summary row, the TC-2091 and TC-2092 rows and the two audit rows in `spec/tests.md`. Read against quoin FR-120 (emitter rules E-2, E-9, E-10), ADR-100, quoin FR-032 (AC-2 withdrawn, AC-17), and the companion ix-trace-rs PR 15 (FR-005, FR-006). `quire validate` exits 0 on both changed files; the only grammar warning (FR-051 line 143) predates this PR.

The AC-32 naming claim matches the extractor: `src/symbols/rust.rs` `parse` calls `walk(root, None, ...)`, so each file starts from an empty container; `mod`, `struct`/`enum`, `trait`, `impl` target type (the `type` field, never `trait`) and an enclosing `fn` push containers. A test in `mod tests` of `src/foo/bar.rs` is therefore `tests::t`, as AC-32 says, and ix-trace-rs FR-006 strips the file's module prefix to the same name (FR-006-AC-3).

Id collisions: CR-211, TC-2091, TC-2092 and FR-051-AC-32/33 are unused on `main` and in open PRs 521, 608, 609, 611, 613 and 614. No hash, pin, digest, tracking record, vendored file or compatibility layer was introduced.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-051-AC-33 and TC-2092 require that editing the requirement text flags the binding stale. Quoin withdrew that behaviour: FR-032-AC-2 (suspect-link on statement-hash change) is withdrawn because bindings carry no hash, and E-10 of FR-120 does not ask for it. No implementation can pass this clause, so TC-2092 can never go green. Delete the clause from the AC and the TC. | spec/functional/FR-051-source-symbol-extraction.md:181, spec/tests.md:1133 |
| FND-002 | medium | FR-051-AC-33's failing variant is underspecified. "The same run with one tagged test made to panic" contradicts itself (a second run is meant), does not say how a test in this repository's real suite is made to panic (a fixture crate or an env-gated test), and "binds nothing for its ids" passes or fails depending on whether another passing test carries the same ids. State the mechanism and that the panicking test's ids are carried by no other test, so the obligation stays unbound (FR-120-AC-5). | spec/functional/FR-051-source-symbol-extraction.md:181, spec/tests.md:1133 |
| FND-003 | medium | The FR-051 summary row covers AC-1..33 and still reads `✅ Implemented` with no exception, while TC-2091 and TC-2092 are `🚧`. The FR-050 row's precedent (`✅ Implemented except AC-62 (🚧, TC-2090 unimplemented)`) applies: add `except AC-32..33 (🚧, TC-2091/TC-2092 unimplemented)`. | spec/tests.md:149 |
| FND-004 | low | TC-2091 says the runner writes the names as `name` and `file`; FR-051-AC-32 says `name` and `classname`. E-9 needs all three (`name` = `classname` = `qualified_name`, `file` = path). Say the same thing in both rows. | spec/tests.md:1132, spec/functional/FR-051-source-symbol-extraction.md:180 |

## Verdict

Examined: FR-051-AC-32 (naming rule correct against `src/symbols/rust.rs` and identical to FR-120's Rust row), FR-051-AC-33 (the marker, `emit` run, JUnit attributes, `quoin evidence record` and bound-obligation clauses match FR-120 E-9/E-10), the CR-211 note, the FR-051 summary row, the TC-2091/TC-2092 rows and both audit rows. The audit rows and the `AC-1..33` range agree with the new ACs. PR 614 edits the same FR-051 summary line (`AC-1..31, 34..40`), so whichever PR lands second has a textual conflict and must resolve the range to `AC-1..40` (or `AC-1..33, 34..40`) and merge both test lists. Not mergeable until FND-001 is fixed; FND-002 and FND-003 should be fixed in the same round.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | FR-051-AC-33's failing run "binds no obligation, provided no other passing test carries the same ids" is a precondition, not a fixture fact, and it does not say whether the second run is recorded into a fresh evidence store or into the store holding run 1's binding (where that binding persists and FR-032-AC-17 governs a failing newest run). State that the panicking test's ids are carried by no other fixture test and that the second run's entries discharge nothing (FR-120-AC-5), or that it is recorded into a fresh store. | spec/functional/FR-051-source-symbol-extraction.md:181, spec/tests.md:1141 |
| FND-006 | low | The fixture's `#[rstest]` function `param` does not state its attribute order. ix-trace-rs FR-005 supports only `#[trace]` written above `#[rstest]`; the other order registers per-case symbols, so AC-33's name equality would fail for a reason outside quire-rs. Say `#[trace]` is written above `#[rstest]`. | spec/functional/FR-051-source-symbol-extraction.md:180 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Commit c38df62 deletes the staleness clause from FR-051-AC-33 and TC-2092. |
| FND-002 | fixed | Commit c38df62 makes the failing variant a second run, with the panic driven by `IX_TRACE_E2E_FAIL`, over a fixture crate rather than the real suite. |
| FND-003 | fixed | The FR-051 summary row now reads `✅ Implemented except AC-32..33 (🚧, TC-2091/TC-2092 unimplemented)`. |
| FND-004 | fixed | TC-2091 is now extractor-only; the `name`/`classname`/`file` comparison moved to AC-33 and TC-2092, which name all three. |
