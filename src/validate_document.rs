//! `validate_document` — the default markdown validation path (FR-032).
//!
//! Validates an authored **markdown document** against a unified
//! archetype (FR-031). This is distinct from the context/data path
//! ([`crate::validate`], FR-002), which validates a JSON object against
//! the archetype schema and never parses markdown.
//!
//! Pipeline (ADR 0004):
//! 1. Parse via [`crate::parse_document`] (FR-005).
//! 2. Validate the frontmatter against `frontmatter_schema_ref`.
//! 3. Run each `body_extraction` locator in an **asserting posture**:
//!    every `required: true` locator must resolve to non-empty,
//!    non-placeholder content, and its optional `assert:` facet (FR-033,
//!    with `{field}` interpolation FR-034) must hold.
//! 4. Enforce per-level heading uniqueness (FR-035).
//!
//! Diagnostics are line-numbered (1-based document line) and carry a
//! [`ValidationReason`].

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::ast::{QuireDocument, QuireSection};
use crate::extract::assert_eval::{evaluate_assert, AssertReason};
use crate::extract::dsl::ExtractionDsl;
use crate::extract::locator::{Locator, LocatorPrimitive};
use crate::loader::compile::CompiledArchetype;

/// Why a [`ValidationError`] was raised. Mirrors the FR-032 reason
/// vocabulary plus an `UnresolvedField` for FR-034-AC-2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationReason {
    /// A `required` locator resolved to nothing.
    Missing,
    /// A `required` locator resolved to empty/whitespace content.
    Empty,
    /// A `required` locator resolved to placeholder-only content
    /// (`TODO`, `{{...}}`, `none specified`, …).
    Placeholder,
    /// A locator `assert:` facet did not hold.
    Assert,
    /// The frontmatter violated `frontmatter_schema_ref`.
    Frontmatter,
    /// Two headings share text at the same level (FR-035).
    DuplicateHeading,
    /// A `{field}` token referenced an absent frontmatter key (FR-034).
    UnresolvedField,
    /// The frontmatter `object:` named an archetype the registry does not
    /// know (FR-032-AC-12). This is the only **warning**-severity reason;
    /// it is advisory and never fails validation.
    UnknownObjectType,
    /// A frontmatter `relationships` edge declared a `type` that is in
    /// neither the artifact archetype's nor the object archetype's
    /// resolved `allowed_links` vocabulary (FR-040-AC-8, Tier-1).
    /// Warning-severity — advisory, never fails validation.
    DisallowedEdgeType,
    /// A requirement-grammar (EARS, FR-042) finding. Severity is policy:
    /// advisory `warning` by default, promotable to `error`. Carried for both
    /// the warning and error routing of grammar findings.
    Grammar,
}

impl ValidationReason {
    /// Stable machine-readable string used in surfaces (CLI, wheel).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Empty => "empty",
            Self::Placeholder => "placeholder",
            Self::Assert => "assert",
            Self::Frontmatter => "frontmatter",
            Self::DuplicateHeading => "duplicate-heading",
            Self::UnresolvedField => "unresolved-field",
            Self::UnknownObjectType => "unknown-object-type",
            Self::DisallowedEdgeType => "disallowed-edge-type",
            Self::Grammar => "grammar",
        }
    }
}

/// One markdown-validation diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Human-readable message naming the archetype + offending
    /// locator/section.
    pub message: String,
    /// 1-based document line of the offending element, when known.
    pub line: Option<usize>,
    /// Machine-readable reason.
    pub reason: ValidationReason,
}

/// One advisory markdown-validation diagnostic. Distinct from
/// [`ValidationError`]: warnings never fail validation (`is_valid` ignores
/// them). The only warning today is the unknown-`object:` case
/// (FR-032-AC-12) — composed type+object validation where the frontmatter
/// `object:` names an archetype the registry cannot resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationWarning {
    /// Human-readable advisory message.
    pub message: String,
    /// 1-based document line of the offending element, when known.
    pub line: Option<usize>,
    /// Machine-readable reason (e.g. [`ValidationReason::UnknownObjectType`]).
    pub reason: ValidationReason,
}

