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
    /// Required by the embedded semantic-core's `Multiplicity` schema (previously
    /// optional): every producer clamps this to `false` on a non-collection
    /// (`upper` at most one) and otherwise emits it explicitly.
    #[serde(default)]
    pub ordered: bool,
    /// Required by the embedded semantic-core's `Multiplicity` schema (previously
    /// optional): every producer clamps this to `false` on a non-collection
    /// (`upper` at most one) and otherwise emits it explicitly.
    #[serde(default)]
    pub unique: bool,
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
        self.upper.is_none_or(|u| u > 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DecimalPolicy {
    pub precision: u64,
    pub scale: u64,
}

/// The two authoritative semantic-core type-target branches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum TypeRefTarget {
    SemanticId(String),
    KernelScalar(String),
}

impl TypeRefTarget {
    /// The identity or scalar spelling selected by this target.
    pub fn as_str(&self) -> &str {
        match self {
            Self::SemanticId(value) | Self::KernelScalar(value) => value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeRef {
    pub target: TypeRefTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiplicity: Option<Multiplicity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decimal: Option<DecimalPolicy>,
}

/// A bound's exact JSON number or authored string, tagged by its owner branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "lowercase",
    deny_unknown_fields
)]
pub enum NumericConstraintValue {
    Number(serde_json::Number),
    String(String),
}

impl NumericConstraintValue {
    /// The primitive value consumed by semantic-core's lossless lowering contract.
    pub fn to_json_value(&self) -> Value {
        match self {
            Self::Number(value) => Value::Number(value.clone()),
            Self::String(value) => Value::String(value.clone()),
        }
    }
}

/// An enum constraint member with its authoritative scalar branch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "lowercase",
    deny_unknown_fields
)]
pub enum EnumConstraintValue {
    String(String),
    Number(serde_json::Number),
    Boolean(bool),
}

impl EnumConstraintValue {
    /// The primitive value consumed by semantic-core's lossless lowering contract.
    pub fn to_json_value(&self) -> Value {
        match self {
            Self::String(value) => Value::String(value.clone()),
            Self::Number(value) => Value::Number(value.clone()),
            Self::Boolean(value) => Value::Bool(*value),
        }
    }
}

impl From<NumericConstraintValue> for EnumConstraintValue {
    fn from(value: NumericConstraintValue) -> Self {
        match value {
            NumericConstraintValue::Number(value) => Self::Number(value),
            NumericConstraintValue::String(value) => Self::String(value),
        }
    }
}

/// The closed semantic-core constraint keyword carried inside its selected record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConstraintKeyword {
    Min,
    Max,
    ExclusiveMin,
    ExclusiveMax,
    MinLength,
    MaxLength,
    Pattern,
    EnumValues,
    NonEmpty,
    Unique,
    Format,
}

/// The semantic-core adjacent union; each selected record retains its own keyword.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum Constraint {
    #[serde(rename = "MinConstraint")]
    Min {
        keyword: ConstraintKeyword,
        value: NumericConstraintValue,
    },
    #[serde(rename = "MaxConstraint")]
    Max {
        keyword: ConstraintKeyword,
        value: NumericConstraintValue,
    },
    #[serde(rename = "ExclusiveMinConstraint")]
    ExclusiveMin {
        keyword: ConstraintKeyword,
        value: NumericConstraintValue,
    },
    #[serde(rename = "ExclusiveMaxConstraint")]
    ExclusiveMax {
        keyword: ConstraintKeyword,
        value: NumericConstraintValue,
    },
    #[serde(rename = "MinLengthConstraint")]
    MinLength {
        keyword: ConstraintKeyword,
        value: u64,
    },
    #[serde(rename = "MaxLengthConstraint")]
    MaxLength {
        keyword: ConstraintKeyword,
        value: u64,
    },
    #[serde(rename = "PatternConstraint")]
    Pattern {
        keyword: ConstraintKeyword,
        regex: String,
        dialect: String,
    },
    #[serde(rename = "EnumValuesConstraint")]
    EnumValues {
        keyword: ConstraintKeyword,
        values: Vec<EnumConstraintValue>,
    },
    #[serde(rename = "NonEmptyConstraint")]
    NonEmpty { keyword: ConstraintKeyword },
    #[serde(rename = "UniqueConstraint")]
    Unique { keyword: ConstraintKeyword },
    #[serde(rename = "FormatConstraint")]
    Format {
        keyword: ConstraintKeyword,
        name: String,
    },
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
