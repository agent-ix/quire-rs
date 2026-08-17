//! Edge-type + role vocabulary (FR-040).
//!
//! Two mergeable manifest registries give the **object axis** a typed,
//! categorized, self-documenting relationship vocabulary:
//!
//! - `edge_types:` — the controlled verb vocabulary. Each verb carries a
//!   [`EdgeCategory`] and a human description, plus an optional `inverse`
//!   verb (for reverse-edge query labelling; it need not itself be
//!   declared). This is what turns the previously free-string edge `type`
//!   into a controlled vocabulary.
//! - `roles:` — capability tags an object type opts into via its
//!   `roles:` list. Cross-domain `allowed_links` target a *role* instead
//!   of hardcoding another module's concrete type name, so the graph is
//!   exhaustive-by-construction as new object types opt into roles.
//!
//! Per-archetype `allowed_links` is a map of verb → allowed target
//! tokens, where a token is a concrete object-type name, a role name, or
//! `"*"`. The legacy flat-array authoring form normalizes to
//! `{verb: ["*"]}` (every verb allowed against any target) — see
//! [`normalize_allowed_links`]. Iteration order is observable
//! (diagnostics, skeleton), so the map is a `BTreeMap` (NFR-006).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Per-archetype `allowed_links`: verb → allowed target tokens.
///
/// A target token is a concrete object-type name, a role name, or `"*"`.
/// `BTreeMap`/`Vec` keep iteration deterministic (NFR-006).
pub type AllowedLinks = BTreeMap<String, Vec<String>>;

/// The semantic family of an edge verb. Encoded as an enum (not a free
/// string) so the manifest contract is closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeCategory {
    /// Composition / containment (`contains`, `aggregates`).
    Structural,
    /// Control flow (`calls`, `triggers`, `transitions_to`).
    Behavioral,
    /// Data movement (`reads`, `writes`, `publishes`, `consumes`).
    Dataflow,
    /// Requirement of another artifact (`depends_on`, `requires`,
    /// `implements`).
    Dependency,
    /// A concrete layer realizing a more abstract one (`represents`,
    /// `exposes`, `realizes`).
    Realization,
    /// Cross-cutting control (`protects`, `governs`, `classifies`,
    /// `measures`).
    Governance,
    /// Spec-meta traceability (`references`, `specifies`, `satisfied_by`,
    /// `verifies`).
    Traceability,
}

/// One `edge_types` registry entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeTypeDef {
    pub description: String,
    pub category: EdgeCategory,
    /// Optional inverse verb (e.g. `contains` ↔ `part_of`). Need not be
    /// declared in the registry itself — a forward-only verb is valid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inverse: Option<String>,
}

/// One `roles` registry entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleDef {
    pub description: String,
}

/// One `lexicon` registry entry (FR-043). A module-declared **concrete term**
/// the EARS object-aware vague-response check (FR-042) accepts as a verifiable
/// object. The engine uses the term (the map key); `definition` is documentation
/// (and feeds a future glossary surface). `category` is an optional grouping tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LexiconTermDef {
    pub definition: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// One `observable_verbs` registry entry (FR-047). A module-declared
/// **observable-result verb** the `ac` grammar reads as evidence of an
/// externally checkable outcome: it satisfies the `unclassifiable` predicate
/// and suppresses `vacuous-outcome`. (Before CR-014 this vocabulary was the
/// membership test of a retired `no-observable-outcome` check; CR-014 demoted
/// it to a signal source.) The engine uses the verb (the map key) and its
/// inflections; `definition` is documentation.
/// Mirrors [`LexiconTermDef`] — concrete vocabulary is module data (ADR 0009).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservableVerbDef {
    pub definition: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// One `vacuous_predicates` registry entry (FR-047, CR-014). A module-declared
/// predicate the `ac` grammar's `vacuous-outcome` check treats as asserting
/// nothing checkable. The engine uses the predicate (the map key) and its
/// inflections; `definition` is documentation. Mirrors [`ObservableVerbDef`] —
/// the two registries are the closed and open halves of the same judgement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VacuousPredicateDef {
    pub definition: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// One `property_idioms` registry entry (FR-052). A module-declared **idiom