/// Outcome of [`validate_document`] / [`validate_document_in_registry`].
///
/// Carries both exit-failing `errors` and advisory `warnings`.
/// `is_valid` is `errors.is_empty()` — **warnings never fail validation**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    fn new(errors: Vec<ValidationError>, warnings: Vec<ValidationWarning>) -> Self {
        Self {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

/// Implements: FR-030
/// Validate an authored markdown `doc_text` against `archetype` (FR-032).
///
/// This is the **type-only** path: it validates the document against the
/// `type` (artifact) archetype alone and never inspects the frontmatter
/// `object:` field. To compose the `type` archetype with the frontmatter
/// `object:` archetype (FR-032-AC-11..13), use
/// [`validate_document_in_registry`], which has the registry needed to
/// resolve the object archetype by name.
///
/// Frontmatter-schema success is **necessary but not sufficient**: a
/// document with valid frontmatter but a missing/placeholder required
/// section, a failed assert, or a duplicate heading is invalid. The
/// returned [`ValidationResult`] carries no warnings on this path.
pub fn validate_document(archetype: &CompiledArchetype, doc_text: &str) -> ValidationResult {
    let doc = crate::parse_document(doc_text);
    let line_offset = body_line_offset(doc_text);
    let mut errors: Vec<ValidationError> = Vec::new();

    // OKF concept shape: `description`/`tags` (when present) must be typed.
    // `type`-required is enforced at the routing/corpus layer, not here —
    // a `--archetype` override may legitimately validate a typeless doc
    // (FR-004-AC-5).
    let fm = doc.frontmatter.clone().unwrap_or_default();
    errors.extend(crate::concept::validate_concept_shape(&fm));

    validate_frontmatter(archetype, &doc, &mut errors);
    if let Some(dsl) = archetype.body_extraction() {
        validate_body(archetype, &doc, dsl, line_offset, &mut errors);
    }
    check_heading_uniqueness(&doc, line_offset, &mut errors);

    let mut warnings: Vec<ValidationWarning> = Vec::new();
    // Type-only path: no registry, so an empty lexicon (FR-043) — only the
    // engine-generic mechanism/bound/backtick suppression applies.
    // FR-048-AC-7: no registry, so the all-default severity map — every
    // grammar finding surfaces as a warning regardless of any module manifest.
    run_grammar(
        archetype,
        &doc,
        line_offset,
        crate::grammar::GrammarVocabularies::defaults(),
        crate::grammar::default_severity(),
        &mut errors,
        &mut warnings,
    );

    ValidationResult::new(errors, warnings)
}

/// Run the requirement-grammar (EARS, FR-042) bundle bound to `archetype` via
/// its `grammar_ref`, routing findings into `errors`/`warnings` by severity.
/// No `grammar_ref` (or an unknown bundle) is a no-op — grammar checking is
/// advisory by construction and never the reason a document fails to validate
/// by default (severity is policy — `warning` unless a module or the CLI maps
/// the check otherwise, FR-048). `lexicon` is the merged concrete-term lexicon
/// (FR-043) and `severity` the merged per-check severity map (FR-048); the
/// type-only path passes an empty lexicon and the all-default map.
fn run_grammar(
    archetype: &CompiledArchetype,
    doc: &QuireDocument,
    line_offset: usize,
    vocab: crate::grammar::GrammarVocabularies<'_>,
    severity: &crate::grammar::GrammarSeverityMap,
    errors: &mut Vec<ValidationError>,
    warnings: &mut Vec<ValidationWarning>,
) {
    let Some(grammar_ref) = archetype.grammar_ref() else {
        return;
    };
    let findings = crate::grammar::check_document_grammar(
        grammar_ref,
        &archetype.name,
        doc,
        line_offset,
        vocab,
    );
    // FR-048: key each finding against the merged map before routing — `off`
    // drops here, so a suppressed check reaches neither list nor the summary.
    for f in crate::grammar::apply_severity(findings, severity) {
        route_grammar_finding(f, errors, warnings);
    }
}

/// Classify the binding acceptance criteria of `doc_text` by **property
/// shape** (FR-052), using `registry`'s merged vocabularies — the same
/// assembly [`run_grammar`] performs, in the one place that assembly already
/// lives.
///
/// Returns the classification records directly. Nothing here touches
/// [`ValidationResult`]: property classification is metadata, never a verdict,
/// so it has no severity, no routing and no promotion path (FR-052-CON-1). An
/// archetype with no `grammar_ref` (or an unknown bundle) yields no records,
/// exactly as it yields no findings.
/// `path` is where the caller read `doc_text` from, and exists for one reason:
/// an `obligations:` source may declare `exclude:` globs, and without a path
/// this surface cannot honour them while `coverage` does — so an excluded
/// fixture would state no obligation in one payload and state one in the other
/// (FR-053-AC-14, CR-063). `None` for content with no location, such as stdin.
pub fn classify_document_criteria(
    registry: &crate::Registry,
    archetype: &CompiledArchetype,
    doc_text: &str,
    path: Option<&std::path::Path>,
) -> Vec<crate::grammar::property::AcClassification> {
    let Some(grammar_ref) = archetype.grammar_ref() else {
        return Vec::new();
    };
    let doc = crate::parse_document(doc_text);
    let line_offset = body_line_offset(doc_text);
    let mut records = crate::grammar::classify_document_properties(
        grammar_ref,
        &archetype.name,
        &doc,
        line_offset,
        crate::grammar::GrammarVocabularies {
            lexicon: registry.lexicon_matcher(),
            observable: registry.observable_verbs_matcher(),
            vacuous: registry.vacuous_predicates_matcher(),
            idioms: registry.property_idioms_matcher(),
            ambiguous: registry.ambiguity_terms_matcher(),
        },
    );

    // FR-053: attach the obligation each criterion states, matched by row id.
    // Empty for a module declaring no `obligations:` sources, and the whole
    // lookup is skipped in that case so an unadopting corpus pays nothing.
    if let Some(model) = registry
        .traceability()
        .filter(|m| !m.obligations.is_empty())
    {
        let by_id = crate::obligation::for_document(model, &archetype.name, &doc, path);
        for record in &mut records {
            if let Some(id) = &record.row_id {
                record.obligation = by_id.get(id).cloned();
            }
        }
    }
    records
}

/// Route one grammar finding into `errors` or `warnings` by its severity
/// (FR-042-AC-7). `Warning` is advisory (never fails validation); `Error`
/// blocks. Split out from [`run_grammar`] so the severity routing is unit
/// testable without a `CompiledArchetype`.
fn route_grammar_finding(
    f: crate::grammar::GrammarFinding,
    errors: &mut Vec<ValidationError>,
    warnings: &mut Vec<ValidationWarning>,
) {
    let message = format!("[{}:{}] {}", f.grammar, f.check, f.message);
    match f.severity {
        crate::grammar::GrammarSeverity::Warning => warnings.push(ValidationWarning {
            message,
            line: f.line,
            reason: ValidationReason::Grammar,
        }),
        crate::grammar::GrammarSeverity::Error => errors.push(ValidationError {
            message,
            line: f.line,
            reason: ValidationReason::Grammar,
        }),
    }
}

/// Validate an authored markdown `doc_text` against BOTH the `type`
/// (artifact) `archetype` AND the frontmatter `object:` archetype, with
/// `object:` resolved from `registry` (FR-032-AC-11..13).
///
/// Composition is **always on**: the `type` archetype is validated
/// exactly as [`validate_document`], then —
///
/// - if the frontmatter carries no `object:` key, nothing further happens
///   (type-only behaviour, identical to [`validate_document`]);
/// - if `object:` names an archetype `registry` resolves (across
///   artifact_types AND object_types), that archetype's `body_extraction`
///   asserts run in the same asserting posture and any failures are
///   merged into `errors` (tagged with the object archetype name via the
///   existing `[<archetype>]` diagnostic prefix);
/// - if `object:` names an archetype `registry` does not know, a single
///   [`ValidationReason::UnknownObjectType`] **warning** is emitted —
///   never an error.
///
/// The `type` archetype path always produces hard errors; warnings come
/// only from the object layer and never fail validation.
pub fn validate_document_in_registry(
    registry: &crate::Registry,
    archetype: &CompiledArchetype,
    doc_text: &str,
) -> ValidationResult {
    validate_in_registry_core(registry, archetype, doc_text, registry.lexicon_matcher())
}

/// As [`validate_document_in_registry`], but the EARS grammar check (FR-042)
/// runs against an **explicitly supplied** `GrammarLexicon` instead of the
/// registry's own (FR-044). The orchestrator composes `lexicon` from the merged
/// module lexicon plus the repo's harvested Ubiquitous-Language terms (see
/// [`crate::Registry::lexicon_with`] + [`crate::corpus::glossary_terms`]).
pub fn validate_document_in_registry_with_lexicon(
    registry: &crate::Registry,
    archetype: &CompiledArchetype,
    doc_text: &str,
    lexicon: &crate::grammar::GrammarLexicon,
) -> ValidationResult {
    validate_in_registry_core(registry, archetype, doc_text, lexicon)
}

/// Shared body of the two registry-backed validation entry points. The only
/// difference is the `GrammarLexicon` the grammar check consumes.
fn validate_in_registry_core(
    registry: &crate::Registry,
    archetype: &CompiledArchetype,
    doc_text: &str,
    lexicon: &crate::grammar::GrammarLexicon,
) -> ValidationResult {
    let doc = crate::parse_document(doc_text);
    let line_offset = body_line_offset(doc_text);
    let mut errors: Vec<ValidationError> = Vec::new();
    let mut warnings: Vec<ValidationWarning> = Vec::new();

    // ── `type` archetype layer (always hard errors) ──
    let fm = doc.frontmatter.clone().unwrap_or_default();
    errors.extend(crate::concept::validate_concept_shape(&fm));
    validate_frontmatter(archetype, &doc, &mut errors);
    if let Some(dsl) = archetype.body_extraction() {
        validate_body(archetype, &doc, dsl, line_offset, &mut errors);
    }
    check_heading_uniqueness(&doc, line_offset, &mut errors);

    // ── `object:` archetype layer (composed; FR-032-AC-11..13) ──
    let mut object_archetype: Option<&CompiledArchetype> = None;
    if let Some(object_name) = frontmatter_object(&doc) {
        match registry.archetype(object_name) {
            // Resolved: run its body_extraction asserts as hard errors,
            // merged into the same list. The frontmatter schema of the
            // object archetype is intentionally NOT re-validated here —
            // the document's frontmatter is contracted by its `type`
            // archetype; the object layer only asserts body structure.
            Some(arch) => {
                object_archetype = Some(arch);
                if let Some(dsl) = arch.body_extraction() {
                    validate_body(arch, &doc, dsl, line_offset, &mut errors);
                }
            }
            // Unresolved: a single advisory warning, not an error.
            None => warnings.push(ValidationWarning {
                message: format!(
                    "unknown object type '{object_name}' declared in frontmatter `object`"
                ),
                line: None,
                reason: ValidationReason::UnknownObjectType,
            }),
        }
    }

    // ── Tier-1 edge-type validation (FR-040-AC-8) ──
    // Resolve the union of the artifact + object allowed_links and flag
    // any frontmatter `relationships` edge whose `type` is outside it.
    // When `object:` is unknown, `object_archetype` is None and the
    // vocabulary falls back to the artifact axis alone. Advisory only.
    let resolved_links = registry.resolve_allowed_links(archetype, object_archetype);
    // Skip the check entirely when neither axis declares any vocabulary —
    // an undeclared archetype must not flag every edge (open vocabulary).
    if !resolved_links.is_empty() {
        let source = frontmatter_id(&doc).unwrap_or("<document>");
        // Collect, then sort by `(target, edge_type)` so the warnings are
        // ordered by the FR-040-AC-10 key `(source, target, edge_type)`
        // (source is constant within one document) regardless of the order
        // the author listed `relationships:`.
        // FR-041: a declared inverse label is type-allowed (recognition
        // only). Per-archetype allowed_links enforcement for the inverse
        // belongs to the forward source, which the corpus layer resolves
        // (Tier-2 normalization); the document level cannot.
        let inverse = registry.inverse_index();
        let mut disallowed: Vec<(String, String)> = harvest_frontmatter_relationships(&doc)
            .into_iter()
            .filter(|(edge_type, _target)| {
                !resolved_links.contains_key(edge_type) && !inverse.contains_key(edge_type)
            })
            .map(|(edge_type, target)| (target, edge_type))
            .collect();
        disallowed.sort();
        for (target, edge_type) in disallowed {
            warnings.push(ValidationWarning {
                message: format!(
                    "edge type '{edge_type}' on '{source}' (target '{target}') is not in the \
                     resolved allowed_links vocabulary"
                ),
                line: None,
                reason: ValidationReason::DisallowedEdgeType,
            });
        }
    }

    // ── Requirement-grammar layer (EARS, FR-042; advisory by default) ──
    // The caller chose the lexicon: the module lexicon (FR-043) for the plain
    // entry point, or that ∪ the repo's project glossary (FR-044).
    run_grammar(
        archetype,
        &doc,
        line_offset,
        crate::grammar::GrammarVocabularies {
            lexicon,
            observable: registry.observable_verbs_matcher(),
            vacuous: registry.vacuous_predicates_matcher(),
            idioms: registry.property_idioms_matcher(),
            ambiguous: registry.ambiguity_terms_matcher(),
        },
        registry.grammar_severity(),
        &mut errors,
        &mut warnings,
    );

    ValidationResult::new(errors, warnings)
}

/// `(edge_type, target)` pairs from the document's frontmatter
/// `relationships` array, mirroring the corpus harvester
/// ([`crate::corpus::resolve::harvest_edges`]): entries missing `target`
/// are skipped; entries missing `type` default to `references`.
fn harvest_frontmatter_relationships(doc: &QuireDocument) -> Vec<(String, String)> {
    let Some(fm) = doc.frontmatter.as_ref() else {
        return Vec::new();
    };
    let Some(rels) = fm.get("relationships").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    rels.iter()
        .filter_map(|entry| {
            let target = entry.get("target").and_then(|v| v.as_str())?;
            let edge_type = entry
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("references");
            Some((edge_type.to_string(), target.to_string()))
        })
        .collect()
}

/// The frontmatter `id:` value when present and a non-empty string.
fn frontmatter_id(doc: &QuireDocument) -> Option<&str> {
    doc.frontmatter
        .as_ref()?
        .get("id")?
        .as_str()
        .filter(|s| !s.is_empty())
}

/// The frontmatter `object:` value when present and a non-empty string.
fn frontmatter_object(doc: &QuireDocument) -> Option<&str> {
    doc.frontmatter
        .as_ref()?
        .get("object")?
        .as_str()
        .filter(|s| !s.is_empty())
}

/// Number of body lines preceding the parsed body in the raw document —
/// the count of newlines consumed by any frontmatter block (plus a
/// leading BOM). Used to convert a section's 0-based body `start_line`
/// into a 1-based document line.
pub(crate) fn body_line_offset(doc_text: &str) -> usize {
    let stripped = doc_text.strip_prefix('\u{FEFF}').unwrap_or(doc_text);
    // `parse_document` stores the verbatim input in `raw`; the parsed
    // body is `raw` minus frontmatter. Recompute the body the same way
    // the parser did so the prefix length is exact.
    let body = crate::extract_frontmatter(doc_text).body;
    let prefix_len = stripped.len().saturating_sub(body.len());
    stripped[..prefix_len.min(stripped.len())]
        .matches('\n')
        .count()
}

fn to_doc_line(line_offset: usize, body_line: usize) -> usize {
    line_offset + body_line + 1
}

fn validate_frontmatter(
    archetype: &CompiledArchetype,
    doc: &QuireDocument,
    errors: &mut Vec<ValidationError>,
) {
    let Some(validator) = archetype.frontmatter_validator() else {
        return;
    };
    let fm = doc.frontmatter.clone().unwrap_or_default();
    let value = Value::Object(fm);
    let messages: Vec<String> = match validator.validate(&value) {
        Ok(()) => Vec::new(),
        Err(violations) => violations
            .map(|v| {
                format!(
                    "[{}] frontmatter: {} (at {})",
                    archetype.name,
                    v,
                    dotted_path(&v.instance_path.to_string())
                )
            })
            .collect(),
    };
    for message in messages {
        errors.push(ValidationError {
            message,
            line: None,
            reason: ValidationReason::Frontmatter,
        });
    }
}

fn dotted_path(ptr: &str) -> String {
    if ptr.is_empty() {
        "<frontmatter>".to_string()
    } else {
        ptr.trim_start_matches('/').replace('/', ".")
    }
}

fn validate_body(
    archetype: &CompiledArchetype,
    doc: &QuireDocument,
    dsl: &ExtractionDsl,
    line_offset: usize,
    errors: &mut Vec<ValidationError>,
) {
    // Single-yield `match` locators are the structural contract for the
    // document body.
    if let Some(map) = &dsl.yield_pattern.r#match {
        for (key, locator) in map {
            validate_required_locator(archetype, doc, key, locator, line_offset, errors);
        }
    }

    // Multi-yield (`iterate_over` + `per_match`): resolve each iteration
    // unit with the **same** evaluation the extractor uses
    // (`extract::iteration_units`), then run the required-locator +
    // assert checks for every `per_match` locator against each unit's
    // local scope (FR-032 step 3, FR-033). Line numbers stay in
    // body-relative coordinates because the unit scopes preserve each
    // section's `start_line` and inherit the parent `frontmatter`.
    if let (Some(iter), Some(per)) = (
        &dsl.yield_pattern.iterate_over,
        &dsl.yield_pattern.per_match,
    ) {
        for unit in crate::extract::iteration_units(doc, iter) {
            for (key, locator) in per {
                validate_required_locator(
                    archetype,
                    &unit.scope,
                    key,
                    locator,
                    line_offset,
                    errors,
                );
            }
        }
    }
}

