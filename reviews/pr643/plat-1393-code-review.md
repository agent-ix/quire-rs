---
id: SR-20271
title: "code-review and rust-review of trait-qualified Rust symbol ids (quire-rs PR 643)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@3837b815; src/symbols/rust.rs; tests/it/implements_relation.rs; tests/fixtures/corpus_cases/trait-impl-symbol-ids.json"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-051
    type: references
---

# Code review of trait-qualified Rust symbol ids (quire-rs PR 643)

## Summary

Ticket: PLAT-1393. One code-review pass with the rust-review lane over `git diff origin/main...3837b815`. Focused tests: `cargo test --lib symbols::` (142 passed) and `cargo test --test it -- implements_relation symbol` (36 passed). `normalize_type_text` was traced by hand over lifetimes (`&'a mut T`, `& 'a T` -> `&'a T`), paths, nested generics, `dyn A + Send` (-> `dyn A+Send`), `fn(u8) -> u8`, arrays (`[u8; N]` -> `[u8;N]`) and commas (`A ,B` / `A, B` -> `A, B`). It is deterministic: the output depends only on the field text. The where clause, `unsafe` and the negative `!` are separate tree-sitter nodes, so they never reach the `type`/`trait` field text. `impl<T> From<T> for X<T>` yields `<X<T> as From<T>>`. The new fixture test fails on pre-fix code, which minted `Halt::from` three times. No committed artifact in this repo stores a trait-impl qualified name, and `bare_name`/`simple_name` still split the method name off the last `::`. A `#[test]` cannot be an associated function, so no test symbol is re-keyed. An `implements` marker on a trait-impl method still binds, under the new id.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | The module doc says `impl Display for Foo` mints `Function "<Foo as std::fmt::Display>::fmt"`. The code keeps the trait as written and resolves no path, so that source mints `<Foo as Display>::fmt`. Fix: make the example agree with the code. | src/symbols/rust.rs:44-49 | implementation-bug-despite-evidence |
| FND-002 | low | A trait-impl method's `contains` edge now starts at `path#<Foo as Trait>`, which is never a node. Before the change it started at the `Foo` struct node in the same file. The exported graph no longer links a type to its trait methods. No consumer in this repo reads it. Fix: state the new edge shape in FR-051, or also emit a `Foo` -> `<Foo as Trait>` edge. | src/symbols/trace.rs:621-627 | missing-requirement |

## Verdict

Mergeable on code: two low findings, no wrong ids, no panics, determinism holds. The spec gap is recorded in SR-20272 and SR-20273.
