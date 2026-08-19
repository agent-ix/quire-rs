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

use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::declared_tables::ExcludeSet;
use super::spec::Spec;
use super::validate::{pack, posture_tier, BundleFinding, BundlePosture, BundleReport};
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
    let model_exclude = ExcludeSet::compile(&model.exclude);
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

/// Every declared value that no document claims and nothing justifies.
fn check_coverage(
    spec: &Spec,
    registry: &Registry,
    coverage: &VocabularyCoverage,
    root: &Path,
    model_exclude: &ExcludeSet,
    out: &mut Vec<(PathBuf, String, String)>,
) {
    let exclude = ExcludeSet::compile(&coverage.exclude);
    let check = coverage.check.clone();

    let Some(values) = declared_values(registry, coverage) else {
        // The declaration names a vocabulary that does not exist, so it can
        // never report anything. Silence here would be indistinguishable from
        // full coverage — the CR-075 failure mode, in a second place.
        out.push((
            root.to_path_buf(),
            "undeclared-coverage-vocabulary".to_string(),
            format!(
                "vocabulary_coverage '{}' reads '{}' on archetype '{}', which declares no such \
                 enum — the check can never report anything",
                coverage.name, coverage.field, coverage.from
            ),
        ));
        return;
    };

    let claimable = |doc: &&crate::corpus::walk::LoadedDocument| {
        !exclude.excludes(root, &doc.path) && !model_exclude.excludes(root, &doc.path)
    };
    let claim =
        |doc: &crate::corpus::walk::LoadedDocument, field: &str, out: &mut BTreeSet<String>| {
            if let Some(fm) = doc.frontmatter() {
                collect_strings(fm.get(field), out);
            }
        };

    // Claimed: a document of the declared archetype carrying the field.
    let projected: Vec<&crate::corpus::walk::LoadedDocument> = spec
        .by_type(&coverage.from)
        .into_iter()
        .filter(claimable)
        .collect();

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
    if projected.is_empty() {
        out.push((
            root.to_path_buf(),
            check.clone(),
            format!(
                "no '{}' document projects onto '{}' — declared by '{}'; the vocabulary's {}                  values are unowned because nothing claims any of them",
                coverage.from,
                coverage.field,
                coverage.name,
                values.len()
            ),
        ));
        return;
    }

    let mut covered: BTreeSet<String> = BTreeSet::new();
    for doc in projected {
        claim(doc, &coverage.field, &mut covered);
    }

    // Justified: ANY document in the bundle naming the value as not applicable.
    // Deliberately not restricted to `from`'s archetype — the natural home for
    // "this product has no safety characteristic" is the bundle's own spec or
    // master-requirements document, not one of the requirements it is a
    // statement about.
    let mut justified: BTreeSet<String> = BTreeSet::new();
    if let Some(field) = &coverage.justified_absence_field {
        for doc in spec.all().iter().filter(claimable) {
            claim(doc, field, &mut justified);
        }
    }

    let unowned: Vec<&String> = values
        .iter()
        .filter(|v| !covered.contains(v.as_str()) && !justified.contains(v.as_str()))
        .collect();
    if unowned.is_empty() {
        return;
    }

    for value in unowned {
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
