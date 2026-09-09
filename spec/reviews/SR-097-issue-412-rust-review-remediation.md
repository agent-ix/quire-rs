---
id: SR-097
title: "Rust review of issue 412 independent-review remediation"
type: SpecReview
analysis: code-review
scope: "PR #415; src/skeptic.rs; NFR-023; TC-1810..TC-1812; SR-092"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/NFR-023"
    type: reviews
---

## Summary

The mandated `agent-skills/rust-review/SKILL.md` checklist was reapplied after
banking the SR-092 regressions. The traversal observer now wraps the actual
`Regex::captures_iter` call, the direct path performs no raw capture traversal,
and a shared line index is constructed only after direct or helper selection
produces a candidate. The fixed two-key assertion state and peeked binding
iterator add no candidate-proportional collection. No open Rust finding
remains.

## Verdict

**PASS** — zero open critical, high, medium, or low findings.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-112 | high | Closed: reintroducing one assertion traversal per binding now increments the real traversal counter and violates the source allowlist; the mutation fails TC-1810 and TC-1811. | `src/skeptic.rs:646`; `src/skeptic.rs:769`; `src/skeptic.rs:1450` | correct-requirement-no-evidence |
| FND-113 | high | Closed: aliasing the assertion regex no longer evades the audit because every raw direct-path `.captures_iter` is forbidden outside the measured wrapper, independent of the regex variable name. | `src/skeptic.rs:646`; `src/skeptic.rs:1526` | correct-requirement-no-evidence |
| FND-114 | medium | Closed: a span without a binding performs only the binding scan; if neither direct nor helper selection succeeds, it allocates no line index. Direct and helper candidates share the one deferred index. | `src/skeptic.rs:665`; `src/skeptic.rs:776`; `src/skeptic.rs:1473` | missing-requirement |
| FND-115 | low | Closed: Rust 1.98.1 Clippy rejected an explicit `peek().is_none()` return; the path now uses `assignments.peek()?`. | `src/skeptic.rs:778` | implementation-bug-despite-evidence |

## Checklist evidence

| Area | Result | Evidence |
| --- | --- | --- |
| Correctness and parity | Pass | TC-1812 remains a differential against the frozen pre-#412 algorithm across Rust/Python/TypeScript, repeated and unmatched bindings, assertion order, UTF-8, CRLF, and helper selection. |
| Test discrimination | Pass | Before production remediation, TC-1810 failed on eager no-candidate line indexing and TC-1811 failed because the measured wrapper was absent. Adding one extra assertion traversal after remediation makes both fail while TC-1812 remains green. |
| Performance and bounds | Pass | One peeked capture iterator streams bindings without a `Vec`; assertion state is two typed `Option<&str>` fields; assertion scanning is skipped without a binding; line indexing is skipped without a selected candidate. |
| Ownership and idioms | Pass | Inputs and regex captures remain borrowed, outputs become owned only on selection, `OracleBinding` is a closed enum with `TryFrom<&str>`, and no public API or reusable allocation was added. |
| Seam integrity | Pass | The observer is a production-path callback around the real traversal and line-index constructors; no `cfg(test)` behavior branch or bypass feature exists. Static inspection is limited to the structural non-nesting/allowlist property. |
| Panic, unsafe, numeric, async, lifecycle | Pass | No first-party unsafe, new panic path, unchecked slice/index, numeric conversion, async/blocking operation, lock, I/O, or resource lifecycle was introduced. Constant regex `expect` calls are unchanged programmer invariants. |
| Traceability | Pass | TC-1810..TC-1812 retain bare `use ix_trace_rs::trace;` and canonical `#[trace(...)]` attributes resolving every NFR-023 acceptance criterion. |

## Gates

- Exact Rust 1.98.1: `cargo clippy --locked --all-targets -- -D warnings`
  and the full `cargo test --locked` pass.
- Repository-selected Rust 1.94.1: formatting, Clippy, Python-feature check,
  WASM checks and 37 semantic tests, 136 script tests (3 skipped), the full
  612-test library plus integration/doc suites, license deny, unsafe/property,
  and all static audits pass.
- Self-validation against the exact locked schema-provider revisions reports
  168 documents, 0 failures, 41 inherited warnings, and 9 governed exclusions.
- The linked quire-cli engine/capability gate passes. The repository's
  1.94.1/1.75 declarations remain owned by #417 and are not justified by this
  review.
