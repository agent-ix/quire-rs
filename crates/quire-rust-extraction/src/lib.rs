//! Dependency-boundary crate for the Rust symbol adapter (PLAT-843).
//!
//! `src/symbols/rust.rs` (in the `quire-rs` root package) walks a tree-sitter
//! syntax tree to build [`quire_rs::symbols::RawSymbol`]s. It does that
//! through this crate rather than naming `quire-code-parse` (or
//! `tree-sitter` itself) directly, so this crate is the **one place** in the
//! workspace `cargo tree` shows a tree-sitter dependency outside the
//! `quire-code-parse` pin — enforced by `tests/dependency_boundary.rs`,
//! copied from `filament-ide-rs`'s `crates/filament-code-extraction`
//! (FR-072-AC-5 there).
//!
//! This crate adds no logic of its own: it is a pure re-export. The
//! classification, span, and identity decisions all live in `rust.rs` — see
//! that file's own module docs for what a symbol is and how its qualified
//! name is built. Keeping this crate free of extraction logic is what makes
//! "the blast radius is `rust.rs` plus the new dependency" (PLAT-843) true:
//! a reviewer never has to look here to understand what a Rust symbol is.
//!
//! `quire_code_parse::tree_sitter` is re-exported transitively (`pub use
//! quire_code_parse::tree_sitter`), the same single-sourcing
//! `quire-code-parse` itself does one layer down — `rust.rs` walks
//! `tree_sitter::Node`/`TreeCursor` from here, never importing the
//! `tree-sitter` crate under its own name.

#![forbid(unsafe_code)]

pub use quire_code_parse::{parse_file, tree_sitter, Diagnostic, Language, ParseError, ParsedFile};
