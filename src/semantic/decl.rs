//! Semantic-core declaration shapes as Quire emits them (FR-070/FR-071).
//!
//! Field order here is the canonical key order of the normalized form;
//! every optional key is skipped when absent, so `serde_json` output is the
//! normalized `FieldDecl[]` the quoin golden fixtures record.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Multiplicity {
    pub lower: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upper: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ordered: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique: Option<bool>,
}

impl Multiplicity {
    pub fn one() -> Self {
        Self {
            lower: 1,
            upper: Some(1),
            ..Self::default()
        }
    }

    /// `1..1`: the only shape `identity` may carry.
    pub fn is_single(&self) -> bool {
        self.lower == 1 && self.upper == Some(1)
    }

    /// Upper absent or greater than one: where `ordered`/`unique` apply.
    pub fn is_collection(&self) -> bool {
        self.upper.map_or(true, |u| u > 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecimalPolicy {
    pub precision: u64,
    pub scale: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeRef {
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiplicity: Option<Multiplicity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decimal: Option<DecimalPolicy>,
}

/// One constraint of the closed FR-070 keyword set. `serde` emits the
/// `keyword` discriminator first, matching the semantic-core models.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "keyword")]
pub enum Constraint {
    #[serde(rename = "min")]
    Min { value: Value },
    #[serde(rename = "max")]
    Max { value: Value },
    #[serde(rename = "exclusiveMin")]
    ExclusiveMin { value: Value },
    #[serde(rename = "exclusiveMax")]
    ExclusiveMax { value: Value },
    #[serde(rename = "minLength")]
    MinLength { value: u64 },
    #[serde(rename = "maxLength")]
    MaxLength { value: u64 },
    #[serde(rename = "pattern")]
    Pattern { regex: String, dialect: String },
    #[serde(rename = "enumValues")]
    EnumValues { values: Vec<Value> },
    #[serde(rename = "nonEmpty")]
    NonEmpty,
    #[serde(rename = "unique")]
    Unique,
    #[serde(rename = "format")]
    Format { name: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDecl {
    pub name: String,
    #[serde(rename = "type")]
    pub type_ref: TypeRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<Vec<Constraint>>,
}

/// The nine kernel scalars (filament-core-data#35).
pub const KERNEL_SCALARS: &[&str] = &[
    "UUID",
    "Boolean",
    "Integer",
    "Decimal",
    "String",
    "Timestamp",
    "Duration",
    "Bytes",
    "JsonObject",
];

/// Kernel scalars a `unit` may qualify.
pub const UNIT_TARGETS: &[&str] = &["Integer", "Decimal", "Duration"];

pub fn is_identifier(token: &str) -> bool {
    let mut chars = token.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
