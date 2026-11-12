---
id: SR-20276
title: "code-review and rust-review of PLAT-1420/PLAT-1485 (quire-rs PR 658)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@c9ff0947; src/clauses.rs; src/lib.rs; src/loader/manifest.rs; src/loader/mod.rs; tests/it/clause_publishable.rs; tests/it/main.rs; CHANGELOG.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-083
    type: references
  - target: ix://agent-ix/quire-rs/FR-073
    type: references
---

# Code and Rust review of PLAT-1420/PLAT-1485 (quire-rs PR 658)

## Summary

Ticket: PLAT-1420 (with PLAT-1485). One pass over `git diff origin/main...c9ff0947`, with the rust-review lane folded in. Focused run: `cargo test --test it clause_publishable` 5/5 pass; `cargo clippy --lib --test it -D warnings` clean. Mutation checks: removing `check_clause_sets_shape` fails TC-2313; changing the filter to `text != None` fails TC-2314 and TC-2316. Readers of `Manifest.clause_sets` (Vec<PathBuf> to Vec<String>): only `src/loader/mod.rs:581,829`; no use in crates/, fuzz/, python bindings, assurance export, or quire-cli (which uses `Registry::clause_sets()`).

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | `check_clause_sets_shape` re-parses the whole manifest YAML into `serde_yaml::Value` on every load to check one key. Fix: a `#[serde(deserialize_with)]` on `clause_sets` that reads `Vec<serde_yaml::Value>` and rejects non-string entries, so the check runs in the single parse. | src/loader/manifest.rs:589,607-621 | implementation-bug-despite-evidence |
| FND-002 | low | `Manifest.clause_sets` changes public type from `Vec<PathBuf>` to `Vec<String>`; the CHANGELOG lists only additions. No in-repo or quire-cli reader breaks. Fix: add a `### Changed` line naming the field type change. | CHANGELOG.md:10-16; src/loader/manifest.rs:341 | implementation-bug-despite-evidence |

## Verdict

Mergeable after the low findings, which do not block. `check_publishable` is pure (reads only `&LoadedModule`, no IO), restricted iff `rights.text == Licensed`, carries no clause text, and sorts by authority/id/version. `Licensed` passes `validate` without `clearance_ref` and with or without text. No compat layer, pin or hash added.
