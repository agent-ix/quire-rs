---
id: SR-185
title: "STD-228 gap analysis of the Rust spec validation gate"
type: SpecReview
analysis: gap-analysis
scope: "STD-228 spec validation gate in examples/spec_validate.rs, Makefile, .github/workflows/ci.yml, scripts/validate_spec.py and FR-013-AC-16/17"
review_set: subset
---

## Summary

Checked the changed gate's requirement, computed test binding, caller path, and removed Python test. The validator still reads the same `spec/` tree and governed exclusion file; only module-set selection moved into the Rust example.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The initial two example tests inflated FR-013-AC-16 trace evidence without exercising its closed-set document-validation behavior. | examples/spec_validate.rs, FR-013-AC-16 |

## Dispositions

- FND-001: fixed. The new gate-path test now cites FR-013-AC-17 and exercises selection through `load_provider_registry`. The existing loader tests own AC-16 and AC-17's full registry behaviors.

## Verdict

The introduced trace gap is fixed. The computed matrix reports FR-013-AC-16 tagged by its existing closed-set document test and FR-013-AC-17 tagged by the existing ambient-isolation test plus the new gate-path test. The new example test is configured to run under ordinary `cargo test` through `[[example]] test = true`. The matrix still reports 245 untagged criteria across the full repository; those pre-existing criteria are outside this PR's scope and prevent a repository-wide PASS claim.

## Coverage

Plan completion: not assessed. Source review found one live Make caller and one CI caller of the deleted Python wrapper; both now invoke the same Rust example. `scripts/tests/test_validate_spec.py` was the only Python wrapper test and its isolation assertion is implemented by the closed-set registry API. The focused example test passed. End-to-end `make validate` fails identically on unchanged origin/main and this branch because the current provider manifests request semantic core 0.3.0 but the embedded bundle is 0.3.3. The final PR-head full gate remains pending the shared host build slot.
