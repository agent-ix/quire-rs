//! High-performance Rust templating + parsing engine for the Filament/Quire ecosystem.
//!
//! v0.2 scope: parser + query + per-block schema validation + render +
//! writeback. The block-addressable artifact model from `INPUT.md` is
//! restored — see `spec/spec.md` § 2bis Drift Audit. Markdown is
//! canonical.

pub mod ast;
pub mod block_edit;
pub mod corpus;
pub mod diagnostic;
pub mod error;
pub mod extract;
pub mod loader;
pub mod merge;
pub mod parser;
#[cfg(feature = "python")]
pub mod python;
pub mod query;
pub mod registry;
pub mod render;
pub mod validate;
pub mod writeback;

// Parser surface (FR-005..009).
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
// Load-time diagnostics (internal Diagnostic enum, used in error paths).
pub use diagnostic::Diagnostic;
// Loader + registry (FR-013 + FR-014).
pub use loader::compile::CompiledArchetype;
pub use registry::Registry;
// Render + validate (FR-001 + FR-002).
pub use render::{render, render_block, render_by_name, render_with_env, RenderOutput};
pub use validate::{apply_patch, validate, validate_all, validate_block};
// Extract / body-extraction DSL (FR-011 + FR-016).
pub use extract::dsl::{ExtractionDsl, IterateKind, IterateOver, YieldPattern};
pub use extract::locator::{Locator, LocatorPrimitive};
pub use extract::{extract, ExtractionResult};
// Writeback (FR-022).
pub use writeback::{update_block, update_section};
// Block edit API (FR-021).
pub use block_edit::{apply_block_patch, replace_block};
// Corpus: parallel repo walk (FR-024) + Spec corpus (FR-025); resolution/query in FR-026..027.
pub use corpus::walk::{load_repo, load_repo_with, LoadedDocument, RepoLoad, WalkOptions};
pub use corpus::Spec;
