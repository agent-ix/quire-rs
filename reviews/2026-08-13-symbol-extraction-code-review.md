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

**CONDITIONAL** — one high and one low finding, both fixed and covered by new test cases.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Quote state reset per line, so a `/*` on a continuation line of a multi-line template literal still rejected the whole file | src/symbols/typescript.rs:257 |
| FND-002 | low | `brace_delta` counted braces inside a multi-line template literal, so a literal `{` there unbalanced the file | src/symbols/typescript.rs:227 |
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
continuation line of a multi-line template counted as a block open — the file
unbalanced, `check_balanced` rejected it, and every tag in it bound to nothing.
The same zero-symbol outcome as FND-001, reached by a different route.

Fixed at the caller: `strip_comment` now **drops** carried-in literal content
rather than copying it, so `brace_delta` never sees it. Safe only there — a
continuation line is never a declaration and `${…}` is balanced. A single-line
backtick title is still copied, because `registration` has to read it, and that
is asserted rather than assumed.

Writing the fix introduced a second bug in the same edit — the drop loop cleared
the carried flag whether or not it found the closing backtick, so a literal
spanning three lines lost its state on the second. TC-799 caught it.

**FND-003.** A line such as `const re = /['"]/; // note` leaves a quote open, so
the trailing comment is no longer stripped. `brace_delta` derives the same state
and therefore ignores braces in that text too, so the two agree and no brace is
miscounted. Behaviour-neutral in every case found; recorded because the
suppression is new.

## Disposition

FND-001 and FND-002 are fixed on this branch. FND-003 is behaviour-neutral today
and filed with the other residual per-line assumptions as
agent-ix/quire-rs#62 — the real fix there is one lexer pass every consumer reads,
which is a refactor rather than a patch.

## Coverage

- Reconciliation: quire coverage (module spec-artifacts-process, working tree)
- Rows backed by a tagged test: 144 / 907
- Test suites: 19 green, 0 failing (`cargo test`)
- FR-051-AC-12 covered by TC-798 (single-line) and TC-799 (continuation line + brace balance)
- FR-051-AC-13 covered by TC-800 (wrapped Python signature)
- Semantic review: skipped
