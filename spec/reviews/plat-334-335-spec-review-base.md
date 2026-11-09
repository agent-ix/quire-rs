---
id: SR-16370
title: "Base spec review of PLAT-334/PLAT-335 trace emitters FR-079 and FR-080 (quire-rs PR 613)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-079-pytest-trace-marker-plugin.md, spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md, spec/tests.md (PR 613 diff only, head d9b0230)"
review_set: subset
---

## Summary

Tickets: PLAT-334, PLAT-335. PR head `d9b0230`. This is the base checklist over the spec-only diff: two new FRs and 21 TC rows plus 23 audit rows in `spec/tests.md`. The upstream contract is Quoin FR-120 (Emitter contract E-1..E-10), with ADR-100 and FR-033 on quoin `main`. The symbol rules were checked against `src/symbols/python.rs` and `src/symbols/typescript.rs` on quire-rs `main`. `quire validate` exits 0 on the three changed files and raises no grammar warning on them.

The new ids FR-079, FR-080, CR-212 and TC-2120..2140 collide with nothing on `main` or in open PRs 608, 609, 611, 612 and 614. No hash, pin, digest, tracking record, vendored file or compatibility layer is added.

Applicable profile: `spec/assurance/AP-201-detection-minting.md` (scope includes trace binding; `review_policy: require spec-review`). This was a direct review, not an enforced ix-flow run. Enforced acceptance needs a human acknowledgement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-080's `Class method -> Class.method` row and the `a class method reports Class.method` clause of FR-080-AC-5 cannot be met. vitest and jest register tests only through `it`/`test`, so a class method is never a test the reporter sees. The extractor makes a TS method `SymbolKind::Function`, not a test, and names an `it(...)` registration by its raw title even inside a class, because `typescript.rs` `walk` passes `title` and does not call `qualify`. No fixture can satisfy that clause. Drop the row and the clause. If FR-120 needs to change, raise that upstream. | spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md:55, spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md:85 |
| FND-002 | medium | Both FRs say the Rust emitter also uses `QUIRE_TRACE_EMIT=1` (`the Rust emitter uses it too`). That is false against the PLAT-333 spec. ix-trace-rs PR 15 enables Rust emission with the cargo feature `emit` (FR-005-CON-2: `emit SHALL be an explicit cargo feature`), and quire-rs PR 612 says `emit-enabled run`. No `QUIRE_TRACE_EMIT` appears in quire-rs `main`, quoin `main` or ix-trace-rs. Delete the Rust claim, or have PLAT-333 adopt the variable first. | spec/functional/FR-079-pytest-trace-marker-plugin.md:38, spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md:43 |
| FND-003 | medium | The FRs bind packages that live in `agent-ix/ix-trace-py` and `agent-ix/ix-trace-ts` (neither repo resolves yet). Even so, `spec/tests.md` lists TC-2120..2140 as quire-rs Integration tests, and no quire-rs test can implement registration, xdist, cost or helper type-checking. The FR says only that the criteria `are tested against this repository's extractor`, without saying where each TC runs. The Rust precedent splits the work: the emitter spec is in ix-trace-rs PR 15, and quire-rs PR 612 carries only the symbol-equality AC and the E-10 demonstration. Either keep in quire-rs only the symbol-equality and E-10 criteria and spec the package behaviour in the package repos, or name the executing repo per TC. | spec/functional/FR-079-pytest-trace-marker-plugin.md:99, spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md:102, spec/tests.md:1133 |
| FND-004 | low | CR-212 is cited by all 21 TC rows, but no CR-212 note exists anywhere. Repo convention is a `> **CR-NNN note**` block in the owning FR, as FR-050 and FR-051 do. | spec/tests.md:1133 |
| FND-005 | low | FR-080 says a registration that the static extractor mis-shapes (`agent-ix/quire-rs#189`) still reports its title at run time. #189 was closed on 2026-08-21, and `typescript.rs` now extracts `it.skipIf(cond)(...)`. A runtime title that the extractor does not report would also break E-2's equality, so the sentence argues against the contract. Remove it. | spec/functional/FR-080-vitest-jest-trace-helper-and-reporter.md:59-60 |

## Verdict

Examined against E-1..E-10: E-1 (both FRs: JUnit `trace` property only), E-3 (FR-079-AC-4, FR-080-AC-4), E-5 (FR-079-AC-7, FR-080-AC-8), E-6 (FR-079-AC-2, FR-080-AC-2), E-7 (AC-9/AC-10 at the SHALL bound of 5%), E-8 (FR-079-AC-8, FR-080-AC-9, both CON-1) and E-10 (FR-079-AC-10, FR-080-AC-11). E-9 is Rust-only and does not apply. The Python symbol rules match `python.rs`: bare at module level, `Class.method`, nested classes dot-joined, no module prefix. The vitest/jest bare-title, nested-`describe` and `it.each` template-literal rules match `typescript.rs`.

E-2 fails on FND-001. E-4 fails as recorded in SR-16372. Not mergeable until FND-001 to FND-003 are fixed.