fn validate_required_locator(
    archetype: &CompiledArchetype,
    doc: &QuireDocument,
    key: &str,
    locator: &Locator,
    line_offset: usize,
    errors: &mut Vec<ValidationError>,
) {
    let canonical = locator.canonical();
    let frontmatter = doc.frontmatter.as_ref();

    let (values, _pos) = crate::extract::locator::eval_locator(doc, locator);
    // FR-032-AC-9: a `table_row`/`list_item` locator that resolves to a
    // present-but-empty substrate (header-only table, item-less list)
    // is `empty`, not `missing`; a substrate that does not resolve at
    // all is `missing`. The generic `content_status` cannot tell these
    // apart (both yield zero values), so refine for these two kinds.
    let status = match table_or_list_status(doc, canonical, &values) {
        Some(s) => s,
        None => content_status(&values),
    };

    // FR-032-AC-10: an optional locator that does not resolve runs no
    // assert and emits no diagnostic. (Previously only value-shaped
    // asserts no-opped on zero values; a `level` assert against the
    // absent section leaked a spurious 'section not found' failure.)
    if !locator.required() && matches!(status, ContentStatus::Missing) {
        return;
    }

    if locator.required() {
        match status {
            ContentStatus::Missing => {
                errors.push(ValidationError {
                    message: format!(
                        "[{}] required '{key}' ({}) is missing",
                        archetype.name,
                        canonical.describe()
                    ),
                    line: locator_line(doc, canonical, line_offset),
                    reason: ValidationReason::Missing,
                });
                // No content to assert against; stop here for this key.
                return;
            }
            ContentStatus::Empty => {
                errors.push(ValidationError {
                    message: format!(
                        "[{}] required '{key}' ({}) is empty",
                        archetype.name,
                        canonical.describe()
                    ),
                    line: locator_line(doc, canonical, line_offset),
                    reason: ValidationReason::Empty,
                });
            }
            ContentStatus::Placeholder => {
                errors.push(ValidationError {
                    message: format!(
                        "[{}] required '{key}' ({}) contains only placeholder content",
                        archetype.name,
                        canonical.describe()
                    ),
                    line: locator_line(doc, canonical, line_offset),
                    reason: ValidationReason::Placeholder,
                });
            }
            ContentStatus::Substantive => {}
        }
    }

    // Assert facet (FR-033/FR-034) — evaluated for required and optional
    // locators alike (an assert is a structural promise either way).
    if let Some(assert) = canonical.assert() {
        for failure in evaluate_assert(doc, canonical, assert, frontmatter) {
            let reason = match failure.reason {
                AssertReason::Assert => ValidationReason::Assert,
                AssertReason::UnresolvedField => ValidationReason::UnresolvedField,
            };
            // CR-097: a row-scoped failure leads with the row's own id, so two
            // cells failing the same check in one table are two distinguishable
            // findings rather than two byte-identical strings at one locus.
            let message = match &failure.row_id {
                Some(row_id) => format!(
                    "[{}] '{key}': {row_id}: {}",
                    archetype.name, failure.message
                ),
                None => format!("[{}] '{key}': {}", archetype.name, failure.message),
            };
            errors.push(ValidationError {
                message,
                line: failure
                    .line
                    .map(|l| to_doc_line(line_offset, l))
                    .or_else(|| locator_line(doc, canonical, line_offset)),
                reason,
            });
        }
    }
}

