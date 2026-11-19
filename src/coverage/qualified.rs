//! Qualified trace ids in `verifies` markers (NFR-140).
//!
//! A tag may name an artifact of another corpus member, either as
//! `ix://<org>/<repo>/<id>` or through a prefix the traceability model
//! declares in `trace_id_qualifiers`. The qualifier resolves against the
//! corpus member holding that identity; a tag that does not resolve is
//! reported and binds nothing, and never falls back to the bare id.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::corpus::declared_tables;
use crate::corpus::resolve::{extract_target_id, ix_authority};
use crate::corpus::spec::{fold_identity, Spec};
use crate::obligation::Obligation;
use crate::symbols::trace::{SymbolGraph, VerifiesRelation};
use crate::traceability::TraceabilityModel;

use super::{CoverageDiagnostic, MintedTargetRecord};

/// Why a qualified tag names nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cause {
    /// No corpus member declares the repository.
    OutsideCorpus,
    /// The member exists and mints no such id.
    UnknownId,
}

#[derive(Debug)]
struct Unresolved<'g> {
    relation: &'g VerifiesRelation,
    repository: String,
    id: String,
    cause: Cause,
}

/// The qualified tags of one graph, resolved against one corpus.
#[derive(Debug, Default)]
pub(super) struct Qualification<'g> {
    /// Folded member identity -> id -> relations binding it.
    bound: BTreeMap<(String, String), Vec<&'g VerifiesRelation>>,
    /// Qualified tag text, as written.
    qualified: BTreeSet<&'g str>,
    unresolved: Vec<Unresolved<'g>>,
    /// Scope-relative document path -> folded identity of its member.
    document_member: BTreeMap<String, String>,
}

/// The `(repository, id)` a tag names, when it is qualified.
pub(super) fn split_qualified(model: &TraceabilityModel, tag: &str) -> Option<(String, String)> {
    if let Some(repository) = ix_authority(tag) {
        return Some((repository, extract_target_id(tag).to_string()));
    }
    model.trace_id_qualifiers.iter().find_map(|q| {
        let rest = tag.strip_prefix(q.prefix.as_str())?;
        (!rest.is_empty()).then(|| (q.repository.clone(), rest.to_string()))
    })
}

impl<'g> Qualification<'g> {
    pub(super) fn build(
        spec: &Spec,
        model: &TraceabilityModel,
        graph: &'g SymbolGraph,
        root: &Path,
        obligations: &[Obligation],
        minted: &[MintedTargetRecord],
    ) -> Self {
        let mut out = Self::default();
        let mut present: BTreeSet<String> = BTreeSet::new();
        // Folded identity -> the ids that member mints.
        let mut known: BTreeMap<String, BTreeSet<&str>> = BTreeMap::new();
        for member in 0..spec.member_count() {
            let Some(identity) = spec.member_identity(member) else {
                continue;
            };
            let folded = fold_identity(identity);
            present.insert(folded.clone());
            for doc in spec.member_view(member).documents() {
                out.document_member.insert(
                    declared_tables::relative_path(root, &doc.path),
                    folded.clone(),
                );
            }
        }
        for obligation in obligations {
            if let Some(folded) = out.document_member.get(&obligation.document) {
                known
                    .entry(folded.clone())
                    .or_default()
                    .insert(&obligation.id);
            }
        }
        for record in minted {
            if let Some(folded) = out.document_member.get(&record.document) {
                known.entry(folded.clone()).or_default().insert(&record.id);
            }
        }

        for relation in &graph.verifies {
            let Some((repository, id)) = split_qualified(model, &relation.trace_id) else {
                continue;
            };
            out.qualified.insert(relation.trace_id.as_str());
            let folded = fold_identity(&repository);
            let cause = if !present.contains(&folded) {
                Some(Cause::OutsideCorpus)
            } else if known
                .get(&folded)
                .is_some_and(|ids| ids.contains(id.as_str()))
                || spec.by_qualified_id(&repository, &id).is_some()
            {
                None
            } else {
                Some(Cause::UnknownId)
            };
            match cause {
                None => out.bound.entry((folded, id)).or_default().push(relation),
                Some(cause) => out.unresolved.push(Unresolved {
                    relation,
                    repository,
                    id,
                    cause,
                }),
            }
        }
        out
    }

    /// Whether `tag` is written in a qualified form (resolved or not).
    pub(super) fn is_qualified(&self, tag: &str) -> bool {
        self.qualified.contains(tag)
    }

    /// The folded identity of the member holding `document`.
    pub(super) fn member_of(&self, document: &str) -> Option<&str> {
        self.document_member.get(document).map(String::as_str)
    }

    /// The relations whose qualified tag names `id` in the member `document`
    /// belongs to.
    pub(super) fn binders(&self, document: &str, id: &str) -> &[&'g VerifiesRelation] {
        self.member_of(document)
            .and_then(|member| self.bound.get(&(member.to_string(), id.to_string())))
            .map_or(&[], Vec::as_slice)
    }

    pub(super) fn diagnostics(&self) -> Vec<CoverageDiagnostic> {
        self.unresolved
            .iter()
            .map(|u| {
                let relation = u.relation;
                let why = match u.cause {
                    Cause::OutsideCorpus => format!(
                        "`{}` is not a member of the corpus, so the tag references a repository \
                         outside it",
                        u.repository
                    ),
                    Cause::UnknownId => format!("`{}` mints no id `{}`", u.repository, u.id),
                };
                CoverageDiagnostic {
                    declaration: "traceability.trace_id_qualifiers".to_string(),
                    reason: "unresolved-qualified-trace-id".to_string(),
                    message: format!(
                        "the qualified trace id `{}` on `{}` at {}:{} binds nothing: {why}",
                        relation.trace_id, relation.symbol, relation.path, relation.line
                    ),
                    path: Some(relation.path.clone()),
                    line: Some(relation.line),
                    value: Some(relation.trace_id.clone()),
                    guidance: None,
                }
            })
            .collect()
    }
}
