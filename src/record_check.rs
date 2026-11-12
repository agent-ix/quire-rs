//! Located record validation, check half (FR-069 "Located record validation").
//!
//! The record `body_extraction.yield_pattern.match` yields (FR-011) is
//! validated against the archetype's `record_validation.schema`, and every
//! validator error becomes one `semantic.record-invalid` finding at the
//! document line of the value it concerns (FR-033-AC-18 lines). The record is
//! built by [`crate::extract::extract`]; this builds nothing of its own.

use std::collections::BTreeSet;

use jsonschema::error::ValidationErrorKind;
use serde_json::Value;

use crate::ast::QuireDocument;
use crate::extract::match_value_lines;
use crate::loader::compile::CompiledArchetype;
use crate::validate_document::{ValidationError, ValidationReason};

/// Where a finding with no value of its own falls when the absence locator
/// located nothing: the first line of the document.
const FIRST_LINE: usize = 1;

/// The `semantic.record-invalid` findings of `doc` under `archetype`'s
/// `record_validation`, in the validator's error order. Empty when the
/// archetype declares none.
///
/// `locator_keys` are the `match` keys that already have a locator finding in
/// this run; an error about one of them is that defect again and is not
/// reported twice. When extraction aborts because a `required: true` locator
/// located nothing there is no record, and the locator's own finding is the
/// report.
pub(crate) fn record_findings(
    archetype: &CompiledArchetype,
    doc: &QuireDocument,
    locator_keys: &BTreeSet<String>,
) -> Vec<ValidationError> {
    let (Some(declaration), Some(dsl)) =
        (&archetype.record_validation, archetype.body_extraction())
    else {
        return Vec::new();
    };
    let Some(match_map) = &dsl.yield_pattern.r#match else {
        return Vec::new();
    };
    let Ok(extracted) = crate::extract::extract(doc, dsl) else {
        return Vec::new();
    };
    // FR-011 yields no record when every locator is optional and none located
    // anything: that is the empty record, so an absence is still reported.
    let record = Value::Object(extracted.records.into_iter().next().unwrap_or_default());
    let lines = match_value_lines(doc, match_map);
    let absence = lines
        .get(&declaration.locate_absence_at)
        .and_then(|located| located.first().copied().flatten())
        .unwrap_or(FIRST_LINE);

    declaration
        .validator
        .iter_errors(&record)
        .filter_map(|err| {
            let at = Concerns::of(&err);
            if at.property.as_ref().is_some_and(|p| locator_keys.contains(p)) {
                return None;
            }
            let line = match (&at.property, at.absent) {
                (Some(property), false) => lines
                    .get(property)
                    .and_then(|located| {
                        at.element
                            .and_then(|i| located.get(i))
                            .or_else(|| located.first())
                            .copied()
                            .flatten()
                    })
                    .unwrap_or(absence),
                _ => absence,
            };
            let detail = crate::validate::schema_validation_detail(&err);
            Some(ValidationError {
                message: format!(
                    "semantic.record-invalid: [{}] yield record fails the record schema at {} ({}): {}",
                    archetype.name,
                    detail.path,
                    err.schema_path(),
                    detail.message
                ),
                line: Some(line),
                reason: ValidationReason::Semantic,
            })
        })
        .collect()
}

/// What a validator error is about, in terms of the record's keys.
struct Concerns {
    /// The record key the error concerns: the key a root `required` error
    /// names, else the first segment of the error's instance path. `None` for
    /// an error of the record itself (`additionalProperties`, ...).
    property: Option<String>,
    /// The array element index, when the error sits inside one.
    element: Option<usize>,
    /// A property that is missing, so it has no value to point at.
    absent: bool,
}

impl Concerns {
    fn of(err: &jsonschema::ValidationError<'_>) -> Self {
        let pointer = err.instance_path().to_string();
        let mut segments = pointer
            .split('/')
            .skip(1)
            .map(|s| s.replace("~1", "/").replace("~0", "~"));
        let first = segments.next();
        if first.is_none() {
            if let ValidationErrorKind::Required { property } = err.kind() {
                return Self {
                    property: property.as_str().map(str::to_string),
                    element: None,
                    absent: true,
                };
            }
        }
        Self {
            property: first,
            element: segments.next().and_then(|s| s.parse().ok()),
            absent: false,
        }
    }
}
