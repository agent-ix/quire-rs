---
id: SR-20261113203423
title: "Code review — PLAT-1696 Bash redirect extraction"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; PLAT-1696; src/symbols/shell.rs, tests/fixtures/corpus_cases/plat-1696-shell-here-string.json, tests/it/shell_symbols.rs, spec/functional/FR-051-source-symbol-extraction.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-051", type: references}
---

## Summary

Reviewed Quire PR #686 at `f7dbb52` against FR-051-AC-45 and the changed Bash extraction and test paths. Static review found no defects; the regression directly asserts the later function remains extractable and the file has no diagnostics.

The matrix reports FR-051-AC-45 tagged by both the existing malformed-file test and the new regression. The focused test, formatting and Clippy results are coder-reported; PR CI currently reports only CLA checks.

## Verdict

**PASS** — the implementation matches the existing declaration-structure contract, and the regression guards the reported valid Bash form without removing the malformed-file diagnostic.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Scope

- **Examined — FR-051-AC-45:** An unparseable `.sh` file (a tree carrying an error over its declaration structure) yields a per-file diagnostic naming the file and the line the error sits at, while the rest of the tree extracts normally.
- **Context only — FR-051-AC-9:** An unparseable fixture file yields a per-file diagnostic naming that file and the line the error sits at, while the rest of the tree extracts normally.
- **Context only — FR-051-CON-2:** Adapters SHALL degrade per file: one unparseable file never aborts the tree extraction.
- **Examined — `src/symbols/shell.rs`:** The exception accepts only the here-string error node at the diagnostic position when its next sibling is a `file_redirect`; all other declaration-structure diagnostics still return an error.
- **Examined — `tests/it/shell_symbols.rs`:** `tc2114_redirected_here_string_preserves_symbols` asserts the later function is extracted and diagnostics are empty; `tc2112_unparseable_file` still asserts a malformed file diagnostic while a sibling file extracts.
- **Examined — `tests/fixtures/corpus_cases/plat-1696-shell-here-string.json`:** The fixture records the production command, expected function and expected zero diagnostics, and includes PLAT-1696 as `issue_ref`.

## Review evidence

- `git diff --check origin/main...HEAD`: passed.
- `quire matrix --scope . --format json`: exit 0; FR-051-AC-45 is tagged with two binders, including `tc2114_redirected_here_string_preserves_symbols`.
- `gh pr checks 686`: CLA checks passed; no Rust CI gate was present in the check rollup at review time.
- Independent CLI smoke used the installed Quire 0.45.0 binary, which predates PR #686 and still reports the issue; this is not evidence against the branch implementation. Coder-reported focused tests: 14 passed; fmt and focused Clippy passed.
