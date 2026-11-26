---
id: SR-967175026302197768900474181941935084873
title: "PLAT-1915 code and Rust review — slug variant tags"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; src/parser/slug.rs; FR-009-AC-1/2/3/5/6/7"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/FR-009"
    type: references
---

## Summary

Ticket: PLAT-1915. Reviewed PR 740 at `0638be586708c3e6a8644fca60a2d9a2f8915d77`. The diff adds only FR-009-AC-5 markers to the five existing Rust slug-variant tests. Each calls the real `slug_line_id` implementation and asserts a literal expected output for its variant.

## Verdict

**PASS** — no code or trace-binding defect found. The five tests collectively assert the slug variants named by FR-009-AC-1/2/3/6/7 and satisfy the parity criterion FR-009-AC-5. Disposition pass 1 confirms the source head is unchanged, and the lead supplied the completed required `cargo check --locked --all-targets -p quire-rs` result: exit 0. The change is mergeable.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

```yaml
clean: true
scope:
- id: FR-009-AC-1
  path: spec/functional/FR-009-slug-line-id.md
  role: examined
  excerpt: Heading `"Hello, World!"` at line 0 yields id `"hello-world-L0"`.
- id: FR-009-AC-2
  path: spec/functional/FR-009-slug-line-id.md
  role: examined
  excerpt: Heading `"2.1 In Scope"` at line 6 yields id `"2-1-in-scope-L6"`.
- id: FR-009-AC-3
  path: spec/functional/FR-009-slug-line-id.md
  role: examined
  excerpt: Heading `"   leading spaces   "` at line 3 yields id `"leading-spaces-L3"`.
- id: FR-009-AC-5
  path: spec/functional/FR-009-slug-line-id.md
  role: examined
  excerpt: A Rust-owned parity test covering each slug variant passes.
- id: FR-009-AC-6
  path: spec/functional/FR-009-slug-line-id.md
  role: examined
  excerpt: Heading `"Café Menu"` at line 4 yields id `"caf-menu-L4"` (non-ASCII collapses; mirrors TS/Py reference).
- id: FR-009-AC-7
  path: spec/functional/FR-009-slug-line-id.md
  role: examined
  excerpt: Heading `"!!!"` at line 7 yields id `"-L7"` (degenerate empty slug, line index preserved).
findings: []
bindings:
- test_id: tests::lowercase_alphanum_dash_basic
  ac_id: FR-009-AC-5
  trace: correct
- test_id: tests::numeric_and_punctuation_collapse_to_dashes
  ac_id: FR-009-AC-5
  trace: correct
- test_id: tests::leading_and_trailing_whitespace_stripped
  ac_id: FR-009-AC-5
  trace: correct
- test_id: tests::non_ascii_collapses_to_dash
  ac_id: FR-009-AC-5
  trace: correct
- test_id: tests::degenerate_heading_keeps_line_index
  ac_id: FR-009-AC-5
  trace: correct
```

## Checks

- Code review with the Rust lane ran over the one-file diff. No production function, dependency revision, workflow, assertion, fixture or runtime behavior changed.
- `quire trace --file src/parser/slug.rs` (Quire 0.52.0, engine 0.62.2) exited 0 and resolves FR-009-AC-5 to the five real test functions.
- `quire matrix --scope . --format json` exited 0; the changed criterion is tagged. Repository-wide pre-existing diagnostics are outside this change and were not treated as PR findings.
- `git diff --check` exited 0. In disposition pass 1, the lead supplied `cargo check --locked --all-targets -p quire-rs` exit 0, with an optional npm fixture-fetch warning. No tests were run for this tag-only change under the regulated-finish light-check rule.
- AP-201 at `spec/assurance/AP-201-detection-minting.md` was considered. Its engine detection/minting scope does not apply to this slug-test annotation change.
- A separate gap-analysis method was not applicable: this diff changes test metadata only. A separate spec-review method was not applicable: no spec or plan artifact changed.

## Dispositions

Round 1: no real findings required disposition. PR 740 still has head `0638be586708c3e6a8644fca60a2d9a2f8915d77`, with only the five reviewed trace-marker additions. The lead supplied the required all-targets compile-check result, exit 0; the optional npm fixture-fetch warning does not alter that result. No new finding arose. Mergeable: yes.
