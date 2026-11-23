//! Per-declaration validation of a traceability model.
//!
//! A declaration that cannot be executed is dropped alone: every other
//! declaration of the model, and every other module, still loads. [`prune`]
//! removes the offending declarations and reports each one by kind and item.
//!
//! [`prune`]: TraceabilityModel::prune

use std::collections::BTreeSet;

use super::field_rules::{
    check_carried_field, check_field_consistency, check_required_field, check_vocabulary_rule,
};
use super::flow_rules::{check_flow_rule, check_remediation};
use super::{
    bound_target_error, check_capturing_pattern, check_excludes, check_field,
    check_kind_declaration, check_named, check_not_qsl, check_sections, DocumentReference,
    EdgeCondition, ObligationSource, RequiredRelation, SourceLanguage, StatusVocabulary,
    TraceIdQualifier, TraceMarkerForm, TraceMarkerSyntax, TraceTarget, TraceTargetSource,
    TraceabilityModel, VocabularyCoverage,
};

/// The kind of traceability declaration that was dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclarationKind {
    /// The whole `traceability:` section.
    TraceabilitySection,
    /// The model-level `exclude` list.
    Exclude,
    /// The model-level `source_exclude` list.
    SourceExclude,
    /// One `trace_targets` entry.
    TraceTarget,
    /// One `document_references` entry.
    DocumentReference,
    /// One `obligations` source.
    Obligation,
    /// One `required_relations` entry.
    RequiredRelation,
    /// One `design_relations` entry.
    DesignRelation,
    /// One `acyclic_edges` entry.
    AcyclicEdge,
    /// One `vocabulary_coverage` entry.
    VocabularyCoverage,
    /// The `status` vocabulary.
    StatusVocabulary,
    /// One retirement-vocabulary value.
    RetirementVocabulary,
    /// One `no_source_symbol` value.
    NoSourceSymbol,
    /// One `trace_id_qualifiers` entry.
    TraceIdQualifier,
    /// One `trace_tags` form.
    TraceTag,
    /// One `carried_fields` entry.
    CarriedField,
    /// One `field_consistency` entry.
    FieldConsistency,
    /// One `required_fields` entry.
    RequiredField,
    /// One `vocabulary` entry.
    Vocabulary,
    /// One `flow_rules` entry.
    FlowRule,
    /// One `remediations` entry.
    Remediation,
    /// An archetype whose `frontmatter_schema_ref` cannot be resolved.
    Archetype,
}

impl DeclarationKind {
    /// The stable machine-readable spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TraceabilitySection => "traceability_section",
            Self::Exclude => "exclude",
            Self::SourceExclude => "source_exclude",
            Self::TraceTarget => "trace_target",
            Self::DocumentReference => "document_reference",
            Self::Obligation => "obligation",
            Self::RequiredRelation => "required_relation",
            Self::DesignRelation => "design_relation",
            Self::AcyclicEdge => "acyclic_edge",
            Self::VocabularyCoverage => "vocabulary_coverage",
            Self::StatusVocabulary => "status_vocabulary",
            Self::RetirementVocabulary => "retirement_vocabulary",
            Self::NoSourceSymbol => "no_source_symbol",
            Self::TraceIdQualifier => "trace_id_qualifier",
            Self::TraceTag => "trace_tag",
            Self::CarriedField => "carried_field",
            Self::FieldConsistency => "field_consistency",
            Self::RequiredField => "required_field",
            Self::Vocabulary => "vocabulary",
            Self::FlowRule => "flow_rule",
            Self::Remediation => "remediation",
            Self::Archetype => "archetype",
        }
    }
}

/// One declaration a model dropped, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationFailure {
    /// What kind of declaration it was.
    pub kind: DeclarationKind,
    /// The declaration's name (or its value or position when it has none).
    pub item: String,
    /// Why it cannot be executed.
    pub reason: String,
}

/// Failures collected while pruning, and which declarations they dropped.
#[derive(Default)]
struct Pruned {
    failures: Vec<DeclarationFailure>,
}

impl Pruned {
    fn drop(&mut self, kind: DeclarationKind, item: String, reason: String) {
        self.failures
            .push(DeclarationFailure { kind, item, reason });
    }

    fn dropped_names(&self, kind: DeclarationKind) -> BTreeSet<String> {
        self.failures
            .iter()
            .filter(|failure| failure.kind == kind)
            .map(|failure| failure.item.clone())
            .collect()
    }
}

