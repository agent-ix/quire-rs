---
id: SR-20261103118117
title: "PLAT-217 and PLAT-1056 row-width assert and foreign ix authority code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs PR590 head 677dbd60; src/corpus/resolve.rs; src/corpus/validate.rs; src/extract/assert_eval.rs; src/validate_document.rs; docs/USAGE.md"
review_set: subset
---

## Summary

Tickets: PLAT-217, PLAT-1056. One code review with the Rust lane over `git diff origin/main...677dbd60` of agent-ix/quire-rs PR 590. Focused tests run, all pass: `cargo test --lib tc1991` (1), `--lib corpus::` (73), `--test it -- bundle_validate trace_references` (28); no full gate. A scratch-only probe test (deleted after the run, never on the branch) confirmed FND-001 and FND-002: with no package, `ix://agent-ix/own/WRONG-9` gave no `dangling-reference` under Strict or Okf; a source with `relationships: [GONE-1]` plus body `ix://agent-ix/own/GONE-1` produced two `GONE-1 references` edges (authority `None` and `Some("agent-ix/own")`), and with package `agent-ix/own` Strict reported the `GONE-1` dangling error twice. The consumer path was checked in agent-ix/quire-cli `origin/main` `src/commands/validate.rs`: `run_okf` builds `Spec::from_path` and never calls `Spec::with_bundle_package`, although the same file already derives `repo_bundle_package(scope)` for per-document validation.

Row width (PLAT-217): widths are re-read with `split_row_unescaped`, the same splitter the table parser uses for both header and rows, so the count is consistent with the parsed header. An escaped `\|` stays one cell; leading and trailing delimiter pipes are dropped symmetrically, so a row with or without outer pipes counts the same; `| A | |` counts two cells (only one trailing empty fragment is popped). An unescaped pipe inside a code span splits the cell, which matches GFM (pipes in code spans must be escaped there too) and the parser, so a row-width failure in that case is a true positive. Line mapping uses the same `content.split('\n')` numbering as the parser. A header mismatch suppresses row-width checks as FR-033-AC-17 says. No defect found in the counting logic.

Foreign authority (PLAT-1056): see FND-001 to FND-003. Plan completion: not assessed.

## Verdict

Changes requested. The row-width half is sound. The foreign-authority half treats an undeclared package as "everything foreign", which on the shipping CLI path silences dangling detection for own-repo `ix://` links (FND-001), and the widened dedup key breaks single-edge dedup for unresolved targets (FND-002).

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | With no declared bundle package, every unresolved `ix://<org>/<repo>/ID` is excused (`is_some_and(\|a\| spec.bundle_package() != Some(a))` is true when the package is `None`), under Strict and Okf alike. `validate_bundle_at` and quire-cli `run_okf` never declare one, so `quire validate --okf` stops reporting a wrong in-repo id written as an `ix://` link, which is the form PLAT-1056 is about and the reverse of its "a wrong in-repo ID still does" acceptance. Minimal fix: excuse only when both are known and differ (`matches!((edge.authority.as_deref(), spec.bundle_package()), (Some(a), Some(p)) if a != p)`), keep package-less behaviour as dangling, drop "(or no package is declared)" from FR-038-AC-9, and wire `repo_bundle_package(scope)` into `with_bundle_package` in quire-cli `run_okf` so the CLI gets the feature. | src/corpus/validate.rs:315-320 | wrong-requirement |
| FND-002 | medium | The stub dedup key became `(source, target, type, authority)`. An absent target declared once as a bare id (frontmatter) and once as an `ix://` link (body), or under two authorities, now yields two `Edge`s with the same `(source, target, type)` and two `DanglingReference` diagnostics, breaking FR-026-AC-8/AC-11 ("exactly one edge"). Fix: keep the set keyed on the triple and carry the authority beside it (for example a map keyed by the triple, where a bare id or own authority wins). | src/corpus/resolve.rs:75-110 | implementation-bug-despite-evidence |
| FND-003 | low | `ix_authority` takes the first two segments, so the two-segment form `ix://<org>/<ID>` (accepted by `extract_target_id`, e.g. `ix://x/StR-001`) yields authority `<org>/<ID>`. Once a package is declared, such an own-org link is always "foreign" and never dangling. The doc's "one when the URI names only an org" branch is unreachable for regex-harvested links. Fix: return an authority only for three or more segments, or document the two-segment case as repo-level. | src/corpus/resolve.rs:309-319 | missing-requirement |
| FND-004 | low | The `LocatedTable` alias was inserted between `locate_table`'s doc comment and the function, so rustdoc attaches the #254/CR-097 line-arithmetic explanation to the alias and `locate_table` loses it. Fix: move the alias and its doc above the existing doc block. | src/extract/assert_eval.rs:174-196 | implementation-bug-despite-evidence |
