---
id: SR-091
title: "Rust review of issue 412 oracle scan remediation"
type: SpecReview
analysis: code-review
scope: "src/skeptic.rs; issue_412_oracle_selection.json; NFR-023; TC-1810..TC-1812"
review_set: subset
---

## Summary

The mandated `agent-skills/rust-review` checklist was applied after the
retrospective specify/spec-review remediation. The optimized direct join
preserves the former first-binding/first-matching-assertion result while
removing the per-binding assertion rescan; direct and Rust helper selection
reuse one line-start index. No open Rust finding remains.

## Verdict

**PASS** — zero high, medium, or low findings.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-434 | low | No open Rust defect found after the FND-429..FND-431 remediations; the remaining Rust 1.75 declarations are pre-existing and governed by #414 rather than accepted by this review. | NFR-023; TC-1810..TC-1812; agent-ix/quire-rs#414 |

## Review evidence

| Area | Result | Evidence |
| --- | --- | --- |
| Correctness and parity | Pass | TC-1812 executes a frozen copy of the former selection algorithm against the same regex grammar. Its generated matrix covers 18 direct combinations across three languages, LF/CRLF, repeated/unmatched bindings, assertion ordering, and UTF-8, plus both Rust-helper newline forms. The checked-in fixture independently pins concrete candidate fields. |
| Performance invariant | Pass | TC-1810 observes production-path events and proves one line index, one binding traversal, one assertion traversal, and at most `a + b` joins for zero, one, and 129-binding spans. TC-1811 rejects assertion traversal below the binding loop and any production prefix newline count. |
| State and determinism | Pass | The join is a borrowed `BTreeMap<&str, &str>` whose keys come only from the grammar's `expected|oracle` alternatives. Lookup is by binding name; no result is derived from map iteration order. |
| Test seam integrity | Pass | The observer is a normal generic callback used by the production function with a no-op closure; there is no `cfg(test)` behavior branch, bypass feature, mock of the unit under test, clock assertion, ambient state, or external service. Source inspection is limited to structural complexity properties that output tests cannot prove. |
| Rust idioms and ownership | Pass | Hot-path inputs and regex captures stay borrowed, output ownership is unchanged, static regexes remain `OnceLock`-compiled, matches are exhaustive, and the new internal types state the direct/helper and scan-event distinctions. No public API was added. |
| Panic, unsafe, numeric, async, and lifecycle surface | Pass | No first-party `unsafe`, async/locking, recursion, integer conversion, wire boundary, or resource-owning lifecycle was added. Production `expect` calls remain compile-once constant-regex programmer invariants; all other new panic helpers are test-only assertions. |
| Traceability | Pass | TC-1810..TC-1812 carry bare `use ix_trace_rs::trace;` attributes and canonical names, resolving NFR-023-AC-1..4 without reusing TC-1061. |

## Mutation sensitivity

- Moving `assertion_re.captures_iter(span)` beneath the binding loop fails
  TC-1811 even when candidate output remains identical.
- Reintroducing `span[..offset].matches('\n').count()` or constructing a second
  line index fails TC-1811.
- Changing pass counts or adding candidate-proportional traversals fails
  TC-1810.
- Changing selection, expression bytes, function normalization, or UTF-8/CRLF
  line offsets fails TC-1812 against the frozen former algorithm and fixture.

## Gates

- Rust 1.98.1: formatting check, clippy with warnings denied, native tests,
  Python-feature check, WASM build and semantic suites, license deny, and all
  static audits pass.
- Script tests: 136 passed, 3 skipped.
- Spec self-validation: 165 documents, 0 failures, 41 pre-existing advisory
  warnings, 9 governed Phase-7 exclusions.
- `quire-cli` consumer check: pass; the built consumer reports the pinned
  engine and all required capability tokens.

The repository still carries inherited Rust 1.75 declarations. This review
does not treat their existence as justification or compatibility evidence;
their correction is explicitly governed by #414/ADR-0012.