/// The name of an entry in a failure: its name, or its position when blank.
fn item_label(name: &str, index: usize) -> String {
    if name.trim().is_empty() {
        format!("#{}", index + 1)
    } else {
        name.to_string()
    }
}

/// Keep the entries of `list` that execute. With `unique`, an entry whose name
/// another entry shares no longer identifies one declaration: identical entries
/// merge to the first, differing ones are all dropped with one failure.
fn screen<T: Clone + PartialEq>(
    kind: DeclarationKind,
    list: Vec<T>,
    unique: bool,
    name: impl Fn(&T) -> &str,
    mut check: impl FnMut(&T) -> Result<(), String>,
    out: &mut Pruned,
) -> Vec<T> {
    let mut kept: Vec<T> = Vec::new();
    let mut handled: BTreeSet<String> = BTreeSet::new();
    for (index, entry) in list.iter().enumerate() {
        let label = name(entry);
        if unique && !label.trim().is_empty() {
            let twins: Vec<&T> = list.iter().filter(|other| name(other) == label).collect();
            if twins.len() > 1 {
                if !handled.insert(label.to_string()) {
                    continue;
                }
                if twins.iter().any(|twin| *twin != entry) {
                    out.drop(
                        kind,
                        label.to_string(),
                        format!(
                            "traceability: {} entry '{label}' is declared more than once with \
                             different bodies, so the name no longer identifies one declaration",
                            kind.as_str()
                        ),
                    );
                    continue;
                }
            }
        }
        match check(entry) {
            Ok(()) => kept.push(entry.clone()),
            Err(reason) => out.drop(kind, item_label(label, index), reason),
        }
    }
    kept
}

impl TraceabilityModel {
    /// The declarations of this model that cannot be executed. The model itself
    /// is not changed; use [`prune`](Self::prune) to drop them.
    pub fn validate(&self) -> Vec<DeclarationFailure> {
        self.clone().prune()
    }