/// The 1-based document line of the section a locator addresses by name
/// (when it has one), for diagnostic attribution.
fn locator_line(
    doc: &QuireDocument,
    primitive: &LocatorPrimitive,
    line_offset: usize,
) -> Option<usize> {
    let name = match primitive {
        LocatorPrimitive::SectionBody { after_heading, .. } => Some(after_heading.as_str()),
        LocatorPrimitive::TableRow {
            under_section: Some(s),
            ..
        }
        | LocatorPrimitive::ListItem {
            under_section: Some(s),
            ..
        }
        | LocatorPrimitive::CodeBlock {
            under_section: Some(s),
            ..
        } => Some(s.as_str()),
        _ => None,
    }?;
    crate::query::section(doc, name).map(|s| to_doc_line(line_offset, s.start_line))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContentStatus {
    Missing,
    Empty,
    Placeholder,
    Substantive,
}

/// Classify a locator's resolved values as substantive / placeholder /
/// empty / missing. Placeholder set per FR-032: `TODO`, `TBD`,
/// unresolved `{{...}}`, case-insensitive `placeholder`, generic
/// empty-state phrases such as `none specified`, and empty tables/lists.
fn content_status(values: &[Value]) -> ContentStatus {
    if values.is_empty() {
        return ContentStatus::Missing;
    }
    // If every resolved value is empty/whitespace → Empty.
    // If every resolved value is placeholder-only → Placeholder.
    // Otherwise Substantive.
    let mut all_empty = true;
    let mut all_placeholder = true;
    for v in values {
        let s = value_to_text(v);
        let trimmed = s.trim();
        if trimmed.is_empty() {
            // empty contributes to both empty + placeholder buckets
            continue;
        }
        all_empty = false;
        if !is_placeholder(trimmed) {
            all_placeholder = false;
        }
    }
    if all_empty {
        ContentStatus::Empty
    } else if all_placeholder {
        ContentStatus::Placeholder
    } else {
        ContentStatus::Substantive
    }
}

fn value_to_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        Value::Array(a) if a.is_empty() => String::new(),
        other => other.to_string(),
    }
}

/// Refine the status of a `table_row` / `list_item` locator (FR-032-AC-9).
///
/// Returns `None` for any other locator kind (caller falls back to the
/// generic `content_status`). For a `table_row`/`list_item`:
/// - `Substantive` when it yielded data values (caller still re-checks
///   placeholder/empty content via `content_status`, so we only special-
///   case the zero-value case here),
/// - `Empty` when the substrate **resolves** (the table/list exists under
///   the named section, or a table is present for `under_section: None`)
///   but has no data rows / items,
/// - `Missing` when the substrate does not resolve at all.
fn table_or_list_status(
    doc: &QuireDocument,
    primitive: &LocatorPrimitive,
    values: &[Value],
) -> Option<ContentStatus> {
    // Only zero-value table/list locators need the empty-vs-missing
    // distinction; once values exist, defer to `content_status`.
    if !values.is_empty() {
        return None;
    }
    match primitive {
        LocatorPrimitive::TableRow { under_section, .. } => {
            let substrate_present = match under_section {
                Some(name) => crate::query::table_from_section(doc, name).is_some(),
                None => doc
                    .sections
                    .iter()
                    .any(|s| !crate::query::parse_tables(&s.content).is_empty()),
            };
            Some(if substrate_present {
                ContentStatus::Empty
            } else {
                ContentStatus::Missing
            })
        }
        LocatorPrimitive::ListItem { under_section, .. } => {
            let section_present = match under_section {
                Some(name) => crate::query::section(doc, name).is_some(),
                // For under_section:None the substrate is the joined body;
                // it is "present" whenever the document has any section.
                None => !doc.sections.is_empty(),
            };
            Some(if section_present {
                ContentStatus::Empty
            } else {
                ContentStatus::Missing
            })
        }
        _ => None,
    }
}

/// Whether `text` (already trimmed, non-empty) is placeholder-only.
///
/// Placeholder sentinel set (FR-032-AC-7, decision 2026-06-04). The set
/// is **reduced**: bare `none` and `n/a` are NOT sentinels — they reject
/// legitimate content such as `Upstream: none` (FR-032-AC-8). The exact
/// set is:
///
/// - `TODO` / `TBD` — case-insensitive, matched as a **prefix**.
/// - `{{…}}` — whole-value unresolved template marker.
/// - `placeholder` — whole-value, case-insensitive.
/// - `none specified` — whole-value, case-insensitive.
/// - empty value — handled by the caller (the empty string).
fn is_placeholder(text: &str) -> bool {
    let lower = text.to_lowercase();
    // Whole-value sentinels.
    const WHOLE_VALUE_SENTINELS: &[&str] = &["placeholder", "none specified"];
    if WHOLE_VALUE_SENTINELS.contains(&lower.as_str()) {
        return true;
    }
    // Unresolved Jinja mustache: the whole content is one `{{...}}`.
    let stripped = lower.trim();
    if stripped.starts_with("{{") && stripped.ends_with("}}") {
        return true;
    }
    // A value whose content begins with a sentinel keyword (e.g.
    // "TODO: ...", "TBD"). Bare `none`/`n/a` are intentionally absent.
    if lower.starts_with("todo") || lower.starts_with("tbd") {
        return true;
    }
    false
}

/// FR-035: within the document, no two headings share text at the same
/// level. Same text at different levels is allowed (level disambiguates).
fn check_heading_uniqueness(
    doc: &QuireDocument,
    line_offset: usize,
    errors: &mut Vec<ValidationError>,
) {
    // (level, text) -> first start_line seen. A second occurrence is a
    // duplicate, reported at the *second* heading's line (FR-035-AC-4).
    let mut seen: BTreeMap<(u8, String), usize> = BTreeMap::new();
    let mut all: Vec<&QuireSection> = Vec::new();
    collect(&doc.sections, &mut all);
    // Document order so the "second" heading is the later one.
    all.sort_by_key(|s| s.start_line);
    for s in all {
        let key = (s.level, s.heading.clone());
        match seen.entry(key) {
            std::collections::btree_map::Entry::Occupied(_) => {
                errors.push(ValidationError {
                    message: format!(
                        "duplicate heading '{}' at level {} (heading text must be unique per level)",
                        s.heading, s.level
                    ),
                    line: Some(to_doc_line(line_offset, s.start_line)),
                    reason: ValidationReason::DuplicateHeading,
                });
            }
            std::collections::btree_map::Entry::Vacant(v) => {
                v.insert(s.start_line);
            }
        }
    }
}

fn collect<'a>(sections: &'a [QuireSection], out: &mut Vec<&'a QuireSection>) {
    for s in sections {
        out.push(s);
        collect(&s.children, out);
    }
}

