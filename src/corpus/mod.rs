//! Bounded, in-memory spec corpus (v0.3).
//!
//! A *spec* is a bounded set of related markdown artifacts (a `spec/`
//! tree: StR, US, FR, NFR, …) whose references point at each other.
//! This module loads such a set, resolves the references *within* it,
//! and answers whole-spec read-only queries — a data structure, not a
//! stateful engine (StR-006 / ADR-0002).
//!
//! Lifecycle: load → resolve → query → drop. No persistence, no
//! filesystem watching, no resolution outside the loaded set.
//!
//! - [`walk`] — `load_repo`: parallel, ignore-aware directory walk +
//!   parse (FR-024).
//! - `spec` — the `Spec` corpus value (FR-025; lands with Task 029).
//! - `resolve` — intra-spec reference resolution (FR-026; Task 030).
//! - `query` — whole-spec query API (FR-027; Task 031).

mod body_cache;
pub mod declared_tables;
pub mod design_matrix;
pub mod field_rules;
pub mod glossary;
pub mod query;
mod relation_edges;
pub mod relation_report;
pub mod required_relations;
pub mod resolve;
pub mod spec;
pub mod trace_refs;
pub mod unlinked;
pub mod validate;
pub mod vocabulary_coverage;
pub mod walk;

pub use design_matrix::{
    design_matrix, DesignBlock, DesignElement, DesignElementRef, DesignMatrixCriterion,
    DesignMatrixEntry,
};
pub use glossary::{glossary_terms, glossary_terms_from_path};
pub use relation_report::{
    design_join, relation_report, Criteria, DanglingEntry, KindMismatchEntry, LinkResolution,
    LinkedEntry, RelationReport, RelationReportError, RelationReportMember, RelationRow,
    RelationSection, RelationSummary, RowDocument, RowSelector, RowState, StateCounts,
    Verification, VerificationCounts,
};
pub use resolve::{harvest_edges, Edge, Resolution};
pub use spec::{CorpusError, CorpusMember, Spec};
pub use unlinked::{unlinked_references, UnlinkedFix, UnlinkedReason, UnlinkedReference};
pub use validate::{
    validate_bundle, validate_bundle_at, validate_corpus, BundleFinding, BundlePosture,
    BundleReport,
};

/// Opaque, stable human artifact id (e.g. `"FR-023"`). Used as the
/// intra-spec resolution key — `ix://` link targets and frontmatter
/// `relationships` reference it. No grammar is imposed by the corpus
/// (the schema's `pattern` governs that, per spec.md §2.2).
pub type ArtifactId = String;