    /// Drop every declaration that cannot be executed and report each one.
    ///
    /// A declaration depending on a dropped one is dropped as well, with a
    /// failure naming the declaration it depended on. Dependencies that cross
    /// modules wait for the merged registry.
    pub fn prune(&mut self) -> Vec<DeclarationFailure> {
        let mut out = Pruned::default();
        for (kind, key, list) in [
            (DeclarationKind::Exclude, "exclude", &mut self.exclude),
            (
                DeclarationKind::SourceExclude,
                "source_exclude",
                &mut self.source_exclude,
            ),
        ] {
            if let Err(reason) = check_excludes("model-level", key, list) {
                list.clear();
                out.drop(kind, key.to_string(), reason);
            }
        }

        if let Some(status) = &self.status {
            if let Err(reason) = check_status(status) {
                out.drop(
                    DeclarationKind::StatusVocabulary,
                    status.column.clone(),
                    reason,
                );
                self.status = None;
            }
        }

        self.trace_id_qualifiers = screen(
            DeclarationKind::TraceIdQualifier,
            std::mem::take(&mut self.trace_id_qualifiers),
            true,
            |q| q.prefix.as_str(),
            check_qualifier,
            &mut out,
        );

        self.trace_targets = screen(
            DeclarationKind::TraceTarget,
            std::mem::take(&mut self.trace_targets),
            true,
            |t| t.name.as_str(),
            check_trace_target,
            &mut out,
        );
        let target_names: BTreeSet<String> =
            self.trace_targets.iter().map(|t| t.name.clone()).collect();
        let dropped_targets = out.dropped_names(DeclarationKind::TraceTarget);
        let status_dropped = !out
            .dropped_names(DeclarationKind::StatusVocabulary)
            .is_empty();
        let status_declared = self.status.is_some();

        self.document_references = screen(
            DeclarationKind::DocumentReference,
            std::mem::take(&mut self.document_references),
            true,
            |r| r.name.as_str(),
            |reference| {
                check_document_reference(
                    reference,
                    &target_names,
                    &dropped_targets,
                    status_declared,
                    status_dropped,
                )
            },
            &mut out,
        );

        let targets = self.trace_targets.clone();
        self.obligations = screen(
            DeclarationKind::Obligation,
            std::mem::take(&mut self.obligations),
            true,
            |o| o.name.as_str(),
            |source| check_obligation(source, &targets, &target_names, &dropped_targets),
            &mut out,
        );

        self.required_relations = screen(
            DeclarationKind::RequiredRelation,
            std::mem::take(&mut self.required_relations),
            true,
            |r| r.name.as_str(),
            check_required_relation,
            &mut out,
        );

        // Whether a non-blank entry names a declared relation waits for the
        // merged model, because the relation may come from another module.
        self.design_relations = screen(
            DeclarationKind::DesignRelation,
            std::mem::take(&mut self.design_relations),
            true,
            String::as_str,
            |entry| {
                if entry.trim().is_empty() {
                    Err(
                        "traceability: design_relations contains a blank entry, which names \
                         no required relation"
                            .to_string(),
                    )
                } else {
                    Ok(())
                }
            },
            &mut out,
        );

        // An empty verb would make the cycle check walk a graph no edge
        // matches, so it would cover nothing while appearing declared.
        self.acyclic_edges = screen(
            DeclarationKind::AcyclicEdge,
            std::mem::take(&mut self.acyclic_edges),
            true,
            String::as_str,
            |edge| {
                if edge.trim().is_empty() {
                    Err(
                        "traceability: acyclic_edges contains an empty verb, which would \
                         check nothing while appearing to be declared"
                            .to_string(),
                    )
                } else {
                    Ok(())
                }
            },
            &mut out,
        );

        self.vocabulary_coverage = screen(
            DeclarationKind::VocabularyCoverage,
            std::mem::take(&mut self.vocabulary_coverage),
            true,
            |c| c.name.as_str(),
            check_vocabulary_coverage,
            &mut out,
        );

        self.carried_fields = screen(
            DeclarationKind::CarriedField,
            std::mem::take(&mut self.carried_fields),
            false,
            |c| c.name.as_str(),
            check_carried_field,
            &mut out,
        );
        self.field_consistency = screen(
            DeclarationKind::FieldConsistency,
            std::mem::take(&mut self.field_consistency),
            true,
            |r| r.name.as_str(),
            check_field_consistency,
            &mut out,
        );
        self.required_fields = screen(
            DeclarationKind::RequiredField,
            std::mem::take(&mut self.required_fields),
            true,
            |r| r.name.as_str(),
            check_required_field,
            &mut out,
        );
        self.vocabulary = screen(
            DeclarationKind::Vocabulary,
            std::mem::take(&mut self.vocabulary),
            true,
            |r| r.name.as_str(),
            check_vocabulary_rule,
            &mut out,
        );
        self.flow_rules = screen(
            DeclarationKind::FlowRule,
            std::mem::take(&mut self.flow_rules),
            true,
            |r| r.name.as_str(),
            check_flow_rule,
            &mut out,
        );
        self.remediations = screen(
            DeclarationKind::Remediation,
            std::mem::take(&mut self.remediations),
            true,
            |r| r.name.as_str(),
            check_remediation,
            &mut out,
        );

        // CR-041: the exemption reads a column, so the column must be named, and
        // a value outside the declared vocabulary would silently exempt nothing.
        let column = self.vocabularies.test_type_column.is_some();
        let test_types = self.vocabularies.test_type.clone();
        self.vocabularies.no_source_symbol = screen(
            DeclarationKind::NoSourceSymbol,
            std::mem::take(&mut self.vocabularies.no_source_symbol),
            false,
            String::as_str,
            |value| {
                if !column {
                    return Err(format!(
                        "traceability: vocabularies.no_source_symbol value '{value}' needs \
                         vocabularies.test_type_column to say which column carries the value"
                    ));
                }
                if test_types
                    .iter()
                    .any(|t| t.trim().eq_ignore_ascii_case(value.trim()))
                {
                    Ok(())
                } else {
                    Err(format!(
                        "traceability: vocabularies.no_source_symbol value '{value}' is not in \
                         the declared test_type vocabulary"
                    ))
                }
            },
            &mut out,
        );
        self.vocabularies.retired = screen(
            DeclarationKind::RetirementVocabulary,
            std::mem::take(&mut self.vocabularies.retired),
            false,
            String::as_str,
            |value| {
                if value.trim().is_empty() {
                    Err("traceability: vocabularies.retired values must be nonblank".to_string())
                } else {
                    Ok(())
                }
            },
            &mut out,
        );

        let TraceabilityModel { trace_tags, .. } = self;
        trace_tags.markers = screen(
            DeclarationKind::TraceTag,
            std::mem::take(&mut trace_tags.markers),
            true,
            |m| m.name.as_str(),
            check_marker,
            &mut out,
        );
        trace_tags.implements = screen(
            DeclarationKind::TraceTag,
            std::mem::take(&mut trace_tags.implements),
            false,
            |m| m.name.as_str(),
            check_implements,
            &mut out,
        );
        trace_tags.formalizes = screen(
            DeclarationKind::TraceTag,
            std::mem::take(&mut trace_tags.formalizes),
            true,
            |m| m.name.as_str(),
            check_formalizes,
            &mut out,
        );
        out.failures
    }