/// Validate a JSON `value` against the archetype's primary schema
/// (FR-002 context path). This is the **explicitly selected** legacy
/// path — it validates a JSON object and does **not** parse markdown,
/// distinct from [`validate_document`] (FR-032-AC-5). Provided here as a
/// thin alias so the two entry points sit side by side; callers may also
/// use [`crate::validate`] directly.
pub fn validate_context(
    archetype: &CompiledArchetype,
    value: &Map<String, Value>,
) -> Result<(), crate::error::QuireError> {
    crate::validate(archetype, &Value::Object(value.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::compile::{compile_schema, ArchetypeCarryOver, CompiledArchetype};
    use ix_trace_rs::trace;
    use std::sync::Arc;

    /// Build a CompiledArchetype with an optional frontmatter schema and
    /// an optional body_extraction DSL for tests.
    fn archetype(frontmatter_schema: Option<Value>, dsl_yaml: Option<&str>) -> CompiledArchetype {
        let (raw_schema, validator, fv) = match frontmatter_schema {
            Some(s) => {
                let v = Arc::new(compile_schema(&s).expect("compile"));
                let raw = Arc::new(s);
                (Arc::clone(&raw), Arc::clone(&v), Some((raw, v)))
            }
            None => {
                let s = Value::Object(Map::new());
                let v = Arc::new(compile_schema(&s).expect("compile"));
                (Arc::new(s), v, None)
            }
        };
        let body_extraction = dsl_yaml.map(|y| serde_yaml::from_str(y).expect("dsl"));
        let (frontmatter_schema, frontmatter_validator) = match fv {
            Some((raw, v)) => (Some(raw), Some(v)),
            None => (None, None),
        };
        CompiledArchetype {
            name: "FR".into(),
            module: "test".into(),
            raw_schema,
            validator,
            frontmatter_schema,
            frontmatter_validator,
            data_schema: None,
            data_validator: None,
            body_extraction,
            carry_over: ArchetypeCarryOver::default(),
        }
    }

    const FR_SCHEMA: &str = r#"{
        "type": "object",
        "required": ["id", "title"],
        "properties": {"id": {"type": "string"}, "title": {"type": "string"}}
    }"#;

    const FR_DSL: &str = r#"
yield_pattern:
  match:
    description:
      from: section_body
      after_heading: Description
      required: true
    specification:
      from: section_body
      after_heading: Specification
      required: true
"#;

    fn fr_schema_value() -> Value {
        serde_json::from_str(FR_SCHEMA).unwrap()
    }

    const FR_DSL_DESC: &str = r#"
yield_pattern:
  match:
    description:
      from: section_body
      after_heading: Description
      required: true
"#;

    #[trace("TC-663", "FR-042-AC-7")]
    // grammar warnings never fail validation; an
    // error-severity finding routes to `errors` and would block.
    #[test]
    fn tc663_grammar_severity_routing() {
        // End-to-end: an FR bound to the EARS grammar (`grammar_ref`) with a
        // vague Description yields a Grammar *warning* yet still validates.
        let mut a = archetype(Some(fr_schema_value()), Some(FR_DSL_DESC));
        a.carry_over.grammar_ref = Some("iso-spec-core".into());
        let doc = "---\nid: FR-001\ntitle: A Thing\n---\n\
                   ## Description\nThe system shall support publishing.\n";
        let r = validate_document(&a, doc);
        assert!(
            r.is_valid,
            "advisory grammar findings must not fail validation"
        );
        assert!(r
            .warnings
            .iter()
            .any(|w| w.reason == ValidationReason::Grammar));

        // Routing: an Error-severity finding lands in `errors` (would block).
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        route_grammar_finding(
            crate::grammar::GrammarFinding {
                grammar: "ears".into(),
                check: "non-singular".into(),
                pattern: None,
                message: "x".into(),
                line: Some(3),
                statement: "s".into(),
                severity: crate::grammar::GrammarSeverity::Error,
            },
            &mut errors,
            &mut warnings,
        );
        assert_eq!(errors.len(), 1);
        assert!(warnings.is_empty());
        assert_eq!(errors[0].reason, ValidationReason::Grammar);
    }

    #[trace("TC-718", "FR-048-AC-3")]
    // with `ac:unclassifiable` mapped to `error`, that
    // finding lands in `ValidationResult.errors` and clears `is_valid`, while
    // an `ears` finding with no map entry stays a warning. The `ac` grammar
    // itself lands in Task-002; this pins the framework's severity-application
    // + routing contract that FR-047's checks ride on.
    #[test]
    fn tc718_per_check_error_routing() {
        let mut map = crate::grammar::GrammarSeverityMap::new();
        map.insert(
            "ac:unclassifiable".into(),
            crate::grammar::GrammarSeverityLevel::Error,
        );
        let emitted = vec![
            crate::grammar::GrammarFinding {
                grammar: "ac".into(),
                check: "unclassifiable".into(),
                pattern: Some("unclassifiable".into()),
                message: "criteria cell matches no canonical shape".into(),
                line: Some(9),
                statement: "It works".into(),
                severity: crate::grammar::GrammarSeverity::Warning,
            },
            crate::grammar::GrammarFinding {
                grammar: "ears".into(),
                check: "vague-response".into(),
                pattern: None,
                message: "vague response".into(),
                line: Some(4),
                statement: "The system shall support it.".into(),
                severity: crate::grammar::GrammarSeverity::Warning,
            },
        ];

        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        for f in crate::grammar::apply_severity(emitted, &map) {
            route_grammar_finding(f, &mut errors, &mut warnings);
        }

        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("[ac:unclassifiable]"));
        assert_eq!(errors[0].reason, ValidationReason::Grammar);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("[ears:vague-response]"));
        let result = ValidationResult::new(errors, warnings);
        assert!(!result.is_valid);
    }

    #[trace("TC-528", "FR-032-AC-1")]
    // conformant document validates.
    #[test]
    fn tc528_conformant_document_is_valid() {
        let a = archetype(Some(fr_schema_value()), Some(FR_DSL));
        let doc = "---\nid: FR-001\ntitle: A Thing\n---\n\
                   ## Description\nReal description content.\n\
                   ## Specification\nReal specification content.\n";
        let r = validate_document(&a, doc);
        assert!(r.is_valid, "{:?}", r.errors);
        assert!(r.errors.is_empty());
    }

    #[trace("TC-529", "FR-032-AC-2")]
    // missing required section → reason missing,
    // line-numbered, names archetype + section.
    #[test]
    fn tc529_missing_required_section() {
        let a = archetype(Some(fr_schema_value()), Some(FR_DSL));
        let doc = "---\nid: FR-001\ntitle: A Thing\n---\n\
                   ## Description\nReal content.\n";
        let r = validate_document(&a, doc);
        assert!(!r.is_valid);
        let e = r
            .errors
            .iter()
            .find(|e| e.reason == ValidationReason::Missing)
            .expect("missing error");
        assert!(e.message.contains("FR"), "{}", e.message);
        assert!(e.message.contains("Specification"), "{}", e.message);
    }

    #[trace("TC-530", "FR-032-AC-3")]
    // placeholder-only required section → reason
    // placeholder, even when frontmatter schema passes.
    #[test]
    fn tc530_placeholder_section() {
        let a = archetype(Some(fr_schema_value()), Some(FR_DSL));
        let doc = "---\nid: FR-001\ntitle: A Thing\n---\n\
                   ## Description\nReal content.\n\
                   ## Specification\nTODO\n";
        let r = validate_document(&a, doc);
        assert!(!r.is_valid);
        assert!(r
            .errors
            .iter()
            .any(|e| e.reason == ValidationReason::Placeholder));

        // `{{...}}` is also placeholder.
        let doc2 = "---\nid: FR-001\ntitle: A Thing\n---\n\
                    ## Description\nReal content.\n\
                    ## Specification\n{{ specification }}\n";
        let r2 = validate_document(&a, doc2);
        assert!(r2
            .errors
            .iter()
            .any(|e| e.reason == ValidationReason::Placeholder));
    }

    #[trace("TC-531", "FR-032-AC-4")]
    // frontmatter violation → reason frontmatter,
    // independent of body.
    #[test]
    fn tc531_frontmatter_violation() {
        let a = archetype(Some(fr_schema_value()), Some(FR_DSL));
        // Missing required `title`.
        let doc = "---\nid: FR-001\n---\n\
                   ## Description\nReal content.\n\
                   ## Specification\nReal content.\n";
        let r = validate_document(&a, doc);
        assert!(!r.is_valid);
        assert!(r
            .errors
            .iter()
            .any(|e| e.reason == ValidationReason::Frontmatter));
    }

    #[trace("TC-532", "FR-032-AC-5")]
    // the context/data path validates a JSON
    // object and does not parse markdown.
    #[test]
    fn tc532_context_path_is_distinct() {
        let a = archetype(Some(fr_schema_value()), Some(FR_DSL));
        // Valid JSON object → Ok, no markdown parsing.
        let mut obj = Map::new();
        obj.insert("id".into(), Value::String("FR-1".into()));
        obj.insert("title".into(), Value::String("T".into()));
        assert!(validate_context(&a, &obj).is_ok());
        // Invalid JSON object → Err (schema violation), still no markdown.
        let mut bad = Map::new();
        bad.insert("id".into(), Value::String("FR-1".into()));
        assert!(validate_context(&a, &bad).is_err());
    }

    #[trace("TC-533", "FR-032-AC-6")]
    // archetype with no body_extraction validates
    // by frontmatter + heading-uniqueness only; no body-structure errors.
    #[test]
    fn tc533_no_body_extraction_only_frontmatter_and_headings() {
        let a = archetype(Some(fr_schema_value()), None);
        let doc = "---\nid: FR-001\ntitle: T\n---\n## Anything\nwhatever\n";
        let r = validate_document(&a, doc);
        assert!(r.is_valid, "{:?}", r.errors);
        // No frontmatter schema → still no body diagnostics.
        let a2 = archetype(None, None);
        let r2 = validate_document(&a2, doc);
        assert!(r2.is_valid, "{:?}", r2.errors);
    }

    // A multi-yield archetype: iterate the bullet-list items under
    // `## Algorithms` (one record per item). Each unit requires its own
    // `heading` (the list-item text, which the iterator places at the
    // unit-scope root) and asserts it matches `^Algo-\d+:` — a per-unit
    // required-locator + assert combination (FR-032 step 3, FR-033).
    // List-item iteration keeps every unit scope-local (no `raw`-wide
    // code_block leakage, no per-level heading collisions).
    const MULTI_DSL: &str = r#"
