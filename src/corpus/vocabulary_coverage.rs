//! Declared-vocabulary coverage (FR-059).
//!
//! The generic primitive behind "does this corpus have a reliability
//! requirement at all":
//!
//! > Given a declared vocabulary and a declared projection from documents onto
//! > it, which vocabulary values does no document claim?
//!
//! ISO 25010 quality characteristics are one instance. Test-type coverage over
//! a Test Matrix and STRIDE-category coverage over declared threats are others,
//! and all three are the same walk with different declarations.
//!
//! Nothing about 25010 — or about any vocabulary — is engine knowledge. The
//! **vocabulary itself is read from the archetype's own frontmatter schema**
//! rather than restated in the manifest: `field` names a property whose `enum`
//! the schema already declares, and that enum *is* the vocabulary. A second
//! list in the manifest would be free to drift from the first, which is the
//! defect CR-015 closed.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::declared_tables::{relative_path, ExcludeSet};
use super::spec::Spec;
use super::validate::{pack, posture_tier, BundleFinding, BundlePosture, BundleReport};
use super::walk::LoadedDocument;
use crate::coverage::{CoverageDiagnostic, VocabularyValueRecord, VocabularyValueState};
use crate::registry::Registry;
use crate::traceability::VocabularyCoverage;

/// Implements: FR-059
/// Check every declared vocabulary coverage.
///
/// A no-op when the module declares none (FR-059-AC-6), the same early return
/// `validate_trace_references` and `validate_required_relations` make.
pub(crate) fn validate_vocabulary_coverage(
    spec: &Spec,
    registry: &Registry,
    posture: BundlePosture,
    root: &Path,
    report: &mut BundleReport,
) {
    let Some(model) = registry.traceability() else {
        return;
    };
    if model.vocabulary_coverage.is_empty() {
        return;
    }

    let severity = registry.grammar_severity();
    let model_exclude = ExcludeSet::compile_validated(&model.exclude);
    let mut findings: Vec<(PathBuf, String, String)> = Vec::new();

    for coverage in &model.vocabulary_coverage {
        check_coverage(
            spec,
            registry,
            coverage,
            root,
            &model_exclude,
            &mut findings,
        );
    }

    findings.sort();
    findings.dedup();
    for (path, check, message) in findings {
        report.route(
            severity,
            posture_tier(posture),
            BundleFinding::in_pack(pack::TRACE, check, path, message),
        );
    }
}

/// The declared vocabulary for one coverage entry, read from the archetype's
/// frontmatter schema.
///
/// Returns `None` when the archetype is unknown, declares no frontmatter
/// schema, or that schema does not give `field` an `enum` — three different
/// ways for the declaration to name a vocabulary that does not exist, all
/// reported by the caller rather than silently treated as "no values".
fn declared_values(registry: &Registry, coverage: &VocabularyCoverage) -> Option<Vec<String>> {
    let archetype = registry.archetype(&coverage.from)?;
    let schema = archetype.frontmatter_schema.as_ref()?;
    let values = enum_at(schema, &coverage.field)?;
    (!values.is_empty()).then_some(values)
}

/// The `enum` declared for `field`, searched breadth-first through the schema.
///
/// Breadth-first rather than a fixed `properties.<field>.enum` path because a
/// frontmatter schema legitimately wraps its properties — `allOf`, `$defs`, a
/// `oneOf` branch per document flavour — and a check that only understood the
/// flat shape would report "no such vocabulary" for a schema that plainly
/// declares one. Depth is bounded by the schema, which is finite and loaded.
///
/// When a schema declares the same field in two branches with different enums,
/// the **shallowest** match wins, then the alphabetically-first key —
/// `serde_json`'s `Map` is a sorted-key `BTreeMap`, so the walk is stable
/// across runs (NFR-006) rather than merely happening to be.
fn enum_at(schema: &Value, field: &str) -> Option<Vec<String>> {
    let mut queue: VecDeque<&Value> = VecDeque::new();
    queue.push_back(schema);
    while let Some(node) = queue.pop_front() {
        if let Some(properties) = node.get("properties").and_then(Value::as_object) {
            if let Some(values) = properties
                .get(field)
                .and_then(|f| f.get("enum"))
                .and_then(Value::as_array)
            {
                return Some(
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_string)
                        .collect(),
                );
            }
        }
        match node {
            Value::Object(map) => queue.extend(map.values()),
            Value::Array(items) => queue.extend(items.iter()),
            _ => {}
        }
    }
    None
}

