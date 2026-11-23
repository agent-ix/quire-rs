---
id: SR-319
title: "failure-domain review of FR-051 source walk amendment (PR #708)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md (Source walk, FR-051-AC-54, FR-051-AC-55) @ ab0fa9958126956ee82d21b061ace08e73b8710c"
review_set: subset
---

## Summary

Ticket: PLAT-1793. Whether the triage's two silent, machine-dependent omissions
(gitignored and symlinked sources) become deterministic and observable, and whether
the HOME/XDG and ancestor ignore boundary is bounded. HOME/XDG, `.git/info/exclude`
and `require_git` are bounded with measurable oracles (AC-55, clean). Symlinks become
observable, but AC-54 decides reporting from the target, so its own determinism clause
fails on dangling links, and its precedence against the other prunes is unstated.
Gitignored sources become deterministic but remain silent. The ancestor cut changes
subdirectory-scope results without saying so.

## Scope examined

- FR-051 Source walk section — examined
- FR-051-AC-54 — examined (findings)
- FR-051-AC-55 — examined (findings; HOME/XDG and `.git` presence clauses clean)
- FR-050-AC-22 (`source_exclude`), FR-050 two-roots (code root = `<scope>`),
  FR-024-AC-5 — context_only
- Code: src/symbols/mod.rs:376-500 (`extract_tree_scoped`, `count_source_files`) — context_only

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | high | AC-54 reports a symlink "that is a directory" or has a source extension, and also requires identical extraction "whether or not the symlink targets exist". An extension-less symlink `vendor -> ../x` is a directory when `../x` exists (diagnostic) and not when dangling (no diagnostic), so the AC contradicts itself; classifying requires reading the target the rule says is never read. | spec/functional/FR-051-source-symbol-extraction.md:45-48,281 |
| FND-002 | medium | AC-54 reports any symlink "under the code root" without precedence against the other walk rules: a symlink inside a `.gitignore`d dir, a hidden symlink, one matching `source_exclude` (applied per file after the walk, mod.rs:496), one inside a template tree, or a symlinked document root `spec -> real_spec` (tc809) — reported or not is undecided, and two conforming implementations differ. | spec/functional/FR-051-source-symbol-extraction.md:281 |
| FND-003 | medium | Gitignored sources become deterministic but stay silent: AC-55 requires no diagnostic and no count, and `excluded_source_files` counts only `source_exclude` and template prunes. A `.gitignore` line such as `tests/` removes every test symbol with nothing in the report, the triage's "no diagnostic" gap left open for this half. | spec/functional/FR-051-source-symbol-extraction.md:282 |
| FND-004 | medium | "Any ignore file above the code root is never consulted" also discards committed in-repo ignore files when `--scope` is a subdirectory: `packages/x/node_modules/**/*.ts` ignored only by the repository-root `.gitignore` is now extracted. Those files are repository content, not machine state; the CR note does not record this behaviour change and no AC or control covers a subdirectory scope. | spec/functional/FR-051-source-symbol-extraction.md:42-44,282,1116 |
| FND-005 | low | The Source walk says hidden *entries* are not source, but AC-55 asserts only that a hidden-named directory is not walked; a hidden-named source file (`.gen.rs`) has no observable and no control. | spec/functional/FR-051-source-symbol-extraction.md:49,282 |

## Verdict

FAIL. FND-001 must be resolved (decide reporting from the link entry alone, e.g. every
symlink not pruned by an earlier rule) before AC-54 is implementable; FND-002 needs a
stated rule order; FND-003/FND-004 need either an explicit decision in the CR note or a
reported count.

## Dispositions

