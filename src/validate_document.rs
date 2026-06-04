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

/// Outcome of [`validate_document`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    fn from_errors(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: errors.is_empty(),
            errors,
        }
    }
}

/// Validate an authored markdown `doc_text` against `archetype` (FR-032).
///
/// Frontmatter-schema success is **necessary but not sufficient**: a
/// document with valid frontmatter but a missing/placeholder required
/// section, a failed assert, or a duplicate heading is invalid.
pub fn validate_document(archetype: &CompiledArchetype, doc_text: &str) -> ValidationResult {
    let doc = crate::parse_document(doc_text);
    let line_offset = body_line_offset(doc_text);
    let mut errors: Vec<ValidationError> = Vec::new();

    validate_frontmatter(archetype, &doc, &mut errors);
    if let Some(dsl) = archetype.body_extraction() {
        validate_body(archetype, &doc, dsl, line_offset, &mut errors);
    }
    check_heading_uniqueness(&doc, line_offset, &mut errors);

    ValidationResult::from_errors(errors)
}

/// Number of body lines preceding the parsed body in the raw document —
/// the count of newlines consumed by any frontmatter block (plus a
/// leading BOM). Used to convert a section's 0-based body `start_line`
/// into a 1-based document line.
fn body_line_offset(doc_text: &str) -> usize {
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

    if locator.required() {
        let (values, _pos) = crate::extract::locator::eval_locator(doc, locator);
        match content_status(&values) {
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
            errors.push(ValidationError {
                message: format!("[{}] '{key}': {}", archetype.name, failure.message),
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

/// Whether `text` (already trimmed, non-empty) is placeholder-only.
fn is_placeholder(text: &str) -> bool {
    let lower = text.to_lowercase();
    // Exact / leading sentinels.
    const SENTINELS: &[&str] = &[
        "todo",
        "tbd",
        "placeholder",
        "none specified",
        "n/a",
        "none",
    ];
    if SENTINELS.contains(&lower.as_str()) {
        return true;
    }
    // Unresolved Jinja mustache: the whole content is one `{{...}}`.
    let stripped = lower.trim();
    if stripped.starts_with("{{") && stripped.ends_with("}}") {
        return true;
    }
    // A line that is solely a sentinel keyword (e.g. "TODO: ...").
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
            template_path: None,
            template_name: None,
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

    // TC-528 (FR-032-AC-1): conformant document validates.
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

    // TC-529 (FR-032-AC-2): missing required section → reason missing,
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

    // TC-530 (FR-032-AC-3): placeholder-only required section → reason
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

    // TC-531 (FR-032-AC-4): frontmatter violation → reason frontmatter,
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

    // TC-532 (FR-032-AC-5): the context/data path validates a JSON
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

    // TC-533 (FR-032-AC-6): archetype with no body_extraction validates
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

    // TC-561 (FR-032 step 3 / FR-033): multi-yield archetype — a
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

    // TC-561: a unit missing its required per_match sub-locator (no
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

    // TC-562 (FR-033): a unit whose required sub-locator violates its
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

    // TC-544 (FR-035-AC-1): two `## Description` → duplicate-heading,
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

    // TC-545 (FR-035-AC-2): same text at different levels passes.
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

    // TC-546 (FR-035-AC-3): iterate_over distinct child headings pass;
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

    // TC-547 (FR-035-AC-4): the duplicate diagnostic carries the line of
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
}
