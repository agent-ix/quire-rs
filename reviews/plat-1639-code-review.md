---
id: SR-2026112016223475846355047
title: "Code review — quire-rs self-test resources"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/lib.rs, src/selftest.rs, selftest/cases/fr-032-placeholder-detection/**, spec/functional/FR-032-validate-document.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-032
    type: references
---

## Summary

Ticket: PLAT-1639. Reviewed PR #722 at `00710b424b22802244a9315cfcd0eee301559b92` against refreshed `origin/main`. The public resource table is small and deterministic, but the initial case's `required-tools` resource is a newline-only file rather than the empty file required to represent no external dependencies.

## Verdict

**CONDITIONAL** — one medium fixture-format defect remains; the focused compile and manual-case evidence are otherwise consistent with the changed scope.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The `required-tools` resource contains one newline byte instead of being empty. The consuming FR-047 contract allows only bare executable names per line and defines an empty file as no required tools; a line parser can treat this as an empty executable name and skip or reject the only quire-rs case. Make the file zero bytes or explicitly define and test blank-line handling. | selftest/cases/fr-032-placeholder-detection/required-tools:1 |

## Coverage

- Examined the `EmbeddedCase` and `EmbeddedCaseFile` public API, all seven embedded file paths, the shell case, and FR-032-AC-3.
- The exported table uses `include_bytes!` for each owned path; no foreign case assets or runtime filesystem lookup are added.
- `git diff --check` passed. The coder reported `cargo check --locked -p quire-rs --lib` through `locked-build.sh`, rustfmt, shell syntax validation, and a manual case run with quire 0.52.0; the review did not repeat builds or suites.
- Computed `quire matrix --scope . --format json` exited 0. FR-032-AC-3 currently has two existing tagged test binders; this review did not treat the self-test case source marker as a repository matrix binder.
