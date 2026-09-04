//! Semantic module contract and declaration extraction (US-019, FR-069..FR-072).
//!
//! `contract` reads the module `semantic` block and reference-form
//! `data_schema` at load (FR-069); `resolver` compiles a module schema
//! against the embedded semantic-core bundle without touching the filesystem
//! or the network; `vendored` is the embedded bundle itself. Extraction
//! (FR-070..FR-072) lands in the sibling modules of Plan-003.

pub mod context;
pub mod contract;
pub mod decl;
pub mod properties;
pub mod resolver;
pub mod scan;
pub mod vendored;

pub use context::{BundleEntry, BundleIndex, SemanticContext};
pub use contract::{
    read_semantic_block, reference_form, DataSchemaRef, SemanticFailure, SemanticModule,
    SemanticSeverity,
};
pub use decl::{Constraint, DecimalPolicy, FieldDecl, Multiplicity, TypeRef};
pub use properties::{extract_fields, FieldsForm, FieldsOutcome};
pub use resolver::{compile_module_schema, ResolvedSchema, SchemaSource};

use serde::{Deserialize, Serialize};

/// Availability of one declaration kind (FR-072 Behavior).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityState {
    Available,
    NotApplicable,
    Missing,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KindAvailability {
    pub state: AvailabilityState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// True when the kind carries an entry the engine did not interpret
    /// (an unresolved placeholder, an unchecked clause language, opaque
    /// brace text) or the module declares `compatibility_posture:
    /// declared-lossy`.
    pub lossy: bool,
}

impl KindAvailability {
    pub fn available(lossy: bool) -> Self {
        Self {
            state: AvailabilityState::Available,
            reason: None,
            lossy,
        }
    }
    pub fn not_applicable() -> Self {
        Self {
            state: AvailabilityState::NotApplicable,
            reason: None,
            lossy: false,
        }
    }
    pub fn missing(reason: impl Into<String>) -> Self {
        Self {
            state: AvailabilityState::Missing,
            reason: Some(reason.into()),
            lossy: false,
        }
    }
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            state: AvailabilityState::Unavailable,
            reason: Some(reason.into()),
            lossy: false,
        }
    }
}

/// One extraction diagnostic with its locus (FR-070/FR-071 Outputs).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticDiagnostic {
    pub code: String,
    pub severity: SemanticSeverity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    /// Machine-readable sub-reason (`unknown-token`, `no-bundle-index`, …).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl SemanticDiagnostic {
    pub fn new(
        code: &str,
        severity: SemanticSeverity,
        line: usize,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.to_string(),
            severity,
            message: message.into(),
            line: Some(line),
            column: Some(1),
            reason: None,
        }
    }
    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }
    pub fn is_error(&self) -> bool {
        self.severity == SemanticSeverity::Error
    }
}
