---
id: SR-16352
title: "Integrity analysis of PLAT-333 FR-051-AC-32/33 (quire-rs PR 612)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md, spec/tests.md (PR 612 diff only, head 8186938)"
review_set: subset
---

## Summary

Ticket: PLAT-333. Completeness, consistency and atomicity of FR-051-AC-32, FR-051-AC-33, the CR-211 note and their TC and audit rows. Every new AC has a TC row and an audit row, and every new TC row names an AC that exists. The relationship to quoin FR-120 is cited as an `ix://` link in prose, consistent with how FR-051 already cites cross-repo sources.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-051-AC-32's fixture clause says `quire coverage --json` reports "exactly these names" without listing them: the rstest case's expected name is never stated, and the clause that they equal what `ix-trace-run` writes can only fail if TC-2091 runs the runner, which neither the AC nor TC-2091 says. Enumerate the four expected `(path, qualified_name)` pairs literally (for example `tests::name` for the rstest source function) and either state that TC-2091 runs `ix-trace-run` over the fixture or leave the runner equality to AC-33, where the join actually runs. | spec/functional/FR-051-source-symbol-extraction.md:180, spec/tests.md:1132 |

## Verdict

Examined: FR-051-AC-32, FR-051-AC-33, the CR-211 note, TC-2091, TC-2092, the FR-051 summary row and the two audit rows. Consistency with quoin FR-120 holds for the naming rule (same container list, crate name and file stem excluded, `impl` target not trait) and for the JUnit shape (E-9). AC-33's staleness clause and its failing-variant wording are reported in SR-16350 and not repeated here.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Commit c38df62 lists the four `(path, qualified_name)` pairs literally in AC-32 and TC-2091, including `(tests/x.rs, param)` for the rstest source function, and moves the runner comparison to AC-33. |
