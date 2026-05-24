//! High-performance Rust templating + parsing engine for the Filament/Quire ecosystem.
//!
//! This crate is currently in the skeleton phase. Modules are wired but mostly empty;
//! types and functions are filled in by subsequent tasks (see `plan/plan.md`).
//
// TODO(skeleton): remove `#![allow(dead_code)]` once Task 001+ populate the modules
// with real public APIs. The allow is temporary scaffolding so the empty modules
// don't trip clippy `-D warnings`.
#![allow(dead_code)]

pub mod ast;
pub mod diagnostic;
pub mod edges;
pub mod error;
pub mod extract;
pub mod loader;
pub mod merge;
pub mod parser;
pub mod query;
pub mod registry;
pub mod render;
pub mod validate;

// Public re-exports for Task 001 parser primitives (FR-006/007/008/009).
// `parse_document` (Task 002) will be the higher-level entry point that
// composes these primitives.
pub use parser::{extract_frontmatter, FrontmatterResult, Heading};
