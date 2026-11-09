---
id: SR-16364
title: "Failure-domain review of PLAT-362 shell symbol adapter (quire-rs PR 614)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md (PR 614 diff only, head 63ac3f0); context FR-050-AC-25, AP-201, scripts/"
review_set: subset
---

## Summary

Ticket: PLAT-362. This pass looked for unstated failure modes in the shell adapter: syntax the line-structural reader can misread, `entry_paths` edge cases, and silent non-binding. AP-201's material scenario is a coverage figure built from a population the engine did not read. Several gaps here produce exactly that silent zero.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-35 names three spellings, each with `{` on the same line. Several common forms are unspecified, so two implementers would read them differently: `name()` with `{` on the next line, a subshell body `name() ( … )`, other compound bodies, and bash names containing `-`, `.` or `:`. | spec/functional/FR-051-source-symbol-extraction.md:202 |
| FND-002 | medium | The decoy lists in AC-35 and AC-37 miss constructs a line reader gets wrong: a here-string `<<<` (not a heredoc), quoted or tab-stripped heredoc delimiters (`<<'EOF'`, `<<-EOF`), backticks, `$'…'`, `$(( ))`, a brace group `{ …; }`, and `}` inside `${x:-}}` during brace matching. A word-internal `#` (`a#b`, a URL fragment) is not a comment. CRLF line endings are also unaddressed. Each can mint a false function, hide a real one, or raise a false unterminated-brace diagnostic that drops the file (AC-39). | spec/functional/FR-051-source-symbol-extraction.md:204, spec/functional/FR-051-source-symbol-extraction.md:202, spec/functional/FR-051-source-symbol-extraction.md:206 |
| FND-003 | medium | A shell marker that does not bind is silent. A marker in a script outside every `entry_paths` glob, or a marker that is not in a channel block, binds nothing, and AC-22 reports only legacy forms. So a misplaced audit, or a glob typo, leaves its `Static` row unbacked with no diagnostic: AP-201's silent zero. AC-22's reason for excusing canonical markers (attribute syntax the language attaches to a declaration) does not hold for a comment. Require a report naming the file, the id and the reason. | spec/functional/FR-051-source-symbol-extraction.md:203, spec/functional/FR-051-source-symbol-extraction.md:190, spec/functional/FR-051-source-symbol-extraction.md:207 |
| FND-004 | medium | The semantics of `entry_paths` are unstated. The spec does not give the glob dialect, whether matching is against the repo root or the `--scope` root, or how an invalid glob is handled (FR-050-AC-25 refuses `source_exclude` as one unit). It does not say how `entry_paths` interacts with `source_exclude`, or that globs from several shell forms are a union. The glob is module (fleet) data, so it does not say how a repo whose audits live elsewhere gets them bound; this repo's own `scripts/check_property_purity.sh` is outside `scripts/audits/`. | spec/functional/FR-051-source-symbol-extraction.md:203, spec/functional/FR-051-source-symbol-extraction.md:205 |
| FND-005 | low | AC-37's file header is "the file's leading `#` comment block after an optional `#!` line". It does not say whether a blank line between the shebang and the block, or a `set -euo pipefail` line before it, still counts as leading. Both are common in this repo's scripts. A wrong reading silently drops the audit's tag. | spec/functional/FR-051-source-symbol-extraction.md:204 |

## Verdict

Examined: the Shell adapter section, FR-051-AC-34..40 and TC-2101..2107. Identity holds. Functions are named by bare name, so a redefinition or a same-named nested helper collides under AC-31's existing advisory diagnostic. `entry_point` differs from a function named `entry_point` by kind. Fail-per-file holds (AC-39). FND-002 is the main cost of the line-structural choice. With tree-sitter-bash (SR-16360 FND-001) it reduces to grammar coverage.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | medium | A shell marker in a function's comment block binds nothing and is reported nowhere. AC-38 says a shell function binds no `verifies`. AC-22 hands shell markers to AC-44. AC-44 reports only a header outside every glob (`outside-entry-paths`) and a comment that is "neither a function's block nor a header" (`not-in-tag-channel`), so a function's block is excluded. Tagging the function an audit script runs is the most likely authoring mistake, and it leaves the `Static` row unbacked in silence (AP-201). Add a reason for this case, for example `function-not-evidence`. | spec/functional/FR-051-source-symbol-extraction.md:232, spec/functional/FR-051-source-symbol-extraction.md:226, spec/functional/FR-051-source-symbol-extraction.md:209 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-35 mints a function for every `function_definition` node: the three spellings, `{` on the next line, a subshell body, names with `-`, `.` or `:`, and nested definitions. |
| FND-002 | fixed | AC-36 covers `$'…'`, backticks, `$( )`, `<<<`, `<<EOF`/`<<'EOF'`/`<<-EOF`, `case` patterns, brace groups, `${x:-}}`, `$(( ))` and CRLF. AC-41 covers `$#`, `${#x}`, `${x#pat}` and word-internal `#`. Reading comments from tree-sitter-bash comment nodes removes the line-lexer failure class. |
| FND-003 | fixed | AC-44 reports a non-binding shell marker with reason `outside-entry-paths` or `not-in-tag-channel`, naming the id, path and line, and AC-46 asserts the report end to end. The residual function-block case is new finding FND-006. |
| FND-004 | fixed | AC-39 states the `source_exclude` dialect and compile check, matching relative to the code root, a union across all forms and modules, `source_exclude` applied first, rejection at load naming `entry_paths`, and a diagnostic for uncompiled globs. |
| FND-005 | fixed | AC-40 defines the header as the first run of consecutive `#` lines before the first command, skipping a `#!` line and blank lines. A comment after `set -euo pipefail` is not header. |
| FND-006 | fixed | AC-44 adds reason `function-not-evidence` for a marker in a shell function's comment block (a shell function binds no `verifies`, AC-38), and TC-2111 lists all three reasons (round 2, head 1e924db). |
