---
id: SR-16317
title: "Gap analysis of the audit-to-clippy migration against NFR-006-AC-3 and StR-001-AC-2 (quire-rs PR 602)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs@65e3fc8; clippy.toml; Makefile; src/lib.rs; src/extract/mod.rs; src/merge.rs; src/parser/mod.rs; src/validate_document.rs; spec/tests.md; plan/tasks/025-static-audits.md; plan/tasks/README.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/NFR-006
    type: reviews
---

# Gap analysis of quire-rs PR 602

## Summary

Ticket: PLAT-344. Planless gap analysis of the change set. Plan completion: not assessed.
Run with quire 0.36.2 (engine 0.50.2) and quoin 0.29.1.

NFR-006-AC-3 reads: "A static check via clippy lint or audit confirms no
`std::collections::HashMap` usage in parse / extract / validate paths ... (the audit path
covers `src/parser`, `src/extract`, `src/validate_document.rs`, `src/merge.rs`)". The AC
already allows a clippy lint, and the four `#![deny(clippy::disallowed_types)]` opt-ins
cover exactly those paths (probes in SR-16316). StR-001-AC-2 reads: "No call site in
`quire-rs` shells out to Python, Node, or any other interpreter". The crate-wide
`disallowed-methods` ban on `Command::new` covers that more fully than the deleted
literal-name grep did.

`quire matrix --scope .` shows NFR-006-AC-3 as `(none) | method-without-symbol`. That was
also true before this PR: a bash script was never a symbol the matrix could see. This PR
neither fixes nor causes it.

## Verdict

**PASS WITH LOW FINDINGS.** Both ACs are enforced at the same or broader scope. The
TC rows and plan notes that describe the enforcement now describe the deleted scripts.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `spec/tests.md` still describes the deleted scripts. TC-058 reads "Static audit: no HashMap in render/parse code paths", although the render path is gone and the scope is four parse, extract and validate paths. TC-201 reads "Static grep: no `std::process::Command` invocations targeting python/node/npm/pip in src/". Both checks are now clippy `disallowed-*` lints run by `make lint`, and TC-201's check is now every `Command::new`. Update both rows to match. | spec/tests.md:223; spec/tests.md:270 |
| FND-002 | low | Plan task notes name the deleted scripts as the enforcement: `025-static-audits.md` subtasks and Owns list, and the README's task 020 row ("tests/it/determinism.rs + check_hashmap_audit.sh"). Change them to the clippy bans. | plan/tasks/025-static-audits.md:13,15,36,38; plan/tasks/README.md:38 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (fix bfbed72, head 95db5a4): TC-058 and TC-201 rows now describe the clippy `disallowed-types` / `disallowed-methods` checks with the current scope. |
| FND-002 | fixed | Round 1: `plan/tasks/025-static-audits.md` subtasks and Owns list, and the `plan/tasks/README.md` task 020 row, now name the clippy bans. |
