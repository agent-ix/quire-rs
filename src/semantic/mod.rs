//! Semantic module contract and declaration extraction (US-019, FR-069..FR-072).
//!
//! `contract` reads the module `semantic` block and reference-form
//! `data_schema` at load (FR-069); `resolver` compiles a module schema
//! against the embedded semantic-core bundle without touching the filesystem
//! or the network; `vendored` is the embedded bundle itself. Extraction
//! (FR-070..FR-072) lands in the sibling modules of Plan-003.

pub mod contract;
pub mod resolver;
pub mod vendored;

pub use contract::{
    read_semantic_block, reference_form, DataSchemaRef, SemanticFailure, SemanticModule,
    SemanticSeverity,
};
pub use resolver::{compile_module_schema, ResolvedSchema, SchemaSource};
