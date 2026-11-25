---
id: SR-967175026302197768900474181941935084796
title: "Code review — quire-rs shell trace coverage"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; tests/it/coverage_rollup.rs; spec/functional/FR-050-declarative-coverage-computation.md"
review_set: subset
---

## Summary

Ticket: PLAT-1640. Reviewed PR #732 at `db7db3872b1f86e48ea803915ea30719f6f0671e`. The focused regression exercises an unknown, unqualified shell `Trace` marker and verifies its ID, script path, entry point, and line; the PR reports that 86 high risk FRs still lack dedicated self-test cases.

## Verdict

**CONDITIONAL** — the focused regression is correctly tagged and its assertions match FR-050-AC-5, but the PR's reported 86 uncovered high risk FRs mean PLAT-1640 AC5 remains unmet.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The PR reports 86 high risk FRs without dedicated self-test cases. FR-050-AC-5 requires zero uncovered high risk FRs at merge across quoin, quire-rs, and quire-cli; this integration regression is not a self-test case for those FRs. Keep AC5 and PLAT-1640 open until the missing cases are supplied and the coverage check reports zero uncovered FRs. | tests/it/coverage_rollup.rs:977; spec/functional/FR-050-declarative-coverage-computation.md:408 |

## Coverage

- Examined the changed test `tc2202_untracked_shell_marker_names_its_script`, its `#[trace("FR-050-AC-5")]` tag, and FR-050-AC-5.
- `FR-050-AC-5` is owned by the quire-rs coverage requirement. The changed test carries the correct criterion tag. No production function changed, so no `implements` marker is due.
- The fixture feeds `# Trace: FR-999-AC-1` from `scripts/selftest.sh`; the focused test asserts that `untracked_symbols` contains exactly that ID, path `scripts/selftest.sh`, symbol `entry_point`, and line 1.
- Focused verification: `locked-build.sh /tmp/quire-rs-plat1640-review-target cargo test -p quire-rs --test it tc2202_untracked_shell_marker_names_its_script -- --nocapture` — passed (1 test; 1001 filtered out).
- `cargo fmt --all -- --check` — passed.
- `quire trace --file tests/it/coverage_rollup.rs --json` reports `tc2202_untracked_shell_marker_names_its_script` as verifying FR-050-AC-5 at line 981.
- No full suite or long validation sweep was run.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | rejected | This is not a code defect in the focused quire-rs PR slice: the 86 uncovered high risk FRs concern PLAT-1640's broader AC5 coverage target, and bulk case creation is paused pending the owner's decision. AC5 remains explicitly pending at the PLAT-1640 ticket level. |

## Disposition Review

**PASS** for the focused PR slice at `db7db3872b1f86e48ea803915ea30719f6f0671e`. The regression test satisfies the quire-rs behavior reviewed here; the reported 86 uncovered high risk FRs remain a ticket-level AC5 limitation, not a finding against this PR.