/// One declaration's classification: every claimed and every excused value,
/// with the scope-relative documents that decide it.
///
/// The single walk both surfaces read (CR-091). The warning stream and the
/// coverage records answer different questions over the same facts, and two
/// independent walks would be free to drift — the CR-015 defect, between two
/// outputs instead of two declarations.
struct Classification {
    /// The declared vocabulary, in the schema enum's order.
    values: Vec<String>,
    /// No claimable document of the projected archetype exists at all.
    projected_empty: bool,
    /// Claimed value → the projected documents claiming it.
    owned: BTreeMap<String, BTreeSet<String>>,
    /// Justified value → the documents (any archetype) recording the absence.
    excused: BTreeMap<String, BTreeSet<String>>,
}

/// The dead-declaration message, shared verbatim by the bundle warning and the
/// coverage diagnostic so the two surfaces cannot disagree about the fault.
fn dead_declaration_message(coverage: &VocabularyCoverage) -> String {
    format!(
        "vocabulary_coverage '{}' reads '{}' on archetype '{}', which declares no such \
         enum — the check can never report anything",
        coverage.name, coverage.field, coverage.from
    )
}

/// Classify one declaration, or `None` when its vocabulary does not exist.
fn classify(
    spec: &Spec,
    registry: &Registry,
    coverage: &VocabularyCoverage,
    root: &Path,
    model_exclude: &ExcludeSet,
) -> Option<Classification> {
    let values = declared_values(registry, coverage)?;
    let exclude = ExcludeSet::compile_validated(&coverage.exclude);
    let claimable = |doc: &&LoadedDocument| {
        !exclude.excludes(root, &doc.path) && !model_exclude.excludes(root, &doc.path)
    };
    let claims = |doc: &LoadedDocument, field: &str| {
        let mut out = BTreeSet::new();
        if let Some(fm) = doc.frontmatter() {
            collect_strings(fm.get(field), &mut out);
        }
        out
    };

    // Claimed: a document of the declared archetype carrying the field.
    let projected: Vec<&LoadedDocument> = spec
        .by_type(&coverage.from)
        .into_iter()
        .filter(claimable)
        .collect();
    let mut owned: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for doc in &projected {
        for value in claims(doc, &coverage.field) {
            owned
                .entry(value)
                .or_default()
                .insert(relative_path(root, &doc.path));
        }
    }

    // Justified: ANY document in the bundle naming the value as not applicable.
    // Deliberately not restricted to `from`'s archetype — the natural home for
    // "this product has no safety characteristic" is the bundle's own spec or
    // master-requirements document, not one of the requirements it is a
    // statement about.
    let mut excused: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    if let Some(field) = &coverage.justified_absence_field {
        for doc in spec.all().iter().filter(claimable) {
            for value in claims(doc, field) {
                excused
                    .entry(value)
                    .or_default()
                    .insert(relative_path(root, &doc.path));
            }
        }
    }

    Some(Classification {
        projected_empty: projected.is_empty(),
        values,
        owned,
        excused,
    })
}

/// Every declared value that no document claims and nothing justifies.
fn check_coverage(
    spec: &Spec,
    registry: &Registry,
    coverage: &VocabularyCoverage,
    root: &Path,
    model_exclude: &ExcludeSet,
    out: &mut Vec<(PathBuf, String, String)>,
) {
    let check = coverage.check.clone();

    let Some(classification) = classify(spec, registry, coverage, root, model_exclude) else {
        // The declaration names a vocabulary that does not exist, so it can
        // never report anything. Silence here would be indistinguishable from
        // full coverage — the CR-075 failure mode, in a second place.
        out.push((
            root.to_path_buf(),
            "undeclared-coverage-vocabulary".to_string(),
            dead_declaration_message(coverage),
        ));
        return;
    };

    // An EMPTY projection is one fact, not `values.len()` of them. Measured
    // over 243 `~/dev` bundles: 90 carry no NFR document at all, and reporting
    // each of the 12 characteristics as unowned turned that single fact into
    // 1080 of the sweep's 2792 findings — every one of them saying "no document
    // claims security", "no document claims safety", and so on, when what is
    // true and useful is "nothing here projects onto this vocabulary".
    //
    // This is not a widening to lower a count, which the rule-quality doctrine
    // forbids. It is the same information reported at the granularity it
    // actually has: a check should make the most specific true statement it
    // can, and twelve restatements of one fact are less actionable than the
    // fact.
    if classification.projected_empty {
        out.push((
            root.to_path_buf(),
            check.clone(),
            format!(
                "no '{}' document projects onto '{}' — declared by '{}'; the vocabulary's {}                  values are unowned because nothing claims any of them",
                coverage.from,
                coverage.field,
                coverage.name,
                classification.values.len()
            ),
        ));
        return;
    }

    for value in &classification.values {
        if classification.owned.contains_key(value) || classification.excused.contains_key(value) {
            continue;
        }
        out.push((
            root.to_path_buf(),
            check.clone(),
            format!(
                "no document claims '{}' for '{}' — declared by '{}'{}",
                value,
                coverage.field,
                coverage.name,
                match &coverage.justified_absence_field {
                    Some(f) => format!(", and nothing records it under '{f}'"),
                    None => String::new(),
                }
            ),
        ));
    }
}

