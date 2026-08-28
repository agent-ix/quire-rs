//! High-performance Rust parsing + validation engine for the Filament/Quire ecosystem.
//!
//! Scope: parser + query + schema validation + markdown document
//! validation + body-extraction + byte-splice writeback. The render /
//! templating feature has been removed (no backward-compatibility layer);
//! markdown is canonical and authored directly. See `spec/spec.md` §2bis.

// Compile-time guarantee: no first-party `unsafe`. The `python` feature pulls in
// PyO3 macros that expand to `unsafe` in-crate, so the forbid is scoped to the
// non-python build; first-party `src/python` is itself `unsafe`-free (NFR-003).
#![cfg_attr(not(feature = "python"), forbid(unsafe_code))]

pub mod ast;
pub mod combinatorial;
pub mod concept;
pub mod contract;
pub mod corpus;
pub mod coverage;
pub mod diagnostic;
pub mod error;
pub mod extract;
pub mod filament;
pub mod finding;
pub mod grammar;
pub mod lint;
pub mod loader;
pub mod merge;
pub mod metric;
pub mod obligation;
pub mod parser;
#[cfg(feature = "python")]
pub mod python;
pub mod query;
pub mod registry;
pub mod skeptic;
pub mod symbol_table;
pub mod symbols;
pub mod traceability;
pub mod validate;
pub mod validate_document;
pub mod vocab;
pub mod writeback;

// Parser surface (FR-005..009).
pub use ast::{QuireDocument, QuireSection};
pub use parser::{
    extract_frontmatter, parse_body, parse_document, parse_header, FrontmatterResult,
    FrontmatterStatus, Header, Heading,
};
// Query API (FR-010).
pub use query::{
    concept_type, diagrams_from_content, extract_diagrams, parse_bullet_list, parse_table,
    parse_tables, search, section, sections, table_from_section, DiagramBlock, ListItem,
    ListPattern, SearchMatch, SearchResult, TableResult,
};
// Error shape (NFR-005).
pub use error::{format_violation, ArchetypeLoadFailure, QuireError, VIOLATION_PREVIEW_MAX};
// Load-time diagnostics (internal Diagnostic enum, used in error paths).
pub use diagnostic::{Diagnostic, PathTraversalReason};
// Loader + registry (FR-013 + FR-014).
pub use loader::compile::CompiledArchetype;
pub use loader::manifest::DeclarationOrigin;
pub use obligation::{normalize_statement, statement_hash, Obligation};
pub use registry::Registry;
// Schema validation (FR-002).
pub use validate::{apply_patch, validate, validate_all, validate_block};
// Markdown-default validation (FR-032 + FR-035).
pub use validate_document::{
    validate_context, validate_document, validate_document_in_registry,
    validate_document_in_registry_with_lexicon, ValidationError, ValidationReason,
    ValidationResult, ValidationWarning,
};
// Base concept frontmatter contract (OKF: required `type` + optional
// `description`/`tags`), validated before archetype routing.
pub use concept::{base_concept_schema, validate_base_concept, validate_concept_shape};
// Archetype input contract + authoring skeleton (FR-029 recast).
pub use contract::{
    input_contract_for, input_contract_for_object, ContractRelationship, ContractSection,
    InputContract,
};
// Extract / body-extraction DSL (FR-011 + FR-016).
pub use extract::dsl::{ExtractionDsl, IterateKind, IterateOver, YieldPattern};
pub use extract::locator::{Locator, LocatorAssert, LocatorKind, LocatorPrimitive};
pub use extract::{extract, ExtractionResult};
pub use finding::{FindingGuidance, FindingNextMove};
// Canonical Filament core-data extraction (FR-045 / FR-046).
pub use filament::{
    extract_filament_core, CoreExtractionDiagnostic, CoreExtractionResult, CoreGraphEdgeRef,
    CoreGraphNodeRef, CoreObjectTypeRecord, FilamentExtractionInput, FilamentObjectType,
};
// Assert facet (FR-033) + `{field}` interpolation (FR-034).
pub use extract::assert_eval::{evaluate_assert, AssertFailure, AssertReason};
pub use extract::interpolate::{interpolate, UnresolvedField};
// Declarative lint rules (FR-036) — advisory, never blocks extraction.
pub use grammar::{check_document_grammar, GrammarFinding, GrammarLexicon, GrammarSeverity};
// Acceptance-criteria property-shape classification (FR-052) — metadata, never
// a finding: this surface carries no severity and no promotion path.
pub use grammar::property::{
    AcClassification, Extraction, PropertyIdioms, PropertyShape, Span as PropertySpan,
    Spans as PropertySpans,
};
pub use grammar::{classify_document_properties, AcPropertyCounts};
pub use lint::{lint_document, LintFinding, LintRule, LintSeverity};
pub use validate_document::classify_document_criteria;
// Writeback (FR-022) — byte-splice section/block edit.
pub use writeback::{update_block, update_section};
// Declarative coverage computation (FR-050) + source-symbol extraction (FR-051).
pub use coverage::{compute as compute_coverage, CoverageError, CoverageReport};
pub use symbol_table::{build as build_symbol_table, SymbolTableReport};
// Corpus: parallel repo walk (FR-024) + Spec corpus (FR-025); resolution/query in FR-026..027.
pub use corpus::walk::{load_repo, load_repo_with, LoadedDocument, RepoLoad, WalkOptions};
pub use corpus::{glossary_terms, glossary_terms_from_path, harvest_edges, Spec};
// Bundle validation postures (OKF): strict archetype-conformance vs.
// permissive foreign-bundle reading, plus index-completeness.
pub use corpus::{validate_bundle, validate_bundle_at, BundleFinding, BundlePosture, BundleReport};
// Unlinked-reference detection + autofix suggestions (FR-039, ADR 0007).
pub use corpus::{unlinked_references, UnlinkedFix, UnlinkedReason, UnlinkedReference};
