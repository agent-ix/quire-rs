//! Carried fields and field-value rules (FR-090, FR-091).
//!
//! Four declarations, all module data. This file holds no archetype, field,
//! verb, rank or value: each is read from the declaring module's manifest, and
//! the rank order of a value is the position in an `enum` of a frontmatter
//! schema (as FR-059 reads a vocabulary). It reports; it never assigns, fills
//! in, suggests or writes a value.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::declared_tables::ExcludeSet;
use super::relation_edges::RelationContext;
use super::spec::Spec;
use super::validate::{pack, BundleFinding, BundleReport};
use super::vocabulary_coverage::{declares_property, enum_at};
use super::walk::LoadedDocument;
use crate::grammar::GrammarSeverity;
use crate::registry::Registry;
use crate::traceability::{FieldConsistency, RequiredField, TraceabilityModel, VocabularyRule};

/// The `trace:` check token of a stated value outside its ranking `enum`.
const UNDECLARED_VALUE: &str = "undeclared-field-value";
/// The `trace:` check token of a declaration that names something nothing has.
const UNDECLARED_VOCABULARY: &str = "undeclared-relation-vocabulary";
/// The `trace:` check token of carried-field `enum`s that disagree.
const VOCABULARY_MISMATCH: &str = "carried-vocabulary-mismatch";

/// The ordered permitted values of each carried field (FR-090), and the carried
/// fields whose archetypes disagree about them.
#[derive(Debug, Default)]
pub(crate) struct CarriedVocabulary {
    /// Carried field name to the `enum` of its key, in schema order.
    pub(crate) values: BTreeMap<String, Vec<String>>,
    /// One message per carried field whose `enum`s differ.
    mismatches: Vec<String>,
}

/// Read the vocabulary of every carried field from the merged registry.
///
/// The `from` archetypes are visited in name order, so the result and the
/// archetypes a mismatch names do not depend on module load order
/// (FR-090-AC-13). An archetype with no `enum` for the key contributes nothing.
pub(crate) fn carried_vocabulary(registry: &Registry) -> CarriedVocabulary {
    let mut out = CarriedVocabulary::default();
    let Some(model) = registry.traceability() else {
        return out;
    };
    for carried in &model.carried_fields {
        let from: BTreeSet<&String> = carried.from.iter().collect();
        let declared: Vec<(&str, &str, Vec<String>)> = from
            .into_iter()
            .filter_map(|name| {
                let archetype = registry.archetype(name)?;
                let schema = archetype.frontmatter_schema.as_deref()?;
                let values = enum_at(schema, &carried.key).filter(|v| !v.is_empty())?;
                Some((archetype.name.as_str(), archetype.module.as_str(), values))
            })
            .collect();
        let Some((first_archetype, first_module, first)) = declared.first() else {
            continue;
        };
        match declared.iter().find(|(_, _, values)| values != first) {
            None => {
                out.values.insert(carried.name.clone(), first.clone());
            }
            Some((archetype, module, _)) => out.mismatches.push(format!(
                "carried_fields entry '{}' reads `{}` from archetype '{first_archetype}' \
                 (module '{first_module}') and archetype '{archetype}' (module '{module}'), whose \
                 enums differ in content or order, so no carried vocabulary is reported for it",
                carried.name, carried.key
            )),
        }
    }
    out
}

/// One finding before routing. The field order is the report order
/// (FR-091): `check`, `from` document path, lowest `via` id, target id, all
/// lexical, so it does not depend on the order documents were read.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Found {
    check: String,
    path: PathBuf,
    via: String,
    target: String,
    message: String,
}

/// Everything one run reads.
struct Run<'a> {
    spec: &'a Spec,
    registry: &'a Registry,
    root: &'a Path,
    context: &'a RelationContext,
    exclude: ExcludeSet,
    carried: CarriedVocabulary,
    found: Vec<Found>,
}

