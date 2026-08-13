---
id: SR-004
title: "Code review — CR-035 / CR-036 coverage and symbol-extraction changes"
type: SpecReview
analysis: code-review
scope: "src/symbols/typescript.rs, quire-cli/src/commands/coverage.rs"
review_set: subset
---

# SR-004: Code review — CR-035 / CR-036 coverage and symbol-extraction changes

## Summary

Reviewed the two changes made for agent-ix/quire-rs#58 and agent-ix/quoin#61.
The `--strict` / percentage change is sound. The string-aware comment stripper
was **incomplete in the way that mattered** — it handled the single-line form and
not the form the corpus actually writes — and was completed on the branch.

## Verdict

**CONDITIONAL** — one high finding, fixed and covered by a new test case.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Quote state reset per line, so a `/*` on a continuation line of a multi-line template literal still rejected the whole file | src/symbols/typescript.rs:257 |
| FND-002 | low | `brace_delta` still counts braces inside a multi-line template literal, so a literal `{` there can still unbalance a file | src/symbols/typescript.rs:227 |
| FND-003 | low | A line with an unterminated `'` or `"` now suppresses comment stripping for the rest of that line | src/symbols/typescript.rs:257 |

## Detail

**FND-001.** The fix for CR-036 tracked quote state per line. The corpus form
that triggered CR-036 writes the refspec on a *continuation* line:

```
const cfg = `
[remote "origin"]
fetch = +refs/heads/*:refs/remotes/origin/*
`;
```

A per-line scanner re-enters the third line believing it is in code, re-opens the
block comment one line later than before, and `check_balanced` rejects the file
exactly as it did — which under FR-051-CON-2 means zero symbols and every trace
tag in it binds to nothing.

Proven, not inferred: a probe file with that literal reported its tag unbacked;
the identical file with `/*` removed reported it backed. That was the only
difference. Fixed by carrying template state in a `ScanState` alongside the
block-comment flag, so the three call sites cannot carry one and forget the
other. TC-799 asserts the state at all three boundaries — opens, carries, closes.

**FND-002.** `brace_delta` re-derives quote state per line, so a literal `{` on a
continuation line of a multi-line template is counted as a block open. This
predates both changes and is not made worse by them; `${…}` interpolation is
balanced, so the residual case is a bare brace inside a multi-line literal. Not
fixed — recorded rather than silently accepted.

**FND-003.** A line such as `const re = /['"]/; // note` leaves a quote open, so
the trailing comment is no longer stripped. `brace_delta` derives the same state
and therefore ignores braces in that text too, so the two agree and no brace is
miscounted. Behaviour-neutral in every case found; recorded because the
suppression is new.

## Coverage

- Reconciliation: quire coverage (module spec-artifacts-process, working tree)
- Rows backed by a tagged test: 143 / 905
- Test suites: 19 green, 0 failing (`cargo test`)
- FR-051-AC-12 covered by TC-798 (single-line) and TC-799 (continuation line)
- Semantic review: skipped
