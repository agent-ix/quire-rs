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
