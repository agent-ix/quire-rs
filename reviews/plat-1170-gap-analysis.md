---
id: SR-16381
title: "Gap analysis of PLAT-1170 FR-050-AC-62 implementation (quire-rs PR 615)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs; spec/functional/FR-050-declarative-coverage-computation.md (FR-050-AC-62, CR-209 note), spec/tests.md (TC-2090, FR-050 row), src/symbols/trace.rs, src/coverage/binding_diagnostics.rs, tests/it/legacy_implements_warning.rs (PR 615 diff only)"
review_set: subset
---

## Summary

Ticket: PLAT-1170. PR head `abc3875`. Planless gap analysis. Plan completion: not assessed. `quire trace --id FR-050-AC-62` (quire 0.36.2, engine 0.50.2) resolves six structural `verifies` claims, all in `tests/it/legacy_implements_warning.rs` and tagged `#[trace("TC-2090", "FR-050-AC-62")]`. The production code carries only mentions. Clauses mapped to tests:

- Warning per legacy use, with file, line, symbol and replacement: the first test.
- Two-id rendering: the first test.
- Unchanged binding, counts and no `RewriteSuggestion`: the first test, against a control module.
- Renaming changes nothing: the renaming test.
- No template gives no finding: the no-template test.
- The first declared template wins: the first-template test.
- A `rust_attribute` binding gives none: the attribute-binding test.
- A module without the form gives none: the control in the first test.
- Non-Rust gives none: the Python test.

The FR-057 severity clause has no engine-side field and no test. That matches the existing `range-in-trace-tag` precedent, and the engine-wide severity assertion is TC-2055 (🚧), so it is not filed here. No stubs and no coverage inflation. The `spec/tests.md` edits (TC-2090 ✅, FR-050 row "Implemented") agree with the bound tests. The FR-050-AC-62 audit row (`| FR-050-AC-62 | TC-2090 |`) is unchanged and correct. quoin:spec-review was not run separately. The only spec diff is two status cells, and they are verified here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-2090 fixtures only put legacy lines on top-level functions. No test covers a legacy occurrence nested inside an inline `mod`, a `trait` or a function body. That is the case where the implementation emits a duplicate container finding (SR-16380 FND-001), so the one-finding-per-occurrence reading of FR-050-AC-62 is not tested. | tests/it/legacy_implements_warning.rs:62-68 |

## Verdict

Examined: FR-050-AC-62, TC-2090 and the FR-050 and TC-2090 rows in `spec/tests.md`. The AC is otherwise fully and falsifiably covered. Add a nested-occurrence test that asserts exactly one finding naming the innermost symbol. It should be written alongside the SR-16380 FND-001 fix. Known and not a finding: `tc1026_the_declared_vocabulary_matches_the_engine` fails until qa-corpus `corpus.yaml` `emitted` lists `legacy-implements-form`. That test also requires every emitted reason to be asserted present in one corpus case and absent in another, so the qa-corpus PR needs both cases, not only the `emitted` entry.
