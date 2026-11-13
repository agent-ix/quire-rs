//! Declared carried fields and field-value rules (FR-090, FR-091).
//!
//! Four declarations, all module data, all reporting only:
//!
//! * [`CarriedField`] — a frontmatter value a person states, carried as written
//!   onto the `requirements[]` entry of the coverage matrix (FR-090).
//! * [`FieldConsistency`] — two documents an edge joins must agree on an
//!   ordered value (FR-091).
//! * [`RequiredField`] — documents of a kind must state a value (FR-091).
//! * [`VocabularyRule`] — a stated value must be in a carried field's
//!   vocabulary (FR-091).
//!
//! Nothing here names an archetype, a field, a verb or a value. The engine
//! learns the *shape* of each declaration and reads every name from the module.

use serde::{Deserialize, Serialize};

use super::{check_excludes, check_field, WhereSelector};

/// A frontmatter value carried as written onto a coverage-matrix requirement
/// entry (FR-090).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CarriedField {
    /// The key the value is carried under, matching `^[a-z][a-z0-9_]*$`.
    #[serde(default)]
    pub name: String,
    /// Archetypes whose documents state the value.
    #[serde(
        default,
        deserialize_with = "deserialize_names",
        serialize_with = "serialize_names"
    )]
    pub from: Vec<String>,
    /// The top-level frontmatter key the value is read from.
    #[serde(default)]
    pub key: String,
}

/// Which way a hop reads an edge from the document it starts at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HopDirection {
    /// The far document authors the verb toward the starting document.
    Incoming,
    /// The starting document authors the verb toward the far document.
    Outgoing,
}

impl HopDirection {
    fn parse(entry: &str, key: &str, text: Option<&str>) -> Result<Self, String> {
        match text {
            Some("incoming") => Ok(Self::Incoming),
            Some("outgoing") => Ok(Self::Outgoing),
            Some(other) => Err(format!(
                "traceability: field_consistency entry '{entry}' has {key} '{other}', which is \
                 not `incoming` or `outgoing`"
            )),
            None => Err(format!(
                "traceability: field_consistency entry '{entry}' has no {key}; it must be \
                 `incoming` or `outgoing`"
            )),
        }
    }

    /// The direction as the FR-058 relation reader names it.
    pub(crate) fn as_relation(self) -> super::RelationDirection {
        match self {
            Self::Incoming => super::RelationDirection::Incoming,
            Self::Outgoing => super::RelationDirection::Outgoing,
        }
    }
}

/// The comparison a [`FieldConsistency`] makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsistencyRule {
    /// The `from` value must not rank below the target value.
    NotBelow,
}

/// The intermediate hop between the `from` document and the target (FR-091).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ViaHop {
    /// Archetype of the intermediate document.
    pub archetype: String,
    /// Accepted edge verbs between the `from` document and the intermediate.
    pub edges: Vec<String>,
    /// Which way the hop is read from the `from` document.
    pub direction: HopDirection,
}

/// The enum that ranks a [`FieldConsistency`]'s values (FR-091).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrderFrom {
    /// Archetype whose frontmatter schema holds the `enum`.
    #[serde(default)]
    pub archetype: String,
    /// The field whose `enum` is the rank order.
    #[serde(default)]
    pub field: String,
}

/// Two documents joined by an edge must agree on an ordered value (FR-091).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldConsistency {
    /// Declaration name, reported in diagnostics.
    pub name: String,
    /// Archetypes stating the value.
    pub from: Vec<String>,
    /// Top-level frontmatter key on the `from` document.
    pub field: String,
    /// Optional intermediate hop.
    pub via: Option<ViaHop>,
    /// Accepted edge verbs from the `via` document, or from the `from`
    /// document when there is no `via`.
    pub edges: Vec<String>,
    /// Which way the `edges` hop is read.
    pub direction: HopDirection,
    /// Archetypes of the target document.
    pub to: Vec<String>,
    /// Top-level frontmatter key on the target document.
    pub to_field: String,
    /// Where the rank order is read.
    pub order_from: OrderFrom,
    /// The comparison.
    pub rule: ConsistencyRule,
    /// The `<check>` half of the `trace:<check>` severity key.
    pub check: String,
}

/// Documents of a kind must state a value (FR-091).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RequiredField {
    /// Declaration name, reported in diagnostics.
    pub name: String,
    /// Archetypes whose documents are obliged.
    pub from: Vec<String>,
    /// Top-level frontmatter key the document must state.
    pub field: String,
    /// Scope-relative globs exempt from the obligation.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    /// Optional FR-058 `where` selector narrowing the obliged documents.
    #[serde(rename = "where", skip_serializing_if = "Option::is_none")]
    pub selector: Option<WhereSelector>,
    /// The `<check>` half of the `trace:<check>` severity key.
    pub check: String,
}