yield_pattern:
  iterate_over:
    section_path: [Algorithms]
    kind: list_item
  per_match:
    name:
      from: heading
      required: true
      assert:
        id_pattern: '^Algo-\d+:'
"#;

    #[trace("TC-561")]
    // multi-yield archetype — a (FR-032 step 3 / FR-033)
    // conformant document (every iteration unit carries its required
    // sub-locator with a satisfied assert) validates.
    #[test]
    fn tc561_multi_yield_conformant_validates() {
        let a = archetype(None, Some(MULTI_DSL));
        let doc = "## Algorithms\n\
                   - Algo-1: first algorithm\n\
                   - Algo-2: second algorithm\n";
        let r = validate_document(&a, doc);
        assert!(r.is_valid, "{:?}", r.errors);
    }

    // A second multi-yield archetype exercising the **missing required
    // sub-locator** path: iterate `### child` headings under
    // `## Steps`, each unit requiring a `#### Detail` (level-4) heading
    // inside it. A unit without that child sub-heading fails `missing`.
    const MULTI_DSL_MISSING: &str = r#"
yield_pattern:
  iterate_over:
    section_path: [Steps]
    kind: heading
    depth: 1
  per_match:
    detail:
      from: heading
      level: 4
      required: true
"#;

    #[trace("TC-561")]
    // a unit missing its required per_match sub-locator (no
    // `#### Detail` child) fails with reason `missing`, naming the key.
    #[test]
    fn tc561_multi_yield_unit_missing_required_fails() {
        let a = archetype(None, Some(MULTI_DSL_MISSING));
        // Unit `Two` has no `#### ...` child heading.
        let doc = "## Steps\n\
                   ### One\n#### Detail one\nx\n\
                   ### Two\nno detail child\n";
        let r = validate_document(&a, doc);
        assert!(!r.is_valid);
        let e = r
            .errors
            .iter()
            .find(|e| e.reason == ValidationReason::Missing)
            .expect("missing per_match locator");
        assert!(e.message.contains("detail"), "{}", e.message);
    }

    // The same archetype: when every unit carries its required child, the
    // document validates (distinct `#### Detail N` headings avoid the
    // per-level uniqueness rule, FR-035).
    #[test]
    fn tc561_multi_yield_missing_dsl_conformant_validates() {
        let a = archetype(None, Some(MULTI_DSL_MISSING));
        let doc = "## Steps\n\
                   ### One\n#### Detail one\nx\n\
                   ### Two\n#### Detail two\ny\n";
        let r = validate_document(&a, doc);
        assert!(r.is_valid, "{:?}", r.errors);
    }

    #[trace("TC-562", "FR-033")]
    // a unit whose required sub-locator violates its
    // assert (list-item text does not match the id_pattern) fails with
    // reason `assert`; a conformant sibling does not.
    #[test]
    fn tc562_multi_yield_unit_assert_fails() {
        let a = archetype(None, Some(MULTI_DSL));
        let doc = "## Algorithms\n\
                   - Algo-1: ok\n\
                   - Bogus: not matching\n";
        let r = validate_document(&a, doc);
        assert!(!r.is_valid);
        assert!(
            r.errors
                .iter()
                .any(|e| e.reason == ValidationReason::Assert),
            "{:?}",
            r.errors
        );
    }

    #[trace("TC-544", "FR-035-AC-1")]
    // two `## Description` → duplicate-heading,
    // names text + level 2.
    #[test]
    fn tc544_duplicate_heading_level_2() {
        let a = archetype(None, None);
        let doc = "## Description\nfirst\n## Description\nsecond\n";
        let r = validate_document(&a, doc);
        assert!(!r.is_valid);
        let e = r
            .errors
            .iter()
            .find(|e| e.reason == ValidationReason::DuplicateHeading)
            .expect("dup");
        assert!(e.message.contains("Description"), "{}", e.message);
        assert!(e.message.contains("level 2"), "{}", e.message);
    }

    #[trace("TC-545", "FR-035-AC-2")]
    // same text at different levels passes.
    #[test]
    fn tc545_same_text_different_levels_ok() {
        let a = archetype(None, None);
        let doc = "## Properties\nx\n### Properties\ny\n";
        let r = validate_document(&a, doc);
        assert!(
            !r.errors
                .iter()
                .any(|e| e.reason == ValidationReason::DuplicateHeading),
            "{:?}",
            r.errors
        );
    }

    #[trace("TC-546", "FR-035-AC-3")]
    // iterate_over distinct child headings pass;
    // a duplicate child fails.
    #[test]
    fn tc546_iterate_over_children() {
        let a = archetype(None, None);
        let ok = "## Algorithms\n### A\nx\n### B\ny\n";
        assert!(validate_document(&a, ok)
            .errors
            .iter()
            .all(|e| e.reason != ValidationReason::DuplicateHeading));
        let bad = "## Algorithms\n### A\nx\n### A\ny\n";
        let r = validate_document(&a, bad);
        assert!(r
            .errors
            .iter()
            .any(|e| e.reason == ValidationReason::DuplicateHeading));
    }

    #[trace("TC-547", "FR-035-AC-4")]
    // the duplicate diagnostic carries the line of
    // the offending (second) heading.
    #[test]
    fn tc547_duplicate_line_is_second_heading() {
        let a = archetype(None, None);
        // Frontmatter pushes body down: lines (1-based doc):
        // 1 ---, 2 id, 3 ---, 4 ## Description, 5 first, 6 ## Description
        let doc = "---\nid: x\n---\n## Description\nfirst\n## Description\nsecond\n";
        let r = validate_document(&a, doc);
        let e = r
            .errors
            .iter()
            .find(|e| e.reason == ValidationReason::DuplicateHeading)
            .expect("dup");
        assert_eq!(e.line, Some(6), "second heading is on doc line 6");
    }

    // ── Placeholder sentinel set + empty/missing reason (FR-032-AC-7..9) ──

    const SPEC_DSL: &str = r#"
yield_pattern:
  match:
    specification:
      from: section_body
      after_heading: Specification
      required: true