    /// Remove the declaration of `kind` named `item`. A kind this does not
    /// hold, or an item it does not declare, leaves the model unchanged.
    pub fn remove_declaration(&mut self, kind: DeclarationKind, item: &str) {
        use DeclarationKind as K;
        match kind {
            K::TraceabilitySection => *self = Self::default(),
            K::Exclude => self.exclude.clear(),
            K::SourceExclude => self.source_exclude.clear(),
            K::TraceTarget => self.trace_targets.retain(|d| d.name != item),
            K::DocumentReference => self.document_references.retain(|d| d.name != item),
            K::Obligation => self.obligations.retain(|d| d.name != item),
            K::RequiredRelation => self.required_relations.retain(|d| d.name != item),
            K::DesignRelation => self.design_relations.retain(|d| d != item),
            K::AcyclicEdge => self.acyclic_edges.retain(|d| d != item),
            K::VocabularyCoverage => self.vocabulary_coverage.retain(|d| d.name != item),
            K::StatusVocabulary => self.status = None,
            K::RetirementVocabulary => self.vocabularies.retired.retain(|d| d != item),
            K::NoSourceSymbol => self.vocabularies.no_source_symbol.retain(|d| d != item),
            K::TraceIdQualifier => self.trace_id_qualifiers.retain(|d| d.prefix != item),
            K::TraceTag => {
                self.trace_tags.markers.retain(|d| d.name != item);
                self.trace_tags.implements.retain(|d| d.name != item);
                self.trace_tags.formalizes.retain(|d| d.name != item);
            }
            K::CarriedField => self.carried_fields.retain(|d| d.name != item),
            K::FieldConsistency => self.field_consistency.retain(|d| d.name != item),
            K::RequiredField => self.required_fields.retain(|d| d.name != item),
            K::Vocabulary => self.vocabulary.retain(|d| d.name != item),
            K::FlowRule => self.flow_rules.retain(|d| d.name != item),
            K::Remediation => self.remediations.retain(|d| d.name != item),
            K::Archetype => {}
        }
    }
}

fn check_status(status: &StatusVocabulary) -> Result<(), String> {
    check_field("status", &status.column, "column", &status.column)?;
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for value in status
        .complete
        .iter()
        .chain(&status.pending)
        .chain(&status.failed)
    {
        if !seen.insert(value.as_str()) {
            return Err(format!(
                "traceability: status value '{value}' is declared in more than one class"
            ));
        }
    }
    Ok(())
}

/// NFR-140: a qualifier names a non-blank prefix and an `<org>/<repo>` identity.
/// That no prefix contains another waits for the merged registry.
fn check_qualifier(qualifier: &TraceIdQualifier) -> Result<(), String> {
    let prefix = &qualifier.prefix;
    if prefix.trim().is_empty() || prefix.chars().any(char::is_whitespace) {
        return Err(format!(
            "traceability: trace_id_qualifiers entry '{prefix}' declares a blank prefix or one \
             containing whitespace"
        ));
    }
    let segments: Vec<&str> = qualifier.repository.split('/').collect();
    if segments.len() != 2 || segments.iter().any(|s| s.trim().is_empty()) {
        return Err(format!(
            "traceability: trace_id_qualifiers entry '{prefix}' names repository '{}', which is \
             not `<org>/<repo>`",
            qualifier.repository
        ));
    }
    Ok(())
}