/// A stated value must be in a carried field's vocabulary (FR-091).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VocabularyRule {
    /// Declaration name, reported in diagnostics.
    #[serde(default)]
    pub name: String,
    /// Archetypes whose documents are held to the vocabulary.
    #[serde(
        default,
        deserialize_with = "deserialize_names",
        serialize_with = "serialize_names"
    )]
    pub from: Vec<String>,
    /// Top-level frontmatter key holding the value.
    #[serde(default)]
    pub field: String,
    /// The `carried_fields` name whose vocabulary applies.
    #[serde(default)]
    pub carried: String,
    /// The `<check>` half of the `trace:<check>` severity key.
    #[serde(default)]
    pub check: String,
}

fn deserialize_names<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    super::deserialize_scalar_or_sequence(d, "an archetype name, or a sequence of them")
}

fn serialize_names<S: serde::Serializer>(names: &[String], s: S) -> Result<S::Ok, S::Error> {
    s.collect_seq(names)
}

// ── authored shapes ────────────────────────────────────────────────────────
//
// The `Raw*` types exist so a bad `direction`, `rule` or `where` is rejected
// naming the entry, and so a missing mandatory key is a load failure that says
// which key rather than a bare serde "missing field".

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawVia {
    #[serde(default)]
    archetype: String,
    #[serde(default)]
    edges: Vec<String>,
    #[serde(default)]
    direction: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawFieldConsistency {
    #[serde(default)]
    name: String,
    #[serde(default, deserialize_with = "deserialize_names")]
    from: Vec<String>,
    #[serde(default)]
    field: String,
    #[serde(default)]
    via: Option<RawVia>,
    #[serde(default)]
    edges: Vec<String>,
    #[serde(default)]
    direction: Option<String>,
    #[serde(default, deserialize_with = "deserialize_names")]
    to: Vec<String>,
    #[serde(default)]
    to_field: String,
    #[serde(default)]
    order_from: Option<OrderFrom>,
    #[serde(default)]
    rule: Option<String>,
    #[serde(default)]
    check: String,
}

impl TryFrom<RawFieldConsistency> for FieldConsistency {
    type Error = String;

    fn try_from(raw: RawFieldConsistency) -> Result<Self, String> {
        let entry = raw.name.as_str();
        let rule = match raw.rule.as_deref() {
            Some("not_below") => ConsistencyRule::NotBelow,
            Some(other) => {
                return Err(format!(
                    "traceability: field_consistency entry '{entry}' has rule '{other}', which \
                     is not `not_below`"
                ))
            }
            None => {
                return Err(format!(
                    "traceability: field_consistency entry '{entry}' has no `rule`"
                ))
            }
        };
        let direction = HopDirection::parse(entry, "`direction`", raw.direction.as_deref())?;
        let via = raw
            .via
            .map(|via| {
                Ok::<_, String>(ViaHop {
                    direction: HopDirection::parse(
                        entry,
                        "`via` `direction`",
                        via.direction.as_deref(),
                    )?,
                    archetype: via.archetype,
                    edges: via.edges,
                })
            })
            .transpose()?;
        let order_from = raw.order_from.ok_or_else(|| {
            format!("traceability: field_consistency entry '{entry}' has no `order_from`")
        })?;
        Ok(Self {
            name: raw.name,
            from: raw.from,
            field: raw.field,
            via,
            edges: raw.edges,
            direction,
            to: raw.to,
            to_field: raw.to_field,
            order_from,
            rule,
            check: raw.check,
        })
    }
}

impl<'de> Deserialize<'de> for FieldConsistency {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        RawFieldConsistency::deserialize(d)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRequiredField {
    #[serde(default)]
    name: String,
    #[serde(default, deserialize_with = "deserialize_names")]
    from: Vec<String>,
    #[serde(default)]
    field: String,
    #[serde(default)]
    exclude: Vec<String>,
    /// `where: null` stays `Some(Null)` so it is rejected, not read as absent.
    #[serde(
        default,
        rename = "where",
        deserialize_with = "super::deserialize_present"
    )]
    selector: Option<serde_json::Value>,
    #[serde(default)]
    check: String,
}

impl<'de> Deserialize<'de> for RequiredField {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = RawRequiredField::deserialize(d)?;
        let selector = raw
            .selector
            .as_ref()
            .map(|value| WhereSelector::parse("required_fields", &raw.name, value))
            .transpose()
            .map_err(serde::de::Error::custom)?;
        Ok(Self {
            name: raw.name,
            from: raw.from,
            field: raw.field,
            exclude: raw.exclude,
            selector,
            check: raw.check,
        })
    }
}

