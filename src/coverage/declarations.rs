//! Declaration-scoped corpus classification used by coverage.

use std::path::Path;

use crate::corpus::declared_tables;
use crate::corpus::spec::Spec;
use crate::grammar::{AcPropertyCounts, GrammarVocabularies};
use crate::registry::Registry;
use crate::traceability::TraceabilityModel;

use super::{relative, CatchAllCriterion, CriteriaCounts};

/// Classify criteria only after path, archetype, and grammar scoping succeeds.
/// Excluded and undeclared documents remain body-unparsed (FR-050-CON-2).
pub(super) fn criteria_counts(
    spec: &Spec,
    registry: &Registry,
    model: &TraceabilityModel,
    root: &Path,
) -> Vec<CriteriaCounts> {
    let vocab = GrammarVocabularies {
        lexicon: registry.lexicon_matcher(),
        observable: registry.observable_verbs_matcher(),
        vacuous: registry.vacuous_predicates_matcher(),
        idioms: registry.property_idioms_matcher(),
        ambiguous: registry.ambiguity_terms_matcher(),
    };
    let excluded = declared_tables::ExcludeSet::compile_validated(&model.exclude);

    let mut out: Vec<CriteriaCounts> = Vec::new();
    for entry in &spec.inner.documents {
        // Path-only, and ahead of every other gate: an excluded document must
        // not be classified *or* body-parsed (CR-060). It matches on the same
        // `relative_path` derivation a report path uses, so a glob and a
        // reported path compare as the same string (CR-038).
        if excluded.excludes(root, &entry.path) {
            continue;
        }
        let Some(archetype) =
            crate::corpus::spec::artifact_type(entry).and_then(|ty| registry.archetype(&ty))
        else {
            continue;
        };
        let Some(grammar_ref) = archetype.grammar_ref() else {
            continue;
        };
        // The line offset is NO LONGER immaterial (#261): the catch-all
        // example carries a document line, so the same conversion every other
        // located finding uses is needed here too. It was `0` while nothing
        // read a record's line.
        //
        // The body touch happens only past the archetype/grammar gates
        // above (CR-047): a document under a module declaring no grammar
        // stays unparsed.
        let records = crate::grammar::classify_document_properties(
            grammar_ref,
            &archetype.name,
            entry.body(),
            crate::validate_document::body_line_offset(&entry.body().raw),
            vocab,
        );
        if records.is_empty() {
            continue;
        }
        let counts = AcPropertyCounts::tally(records.iter());
        // The lowest-lined `universal` criterion, and only when the document
        // named a specific shape for none of them. A document with one
        // specific criterion is not what this finding is about, and carrying
        // an example there would make the field noise rather than a locus.
        let catch_all_example = (counts.property_shaped > 0 && counts.specific_shaped == 0)
            .then(|| {
                records
                    .iter()
                    .filter(|r| r.extractable)
                    .min_by_key(|r| (r.line.unwrap_or(usize::MAX), r.row_id.clone()))
                    .map(|r| CatchAllCriterion {
                        row_id: r.row_id.clone(),
                        line: r.line,
                    })
            })
            .flatten();
        out.push(CriteriaCounts {
            document: relative(root, &entry.path),
            archetype: archetype.name.clone(),
            criteria: counts.criteria,
            property_shaped: counts.property_shaped,
            specific_shaped: counts.specific_shaped,
            by_property: counts.by_property,
            grounding: counts.grounding,
            catch_all_example,
        });
    }
    out.sort_by(|a, b| (&a.document, &a.archetype).cmp(&(&b.document, &b.archetype)));
    out
}