"#;

    fn spec_doc(body: &str) -> String {
        format!("## Specification\n{body}\n")
    }

    #[trace("TC-573", "FR-032-AC-7")]
    // the exact placeholder sentinel set. `TODO:`
    // /`TBD` prefix (case-insensitive) and whole-value `{{…}}` /
    // `placeholder` / `none specified` / empty fail with reason
    // `placeholder`; substantive prose merely containing `todo`
    // mid-sentence or an embedded `{{x}}` token does NOT.
    #[test]
    fn tc573_placeholder_sentinel_set_exact() {
        let a = archetype(None, Some(SPEC_DSL));
        let placeholders = [
            "TODO: fill this in",
            "tbd",
            "{{ specification }}",
            "Placeholder",
            "none specified",
        ];
        for body in placeholders {
            let r = validate_document(&a, &spec_doc(body));
            assert!(
                r.errors
                    .iter()
                    .any(|e| e.reason == ValidationReason::Placeholder),
                "expected placeholder for {body:?}, got {:?}",
                r.errors
            );
        }
        // Substantive prose that merely mentions the words / embeds a token.
        let substantive = [
            "We will not do the todo list shuffle here; this is real content.",
            "The id field interpolates as {{id}} inside otherwise real prose.",
        ];
        for body in substantive {
            let r = validate_document(&a, &spec_doc(body));
            assert!(
                r.is_valid,
                "expected substantive for {body:?}, got {:?}",
                r.errors
            );
        }
    }

    #[trace("TC-574", "FR-032-AC-8")]
    // a required section whose only content is
    // `none` or `n/a` is substantive and passes — bare `none`/`n/a` are
    // not sentinels.
    #[test]
    fn tc574_bare_none_and_na_are_substantive() {
        let a = archetype(None, Some(SPEC_DSL));
        for body in ["none", "n/a", "Upstream: none"] {
            let r = validate_document(&a, &spec_doc(body));
            assert!(
                r.is_valid,
                "expected substantive for {body:?}, got {:?}",
                r.errors
            );
        }
    }

    const TABLE_DSL: &str = r#"
yield_pattern:
  match:
    rows:
      from: table_row
      under_section: Acceptance Criteria
      required: true
"#;

    const LIST_DSL: &str = r#"
yield_pattern:
  match:
    items:
      from: list_item
      under_section: Dependencies
      required: true
"#;

    #[trace("TC-575", "FR-032-AC-9")]
    // a required `table_row` resolving to a
    // header-only table fails `empty`; a required `list_item` resolving to
    // an item-less list fails `empty`; a non-resolving locator fails
    // `missing` (none report `placeholder`).
    #[test]
    fn tc575_empty_table_and_list_reason_is_empty_not_placeholder() {
        // Header-only table under a present section → empty.
        let a = archetype(None, Some(TABLE_DSL));
        let doc = "## Acceptance Criteria\n\
                   | ID | Criteria |\n\
                   |----|----------|\n";
        let r = validate_document(&a, doc);
        assert!(
            r.errors.iter().any(|e| e.reason == ValidationReason::Empty),
            "{:?}",
            r.errors
        );
        assert!(r
            .errors
            .iter()
            .all(|e| e.reason != ValidationReason::Placeholder));

        // Section absent entirely → missing.
        let r_missing = validate_document(&a, "## Other\nx\n");
        assert!(r_missing
            .errors
            .iter()
            .any(|e| e.reason == ValidationReason::Missing));

        // Item-less list under a present section → empty.
        let a2 = archetype(None, Some(LIST_DSL));
        let doc2 = "## Dependencies\n\nNo bullet items here, just prose.\n";
        let r2 = validate_document(&a2, doc2);
        assert!(
            r2.errors
                .iter()
                .any(|e| e.reason == ValidationReason::Empty),
            "{:?}",
            r2.errors
        );
        // Section absent → missing.
        let r2_missing = validate_document(&a2, "## Other\nx\n");
        assert!(r2_missing
            .errors
            .iter()
            .any(|e| e.reason == ValidationReason::Missing));
    }

    const OPTIONAL_ASSERT_DSL: &str = r#"
yield_pattern:
  match:
    code:
      from: section_body
      after_heading: Code
      required: false
      assert:
        id_pattern: '^OK-'
