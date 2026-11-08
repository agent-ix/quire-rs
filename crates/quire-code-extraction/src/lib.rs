//! Dependency-boundary crate for the tree-sitter-backed symbol adapters
//! (PLAT-843 for Rust; widened to all three `quire-code-parse` grammars by
//! PLAT-851, the shared enabling slice for the Python/TypeScript ports).
//!
//! `src/symbols/{rust,python,typescript}.rs` (in the `quire-rs` root package)
//! each walk a tree-sitter syntax tree to build
//! [`quire_rs::symbols::RawSymbol`]s. They do that through this crate rather
//! than naming `quire-code-parse` (or `tree-sitter` itself) directly, so this
//! crate is the **one place** in the workspace `cargo tree` shows a
//! tree-sitter dependency outside the `quire-code-parse` pin — enforced by
//! `tests/dependency_boundary.rs`, copied from `filament-ide-rs`'s
//! `crates/filament-code-extraction` (FR-072-AC-5 there).
//!
//! This crate adds no logic of its own: it is a pure re-export. The
//! classification, span, and identity decisions all live in each language's
//! own adapter module — see those files' module docs for what a symbol is
//! and how its qualified name is built. Keeping this crate free of
//! extraction logic means a reviewer never has to look here to understand
//! what a symbol is.
//!
//! `quire_code_parse::tree_sitter` is re-exported transitively (`pub use
//! quire_code_parse::tree_sitter`), the same single-sourcing
//! `quire-code-parse` itself does one layer down — the adapters walk
//! `tree_sitter::Node`/`TreeCursor` from here, never importing the
//! `tree-sitter` crate under its own name.
//!
//! This crate's own `rust`/`python`/`typescript` Cargo features (see
//! `Cargo.toml`) each forward to the matching `quire-code-parse` feature, so
//! a consumer that enables only one still links only that one grammar's
//! tree-sitter C parser — see the `Cargo.toml` dependency comment for why
//! that matters for the `wasm` build.

#![forbid(unsafe_code)]

pub use quire_code_parse::{parse_file, tree_sitter, Diagnostic, Language, ParseError, ParsedFile};