/// Implements: FR-090, FR-091
/// Check the declared carried fields, field consistency, required fields and
/// vocabulary rules.
///
/// A no-op when the module declares none of the four (FR-091-AC-6), so a module
/// that has not adopted them sees byte-identical output. Every finding is
/// advisory unless the FR-057 registry says otherwise.
pub(crate) fn validate_field_rules(
    spec: &Spec,
    registry: &Registry,
    root: &Path,
    context: &RelationContext,
    report: &mut BundleReport,
) {
    let Some(model) = registry.traceability() else {
        return;
    };
    if model.carried_fields.is_empty()
        && model.field_consistency.is_empty()
        && model.required_fields.is_empty()
        && model.vocabulary.is_empty()
    {
        return;
    }
    let mut run = Run {
        spec,
        registry,
        root,
        context,
        exclude: ExcludeSet::compile_validated(&model.exclude),
        carried: carried_vocabulary(registry),
        found: Vec::new(),
    };

    for message in std::mem::take(&mut run.carried.mismatches) {
        run.module_level(VOCABULARY_MISMATCH, message);
    }
    run.check_carried_fields(model);
    for rule in &model.field_consistency {
        run.check_consistency_is_live(rule);
        run.consistency(rule);
    }
    for rule in &model.required_fields {
        run.check_required_is_live(rule);
        run.required(rule);
    }
    for rule in &model.vocabulary {
        run.check_vocabulary_is_live(model, rule);
        run.vocabulary(rule);
    }

    run.found.sort();
    run.found.dedup();
    let severity = registry.grammar_severity();
    for found in run.found {
        report.route(
            severity,
            GrammarSeverity::Warning,
            BundleFinding::in_pack(pack::TRACE, found.check, found.path, found.message),
        );
    }
}

/// A frontmatter value a document states: present, and not null, blank or an
/// empty collection.
fn stated<'a>(doc: &'a LoadedDocument, key: &str) -> Option<&'a Value> {
    let value = doc.frontmatter()?.get(key)?;
    let empty = match value {
        Value::Null => true,
        Value::String(text) => text.trim().is_empty(),
        Value::Array(items) => items.is_empty(),
        Value::Object(map) => map.is_empty(),
        _ => false,
    };
    (!empty).then_some(value)
}

/// A stated value as it was written, for a finding.
fn written(value: &Value) -> String {
    value
        .as_str()
        .map_or_else(|| value.to_string(), str::to_string)
}

/// A stated value against an ordered `enum`.
enum Standing {
    /// States nothing.
    Absent,
    /// The position in the `enum`.
    Rank(usize),
    /// A value outside the `enum`, as written.
    Outside(String),
}

fn standing(doc: &LoadedDocument, key: &str, order: &[String]) -> Standing {
    match stated(doc, key) {
        None => Standing::Absent,
        Some(value) => match value
            .as_str()
            .and_then(|text| order.iter().position(|member| member == text))
        {
            Some(rank) => Standing::Rank(rank),
            None => Standing::Outside(written(value)),
        },
    }
}