fn check_trace_target(target: &TraceTarget) -> Result<(), String> {
    check_named("trace_targets", &target.name)?;
    check_field(
        "trace_targets",
        &target.name,
        "archetype",
        &target.archetype,
    )?;
    check_excludes(
        &format!("trace_targets entry '{}'", target.name),
        "exclude",
        &target.exclude,
    )?;
    check_sections("trace_targets", &target.name, &target.section)?;
    match &target.source {
        TraceTargetSource::TableRow { id_column } => {
            if id_column.is_empty() {
                return Err(format!(
                    "traceability: trace_targets entry '{}' declares an empty `id_column` list, \
                     so it reads no column",
                    target.name
                ));
            }
            for value in id_column.names() {
                check_field("trace_targets", &target.name, "id_column", value)?;
            }
        }
        TraceTargetSource::ListItem { pattern } | TraceTargetSource::Heading { pattern } => {
            check_capturing_pattern("trace_targets", &target.name, pattern)?;
        }
    }
    Ok(())
}

fn check_document_reference(
    reference: &DocumentReference,
    target_names: &BTreeSet<String>,
    dropped_targets: &BTreeSet<String>,
    status_declared: bool,
    status_dropped: bool,
) -> Result<(), String> {
    check_named("document_references", &reference.name)?;
    check_field(
        "document_references",
        &reference.name,
        "archetype",
        &reference.archetype,
    )?;
    check_excludes(
        &format!("document_references entry '{}'", reference.name),
        "exclude",
        &reference.exclude,
    )?;
    check_sections("document_references", &reference.name, &reference.section)?;
    check_field(
        "document_references",
        &reference.name,
        "column",
        &reference.column,
    )?;
    check_capturing_pattern("document_references", &reference.name, &reference.pattern)?;
    if let Some(column) = &reference.status_column {
        check_field(
            "document_references",
            &reference.name,
            "status_column",
            column,
        )?;
        if !status_declared {
            return Err(if status_dropped {
                format!(
                    "traceability: document_references entry '{}' declares status_column, which \
                     depends on the dropped traceability.status vocabulary",
                    reference.name
                )
            } else {
                format!(
                    "traceability: document_references entry '{}' declares status_column \
                     without a traceability.status vocabulary",
                    reference.name
                )
            });
        }
    }
    if reference.targets.is_empty() {
        return Err(format!(
            "traceability: document_references entry '{}' declares no targets",
            reference.name
        ));
    }
    for target in &reference.targets {
        if dropped_targets.contains(target) {
            return Err(format!(
                "traceability: document_references entry '{}' depends on dropped trace_targets \
                 entry '{target}'",
                reference.name
            ));
        }
        if !target_names.contains(target) {
            return Err(format!(
                "traceability: document_references entry '{}' references undeclared target \
                 '{target}'",
                reference.name
            ));
        }
    }
    check_kind_declaration(reference)
}

/// FR-053: an obligation source resolves its minting documents exactly one way.
fn check_obligation(
    source: &ObligationSource,
    targets: &[TraceTarget],
    target_names: &BTreeSet<String>,
    dropped_targets: &BTreeSet<String>,
) -> Result<(), String> {
    check_named("obligations", &source.name)?;
    match (&source.target, &source.archetype) {
        (Some(_), Some(_)) => {
            return Err(format!(
                "traceability: obligations entry '{}' declares both 'target' and 'archetype'; a \
                 source inherits from a trace target or names its own archetype, never both",
                source.name
            ))
        }
        (None, None) => {
            return Err(format!(
                "traceability: obligations entry '{}' declares neither 'target' nor \
                 'archetype', so nothing says which documents state it",
                source.name
            ))
        }
        (Some(target), None) => {
            if dropped_targets.contains(target) {
                return Err(format!(
                    "traceability: obligations entry '{}' depends on dropped trace_targets \
                     entry '{target}'",
                    source.name
                ));
            }
            if !target_names.contains(target) {
                return Err(format!(
                    "traceability: obligations entry '{}' references undeclared target \
                     '{target}'",
                    source.name
                ));
            }
            bound_target_error(targets, &source.name, target)?;
        }
        (None, Some(archetype)) => {
            check_field("obligations", &source.name, "archetype", archetype)?;
            let Some(section) = &source.section else {
                return Err(format!(
                    "traceability: obligations entry '{}' declares 'archetype' without \
                     'section'",
                    source.name
                ));
            };
            check_sections("obligations", &source.name, section)?;
            let Some(id_format) = &source.id_format else {
                return Err(format!(
                    "traceability: obligations entry '{}' declares 'archetype' without \
                     'id_format'; rows under an archetype-bound section mint no id of their own",
                    source.name
                ));
            };
            // A template naming neither placeholder renders one id for every
            // row, so every obligation after the first would overwrite it.
            if !id_format.contains("{document}") && !id_format.contains("{row}") {
                return Err(format!(
                    "traceability: obligations entry '{}' has an id_format naming neither \
                     {{document}} nor {{row}}, so every row would render the same id",
                    source.name
                ));
            }
        }
    }
    check_excludes(
        &format!("obligations entry '{}'", source.name),
        "exclude",
        &source.exclude,
    )?;
    check_field(
        "obligations",
        &source.name,
        "statement_column",
        &source.statement_column,
    )?;
    if let Some(field) = &source.criticality_field {
        check_field("obligations", &source.name, "criticality_field", field)?;
    }
    if let Some(combinatorial) = &source.combinatorial {
        check_field(
            "obligations",
            &source.name,
            "combinatorial.dimension_column",
            &combinatorial.dimension_column,
        )?;
        check_field(
            "obligations",
            &source.name,
            "combinatorial.values_column",
            &combinatorial.values_column,
        )?;
        if let Some(column) = &combinatorial.excludes_column {
            check_field(
                "obligations",
                &source.name,
                "combinatorial.excludes_column",
                column,
            )?;
        }
        // Strength 0 mints an obligation over nothing, which reads as a declared
        // configuration space that demands no coverage while looking answered.
        if combinatorial.strength == 0 {
            return Err(format!(
                "traceability: obligations entry '{}' declares combinatorial strength 0, which \
                 demands coverage of nothing while reading as declared",
                source.name
            ));
        }
    }
    for (key, column) in &source.parameters {
        if key.trim().is_empty() {
            return Err(format!(
                "traceability: obligations entry '{}' declares a parameter with an empty key",
                source.name
            ));
        }
        check_field(
            "obligations",
            &source.name,
            &format!("parameters.{key}"),
            column,
        )?;
    }
    Ok(())
}

