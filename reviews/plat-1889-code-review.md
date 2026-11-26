---
id: SR-967175026302197768900474181935084797
title: "Code review — PLAT-1889 quire-rs impact engine"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; Cargo.toml, src/impact.rs, src/lib.rs; FR-099"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-099", type: references}
---

## Summary

Reviewed PR #734 at `cb5f5d444e16b0f9e4c9d18a83f4f953b52489db`, including the Rust review lane, test changes, API surface, and traced behavior. Two medium findings remain; the implementation has an incomplete-extraction path that can claim complete selection and the AC-6 test omits required identity distinctions.

## Verdict

**CONDITIONAL** — two medium findings need disposition before merge.

## Assurance Context

AP-201 (`spec/assurance/AP-201-detection-minting.md`) applies because this change alters trace binding and impact reporting. The profile evaluator reported no source profile selected, so no clause set was evaluated. The assurance report had no unreadable entries, but showed no run evidence for the inspected matrix and 29 unreachable FR/NFR claims. Selected architecture, impact, measurement, exception, independence, structural-gap, and producer-reliance evidence was unavailable; there are no active exceptions. AP-201's required `make ci`, controlled-corpus, and mutation/self-test evidence was not available in this review run.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Snapshot extraction errors outside the changed path are hidden, allowing incomplete selection to be reported as complete | src/impact.rs:449-467 |
| FND-002 | medium | The AC-6 test omits requirement-level versus criterion-level and repository-prefixed id distinctions | src/impact.rs:773-810 |

## Scope

- `FR-099-AC-1` examined: Given readable base and head revisions, the report maps changed Rust lines on both sides to the containing symbols, includes enclosing containers, and reports deleted base-side and added head-side symbols. An unavailable revision is an explicit error, not an empty report.
- `FR-099-AC-2` examined: Every changed line outside an extracted symbol is returned as an unmapped-change record with repository-relative path, revision side, and line range.
- `FR-099-AC-3` examined: Every test symbol added or edited by the diff is selected with a Rust test-runner filter and a direct-change reason; a deleted test is not selected.
- `FR-099-AC-4` examined: For a changed symbol implementing an exact id, the report selects every test verifying that id and lists every other symbol implementing it. A test or symbol with only a different id is not included by this rule.
- `FR-099-AC-5` examined: Every selected test and related symbol carries a complete reason chain from a changed path and line range through the relevant symbol and exact relation id; a directly edited test carries its direct-change chain.
- `FR-099-AC-6` examined: Requirement-level ids match only the same normalized requirement-level id and do not expand to criterion ids; criterion ids match only the same normalized criterion id; repository-prefixed ids remain distinct from local ids.
- `FR-099-AC-7` examined: A changed production symbol with no bound relation is reported as unlinked. An id that is absent from supplied specs, a spec that is unavailable, and a source marker that fails to bind produce distinct findings; a path-qualified non-binding Rust attribute is not reported as a successful implementation link.
- `FR-099-AC-8` examined: When extraction, traceability forms, or a referenced specification are unavailable, the affected files or ids are identified as unavailable or unverifiable, and the report does not claim an empty selection or unaffected tests.
- `FR-099-AC-9` context_only: Caller following resolves only statically named paths that exactly match one extracted Rust symbol path; dynamic, ambiguous, aliased, or unmatched calls are unresolved. Following stops at its finite depth, returns only nearest linked callers once even in a cycle, adds their verified tests without removing earlier selections, reports unresolved call sites, and marks depth truncation when more caller edges remain.
- `FR-099-AC-10` context_only: A dependency revision change reports changed upstream files and extracted items, local references, selected tests with upstream-to-local-to-test reason chains, and every unmatched upstream change as uncovered. An unavailable upstream tree is not reported as having no impact.
- `FR-099-AC-11` examined: Identical analysis inputs produce byte-identical report output and stable result ordering, without timestamps, temporary paths, or host-specific values.
- `FR-062` context_only: Existing `implements` binding behavior and shared relation semantics.
- `FR-077` context_only: Existing exact trace search behavior used to find verifier tests and sibling implementations.

## Review Method and Gates

- Rust lane: reviewed changed API and implementation, repository conventions and applicable Rust style references, behavior, error handling, deterministic ordering, and trace-tagged tests.
- `cargo fmt --all -- --check`: pass.
- `cargo clippy -p quire-rs --all-targets --all-features -- -D warnings`: pass; the environment could not fetch `@agent-ix/spec-objects-architecture@0.10.3` for two unrelated tests.
- `cargo test -p quire-rs`: 789 passed, one unrelated environment failure in `loader::paths::tests::env_roots_keep_default_root` (`/private/var/...` versus `/var/...`).
- `cargo deny check`: pass with existing allow/exception and wildcard-dependency warnings.
- `git diff --check`: pass.
- Trace check confirmed implementation links for AC-1..8 and AC-11. The FR-099-AC-9/10 implementation is staged as debt under PLAT-1889 and is not claimed by this PR.

## Verdict Detail

The new code is largely cohesive and uses the existing extractor, binder, and exact trace-search APIs. The diagnostics filter misses malformed untouched files even though those files participate in whole-snapshot test lookup. The AC-6 test only checks a normalized criterion id against a different criterion id; it does not protect the other explicitly required exact-identity cases.

## Dispositions

| FND | outcome | reason / after_excerpt |
| --- | --- | --- |
| FND-001 | fixed | after_excerpt: `for (side, extraction) in [` |
| FND-002 | fixed | after_excerpt: `assert!(!criterion` |
