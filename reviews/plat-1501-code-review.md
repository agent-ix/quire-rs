---
id: SR-20261112221720
title: "PLAT-1501 quire-rs PR662 code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs PR662 head 88d6e7b; spec/tests.md (deleted), src/audit_table.rs, examples/spec_validate.rs, tests/it/{audit_table,trace_dogfood,coverage_matrix}.rs, benches/{parse,plain_language}.rs, fuzz/fuzz_targets/fuzz_validate_extract_query.rs, tests/python/test_bindings.py, spec/**, plan/**, CLAUDE.md, Makefile, CHANGELOG.md"
review_set: subset
---

## Summary

Ticket: PLAT-1501. Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD`. The PR deletes `spec/tests.md`, deletes the public `audit_table::audit_spec_dir` (which hardcoded `<spec>/tests.md`), its dogfood test and the audit stage of `examples/spec_validate.rs`, deletes the TC-1880/TC-1881 pins in `trace_dogfood.rs`, and strips TC ids from markers and spec verification cells.

Gate check: `make validate` (the `spec_validate` example) on a copy of the PR head reports `204 document(s), 0 failed`. With the `id:` line deleted from FR-050 it exits 1 with three errors and `1 failed`. The bench and fuzz dogfood tests now pin NFR-002-AC-4, NFR-007-AC-1 and NFR-019-AC-1. The TC-1880/TC-1881 pins only checked that hand-table rows were honest. The tests they pinned still carry `FR-051-AC-25`, so the computed matrix covers them. Focused tests pass (`audit_table::`, `trace_dogfood::`, 9 tests). Both edited benches compile. pytest collects 41 tests. `src/python` is untouched.

The `audit_spec_dir` deletion is correct. It was a this-repo wrapper around one fixed file name, and its only callers were this repo's own gate and test. Nothing in quire-cli, quoin, the Python bindings, wasm, fuzz, benches or docs calls it. `audit_gaps` and the TestMatrix machinery stay.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-050-AC-58 still says the audit check is "run by `examples/spec_validate.rs` (`make validate`) over the spec tree's requirement documents", and the CR-200 note says the same. The PR removed that run, so the spec now makes a claim the code violates. Reword AC-58 so the check is the `audit_gaps` function over a supplied matrix, and add a CR note recording that this repo carries no matrix. | spec/functional/FR-050-declarative-coverage-computation.md:383, spec/functional/FR-050-declarative-coverage-computation.md:1200 |
| FND-002 | low | Public `pub fn audit_spec_dir` is removed without a CHANGELOG entry. The repo convention is a `**Breaking, PLAT-nnn**` line under `[Unreleased]`. | src/audit_table.rs:421, CHANGELOG.md:8 |
| FND-003 | low | Two verification cells still cite TC ids: `Test (TC-992..TC-996)` and `Test (TC-1021, ...)`. Stale doc comments also still say a test "backs TC-1880". | spec/functional/FR-050-declarative-coverage-computation.md:429, spec/functional/FR-065-controlled-corpus-contract.md:637, src/symbols/python.rs:1000 |
| FND-004 | low | `.github/workflows/ci.yml` still has two `NFR-021-AC-2 (TC-1649)` comments. This is the workflow edit that was dropped. It is comment-only, and CI does not reference `tests.md`, so CI does not break. Plan prose also still cites `spec/tests.md`. | .github/workflows/ci.yml:47, .github/workflows/ci.yml:107, plan/Plan-001-ac-grammar-coverage/tasks/Task-001-fr048-severity-framework.md:64 |

## Verdict

The deletion is sound and the validate gate still bites (204 documents, and the mutation fails). No CI or bench reference to the deleted file remains. FND-001 must be fixed before merge. FND-002..004 are small edits for the same PR. The Rust diff is attribute, comment and test-list edits plus one function deletion, with no idiom issues.