/// phrase** the property classifier reads as evidence of a property shape: the
/// phrase is the map key, `shape` the [`PropertyShape`](crate::grammar::property::PropertyShape)
/// it indicates, `definition` documentation.
///
/// The registry is a **booster, never a prerequisite** (FR-052-CON-4): the
/// closed structural signals carry extraction coverage, and a declared phrase
/// only refines which shape a criterion is labelled with. A missed idiom
/// therefore degrades a label; it can never remove a criterion from extraction,
/// which is what keeps this open phrase space from repeating the
/// `no-observable-outcome` failure (CR-014).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyIdiomDef {
    pub definition: String,
    pub shape: crate::grammar::property::PropertyShape,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// One `verification_catalog` registry entry (FR-054). A module-declared
/// **method by which a requirement can be verified**, keyed by method id.
///
/// `class` is a **free string**, not a closed IADT enum, and that is the point:
/// this ecosystem classifies by ISO 29148's `Inspection | Analysis |
/// Demonstration | Test`, and an external user classifying by 29119-4 technique
/// family or by assurance level must be able to. An engine-side enum would make
/// the catalog agent-ix's rather than theirs (CON-1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerificationMethodDef {
    /// Human-readable method name.
    pub name: String,
    /// Classification axis — IADT here, whatever the declaring module uses
    /// elsewhere. Never interpreted by the engine.
    pub class: String,
    /// What the method does, in the declaring module's words.
    pub definition: String,
    /// The kind of artifact discharging this method produces (a test run, a
    /// coverage report, a signed inspection). Consumed by the evidence store;
    /// opaque here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_kind: Option<String>,
    /// Rule name → values that trigger advising this method. **Stored and
    /// surfaced, never interpreted** (CON-2): deciding which requirement a rule
    /// matches is the advisor's judgement, and an engine that understood
    /// `property_shapes` would owe an understanding of every axis a future
    /// module invents.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub applicability: BTreeMap<String, Vec<String>>,
    /// Example tooling, as documentation. The engine names no tool.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tooling: Vec<String>,
}

impl VerificationMethodDef {
    /// Reject an entry that cannot say what it is. Returns the offending field
    /// name; a catalog entry with an empty name, class or definition advises
    /// nothing and would sit in the registry looking like data.
    pub fn empty_field(&self) -> Option<&'static str> {
        if self.name.trim().is_empty() {
            Some("name")
        } else if self.class.trim().is_empty() {
            Some("class")
        } else if self.definition.trim().is_empty() {
            Some("definition")
        } else {
            None
        }
    }
}

/// Normalize a raw `allowed_links` value (array **or** map) into the
/// canonical [`AllowedLinks`] map (FR-040-AC-4).
///
/// - array `["calls", "publishes"]` → `{calls: ["*"], publishes: ["*"]}`
///   (the artifact-axis authoring form: each verb allowed against any
///   target).
/// - map `{contains: ["value_object"], references: ["*"]}` → itself
///   (the object-axis form, target lists preserved).
///
/// Anything else (or absent) yields an empty map.
pub fn normalize_allowed_links(value: Option<&Value>) -> AllowedLinks {
    match value {
        Some(Value::Array(verbs)) => verbs
            .iter()
            .filter_map(|v| v.as_str())
            .map(|verb| (verb.to_string(), vec!["*".to_string()]))
            .collect(),
        Some(Value::Object(map)) => map
            .iter()
            .map(|(verb, targets)| {
                let targets = targets
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|t| t.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default();
                (verb.clone(), targets)
            })
            .collect(),
        _ => AllowedLinks::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn array_form_normalizes_to_star_targets() {
        let v = json!(["calls", "publishes"]);
        let got = normalize_allowed_links(Some(&v));
        assert_eq!(got.get("calls"), Some(&vec!["*".to_string()]));
        assert_eq!(got.get("publishes"), Some(&vec!["*".to_string()]));
    }

    #[test]
    fn map_form_preserves_target_lists() {
        let v = json!({"contains": ["value_object", "entity"], "references": ["*"]});
        let got = normalize_allowed_links(Some(&v));
        assert_eq!(
            got.get("contains"),
            Some(&vec!["value_object".to_string(), "entity".to_string()])
        );
        assert_eq!(got.get("references"), Some(&vec!["*".to_string()]));
    }

    #[test]
    fn absent_or_scalar_yields_empty() {
        assert!(normalize_allowed_links(None).is_empty());
        assert!(normalize_allowed_links(Some(&json!("nonsense"))).is_empty());
    }

    #[test]
    fn edge_category_deserializes_lowercase() {
        let d: EdgeTypeDef =
            serde_json::from_value(json!({"description": "x", "category": "realization"})).unwrap();
        assert_eq!(d.category, EdgeCategory::Realization);
        assert!(d.inverse.is_none());
    }
}
