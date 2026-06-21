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
