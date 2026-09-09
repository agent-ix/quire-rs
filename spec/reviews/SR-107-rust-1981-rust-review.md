---
id: SR-107
title: "Rust review of the Rust 1.98.1 transition"
type: SpecReview
analysis: code-review
scope: "issue #417 implementation; Rust/config/audit/test diff from cf27f0d"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-022"
    type: reviews
---

## Summary

This review applies `agent-skills/rust-review/SKILL.md` and the repository's
AGENTS.md/CLAUDE.md conventions to the full #417 implementation diff. The two
compiler-driven defect classes were repaired; no unresolved Rust correctness,
safety, lifecycle, resource-bound, API, or test-integrity finding remains.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-4174 | medium | Closed: nine `Option::map_or(true, predicate)` sites blocked 1.98.1 under `-D warnings`; each now uses the equivalent `Option::is_none_or` form, so `None` remains accepting and `Some` still evaluates the same predicate. | `src/corpus/query.rs:93`; `src/semantic/decl.rs:37`; `src/traceability.rs:799`; `tests/corpus_case/grading.rs:22` | correct-requirement-no-evidence |
| FND-4175 | medium | Closed: strict rustdoc found private, broken, ambiguous, and redundant intra-doc links that could make generated API documentation misleading or fail its build; the corrected links pass with warnings denied. | `src/corpus/glossary.rs:46`; `src/registry.rs:156`; `src/validate_document.rs:5`; TC-1833 | correct-requirement-no-evidence |

## Review checklist

- The source-inspection audit is appropriate for compiler declarations and is
  mutation-tested against the old numeric pins and floating `stable`; it does
  not replace a reachable runtime behavior test.
- The `is_none_or` rewrites are ownership-neutral, allocate nothing, preserve
  short-circuiting, and introduce no new trait or visibility surface.
- The dependency change is one locked transitive patch release tied to a named
  RustSec advisory; no manifest range or source policy changed.
- The change adds no public API, unsafe block, panic path, integer/FFI
  conversion, async work, lock, or unbounded resource loop, and weakens no
  lint, audit, test, or coverage threshold.
- #417 adds Python source-inspection tests because configuration drift has no
  behavioral Rust entry point. It adds no Rust acceptance test requiring a new
  `ix_trace_rs::trace` marker; changed Rust helpers remain reached by the
  existing traced suites.
- The all-target/all-feature Clippy gate, full default test suite, strict docs,
  Python ABI3 wheel, WASM, loom, license, advisory, benchmark-build, release,
  unsafe, and repository audit gates were actually run on Rust 1.98.1.

## Review disposition

Pass with no open finding. Independent repository review is still required
before merge; this artifact records the mandated Rust-specific owner review and
does not substitute for that independent gate.