"#;

    #[trace("TC-576", "FR-032-AC-10")]
    // an `assert` on a **resolved** locator is
    // evaluated regardless of `required`. An optional locator that
    // resolves but violates its assert → reason `assert`; an optional
    // locator that does not resolve runs no assert and emits nothing.
    #[test]
    fn tc576_assert_on_resolved_optional_locator() {
        let a = archetype(None, Some(OPTIONAL_ASSERT_DSL));
        // Resolves, violates id_pattern → assert failure.
        let bad = "## Code\nNOPE-123 content\n";
        let r = validate_document(&a, bad);
        assert!(
            r.errors
                .iter()
                .any(|e| e.reason == ValidationReason::Assert),
            "{:?}",
            r.errors
        );
        // Resolves, satisfies → valid.
        let good = "## Code\nOK-123 content\n";
        assert!(validate_document(&a, good).is_valid);
        // Does not resolve (no `## Code`) → no assert, no diagnostic.
        let absent = "## Other\nx\n";
        let r_absent = validate_document(&a, absent);
        assert!(r_absent.is_valid, "{:?}", r_absent.errors);
    }

    // TC-576 regression (FR-032-AC-10): a `level` assert on an OPTIONAL
    // section_body locator must not leak a 'section not found' failure
    // when the section is absent. (Value-shaped asserts already
    // no-opped on zero values; the level assert checked the section by
    // name and reported the miss.)
    #[test]
    fn tc576_level_assert_on_absent_optional_section_is_silent() {
        const DSL: &str = r#"
yield_pattern:
  match:
    inputs:
      from: section_body
      after_heading: Inputs
      required: false
      assert:
        level: 2
"#;
        let a = archetype(None, Some(DSL));
        let r = validate_document(&a, "## Other\nprose\n");
        assert!(r.is_valid, "{:?}", r.errors);
        // Present at the wrong level still fails.
        let r_wrong = validate_document(&a, "## Wrap\n### Inputs\nx\n");
        assert!(r_wrong
            .errors
            .iter()
            .any(|e| e.reason == ValidationReason::Assert));
    }

    // ── Composed type+object validation (FR-032-AC-11..13) ──────────
    //
    // A small inline registry with an `FR` artifact_type (requires a
    // substantive `## Specification` section) and a `process`-like
    // object_type (requires a mermaid `diagram` code_block under
    // `## Workflow`) — mirrors the spec-objects-business `process`
    // archetype's `body_extraction`.

    /// Build a `Registry` with both an `FR` artifact_type and a `process`
    /// object_type for the composed-validation tests.
    fn composed_registry() -> crate::Registry {
        let manifest = br#"
name: composed-test
artifact_types:
- name: FR
  frontmatter_schema_ref: schemas/fr.schema.json
  body_extraction:
    yield_pattern:
      match:
        specification:
          from: section_body
          after_heading: Specification
          required: true
object_types:
- name: process
  data_schema:
    type: object
  body_extraction:
    yield_pattern:
      match:
        diagram:
          from: code_block
          after_heading: Workflow
          required: true
          language: mermaid
"#;
        let mut schemas = BTreeMap::new();
        schemas.insert(
            "schemas/fr.schema.json".to_string(),
            r#"{"type":"object","required":["id","title"],"properties":{"id":{"type":"string"},"title":{"type":"string"}}}"#
                .to_string(),
        );
        let r = crate::Registry::from_inline_parts(manifest, &schemas).expect("inline registry");
        assert!(r.failures().is_empty(), "{:?}", r.failures());
        assert!(r.archetype("FR").is_some(), "FR archetype loaded");
        assert!(r.archetype("process").is_some(), "process archetype loaded");
        r
    }

    #[trace("TC-610", "FR-032-AC-11", "FR-032-AC-13")]
    // `type: FR` + `object: process`
    // with the FR core present but NO `## Workflow` mermaid block → an
    // object ERROR (process required `diagram` missing) merged into
    // `errors`, while the FR part passes independently; is_valid==false.
    #[test]
    fn tc610_composed_object_missing_diagram_is_error() {
        let reg = composed_registry();
        let fr = reg.archetype("FR").expect("FR");
        // FR core conformant (substantive Specification); object: process
        // requires a `## Workflow` mermaid block, which is absent.
        let doc = "---\nid: FR-001\ntitle: A Thing\nobject: process\n---\n\
                   ## Specification\nThe system SHALL do a real, concrete thing.\n";
        let r = validate_document_in_registry(&reg, fr, doc);
        assert!(!r.is_valid, "expected invalid, got {:?}", r);
        // Exactly the object error (process diagram missing) — no FR errors.
        assert!(
            r.errors
                .iter()
                .any(|e| e.message.contains("process") && e.reason == ValidationReason::Missing),
            "expected a process 'diagram' missing error, got {:?}",
            r.errors
        );
        // The FR (type) portion passes — no FR-tagged error.
        assert!(
            !r.errors.iter().any(|e| e.message.contains("[FR]")),
            "FR portion should pass, got {:?}",
            r.errors
        );
        // No warnings: `process` resolved, so this is a hard error path.
        assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    }

    #[trace("TC-611", "FR-032-AC-12")]
    // `type: FR` (conformant) + `object:
    // totally-unknown` → exactly one WARNING (reason unknown-object-type,
    // naming the object), zero errors, is_valid==true.
    #[test]
    fn tc611_unknown_object_is_one_warning_zero_errors() {
        let reg = composed_registry();
        let fr = reg.archetype("FR").expect("FR");
        let doc = "---\nid: FR-001\ntitle: A Thing\nobject: totally-unknown\n---\n\
                   ## Specification\nThe system SHALL do a real, concrete thing.\n";
        let r = validate_document_in_registry(&reg, fr, doc);
        assert!(r.is_valid, "expected valid, got errors {:?}", r.errors);
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert_eq!(r.warnings.len(), 1, "exactly one warning: {:?}", r.warnings);
        let w = &r.warnings[0];
        assert_eq!(w.reason, ValidationReason::UnknownObjectType);
        assert!(
            w.message.contains("totally-unknown"),
            "warning names the object: {}",
            w.message
        );
    }

    #[trace("TC-612", "FR-032-AC-11")]
    // `type: FR` conformant + NO `object:` key
    // (registry-aware entry point) → no object-layer diagnostics at all;
    // errors + warnings unchanged from the type-only path.
    #[test]
    fn tc612_no_object_key_no_object_diagnostics() {
        let reg = composed_registry();
        let fr = reg.archetype("FR").expect("FR");
        let doc = "---\nid: FR-001\ntitle: A Thing\n---\n\
                   ## Specification\nThe system SHALL do a real, concrete thing.\n";
        let composed = validate_document_in_registry(&reg, fr, doc);
        assert!(composed.is_valid, "{:?}", composed.errors);
        assert!(composed.warnings.is_empty(), "{:?}", composed.warnings);
        // Identical errors to the type-only path (no object layer ran).
        let type_only = validate_document(fr, doc);
        assert_eq!(composed.errors, type_only.errors);
    }

    /// Registry with FR + aggregate_root each declaring `allowed_links`
    /// plus the `edge_types` registry, for the Tier-1 edge-vocabulary
    /// tests (FR-040-AC-8).
    fn edge_vocab_registry() -> crate::Registry {
        let manifest = br#"
name: edge-vocab-test
artifact_types:
- name: FR
  frontmatter_schema_ref: schemas/fr.schema.json
  allowed_links: [implements, references]
object_types:
- name: aggregate_root
  allowed_links:
    emits: [event]
edge_types:
  implements: { description: x, category: dependency }
  references: { description: x, category: traceability }
  emits: { description: x, category: dataflow }
"#;
        let mut schemas = BTreeMap::new();
        schemas.insert(
            "schemas/fr.schema.json".to_string(),
            r#"{"type":"object","required":["id","title"],"properties":{"id":{"type":"string"},"title":{"type":"string"}}}"#
                .to_string(),
        );
        let r = crate::Registry::from_inline_parts(manifest, &schemas).expect("inline registry");
        assert!(r.failures().is_empty(), "{:?}", r.failures());
        r
    }

    #[trace("TC-641", "FR-040-AC-8")]
    // a frontmatter edge whose `type` is outside the
    // resolved (artifact ∪ object) vocabulary yields exactly one
    // DisallowedEdgeType warning; in-vocabulary edges yield none; the
    // object axis contributes its verb (`emits`) to the resolved set.
    #[test]
    fn tc641_disallowed_edge_type_is_one_warning() {
        let reg = edge_vocab_registry();
        let fr = reg.archetype("FR").expect("FR");
        // `implements` ∈ FR axis, `emits` ∈ aggregate_root axis (valid);
        // `teleports` ∈ neither (flagged).
        let doc = "---\nid: FR-001\ntitle: A Thing\nobject: aggregate_root\n\
                   relationships:\n\
                   - target: ix://o/r/US-001\n  type: implements\n\
                   - target: ix://o/r/EV-001\n  type: emits\n\
                   - target: ix://o/r/X-001\n  type: teleports\n---\n\
                   ## Body\nText.\n";
        let r = validate_document_in_registry(&reg, fr, doc);
        let edge_warnings: Vec<_> = r
            .warnings
            .iter()
            .filter(|w| w.reason == ValidationReason::DisallowedEdgeType)
            .collect();
        assert_eq!(
            edge_warnings.len(),
            1,
            "exactly one disallowed edge, got {:?}",
            r.warnings
        );
        assert!(
            edge_warnings[0].message.contains("teleports")
                && edge_warnings[0].message.contains("FR-001"),
            "warning names verb + source: {}",
            edge_warnings[0].message
        );
        // Warn-tier: edges never fail validation.
        assert!(r.is_valid, "edges are advisory: {:?}", r.errors);
    }

    #[trace("TC-644", "FR-040-AC-10")]
    // Tier-1 disallowed-edge warnings are emitted
    // sorted by (target, edge_type), independent of the author's
    // `relationships:` ordering.
    #[test]
    fn tc644_tier1_warnings_are_sorted() {
        let reg = edge_vocab_registry();
        let fr = reg.archetype("FR").expect("FR");
        // Three out-of-vocab verbs listed in deliberately unsorted order.
        let doc = "---\nid: FR-9\ntitle: T\n\
                   relationships:\n\
                   - target: ix://o/r/Z-1\n  type: zaps\n\
                   - target: ix://o/r/A-1\n  type: yanks\n\
                   - target: ix://o/r/M-1\n  type: xeroxes\n---\n\
                   ## Body\nText.\n";
        let r = validate_document_in_registry(&reg, fr, doc);
        let targets: Vec<&str> = r
            .warnings
            .iter()
            .filter(|w| w.reason == ValidationReason::DisallowedEdgeType)
            .map(|w| {
                // message embeds "target '<id>'"
                let s = w.message.split("target '").nth(1).unwrap();
                s.split('\'').next().unwrap()
            })
            .collect();
        assert_eq!(
            targets,
            vec!["ix://o/r/A-1", "ix://o/r/M-1", "ix://o/r/Z-1"],
            "sorted by target"
        );
    }

    #[trace("TC-641b", "FR-040-AC-8")]
    // when `object:` is unknown, the vocabulary
    // falls back to the artifact axis alone and Tier-1 still runs.
    #[test]
    fn tc641_unknown_object_falls_back_to_artifact_vocab() {
        let reg = edge_vocab_registry();
        let fr = reg.archetype("FR").expect("FR");
        // `emits` is only on the (now-unresolved) object axis → flagged;
        // `implements` is on the FR axis → fine.
        let doc = "---\nid: FR-002\ntitle: A Thing\nobject: not-a-real-type\n\
                   relationships:\n\
                   - target: ix://o/r/US-001\n  type: implements\n\
                   - target: ix://o/r/EV-001\n  type: emits\n---\n\
                   ## Body\nText.\n";
        let r = validate_document_in_registry(&reg, fr, doc);
        let edge_warnings: Vec<_> = r
            .warnings
            .iter()
            .filter(|w| w.reason == ValidationReason::DisallowedEdgeType)
            .collect();
        assert_eq!(edge_warnings.len(), 1, "{:?}", r.warnings);
        assert!(edge_warnings[0].message.contains("emits"));
        // Plus the unknown-object-type warning is still present.
        assert!(r
            .warnings
            .iter()
            .any(|w| w.reason == ValidationReason::UnknownObjectType));
    }

    #[trace("TC-613", "FR-032-AC-13")]
    // `type: FR` + `object: process` WITH a valid
    // `## Workflow` mermaid block → no object errors, no warnings,
    // is_valid==true.
    #[test]
    fn tc613_composed_conformant_validates() {
        let reg = composed_registry();
        let fr = reg.archetype("FR").expect("FR");
        let doc = "---\nid: FR-001\ntitle: A Thing\nobject: process\n---\n\
                   ## Specification\nThe system SHALL do a real, concrete thing.\n\
                   ## Workflow\n```mermaid\nflowchart TD\n  A --> B\n```\n";
        let r = validate_document_in_registry(&reg, fr, doc);
        assert!(r.is_valid, "expected valid, got errors {:?}", r.errors);
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    }
}