/// FR-058: every rule rejects a declaration whose runtime effect would be
/// silent and wrong rather than absent.
fn check_required_relation(relation: &RequiredRelation) -> Result<(), String> {
    check_named("required_relations", &relation.name)?;
    check_field("required_relations", &relation.name, "from", &relation.from)?;
    for condition in &relation.when_any {
        let others: Vec<&str> = condition.other().iter().map(String::as_str).collect();
        EdgeCondition::new(condition.verb(), condition.direction(), &others).map_err(|why| {
            format!(
                "traceability: required_relations entry '{}' has an invalid `when_any`: {why}",
                relation.name
            )
        })?;
    }
    // No accepted verb means no edge can satisfy the relation, so EVERY `from`
    // document would be reported.
    if relation.edges.is_empty() {
        return Err(format!(
            "traceability: required_relations entry '{}' declares no edges, so no link could \
             ever satisfy it and every '{}' document would be reported",
            relation.name, relation.from
        ));
    }
    for edge in &relation.edges {
        check_field("required_relations", &relation.name, "edges", edge)?;
    }
    // `to` is meaningful when absent (any document satisfies), so a blank string
    // would narrow the accepted set to a target nothing can match.
    for target in &relation.to {
        check_field("required_relations", &relation.name, "to", target)?;
    }
    check_field(
        "required_relations",
        &relation.name,
        "check",
        &relation.check,
    )?;
    // The token is the `<check>` half of a `trace:<check>` severity key. One
    // that cannot round-trip is a relation whose severity can never be tuned.
    if !crate::grammar::is_severity_key(&crate::grammar::severity_key("trace", &relation.check)) {
        return Err(format!(
            "traceability: required_relations entry '{}' has a `check` token '{}' that cannot \
             form a `trace:<check>` severity key, so its severity could never be configured",
            relation.name, relation.check
        ));
    }
    check_excludes(
        &format!("required_relations entry '{}'", relation.name),
        "exclude",
        &relation.exclude,
    )
}

