---
id: SR-101
title: "Independent review of the TypeScript regex literal specification"
type: SpecReview
analysis: code-review
scope: "PR #421 at 5b40266; FR-051-AC-25..AC-28; TC-1836..TC-1839; qa-corpus PR #19 at 0f63069; issues #403, #413"
review_set: subset
---

## Summary

The specification is grounded in facts that check out, and the banked corpus
pair discriminates on exactly the mechanism it claims. The
`filament-core-data@3b75e01` `test/compiler.test.ts` blob digest in AC-28 is
byte-correct, line 1291 is the brace-bearing regex, and the qa-corpus failure
and control differ by one regex atom yet produce a missing binding census
versus `candidates 1, bound 1` against today's engine. Three low observations
remain, all about what happens after this lands rather than about the
specification's correctness.

## Verdict

**CONDITIONAL** — no high or medium findings.

## Verification performed

| Check | Result |
| --- | --- |
| AC-28 blob digest | `git show 3b75e01c…:test/compiler.test.ts \| sha256sum` = `54c96129e59dab60b37f8ee3107ecd703aa8cf0f864279ff8f49b5be6402074a` — **exact match** |
| AC-28 line reference | line 1291 is `/else if \(checkMode\) \{[\s\S]*Missing retained lockfile/` — exactly the described construct |
| Recorded revision published | `3b75e01` is an ancestor of `filament-core-data` `origin/main` |
| qa-corpus failure case, live | `binding_census` absent — the file is refused and its only symbol is lost |
| qa-corpus control case, live | `binding_census: [{language: typescript, candidates: 1, tagged: 1, bound: 1}]` |
| `lex_line` mechanism | inside a quote the brace match is skipped, so the `{` in the failure case's *string* contributes nothing; the sole depth source is `\{` in the regex, and the pair isolates one variable |
| qa-corpus focused gates on #19 | schema self-test, duplicate census, external channel, compatibility-corpus self-test, bounds — all pass |
| `quire validate` on #421 | the 11 failing documents are the `spec/assets/*` frontmatter and AP-201/MP-20x schema mismatches present identically on `main`; no document this PR changes fails |

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | AC-25 classifies `/` after `)` as division, so a regex in statement position after a control-flow head reproduces the #403 refusal | spec/functional/FR-051-source-symbol-extraction.md | correct-requirement-no-evidence |
| FND-002 | low | AC-28 binds an external blob by digest with no retained copy and no reachability requirement | spec/functional/FR-051-source-symbol-extraction.md | missing-requirement |
| FND-003 | low | The #403-before-#413 ordering lives only in the PR body, and both change `lex_line` | src/symbols/typescript.rs:527, src/symbols/typescript.rs:601 | missing-requirement |

## Finding detail

### FND-001 — one expression-ending token is context-dependent

The AC-25 boundary section lists `)` among the tokens after which `/` is a
division operator. That is right for `(a + b) / c` and wrong for a regex in
statement position after a control-flow head:

```ts
if (cond) /\{/.test(subject);
```

Here `)` closes the `if` head, the following `/` opens a regex, and the
classifier will read it as division — counting the `\{` as code and refusing the
file, which is the #403 defect exactly.

Two things make this low rather than higher. AC-26 now requires a diagnostic
naming the construct and its 1-based line, so the failure is reported instead of
silently dropping symbols — the actual harm in #403 was the silence. And the
Scope paragraph already declares the classifier as syntax-level rather than a
full ECMAScript grammar. What is missing is that the paragraph disclaims only
automatic semicolon insertion, and this case is not ASI: it is a keyword-head
paren. Naming it — alongside the ASI sentence, or by excluding `)` that closes an
`if`/`for`/`while`/`with` head from the expression-ending set — costs one clause
and removes the one known way the governed defect returns.

### FND-002 — a digest is not retention

AC-28 is a `Demonstration` over
`filament-core-data@3b75e01c652ba00bb07c352ff5467419401e792b:test/compiler.test.ts`.
I verified the digest and the revision's reachability, so the AC is satisfiable
today. Nothing in FR-051 requires it to stay satisfiable: no copy of the blob is
retained in this repository or in qa-corpus, and no check asserts the revision
remains reachable from that repository's `origin/main`.

`agent-ix/qa-corpus#17`/`#18` established precisely this distinction for the
compatibility corpus — an object present in a local clone is not an object a
clean verifier can fetch — and shipped
`require_published_source_refs` to enforce it. TC-1839 deserves the same
sentence: either retain the blob beside the case, or require the recorded
revision to be an ancestor of the source repository's published main before the
demonstration counts.

### FND-003 — the ordering constraint is not in the specification

`src/symbols/typescript.rs:527` (`let chars: Vec<char> = line.chars().collect()`,
the allocation #413 removes) and `:601` (the `'{' => delta += 1` match #421's
regex state must guard) are in the same function, in the same pass. #421's PR
body says "#413 remains sequenced after #403 because both touch
`src/symbols/typescript.rs`", which is correct and is the right call — but a PR
body is not part of the record after merge. The CR-161 note added to FR-051 does
not mention #413. One clause there, or a blocking note on #413 itself, keeps the
constraint where the next author will read it.

## On qa-corpus #19

Reviewed as the evidence #421's AC-27 binds. It is a good bank:

- The failure and control inputs differ in exactly one regex atom (`\{` → `x`).
  The accompanying change to the asserted string is inert, because `lex_line`
  skips the brace match inside a quote — so the pair isolates one variable with
  respect to the quantity being measured, not merely one line.
- The forward contract is separated from the live contract. `expect.yaml`
  records today's zero-binding state; `expect-pending.yaml` records the
  post-fix `candidates 1 / bound 1` plus the three diagnostic reasons that must
  disappear. The case cannot quietly become "passing" by changing what it
  expects.
- The Python and Rust exclusions are grounded in the language rather than
  asserted: neither has a slash-delimited regex token, so the cell is genuinely
  out of scope rather than untested.
- `make ci` still fails at `verify` with the 18 pre-existing quoin external-
  producer drift mismatches (`0.21.9-76-gd3ed56a` recorded, `0.23.1` installed),
  which reproduce identically on `main`. The PR body says so explicitly and
  names the owning issues, which is the right disclosure.

## What is correct

- AC-25's boundary section is a real specification of a lexer, not a
  restatement of the bug: it enumerates the expression-ending set, the regex
  scanner's closing rule, character-class handling, escape handling and flags,
  and it says where the boundary is rather than implying completeness.
- AC-27 puts a mutation obligation in the acceptance criterion itself —
  "Reverting regex-literal masking makes the corpus case and delimiter test
  fail, and classifying division as regex makes the operator controls fail."
  That is the discipline SR-092 had to establish the hard way on #412.
- AC-26 converts the #403 harm from silent symbol loss into a diagnostic that
  names the construct, the 1-based line, and the file, and requires a readable
  sibling to keep extracting.
- The PR repairs two pre-existing traceability omissions (FR-051-AC-23 and
  AC-24 had no matrix rows) while it is in the file, and marks FR-051's summary
  row `🚧 CR-161 pending` rather than leaving it `✅ Implemented`.
- The AP-201 governance blocker is recorded rather than worked around. I
  reproduced it: `AP-201-detection-minting.md` fails on missing `boundary`,
  `impacts`, `evidence` and `exceptions` sections against the installed
  AssuranceProfile schema, identically on `main`. Not claiming AP-201
  satisfaction is the correct disposition.
