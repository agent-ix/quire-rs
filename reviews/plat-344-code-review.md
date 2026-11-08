---
id: SR-16316
title: "Code review (with Rust lane) of the audit-to-clippy migration and scratch-example removal (quire-rs PR 602)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@65e3fc8; Makefile; clippy.toml; scripts/audits/README.md; scripts/audits/check_hashmap_audit.sh; scripts/audits/check_no_shellout.sh; examples/*.rs (11 deleted); src/lib.rs; src/extract/mod.rs; src/merge.rs; src/parser/mod.rs; src/validate_document.rs"
review_set: subset
---

# Code review of quire-rs PR 602

## Summary

Ticket: PLAT-344. Code-review plus rust-review lane over `git diff origin/main...HEAD`
(head 65e3fc8). The PR deletes `check_hashmap_audit.sh` and `check_no_shellout.sh`, moves
both bans into `clippy.toml` (`disallowed-types` for `std::collections::HashMap`,
`disallowed-methods` for `std::process::Command::new`), opts the scoped modules in with
`#![deny(...)]`, makes `make lint` pass `-A clippy::disallowed-types -A
clippy::disallowed-methods` workspace-wide, and deletes 11 scratch examples.

Scope equivalence was measured in a scratch copy of the head (not the worktree), with
`cargo clippy --locked --lib -- -D warnings -A clippy::disallowed-types -A
clippy::disallowed-methods` (the `make lint` form):

- A `HashMap` type annotation, a `HashMap::new()` path, and a `collect::<HashMap<_, _>>()`
  turbofish in `src/merge.rs` all failed, with the lint level reported from
  `src/merge.rs:8` (`#![deny]`). The in-source attribute beats the command-line `-A`.
- A `HashMap` in the child module `src/parser/slug.rs` failed, with the level from
  `src/parser/mod.rs:11`, so the attribute covers the whole subtree as the script did.
- `Command::new("python3")` in `src/skeptic.rs` failed, with the level from `src/lib.rs:12`.

The new bans match or exceed the scripts. The script only flagged `Command::new` with one of
12 literal runtime names. The lint flags every `Command::new` in the library crate, which is
what StR-001-AC-2 says. The HashMap scope is the same four paths. The script's extra
`::HashMap` regex arm would also have matched `hashbrown::HashMap`, but no hashbrown, ahash or
fxhash crate is a direct dependency, so nothing is lost. There is no `src/main.rs` or
`src/bin/`, so moving the ban from "all of `src/`" to "the lib crate" loses nothing.

None of the 11 deleted examples has a `[[example]]` entry or a Makefile, CI or spec
reference. The references left are in historical review records (`spec/reviews/SR-045`,
`SR-048`, `reviews/2026-08-21-wp3-gap-analysis.md`, which stay) and two live doc comments
(FND-003). The `examples` entry in the `wheels.yml` sparse checkout is still needed for
`spec_validate.rs` and `mutants_scope.rs`.

## Verdict

**CHANGES REQUIRED.** The migration enforces the same scope as the scripts, and the
`#![deny]` attributes do work under `make lint`'s `-A` flags. But `clippy.toml` now
applies to every workspace member, and `.github/workflows/ci.yml` still runs bare clippy
with `-D warnings`, which fails on `crates/build-npm-fetch`. The README's reason for
keeping `check_no_shared_mutable.sh` is the right conclusion with the wrong reason. Two
doc comments and `CLAUDE.md` still name things this PR deleted.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | `ci.yml`'s Lint step still runs `cargo clippy --locked --all-targets -- -D warnings` without the opt-in `-A` flags. `clippy.toml` applies workspace-wide, and both lints are warn-by-default, so a dispatched CI run fails. Reproduced: `crates/build-npm-fetch/src/lib.rs:172` and `:206` (`Command::new`) fail the build. A developer running bare `cargo clippy` hits the same error. Fix: make the Lint step run `make lint`, so the flags have one source (a workflow edit, which needs clearing first). | .github/workflows/ci.yml:72-76; Makefile:50-56; clippy.toml:5-16 |
| FND-002 | low | The README says `check_no_shared_mutable.sh` stays because clippy "cannot scope a type ban to two modules or carry per-line exemptions with a stale check". Both halves are false: this PR scopes a type ban with `#![deny]`, and `#[expect(clippy::disallowed_types, reason = ...)]` gives a per-line exemption that fails when stale (`unfulfilled_lint_expectations`). The real reasons: (1) `disallowed-types` is one lint with one shared list, so opting `src/corpus` into a Mutex/OnceLock ban would also ban `HashMap` there (`src/corpus/spec.rs:39`), and the HashMap modules would also get the sync bans (`src/extract/interpolate.rs:14` uses `OnceLock`). (2) `thread_local!`, `static mut` and `unsafe impl Sync` are not types. Keeping the script is right; fix the reason. | scripts/audits/README.md:7 |
| FND-003 | low | Two doc comments still give "`plat843_audit_list` emits one row per source line" as the reason the multi-line exclusion matters. This PR deletes that example. Restate the reason without it, for example that a newline would split one symbol's `compute_id` across lines. | src/symbols/typescript.rs:897; src/symbols/typescript.rs:1755 |
| FND-004 | medium | `CLAUDE.md` tells agents that NFR-006 "is enforced by `scripts/audits/check_hashmap_audit.sh`", and this PR deletes that script. The repo's agent instructions now name enforcement that does not exist. Point it at the `clippy.toml` `disallowed-types` ban and the `#![deny(clippy::disallowed_types)]` opt-in. | CLAUDE.md:128-130 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1 (fix bfbed72, head 95db5a4). The fix differs from the suggestion: `.cargo/config.toml` gains `[target.'cfg(all())'] rustflags = ["-A", "clippy::disallowed-types", "-A", "clippy::disallowed-methods"]`, and the Makefile `-A` flags are reverted, so `make lint` and `ci.yml` run the same bare commands. All three `ci.yml` clippy commands exit 0 on a scratch copy of 95db5a4. Probes still fail: `HashMap` in `src/merge.rs` and `src/parser/slug.rs` (level from `#![deny]`), and `Command::new` in `src/skeptic.rs` and, under `--features python`, in `src/python/mod.rs` (level from `src/lib.rs:12`). Residual edge: Cargo ignores config `rustflags` whenever a `RUSTFLAGS` env var is set (tested: `RUSTFLAGS="--cfg x" cargo clippy -p build-npm-fetch` fails). No clippy invocation in the Makefile or any workflow sets `RUSTFLAGS`; it is set only for the loom and sanitizer `cargo test` runs. |
| FND-002 | fixed | Round 1: the README now says the script is kept because `disallowed-types` has one shared list, so banning Mutex/OnceLock in `src/corpus` would also ban HashMap there, and `thread_local!`, `static mut` and `unsafe impl Sync` are not types. |
| FND-003 | fixed | Round 1: both doc comments now cite "the PLAT-843 audit differential emitted one row per source line" and no longer name the deleted example. |
| FND-004 | fixed | Round 1: `CLAUDE.md:129` now says NFR-006 is enforced by the clippy `disallowed-types` rule in `clippy.toml`, with an opt-in `#![deny]` per module. |