Round 1, verified against PR #708 head 3afa2b8b1ba5a8a34b94296ccefbf20d9604c2e5.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Rule 5 and AC-54 recognise a symlink from its own link metadata only; a dangling link, a file link and a directory link are one case. |
| FND-002 | fixed | Rules are ordered (a path removed earlier is never judged later) and AC-54 states five precedence outcomes: gitignored, hidden, `source_exclude`, template tree, symlinked document root. |
| FND-003 | fixed | AC-55 counts rule-3 removals, and FR-050-AC-69 adds the `ignored_source_files` report member (omitted when 0, distinct from `excluded_source_files`). |
| FND-004 | fixed | Rule 3 reads ancestor ignore files up to the nearest `.git` ancestor; AC-55 asserts that a repo-root `packages/x/node_modules/` entry still applies to code root `packages/x`. The CR note records the behaviour change. |
| FND-005 | fixed | AC-55 adds hidden controls: a hidden directory holding `.rs` and a hidden `.gen.rs` file, each with a non-hidden twin. |

## New findings (disposition pass 1)

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-006 | medium | AC-54 ends "one tree extracts identically with its link targets present or deleted", but it also covers an in-tree target ("a symlinked file and its in-tree target mint one set"). Failure: `a.rs -> b.rs` with `b.rs` in the tree. Deleting `b.rs` removes b's symbols, so the extraction cannot be identical. Restrict the clause to targets that the walk does not otherwise reach (absent, or outside the root). | spec/functional/FR-051-source-symbol-extraction.md AC-54 |
| FND-007 | medium | Rule 3 bounds ancestor ignore files at the "nearest ancestor holding a `.git` entry". That makes a tree with no `.git` of its own depend on where it is placed. Failure: an extracted tarball or CI artefact under a home directory that is a dotfiles git repo, whose `.gitignore` is `*`, is wholly ignored. Placed elsewhere, the same tree extracts normally. This contradicts the Source walk SHALL (no machine dependence) and AC-55's "identically with and without a `.git` directory". | spec/functional/FR-051-source-symbol-extraction.md Source walk rule 3; AC-55 |
| FND-008 | low | AC-54 says a symlink inside a gitignored directory "is counted by rule 3" without qualification. AC-55 and FR-050-AC-69 count only supported-extension, non-hidden entries. Failure: an extension-less `vendor` symlink inside an ignored `build/` is counted under AC-54 but not under AC-69. | spec/functional/FR-051-source-symbol-extraction.md AC-54; spec/functional/FR-050-declarative-coverage-computation.md AC-69 |

## Dispositions (round 2)

Round 2, verified against PR #708 head 7028b978739e1396f721ffc2cd133b1105dbe670 (rebased on origin/main 2d0836a; diff additive only).

| FND | outcome | reason |
| --- | --- | --- |
| FND-006 | fixed | AC-54 now says an in-root target is read through its own path, so deleting it removes only its symbols. The identity clause is limited to absent or out-of-root targets. |
| FND-007 | fixed | Rule 3 uses an ignore boundary given to the extraction, which defaults to the code root and is never discovered upward. AC-55 asserts the dotfiles-repo control as byte-identical. |
| FND-008 | fixed | AC-54 counts a symlink in a gitignored directory "exactly under the AC-55 predicate", so any other symlink is uncounted. This is consistent with FR-050-AC-69. |

## New findings (disposition pass 2)

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-009 | medium | Rule 3 and the CR-423 note say a caller scoping to a subdirectory "gives that repository's root as the boundary" and that this is "the calling CLI's existing knowledge". No consumer has it. quire-cli (src/commands/coverage.rs:299, symbols.rs:103, assurance.rs:126 at origin/main 61ccfc6) and quoin-quire (coverage.rs:97/160, assurance.rs:63 at 458f13b1) pass only `scope`, and none resolves a repository root. The boundary parameter therefore has no consumer, and the AC-55 "boundary given" case is reachable only from tests. Either drop the parameter (boundary = code root, keeping the already-stated subdirectory consequence) or own the consumer change with a tracked quire-cli/quoin ticket. Also unstated: what happens when the boundary is not an ancestor of the code root. | spec/functional/FR-051-source-symbol-extraction.md Source walk rule 3; CR-423 note |
| FND-010 | low | AC-55 still says a file matched "by an ignore file above the repository root is extracted". Rule 3 no longer defines a repository root, only the ignore boundary. The term should read "above the ignore boundary". | spec/functional/FR-051-source-symbol-extraction.md AC-55 |