// ── structural validation ──────────────────────────────────────────────────

/// A `carried_fields` name: `^[a-z][a-z0-9_]*$`.
fn is_carried_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// The label of an entry in a failure: its name, or its position when blank.
fn label(name: &str, index: usize) -> String {
    if name.trim().is_empty() {
        format!("#{}", index + 1)
    } else {
        name.to_string()
    }
}

fn check_names(section: &str, entry: &str, key: &str, names: &[String]) -> Result<(), String> {
    if names.is_empty() {
        return Err(format!(
            "traceability: {section} entry '{entry}' has an empty `{key}`"
        ));
    }
    names
        .iter()
        .try_for_each(|name| check_field(section, entry, key, name))
}

fn check_token(section: &str, entry: &str, check: &str) -> Result<(), String> {
    check_field(section, entry, "check", check)?;
    if !crate::grammar::is_severity_key(&crate::grammar::severity_key("trace", check)) {
        return Err(format!(
            "traceability: {section} entry '{entry}' has a `check` token '{check}' that cannot \
             form a `trace:<check>` severity key, so its severity could never be configured"
        ));
    }
    Ok(())
}

/// A blank name, or one already seen in the same section.
fn check_entry_name<'a>(
    section: &str,
    name: &'a str,
    index: usize,
    seen: &mut std::collections::BTreeSet<&'a str>,
) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err(format!(
            "traceability: {section} entry {} has an empty name",
            label(name, index)
        ));
    }
    if !seen.insert(name) {
        return Err(format!("traceability: duplicate {section} entry '{name}'"));
    }
    Ok(())
}

impl super::TraceabilityModel {
    /// Shape-validate the four FR-090/FR-091 declarations (AC-7, AC-10).
    ///
    /// Whether an archetype, field or carried name exists waits for the merged
    /// registry: the declaring module may name another module's archetype.
    pub(super) fn validate_field_rules(&self) -> Result<(), String> {
        for (index, carried) in self.carried_fields.iter().enumerate() {
            let entry = label(&carried.name, index);
            if !is_carried_name(&carried.name) {
                return Err(format!(
                    "traceability: carried_fields entry '{entry}' has a name that does not match \
                     `^[a-z][a-z0-9_]*$`"
                ));
            }
            check_names("carried_fields", &entry, "from", &carried.from)?;
            check_field("carried_fields", &entry, "key", &carried.key)?;
        }

        let mut names = std::collections::BTreeSet::new();
        for (index, rule) in self.field_consistency.iter().enumerate() {
            check_entry_name("field_consistency", &rule.name, index, &mut names)?;
            let entry = rule.name.as_str();
            let section = "field_consistency";
            check_names(section, entry, "from", &rule.from)?;
            check_field(section, entry, "field", &rule.field)?;
            if let Some(via) = &rule.via {
                check_field(section, entry, "via` `archetype", &via.archetype)?;
                check_names(section, entry, "via` `edges", &via.edges)?;
            }
            check_names(section, entry, "edges", &rule.edges)?;
            check_names(section, entry, "to", &rule.to)?;
            check_field(section, entry, "to_field", &rule.to_field)?;
            check_field(
                section,
                entry,
                "order_from` `archetype",
                &rule.order_from.archetype,
            )?;
            check_field(section, entry, "order_from` `field", &rule.order_from.field)?;
            check_token(section, entry, &rule.check)?;
        }

        let mut names = std::collections::BTreeSet::new();
        for (index, rule) in self.required_fields.iter().enumerate() {
            check_entry_name("required_fields", &rule.name, index, &mut names)?;
            let (section, entry) = ("required_fields", rule.name.as_str());
            check_names(section, entry, "from", &rule.from)?;
            check_field(section, entry, "field", &rule.field)?;
            check_token(section, entry, &rule.check)?;
            check_excludes(
                &format!("required_fields entry '{entry}'"),
                "exclude",
                &rule.exclude,
            )?;
        }

        let mut names = std::collections::BTreeSet::new();
        for (index, rule) in self.vocabulary.iter().enumerate() {
            check_entry_name("vocabulary", &rule.name, index, &mut names)?;
            let (section, entry) = ("vocabulary", rule.name.as_str());
            check_names(section, entry, "from", &rule.from)?;
            check_field(section, entry, "field", &rule.field)?;
            check_field(section, entry, "carried", &rule.carried)?;
            check_token(section, entry, &rule.check)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::is_carried_name;

    #[test]
    fn carried_names_match_the_declared_pattern() {
        assert!(is_carried_name("software_safety_class"));
        assert!(is_carried_name("a1"));
        for bad in ["", "Class", "1a", "a-b", "a b", "_a"] {
            assert!(!is_carried_name(bad), "{bad:?}");
        }
    }
}