fn check_vocabulary_coverage(coverage: &VocabularyCoverage) -> Result<(), String> {
    check_named("vocabulary_coverage", &coverage.name)?;
    check_field(
        "vocabulary_coverage",
        &coverage.name,
        "from",
        &coverage.from,
    )?;
    check_field(
        "vocabulary_coverage",
        &coverage.name,
        "field",
        &coverage.field,
    )?;
    check_field(
        "vocabulary_coverage",
        &coverage.name,
        "check",
        &coverage.check,
    )?;
    if !crate::grammar::is_severity_key(&crate::grammar::severity_key("trace", &coverage.check)) {
        return Err(format!(
            "traceability: vocabulary_coverage entry '{}' has a `check` token '{}' that cannot \
             form a `trace:<check>` severity key, so its severity could never be configured",
            coverage.name, coverage.check
        ));
    }
    if let Some(field) = &coverage.justified_absence_field {
        check_field(
            "vocabulary_coverage",
            &coverage.name,
            "justified_absence_field",
            field,
        )?;
    }
    check_excludes(
        &format!("vocabulary_coverage entry '{}'", coverage.name),
        "exclude",
        &coverage.exclude,
    )
}

fn check_marker(marker: &TraceMarkerForm) -> Result<(), String> {
    check_named("trace_tags.markers", &marker.name)?;
    check_not_qsl("trace_tags.markers", marker)?;
    if let Some(
        syntax @ (TraceMarkerSyntax::RustAttribute
        | TraceMarkerSyntax::PythonDecorator
        | TraceMarkerSyntax::TypescriptDecorator
        | TraceMarkerSyntax::TypescriptWrapper),
    ) = marker.syntax
    {
        return Err(format!(
            "trace_tags.markers form '{}' cannot declare syntax {}; only trace_tags.implements \
             supports it",
            marker.name,
            syntax.as_str()
        ));
    }
    check_capturing_pattern("trace_tags.markers", &marker.name, &marker.pattern)?;
    // FR-051-AC-42: shell tags are read only through a `syntax: comment` form.
    if marker.language == SourceLanguage::Shell && marker.syntax != Some(TraceMarkerSyntax::Comment)
    {
        return Err(format!(
            "traceability: trace_tags.markers form '{}' is a shell form and must declare \
             `syntax: comment`",
            marker.name
        ));
    }
    if let Some(entry_paths) = &marker.entry_paths {
        if marker.language != SourceLanguage::Shell {
            return Err(format!(
                "traceability: trace_tags.markers form '{}' cannot declare `entry_paths`; only a \
                 `language: shell` form has an entry point",
                marker.name
            ));
        }
        check_excludes(
            &format!("trace_tags.markers form '{}'", marker.name),
            "entry_paths",
            entry_paths,
        )?;
    }
    Ok(())
}

fn check_implements(marker: &TraceMarkerForm) -> Result<(), String> {
    check_named("trace_tags.implements", &marker.name)?;
    check_not_qsl("trace_tags.implements", marker)?;
    check_capturing_pattern("trace_tags.implements", &marker.name, &marker.pattern)?;
    if marker.entry_paths.is_some() {
        return Err(format!(
            "traceability: trace_tags.implements form '{}' cannot declare `entry_paths`; an \
             `implements` form binds no `entry_point`",
            marker.name
        ));
    }
    let required = match marker.syntax {
        Some(TraceMarkerSyntax::RustAttribute) => Some(SourceLanguage::Rust),
        Some(TraceMarkerSyntax::PythonDecorator) => Some(SourceLanguage::Python),
        Some(TraceMarkerSyntax::TypescriptDecorator | TraceMarkerSyntax::TypescriptWrapper) => {
            Some(SourceLanguage::Typescript)
        }
        _ => None,
    };
    if let (Some(required), Some(syntax)) = (required, marker.syntax) {
        if marker.language != required {
            return Err(format!(
                "trace_tags.implements form '{}' requires {} for {}",
                marker.name,
                required.as_str(),
                syntax.as_str()
            ));
        }
    }
    Ok(())
}

fn check_formalizes(marker: &TraceMarkerForm) -> Result<(), String> {
    check_named("trace_tags.formalizes", &marker.name)?;
    if marker.language != SourceLanguage::Qsl {
        return Err(format!(
            "traceability: trace_tags.formalizes form '{}' must declare `language: qsl`, not {}",
            marker.name,
            marker.language.as_str()
        ));
    }
    if marker.syntax != Some(TraceMarkerSyntax::Comment) {
        return Err(format!(
            "traceability: trace_tags.formalizes form '{}' must declare `syntax: comment`",
            marker.name
        ));
    }
    if marker.entry_paths.is_some() {
        return Err(format!(
            "traceability: trace_tags.formalizes form '{}' cannot declare `entry_paths`",
            marker.name
        ));
    }
    check_capturing_pattern("trace_tags.formalizes", &marker.name, &marker.pattern)
}