/// Implements: FR-059
/// Per-value classification records for the coverage payload (FR-059-AC-9,
/// CR-091, `agent-ix/quire-rs#179`).
///
/// The warning stream reports only the unowned **residue**, so a consumer
/// could not tell an owned value from an excused one without opening every
/// document in the bundle and parsing its frontmatter — a second frontmatter
/// reader in a toolchain whose discipline is that quire is the parser. The
/// records carry the whole classification instead: one record per declared
/// value, owned / excused / unowned, with the deciding documents.
///
/// A declaration whose vocabulary does not exist yields a
/// `undeclared-coverage-vocabulary` diagnostic — the same token the bundle
/// warning uses — because on this surface too, silence would be
/// indistinguishable from a module that declared nothing (FR-059-AC-10).
pub(crate) fn coverage_records(
    spec: &Spec,
    registry: &Registry,
    root: &Path,
) -> (Vec<VocabularyValueRecord>, Vec<CoverageDiagnostic>) {
    let Some(model) = registry.traceability() else {
        return (Vec::new(), Vec::new());
    };
    if model.vocabulary_coverage.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let model_exclude = ExcludeSet::compile_validated(&model.exclude);

    let mut records = Vec::new();
    let mut diagnostics = Vec::new();
    for coverage in &model.vocabulary_coverage {
        let Some(classification) = classify(spec, registry, coverage, root, &model_exclude) else {
            diagnostics.push(CoverageDiagnostic {
                declaration: coverage.name.clone(),
                reason: "undeclared-coverage-vocabulary".to_string(),
                message: dead_declaration_message(coverage),
                path: None,
                line: None,
                value: None,
                guidance: None,
            });
            continue;
        };
        for value in &classification.values {
            // A value both claimed and excused reads as OWNED: a requirement
            // exists, and the stale justified-absence record is the consumer's
            // (quoin FR-037's) finding to make from the same records.
            let (state, documents) = if let Some(docs) = classification.owned.get(value) {
                (VocabularyValueState::Owned, docs.iter().cloned().collect())
            } else if let Some(docs) = classification.excused.get(value) {
                (
                    VocabularyValueState::Excused,
                    docs.iter().cloned().collect(),
                )
            } else {
                (VocabularyValueState::Unowned, Vec::new())
            };
            records.push(VocabularyValueRecord {
                vocabulary: coverage.name.clone(),
                archetype: coverage.from.clone(),
                field: coverage.field.clone(),
                check: coverage.check.clone(),
                value: value.clone(),
                state,
                documents,
            });
        }
    }
    (records, diagnostics)
}

/// Collect a frontmatter value as one or more claimed strings.
///
/// A field is a scalar on one document and a list on another — `quality_attribute: security`
/// and `not_applicable: [safety, portability]` are both ordinary authoring — so
/// both shapes count rather than the list form being silently ignored.
fn collect_strings(value: Option<&Value>, out: &mut BTreeSet<String>) {
    match value {
        Some(Value::String(s)) => {
            out.insert(s.clone());
        }
        Some(Value::Array(items)) => {
            for item in items {
                if let Value::String(s) = item {
                    out.insert(s.clone());
                }
            }
        }
        _ => {}
    }
}
