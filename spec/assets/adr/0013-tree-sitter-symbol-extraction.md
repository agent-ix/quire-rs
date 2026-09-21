---
id: ADR-0013
title: "Tree-sitter Adoption for Rust Symbol Extraction"
type: ADR
---

# ADR 0013: Tree-sitter Adoption for Rust Symbol Extraction

**Status**: decided (v1)
**Date**: 2026-09-20
**Decision authority**: PLAT-843 owner

## Context

`src/symbols/rust.rs` ([FR-051](../../functional/FR-051-source-symbol-extraction.md))
extracted Rust symbols with a hand-written, line-structural scanner: a
brace-depth counter plus a string/comment-aware lexer (`LexedLine`,
`ScanState`, `lex_line`, `check_balanced`), recognising declarations by their
leading keyword rather than by parsing them. That design traded a parser
dependency for a class of defects the scanner could not avoid by
construction:

- **Whole-file rejection on desync.** `check_balanced` maintains one global
  brace-depth counter; any edge case that counter does not model correctly
  anywhere in a file returns `Err` for the **whole file**, contributing zero
  symbols and losing every trace tag in it (PLAT-163's shape). CR-040 patched
  the *known* triggers (raw strings, lifetimes, char literals, nested block
  comments) after they cost 33 of this repo's own source files their symbols
  — 78 of 140 reported status lies at the time. The fix pattern does not
  generalise: the architecture is "one undiscovered edge case away from the
  same incident," not "the known incidents are now excluded."
- **Span truncation on multi-line syntax.** `leading_block` walks backward
  line by line, matching `#[`, `#![` and `//` textually. A rustfmt-wrapped
  attribute continuation line starts with none of those, so the leading span
  truncates and a tag written in a genuinely-attached attribute falls outside
  `attached_source()` (PLAT-69). A block doc comment (`/** ... */`) was never
  recognised as an annotation at all (PLAT-846).
- **A formatter can move a coverage number.** Both defects above are
  triggered by *reformatting*, not by a semantic change — the property
  `FR-051-CON-1`/AC-10 exists to guarantee (coverage numbers move only for a
  reason a reader can find in the analysed source) held only as far as the
  scanner's own text-matching happened to cover.

`spec-artifacts-process`'s own precedent for a narrower case
(`ix://agent-ix/quire-cli/FR-025-CON-1`) already admits no hand-written lexer
for Rust-source classification; `quire-qualify` reads `syn`'s tree instead.
`FR-051`'s `CON-1` is amended (PLAT-842, CR-176) to state the same rule for
this extractor's three languages, and this ADR is the corresponding
dependency decision, landing with the code it describes (PLAT-843) rather
than with the spec-only amendment PR — an ADR states current state, and at
PLAT-842 time the dependency this ADR names did not exist yet.

Phase 1 (`PLAT-843`) is Rust only. Python and TypeScript keep their current
line/indentation-structural adapters; the same class of defect is expected
there and is Phase 2's decision (`PLAT-851`), not retroactively decided here.

## Options surveyed

| Option | Pros | Cons |
|---|---|---|
| **`tree-sitter` via `quire-code-parse`** (chosen) | Real grammar, incremental/error-tolerant parsing, per-node local error recovery instead of one global desync; `quire-code-parse` is a shared, borrowed, dependency-free-of-analysis parse layer already built for exactly this consumer set (`quire-rs`, `filament-ide-rs`, a future daemon) | An external grammar becomes an input to the output (grammar-version determinism, addressed below); a native (C) build step per grammar, which does not cross-compile to `wasm32-unknown-unknown` without a wasm-targeting C toolchain this workspace does not carry |
| `syn` | Pure Rust, no C build step, already precedented one repository over (`quire-qualify`) | Parses only *syntactically valid* Rust as a `TokenStream`/AST for macro-adjacent tooling — it is not built to recover locally from a genuinely broken file the way this extractor's own contract (`FR-051-CON-2`, per-file degradation) needs, and has no equivalent for the other two languages this crate's `quire-code-parse` boundary is shared across |
| Extend the hand-written scanner further | No new dependency | Extends the exact architecture whose failure mode (one global counter, one undiscovered edge case) is the problem; each additional case is a fix to a symptom, not the cause, and the class recurs — CR-040 already proved this once |
| A hand-rolled recursive-descent parser in `quire-rs` itself | Full control, no external grammar version to track | Reimplements a grammar `rustc`'s own ecosystem already maintains, for three languages if extended to Phase 2, entirely inside this crate — the highest-maintenance option surveyed, and the one most likely to accumulate its own edge-case backlog |

## Decision

**Selected: `tree-sitter`, reached through the shared `quire-code-parse`
crate** (`agent-ix/quire-code-rs`, workspace member added at commit
`57b83ba`), via a new local dependency-boundary crate,
`crates/quire-rust-extraction`.

Rationale:

1. **Per-node recovery is the property this ADR exists for.** tree-sitter
   parses incrementally and tolerates a local error without losing the rest
   of the tree's declaration structure — `has_declaration_structure_error`'s
   documented rule (`quire-code-parse`) is *"an error inside a declaration's
   own executable body does not count"* — which is the structural opposite of
   one global brace-depth counter. A brace living in a raw string, a
   lifetime, a char literal, or a nested block comment was never a real
   desync, and tree-sitter never treats it as one, by construction rather
   than by an enumerated fix list.
2. **The span defects (PLAT-69, PLAT-846) are fixed as a byproduct, not a
   special case.** A tree-sitter node's span already covers a multi-line
   attribute or a wrapped doc comment; walking preceding **sibling nodes**
   rather than preceding **lines** (`leading_span` in `src/symbols/rust.rs`)
   reaches the true start of the leading annotation block regardless of how
   it wraps.
3. **`quire-code-parse` is the modular boundary this decision needs.** It is
   a workspace member with no dependency edge to `quire-code-rs`'s own fact
   model, type environment, fixpoint call resolver, or record emitter — this
   extractor needs a borrowed syntax tree and source text, nothing else, and
   `quire-code-parse`'s own crate docs state that boundary as their reason to
   exist (PLAT-841 finding 5). Depending on `quire-code-rs` directly would
   compile all four of those for a consumer that reads none of them.
4. **`quire-rust-extraction` copies `filament-ide-rs`'s own boundary
   pattern.** `filament-ide-rs` already answered the same question — how does
   a workspace admit tree-sitter through one pin without a second copy
   reappearing — with a thin integration crate plus a `cargo metadata`-based
   test (`crates/filament-code-extraction`,
   `fr_072_ac_5_tree_sitter_is_reachable_only_through_the_quire_code_rs_pin`).
   `crates/quire-rust-extraction/tests/dependency_boundary.rs` is the same
   mechanism, adapted to this workspace's own two-package shape (this repo
   was not previously a Cargo workspace; it is now, purely to hold this one
   boundary crate — `default-members = ["."]` keeps every existing `cargo
   build`/`cargo test` invocation from the repo root building exactly what it
   built before).
