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

// Public re-exports for the parser surface (FR-005 + FR-006/007/008/009).
pub use ast::{QuireDocument, QuireSection};
pub use parser::{extract_frontmatter, parse_document, FrontmatterResult, Heading};
// Query API (FR-010).
pub use query::{
    extract_diagrams, parse_bullet_list, parse_table, parse_tables, search, section, sections,
    table_from_section, DiagramBlock, ListItem, ListPattern, SearchMatch, SearchResult,
    TableResult,
};
// Error shape (NFR-005).
pub use error::{format_violation, ArchetypeLoadFailure, QuireError, VIOLATION_PREVIEW_MAX};
// Loader + registry (FR-013 + FR-014).
pub use diagnostic::{Diagnostic, DiagnosticKind, Diagnostics};
pub use loader::compile::CompiledArchetype;
pub use registry::Registry;
// Render + validate (FR-001 + FR-002).
pub use render::{render, render_by_name};
pub use validate::{apply_patch, validate, validate_all};
// Extract / body-extraction DSL (FR-011).
pub use extract::dsl::{
    EdgeEmission, EdgeTarget, ExtractionDsl, IterateKind, IterateOver, YieldPattern,
};
pub use extract::locator::{Locator, LocatorPrimitive};
pub use extract::{extract, ExtractionResult, HarvestedEdge};
// Edge harvesting + resolver (FR-015).
pub use edges::{
    harvest_edges, EdgeHarvest, IdentityResolver, MockResolver, RelationshipResolver, SUGAR_FIELDS,
};
