---
id: SR-184
title: "STD-228 code review of the Rust spec validation gate"
type: SpecReview
analysis: code-review
scope: "STD-228 diff in examples/spec_validate.rs, Cargo.toml, Makefile, .github/workflows/ci.yml, scripts/validate_spec.py and scripts/tests/test_validate_spec.py"
review_set: subset
---

## Summary

Reviewed the changed Rust example, caller configuration, and removed Python wrapper against the repository's Rust conventions and the rust-review and code-review checklists. The existing validation loop and governed exclusions remain intact; the example now receives the two provider roots directly and loads them with `Registry::load_module_set` in the wrapper's former sorted order.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The first example tests cited FR-013-AC-16 while only checking argument parsing and directory presence; they did not exercise closed-set loading or document validation. | examples/spec_validate.rs, FR-013-AC-16 |

## Dispositions

- FND-001: fixed. Replaced the two helper-only tests with one test of the actual provider-loading call path. It asserts both selected archetypes resolve, an extra sibling does not, and a missing named provider is refused. The test cites FR-013-AC-17, which owns ambient isolation; the existing loader test continues to own AC-16's document-validation claim.

## Verdict

The introduced review finding is fixed. The registry API is an exact closed set, so ambient module installations and both module-path environment variables cannot change this gate's schema set. Make and CI still invoke the validation example with the same provider checkouts. The removed Python test's isolation claim is covered by the existing FR-013-AC-17 registry test and the new example provider-loading test. No external source or generated artifact was copied into this repository.

## Validation

The focused `cargo test --locked --example spec_validate` and `cargo clippy --locked --example spec_validate -- -D warnings` passed after the review fix. `cargo fmt --all --check`, `git diff --check`, Cargo metadata for the example test target, and a source inventory of live callers passed. `make validate` fails on this branch and on untouched origin/main `9f42d04` with the same 172 unknown-type failures: the current process and ISO provider manifests both request semantic core 0.3.0 while the embedded bundle is 0.3.3. The final PR-head full gate remains to run.