/// The distinct names, sorted.
fn sorted(names: &[String]) -> Vec<&str> {
    names
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

impl Run<'_> {
    fn module_level(&mut self, check: &str, message: String) {
        self.found.push(Found {
            check: check.to_string(),
            path: self.root.to_path_buf(),
            via: String::new(),
            target: String::new(),
            message,
        });
    }

    /// A document's id, qualified by its repository when it lives in another
    /// member than `relative_to` (FR-085).
    fn named(&self, slot: usize, relative_to: usize) -> String {
        let id = &self.spec.doc(slot).id;
        match self.spec.member_identity(self.spec.member_of(slot)) {
            Some(repo) if self.spec.member_of(slot) != self.spec.member_of(relative_to) => {
                format!("{repo}:{id}")
            }
            _ => id.clone(),
        }
    }

    /// Slots of the documents of `archetypes` this view sees, minus the
    /// model-level exclusion (CR-060), in a stable order.
    fn documents_of(&self, archetypes: &[String]) -> Vec<usize> {
        let mut slots: Vec<usize> = sorted(archetypes)
            .into_iter()
            .flat_map(|archetype| self.spec.by_type_slots(archetype))
            .filter(|&slot| !self.exclude.excludes(self.root, &self.spec.doc(slot).path))
            .collect();
        slots.sort_unstable();
        slots.dedup();
        slots
    }

    // ── liveness against the merged registry ──────────────────────────────

    fn undeclared<'n>(&self, archetypes: impl IntoIterator<Item = &'n String>) -> Vec<String> {
        archetypes
            .into_iter()
            .filter(|name| self.registry.archetype(name).is_none())
            .map(|name| format!("archetype '{name}' is declared by no loaded module"))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Whether `field` is a property of a declared archetype's frontmatter
    /// schema or is carried by a document of any named archetype.
    fn field_is_live(&self, archetypes: &[String], field: &str) -> bool {
        archetypes.iter().any(|name| {
            self.registry
                .archetype(name)
                .and_then(|a| a.frontmatter_schema.as_deref())
                .is_some_and(|schema| declares_property(schema, field))
                || self.spec.corpus_by_type(name).iter().any(|doc| {
                    doc.frontmatter()
                        .is_some_and(|frontmatter| frontmatter.contains_key(field))
                })
        })
    }

    fn dead_field(&self, archetypes: &[String], field: &str, role: &str) -> Option<String> {
        let declared: Vec<String> = archetypes
            .iter()
            .filter(|name| self.registry.archetype(name).is_some())
            .cloned()
            .collect();
        (!declared.is_empty() && !self.field_is_live(archetypes, field)).then(|| {
            format!(
                "{role} '{field}' is not a property of the frontmatter of {} and no such document \
                 carries it",
                sorted(archetypes)
                    .iter()
                    .map(|name| format!("'{name}'"))
                    .collect::<Vec<_>>()
                    .join("/")
            )
        })
    }

    fn report_problems(&mut self, kind: &str, name: &str, mut problems: Vec<String>) {
        if problems.is_empty() {
            return;
        }
        problems.sort();
        self.module_level(
            UNDECLARED_VOCABULARY,
            format!(
                "{kind} entry '{name}' names something nothing has: {} — it matches nothing",
                problems.join("; ")
            ),
        );
    }

    fn check_carried_fields(&mut self, model: &TraceabilityModel) {
        for carried in &model.carried_fields {
            let mut problems = self.undeclared(&carried.from);
            problems.extend(self.dead_field(&carried.from, &carried.key, "key"));
            self.report_problems("carried_fields", &carried.name, problems);
        }
    }

    fn check_consistency_is_live(&mut self, rule: &FieldConsistency) {
        let mut kinds: Vec<&String> = rule.from.iter().chain(&rule.to).collect();
        kinds.push(&rule.order_from.archetype);
        if let Some(via) = &rule.via {
            kinds.push(&via.archetype);
        }
        let mut problems = self.undeclared(kinds);
        problems.extend(self.dead_field(&rule.from, &rule.field, "field"));
        problems.extend(self.dead_field(&rule.to, &rule.to_field, "to_field"));
        let order = std::slice::from_ref(&rule.order_from.archetype);
        problems.extend(self.dead_field(order, &rule.order_from.field, "order_from field"));
        if self.rank_order(rule).is_none() && problems.is_empty() {
            problems.push(format!(
                "order_from field '{}' of '{}' has no enum, so no rank exists",
                rule.order_from.field, rule.order_from.archetype
            ));
        }
        self.report_problems("field_consistency", &rule.name, problems);
    }

    fn check_required_is_live(&mut self, rule: &RequiredField) {
        let mut problems = self.undeclared(&rule.from);
        problems.extend(self.dead_field(&rule.from, &rule.field, "field"));
        self.report_problems("required_fields", &rule.name, problems);
    }

    fn check_vocabulary_is_live(&mut self, model: &TraceabilityModel, rule: &VocabularyRule) {
        let mut problems = self.undeclared(&rule.from);
        problems.extend(self.dead_field(&rule.from, &rule.field, "field"));
        if !model.carried_fields.iter().any(|c| c.name == rule.carried) {
            problems.push(format!(
                "carried name '{}' is declared by no carried_fields entry",
                rule.carried
            ));
        } else if !self.carried.values.contains_key(&rule.carried) {
            problems.push(format!(
                "carried name '{}' has no carried_vocabulary entry",
                rule.carried
            ));
        }
        self.report_problems("vocabulary", &rule.name, problems);
    }

    // ── the rules ─────────────────────────────────────────────────────────

    /// The rank order of a consistency entry: the `enum` of `order_from`.
    fn rank_order(&self, rule: &FieldConsistency) -> Option<Vec<String>> {
        self.registry
            .archetype(&rule.order_from.archetype)?
            .frontmatter_schema
            .as_deref()
            .and_then(|schema| enum_at(schema, &rule.order_from.field))
            .filter(|values| !values.is_empty())
    }

    /// Every target a `from` document reaches, with the `via` documents that
    /// reach it. A dangling edge, or an end outside the hop's archetypes, joins
    /// nothing. Several routes to one target are one pair.
    fn reached(&self, rule: &FieldConsistency, slot: usize) -> BTreeMap<usize, BTreeSet<usize>> {
        let mut reached: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
        let Some(hops) = self.context.hops_for(&rule.name) else {
            return reached;
        };
        let starts: Vec<(usize, Option<usize>)> = match (&rule.via, &hops.via) {
            (Some(via), Some(edges)) => edges
                .matches(via.direction.as_relation(), slot)
                .iter()
                .filter_map(|m| m.counterpart)
                .filter(|&found| {
                    self.spec.doc(found).concept_type() == Some(via.archetype.as_str())
                })
                .map(|found| (found, Some(found)))
                .collect(),
            _ => vec![(slot, None)],
        };
        for (start, via) in starts {
            for found in hops.target.matches(rule.direction.as_relation(), start) {
                let Some(target) = found.counterpart.filter(|&t| t != slot) else {
                    continue;
                };
                let kind = self.spec.doc(target).concept_type();
                if rule.to.iter().any(|to| Some(to.as_str()) == kind) {
                    reached.entry(target).or_default().extend(via);
                }
            }
        }
        reached
    }

    fn outside_enum(&mut self, doc: &LoadedDocument, field: &str, value: &str, owner: &str) {
        self.found.push(Found {
            check: UNDECLARED_VALUE.to_string(),
            path: doc.path.clone(),
            via: String::new(),
            target: doc.id.clone(),
            message: format!(
                "'{}' states {field} '{value}', which is outside the enum '{owner}' ranks by, so \
                 it is not compared",
                doc.id
            ),
        });
    }

    fn consistency(&mut self, rule: &FieldConsistency) {
        let Some(order) = self.rank_order(rule) else {
            return;
        };
        let owner = format!("{}.{}", rule.order_from.archetype, rule.order_from.field);
        let spec = self.spec;
        for slot in self.documents_of(&rule.from) {
            let from = spec.doc(slot);
            for (target_slot, vias) in self.reached(rule, slot) {
                let target = spec.doc(target_slot);
                let (from_standing, to_standing) = (
                    standing(from, &rule.field, &order),
                    standing(target, &rule.to_field, &order),
                );
                let mut compared = true;
                if let Standing::Outside(value) = &from_standing {
                    self.outside_enum(from, &rule.field, value, &owner);
                    compared = false;
                }
                if let Standing::Outside(value) = &to_standing {
                    self.outside_enum(target, &rule.to_field, value, &owner);
                    compared = false;
                }
                let (Standing::Rank(low), Standing::Rank(high), true) =
                    (from_standing, to_standing, compared)
                else {
                    continue;
                };
                if low >= high {
                    continue;
                }
                let mut via_ids: Vec<String> =
                    vias.iter().map(|&via| self.named(via, slot)).collect();
                via_ids.sort();
                let through = if via_ids.is_empty() {
                    String::new()
                } else {
                    format!(
                        " reached through {}",
                        via_ids
                            .iter()
                            .map(|id| format!("'{id}'"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                let (from_value, to_value) = (
                    stated(from, &rule.field).map(written).unwrap_or_default(),
                    stated(target, &rule.to_field)
                        .map(written)
                        .unwrap_or_default(),
                );
                let target_id = self.named(target_slot, slot);
                self.found.push(Found {
                    check: rule.check.clone(),
                    path: from.path.clone(),
                    via: via_ids.first().cloned().unwrap_or_default(),
                    target: target_id.clone(),
                    message: format!(
                        "'{}' states {} '{from_value}', below {} '{to_value}' stated by \
                         '{target_id}'{through} — declared by '{}'",
                        from.id, rule.field, rule.to_field, rule.name
                    ),
                });
            }
        }
    }

    fn required(&mut self, rule: &RequiredField) {
        let exclude = ExcludeSet::compile_validated(&rule.exclude);
        let spec = self.spec;
        for slot in self.documents_of(&rule.from) {
            let doc = spec.doc(slot);
            if doc.id.is_empty() || exclude.excludes(self.root, &doc.path) {
                continue;
            }
            if let Some(selector) = &rule.selector {
                let selected = doc
                    .frontmatter()
                    .and_then(|frontmatter| frontmatter.get(selector.field()))
                    .is_some_and(|value| selector.selects(value));
                if !selected {
                    continue;
                }
            }
            if stated(doc, &rule.field).is_none() {
                self.found.push(Found {
                    check: rule.check.clone(),
                    path: doc.path.clone(),
                    via: String::new(),
                    target: doc.id.clone(),
                    message: format!(
                        "'{}' states no `{}` — declared by '{}'",
                        doc.id, rule.field, rule.name
                    ),
                });
            }
        }
    }

    fn vocabulary(&mut self, rule: &VocabularyRule) {
        let Some(allowed) = self.carried.values.get(&rule.carried).cloned() else {
            return;
        };
        let spec = self.spec;
        for slot in self.documents_of(&rule.from) {
            let doc = spec.doc(slot);
            let Some(value) = stated(doc, &rule.field) else {
                continue;
            };
            if value
                .as_str()
                .is_some_and(|text| allowed.iter().any(|member| member == text))
            {
                continue;
            }
            self.found.push(Found {
                check: rule.check.clone(),
                path: doc.path.clone(),
                via: String::new(),
                target: doc.id.clone(),
                message: format!(
                    "'{}' states {} '{}', which is not in the carried vocabulary of '{}' [{}] — \
                     declared by '{}'",
                    doc.id,
                    rule.field,
                    written(value),
                    rule.carried,
                    allowed.join(", "),
                    rule.name
                ),
            });
        }
    }
}
