---
id: SR-16360
title: "Base review of PLAT-362 shell symbol adapter (quire-rs PR 614)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md, spec/tests.md (PR 614 diff only, head 63ac3f0); context src/symbols/mod.rs, src/traceability.rs, deny.toml, crates/quire-code-extraction/Cargo.toml, scripts/"
review_set: subset
---

## Summary

Ticket: PLAT-362. This is the base checklist pass over CR-213: the shell adapter prose, the FR-051-CON-1 exception, FR-051-AC-34..40, TC-2101..2107, the FR-051 coverage row and the TC-502 row. The ids are well formed. They do not collide with main or with open PRs #612/#613 (#609 is merged). Each new AC maps to exactly one TC in both tables. Three problems remain. The CON-1 carve-out is not justified, and the repo's own dependency-boundary design points to tree-sitter-bash instead. The CR-213 note cites shell audits that do not exist. The TC-502 status text points at the wrong ACs.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The FR-051-CON-1 exception ("the shell adapter ... is line-structural") is unjustified. The CR note's reason is "the surface is two constructs". But AC-35, AC-37 and AC-39 require full shell lexing: single and double quotes, `$( )`, heredoc bodies, `case` patterns, `$#`/`${#x}`/`${x#pat}`, and brace matching to an unterminated brace. That is the hand-written lexer class that CR-039 and CR-040 paid for and that PLAT-843/868/882 removed, leaving every adapter on tree-sitter. The repo already has a home for a new grammar. A `bash` feature in `quire-code-parse` (quire-code-rs, which currently ships rust/python/typescript only) would be forwarded by `quire-code-extraction` (the single PLAT-911 boundary crate, enforced by `check_tree_sitter_boundary.sh`) and gated by a `shell-symbols` feature, like the other three. tree-sitter-bash is MIT, within deny.toml's allow list, and needs no new git source. With it, CON-1, the Language-adapters statement and the Trace-tag grammar section all hold with no exception. Specify tree-sitter-bash, or record a real reason it cannot be used (for example wasm availability) and accept the hand-lexer risk explicitly. | spec/functional/FR-051-source-symbol-extraction.md:162, spec/functional/FR-051-source-symbol-extraction.md:927, crates/quire-code-extraction/Cargo.toml, deny.toml |
| FND-002 | medium | The CR-213 note rests on facts that are false at head. It cites "the other shell audits that enforce a `Static` row (TC-050, TC-203)". But `scripts/check_unsafe_comments.sh` and `scripts/audits/verify_cookiecutter_inheritance.sh` do not exist. The repo tracks four `.sh` files: three in `scripts/audits/` and `scripts/check_property_purity.sh`. The note also says "The file is the entry point (AC-35)", but entry points are AC-36. | spec/functional/FR-051-source-symbol-extraction.md:914-921, spec/tests.md:215, spec/tests.md:272 |
| FND-003 | low | The TC-502 status says it is "Specified by FR-051-AC-34..37" and "flips when the engine and the module's shell marker form ship". Binding a `Static` row is AC-38 (marker form) plus AC-40 (end to end). The flip also needs two things nothing tracks: this repo's script header must carry the declared marker (it currently carries prose, "TC-502: this script IS ..."), and a declared `entry_paths` glob must cover `scripts/audits/`. | spec/tests.md:532, scripts/audits/check_no_shared_mutable.sh:4 |

## Verdict

Examined: the Shell adapter section (FR-051:85-104), FR-051-CON-1, FR-051-AC-34..40, the CR-213 note, TC-2101..2107, the FR-051 coverage row and its traceability rows, and the TC-502 row. Ids are clean: CR-213, FR-051-AC-34..40 and TC-2101..2107 appear nowhere on main, #612 or #613. The coverage range `AC-1..31, 34..40` correctly leaves AC-32/33 to #612, and it becomes `AC-1..40` when both land. The spec does not contradict the approved shell ruling (PLAT-390 research comment). Option 4 (port audits to in-language tests) is left as a choice each repo can still make, not reversed. The diff adds no vendoring, compat layer or tracking record. The branch is behind main (#609 merged) and conflicts textually in spec/tests.md, so it needs a rebase. FND-001 needs a decision before merge.