5. **Git-rev pin, not a registry version.** `quire-code-rs` is a public repo
   (PLAT-848 resolved), but crates.io publishing is deliberately deferred, so
   the pin is `rev = "57b83ba"` per this org's convention for unpublished
   crates (`NFR-009`). `deny.toml`'s `allow-git` names both `quire-code-rs`
   and the pre-existing `ix-trace-rs` exception, stating both are deletable
   once registry publishing exists.

## Consequences

- `src/symbols/rust.rs`'s hand-written lexer subsystem (`LexedLine`,
  `ScanState`, `lex`, `lex_line`, `opens_char_literal`, `opens_raw_string`,
  `raw_prefix_len`, `closes_raw`, `check_balanced` — 218 of the file's
  previous 838 lines) is deleted outright, as a consequence of parsing
  correctly rather than a deliberate optimisation.
- Two tests that asserted that machinery's own intermediate state
  (`tc804_lexer_counts_only_code_braces`, `tc804_string_state_carries_across_lines`)
  cannot survive the port and are retired, each replaced by a successor that
  asserts the **property** the old test pinned — extraction outcome, not
  lexer mechanism — confirmed to fail against a deliberately broken
  stand-in before being trusted. `tc804_rust_lexing_is_string_and_lifetime_aware`
  needed no edit: it already asserted an outcome.
- **Grammar-version determinism is a newly-relevant claim, and cross-version
  identity is deliberately not one of the claims made.** `FR-051-AC-10`'s
  byte-identical-output guarantee is amended to name the grammar version and
  the `rust-symbols` feature set as part of the identity it claims — *at* a
  pinned grammar version, not *across* grammar versions (`CR-177`, same FR).
  The pin `rev = "57b83ba"` on `quire-code-parse` pins that crate's own
  *source*, not the registry-resolved `tree-sitter`/`tree-sitter-rust`
  versions its own `Cargo.toml` declares as caret ranges — those live only in
  each consumer's own `Cargo.lock`, so this workspace and another consumer of
  the same pin are not guaranteed to resolve the identical grammar patch
  version, and a bare `cargo update` here can move it with no manifest edit
  (spec review F1). `crates/quire-rust-extraction/tests/dependency_boundary.rs`
  asserts this repo's own locked version exactly, so that drift is a compiled
  gate failure here, not a silent variable.
- **The `wasm` build cannot link this dependency, and that fact is not a
  parse error.** `tree-sitter-rust` compiles a C parser via a `cc`-crate
  build script, which cannot cross-compile to `wasm32-unknown-unknown`
  without a wasm-targeting C toolchain this workspace does not carry — the
  same shape `resolve-file` is already gated off `wasm` for, one dependency
  over. The Rust symbol adapter is gated behind a new `rust-symbols` feature
  (on by default, off under `wasm`); `src/symbols/mod.rs` falls back to a
  per-file **build-configuration** diagnostic for `SourceLanguage::Rust`,
  worded so it cannot be mistaken for a parse failure this file's own
  content caused (review F4), rather than failing to compile when the
  feature is off. Python and TypeScript extraction are unaffected either
  way, and no existing consumer of the default feature set observes any
  change.
- A future revision (a `quire-code-parse` grammar bump, or Phase 2 porting
  Python/TypeScript onto the same crate) requires: re-running the
  symbol differential against a freshly measured pre-change baseline, and
  — for a grammar bump specifically — treating it as the explicit, reviewed
  dependency change `CR-177` says it must be, never a silent variable.
