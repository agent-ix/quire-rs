---
id: SR-16323
title: "Code review — PLAT-876 rename quire-rust-extraction to quire-code-extraction (quire-rs PR 605)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; Cargo.toml, Cargo.lock, Makefile, deny.toml, crates/quire-code-extraction/**, scripts/audits/check_tree_sitter_boundary.sh, scripts/audits/README.md, src/symbols/{mod,python,rust,typescript}.rs, tests/it/implements_relation.rs, .github/workflows/wheels.yml (report only)"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references}
---

## Summary

Ticket: PLAT-876. PR head: `9d8e4d5`. Code review with the rust-review lane folded in. The diff is a mechanical rename: a `git mv` of the crate directory, a new package and lib name, and every `use`, feature edge, deny exception, Makefile comment and audit exemption updated. No logic changes.

## Verdict

**PASS with findings.** The rename itself is correct and complete in every place that has behaviour. `Cargo.lock` is consistent (`cargo test --locked -p quire-code-extraction` passes). `deny.toml` license check passes under `--workspace`. The boundary audit passes, fails on an injected `tree-sitter` line in the root `Cargo.toml`, and fails when the crate sits back at the old path, so its exemption is bound to the new name. `dependency_boundary.rs` passes, and fails naming `quire-rs` when the root manifest takes `tree-sitter = "0.26"` directly. `tests/it` `recovered_attribute_is_a_real_missing_token_not_a_valid_candidate` compiles and passes. The only remaining old-name references are CHANGELOG history, `spec/reviews/SR-117`, and two `wheels.yml` comments.

The defects are three stale "rename deferred to PLAT-876" notes that the blind find-and-replace left in place. One of them now contradicts itself.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Find-and-replace made the root manifest comment contradict itself: "Still named `quire-code-extraction` even though it now also wraps the `python` and `typescript` grammars — the rename is tracked as PLAT-876, deliberately deferred so this slice makes zero edits to `rust.rs`". The rename is done. Delete the sentence. | Cargo.toml:96-101 |
| FND-002 | medium | The crate's `NOTE (PLAT-851)` header says "the name is stale. Renaming it is deliberately deferred, tracked as PLAT-876 … Rename once rust.rs is quiet". This PR is that rename, so the note is now false. Delete it. | crates/quire-code-extraction/Cargo.toml:6-13 |
| FND-003 | medium | The module docs still say "This crate's name is stale as of PLAT-851 … the rename is tracked as PLAT-876". They also say "Only `rust.rs` consumes this crate today — `python.rs` and `typescript.rs` still use their own pre-tree-sitter scanners", which has been false since PLAT-868 and PLAT-882. The same applies to "(`rust.rs` today)" at lines 26-29. Rewrite the paragraph to describe the three current consumers. | crates/quire-code-extraction/src/lib.rs:14-30 |
| FND-004 | low | `wheels.yml` comments still name `quire-rust-extraction`. Behaviour is unaffected because the sparse checkout lists the whole `crates` directory. Workflow edits need clearing first, so this is reported only. | .github/workflows/wheels.yml:77-79 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| Cargo.toml (members, dependency, three `*-symbols` features) | examined | Correct; FND-001 on a comment |
| Cargo.lock | examined | Consistent; `--locked` builds |
| crates/quire-code-extraction/Cargo.toml (package, lib name) | examined | Correct; FND-002 on the header note |
| crates/quire-code-extraction/src/lib.rs | examined | Unchanged by the diff; FND-003 |
| crates/quire-code-extraction/tests/dependency_boundary.rs | examined | Passes and fails correctly |
| deny.toml | examined | License check passes |
| scripts/audits/check_tree_sitter_boundary.sh EXEMPT | examined | Passes and fails correctly |
| Makefile, scripts/audits/README.md | examined | Comments updated |
| src/symbols/{mod,python,rust,typescript}.rs | examined | `use` paths updated; compiles |
| tests/it/implements_relation.rs | examined | Compiles; the targeted test passes |
| .github/workflows/wheels.yml | examined | FND-004 (report only) |

## Verification

These commands ran on a `git archive` copy of the head, with `corpus` symlinked from another checkout, and used a scratch target that was deleted afterwards: `cargo test --locked -p quire-code-extraction`; the boundary test with an injected `tree-sitter = "0.26"` (FAILED as expected); `check_tree_sitter_boundary.sh` at base (exit 0), with an injected dependency (exit 1) and with the crate at its old path (exit 1); `cargo deny --workspace check licenses` (ok); `cargo test --locked --test it implements_relation::recovered_attribute` (ok).
