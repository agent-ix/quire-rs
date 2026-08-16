//! Bundle validation with two postures (OKF).
//!
//! A *bundle* is a directory tree of authored markdown documents. Two
//! postures answer two different questions:
//!
//! - [`BundlePosture::Strict`] — "is this one of *our* archetype-conformant
//!   specs?" Every document must carry a known `type`, satisfy its
//!   archetype (frontmatter schema + `body_extraction` + heading
//!   uniqueness), have resolvable `ix://` references, and every directory's
//!   `index.md` must list its sibling artifacts (with `okf_version` at the
//!   bundle root).
//! - [`BundlePosture::Okf`] — "can we read this *foreign* OKF bundle?"
//!   `type` is still required and non-empty, but unknown types are
//!   tolerated, broken references and index gaps degrade to **warnings**,
//!   and archetype-specific body contracts are not enforced.
//!
//! `index.md`/`log.md` keep their archetypes and are validated like any
//! other document; they are exempt only from *index-completeness*, which
//! asks an index to list its sibling artifacts.
//!
//! Nothing is exempt by filename at walk time (CR-044). Corpus membership
//! is decided by the presence of a frontmatter block, so a `README.md` is
//! not a document and a typed `tests.md` is.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::corpus::resolve::Resolution;
use crate::corpus::spec::Spec;
use crate::corpus::walk::LoadedDocument;
use crate::query::{concept_type, section};
use crate::registry::Registry;

/// Bundle-structure files that an `index.md` is not expected to list among its
/// own siblings: the index cannot be a sibling of itself, and `log.md` is the
/// bundle's history rather than one of its artifacts.
///
/// **Deliberately short** (CR-044). This was a second filename list holding
/// four names, overlapping the walk's and disagreeing with it. `README.md` is
/// gone permanently — with membership decided by the presence of a frontmatter
/// block, a README never becomes a document, so the entry could never fire.
/// `tests.md` is gone because it is an artifact: `TestMatrix` is a registered
/// archetype with a frontmatter schema, an `id_pattern` and a `body_extraction`
/// contract, and an index that omits it is incomplete. **[RAN]** across `~/dev`,
/// 4 of 180 repos with a `spec/tests.md` already name it in `spec/index.md`;
/// the other 172 now report `index-incomplete`, which is authoring debt the
/// suppression was hiding, not a regression this list should absorb.
const NON_ARTIFACT_FILES: &[&str] = &["index.md", "log.md"];

/// Which validation posture to apply to a bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundlePosture {
    /// Our strict, archetype-conformant posture.
    Strict,
    /// The permissive OKF posture for reading foreign bundles.
    Okf,
}

/// One bundle-level finding (error or warning).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleFinding {
    /// Document or directory the finding concerns.
    pub path: PathBuf,
    /// Human-readable message.
    pub message: String,
    /// Stable machine-readable reason token.
    pub reason: &'static str,
}

/// Outcome of [`validate_bundle`]: hard errors + non-fatal warnings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BundleReport {
    pub errors: Vec<BundleFinding>,
    pub warnings: Vec<BundleFinding>,
}

impl BundleReport {
    /// The bundle is valid for its posture when there are no hard errors.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Route a finding to errors (Strict) or warnings (Okf).
    pub(crate) fn degradable(&mut self, posture: BundlePosture, finding: BundleFinding) {
        match posture {
            BundlePosture::Strict => self.errors.push(finding),
            BundlePosture::Okf => self.warnings.push(finding),
        }
    }
}

/// Load `root` as a corpus and validate it under `posture`. Both the
/// document root and the reference root are `root` — the shape for a
/// self-contained bundle whose traceability model declares paths relative
/// to the bundle itself.
pub fn validate_bundle_at(
    root: &Path,
    registry: &Registry,
    posture: BundlePosture,
) -> BundleReport {
    let spec = Spec::from_path(root);
    validate_bundle(&spec, registry, posture, root, root)
}

/// Validate an already-loaded `spec` under `posture`, with the two roots
/// stated separately (CR-045, the same split `compute_coverage` has always
/// had via its `root` parameter):
///
/// - `document_root` — the directory the corpus was walked from; locates
///   the root `index.md` for OKF completeness.
/// - `reference_root` — the base the module's `traceability:` model paths
///   and `exclude:` globs resolve against. Models are authored against the
///   repository scope (`document: spec/tests.md`), so a caller that walked
///   `<scope>/spec` passes `<scope>` here; conflating the two silently
///   un-mints every path-bound trace target.
pub fn validate_bundle(
    spec: &Spec,
    registry: &Registry,
    posture: BundlePosture,
    document_root: &Path,
    reference_root: &Path,
) -> BundleReport {
    let root = document_root;
    let mut report = BundleReport::default();

    // FR-044: harvest the repo's project Ubiquitous-Language terms once and
    // compose the combined (module ∪ project) lexicon the EARS grammar check
    // consumes for every document in the bundle.
    let project_terms = crate::corpus::glossary_terms(spec);
    let lexicon = registry.lexicon_with(&project_terms);

    for doc in &spec.inner.documents {
        validate_concept(doc, registry, posture, &lexicon, &mut report);
    }

    // Reference resolution: a dangling `ix://` ref is a hard error under
    // Strict, a warning under OKF (foreign targets are expected).
    for edge in spec.inner.edges.iter() {
        if edge.resolution == Resolution::Dangling {
            report.degradable(
                posture,
                BundleFinding {
                    path: PathBuf::from(&edge.source),
                    message: format!(
                        "dangling reference '{}' (edge '{}') has no target in the bundle",
                        edge.target, edge.edge_type
                    ),
                    reason: "dangling-reference",
                },
            );
        }
    }

    // Tier-2 edge-target validation (FR-040-AC-9). `spec.inner.edges` is
    // already sorted by `(source, target, type)` (resolver collects into a
    // BTreeSet), so findings come out deterministically. Warn-tier always
    // (FR-040-AC-10) — never degraded to a Strict error this revision.
    validate_edge_targets(spec, registry, &mut report);

    // FR-049: declared table-cell trace references get the same dangling
    // check `ix://` edges get, driven entirely by the module's traceability
    // model. A no-op when no module declares one.
    crate::corpus::trace_refs::validate_trace_references(
        spec,
        registry,
        posture,
        reference_root,
        &mut report,
    );

    check_index_completeness(spec, posture, root, &mut report);

    report
}

/// The frontmatter `object:` value of a loaded document, when present and
/// a non-empty string.
fn document_object(doc: &LoadedDocument) -> Option<&str> {
    doc.doc
        .frontmatter
        .as_ref()?
        .get("object")?
        .as_str()
        .filter(|s| !s.is_empty())
}

/// Tier-2: for each **resolved** edge, check that the target document's
/// `object:` archetype (or any role it carries) satisfies the verb's
/// target list in the source's resolved `allowed_links` (FR-040-AC-9).
///
/// Skipped when the source type/verb is unknown, the verb's target list
/// is `"*"` or empty (unconstrained), the target declares no `object:`
/// (no object type to constrain), or its object archetype is unknown.
/// Cross-repo targets never reach here — they resolve as dangling.
fn validate_edge_targets(spec: &Spec, registry: &Registry, report: &mut BundleReport) {
    for edge in spec.inner.edges.iter() {
        if edge.resolution != Resolution::Dangling {
            // FR-041: normalize an inverse-verb edge `(source, I, target)`
            // to its forward orientation `(target, F, source)` before the
            // target check, so the canonical-direction allowed_links rule
            // applies. A forward edge passes through unchanged.
            let (fwd_source_id, fwd_verb, fwd_target_id): (&str, &str, &str) =
                match registry.inverse_index().get(&edge.edge_type) {
                    Some(forward) => (edge.target.as_str(), forward.as_str(), edge.source.as_str()),
                    None => (
                        edge.source.as_str(),
                        edge.edge_type.as_str(),
                        edge.target.as_str(),
                    ),
                };
            // Resolve the forward source's vocabulary for the forward verb.
            let Some(source_doc) = spec.by_id(fwd_source_id) else {
                continue;
            };
            let Some(source_type) = concept_type(&source_doc.doc) else {
                continue;
            };
            let Some(source_arch) = registry.archetype(source_type) else {
                continue;
            };
            let source_obj_arch = document_object(source_doc).and_then(|o| registry.archetype(o));
            let resolved = registry.resolve_allowed_links(source_arch, source_obj_arch);
            let Some(targets) = resolved.get(fwd_verb) else {
                // Verb not in the resolved vocabulary — that is Tier-1's
                // concern (DisallowedEdgeType), not a target violation.
                continue;
            };
            // Unconstrained target list — nothing to check.
            if targets.is_empty() || targets.iter().any(|t| t == "*") {
                continue;
            }
            // Resolve the forward target's object archetype.
            let Some(target_doc) = spec.by_id(fwd_target_id) else {
                continue;
            };
            let Some(target_obj_name) = document_object(target_doc) else {
                continue; // no `object:` — nothing to constrain
            };
            let Some(target_arch) = registry.archetype(target_obj_name) else {
                continue;
            };
            if !targets
                .iter()
                .any(|t| registry.target_satisfies(t, target_arch))
            {
                // Report with the edge as authored (inverse source/target/
                // verb), per FR-041-AC-4.
                report.warnings.push(BundleFinding {
                    path: PathBuf::from(&edge.source),
                    message: format!(
                        "edge '{}' from '{}' targets '{}' (object type '{}'), which satisfies none of {:?}",
                        edge.edge_type, edge.source, edge.target, target_obj_name, targets
                    ),
                    reason: "disallowed-edge-target",
                });
            }
        }
    }
}

/// Per-document checks: base concept contract + (Strict only) full
/// archetype validation against a known type.
fn validate_concept(
    doc: &LoadedDocument,
    registry: &Registry,
    posture: BundlePosture,
    lexicon: &crate::grammar::GrammarLexicon,
    report: &mut BundleReport,
) {
    let fm = doc.doc.frontmatter.clone().unwrap_or_default();

    // `type` is required + non-empty in BOTH postures — this is the
    // "untyped corpus doc is an error, not a warning" fix.
    match concept_type(&doc.doc) {
        None | Some("") => {
            report.errors.push(BundleFinding {
                path: doc.path.clone(),
                message: "frontmatter is missing the required, non-empty 'type'".to_string(),
                reason: "frontmatter",
            });
            return;
        }
        Some(_) => {}
    }

    match posture {
        BundlePosture::Strict => {
            // Full base concept contract (typing of description/tags too).
            for err in crate::concept::validate_base_concept(&fm) {
                report.errors.push(BundleFinding {
                    path: doc.path.clone(),
                    message: err.message,
                    reason: err.reason.as_str(),
                });
            }

            let ty = concept_type(&doc.doc).unwrap_or_default();
            match registry.archetype(ty) {
                Some(archetype) => {
                    // Composed type+object validation (FR-032-AC-11..13):
                    // the bundle has the registry, so resolve the
                    // frontmatter `object:` archetype too. Object errors
                    // merge into bundle errors; an unknown `object:`
                    // degrades to a bundle warning.
                    let result = crate::validate_document_in_registry_with_lexicon(
                        registry,
                        archetype,
                        &doc.doc.raw,
                        lexicon,
                    );
                    for err in result.errors {
                        report.errors.push(BundleFinding {
                            path: doc.path.clone(),
                            message: err.message,
                            reason: err.reason.as_str(),
                        });
                    }
                    for warn in result.warnings {
                        report.warnings.push(BundleFinding {
                            path: doc.path.clone(),
                            message: warn.message,
                            reason: warn.reason.as_str(),
                        });
                    }
                }
                None => report.errors.push(BundleFinding {
                    path: doc.path.clone(),
                    message: format!("unknown type '{ty}' (no archetype registered for it)"),
                    reason: "unknown-type",
                }),
            }
        }
        BundlePosture::Okf => {
            // Permissive: unknown types are tolerated as a warning; the
            // archetype body contract is not enforced for foreign bundles.
            let ty = concept_type(&doc.doc).unwrap_or_default();
            if registry.archetype(ty).is_none() {
                report.warnings.push(BundleFinding {
                    path: doc.path.clone(),
                    message: format!("unknown type '{ty}' tolerated under OKF posture"),
                    reason: "unknown-type",
                });
            }
        }
    }
}

/// Every directory holding an `index.md` must list its sibling artifacts in
/// the index `## Contents`, and the bundle-root `index.md` must declare
/// `okf_version`. Findings degrade to warnings under OKF.
fn check_index_completeness(
    spec: &Spec,
    posture: BundlePosture,
    root: &Path,
    report: &mut BundleReport,
) {
    // Group documents by parent directory.
    let mut by_dir: BTreeMap<PathBuf, Vec<&LoadedDocument>> = BTreeMap::new();
    for doc in &spec.inner.documents {
        let dir = doc.path.parent().map(Path::to_path_buf).unwrap_or_default();
        by_dir.entry(dir).or_default().push(doc);
    }

    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

    for (dir, docs) in &by_dir {
        let Some(index_doc) = docs.iter().find(|d| file_name(&d.path) == "index.md") else {
            continue;
        };

        // Root index must carry okf_version.
        let dir_canon = dir.canonicalize().unwrap_or_else(|_| dir.clone());
        if dir_canon == root {
            let has_okf = index_doc
                .doc
                .frontmatter
                .as_ref()
                .and_then(|fm| fm.get("okf_version"))
                .is_some();
            if !has_okf {
                report.degradable(
                    posture,
                    BundleFinding {
                        path: index_doc.path.clone(),
                        message: "root index.md is missing 'okf_version' in frontmatter"
                            .to_string(),
                        reason: "index-okf-version",
                    },
                );
            }
        }

        // Every sibling artifact must appear in the index Contents.
        let listed = contents_basenames(index_doc);
        for doc in docs {
            let name = file_name(&doc.path);
            if NON_ARTIFACT_FILES.contains(&name.as_str()) {
                continue;
            }
            if !listed.contains(&name) {
                report.degradable(
                    posture,
                    BundleFinding {
                        path: index_doc.path.clone(),
                        message: format!("index.md does not list sibling artifact '{name}'"),
                        reason: "index-incomplete",
                    },
                );
            }
        }
    }
}

/// Basenames of the files linked from an index document's `## Contents`
/// section. Falls back to the whole document when no `Contents` heading is
/// present (a permissive read).
fn contents_basenames(index_doc: &LoadedDocument) -> BTreeSet<String> {
    let text = section(&index_doc.doc, "Contents")
        .map(|s| s.content.clone())
        .unwrap_or_else(|| index_doc.doc.raw.clone());

    let mut out = BTreeSet::new();
    for target in markdown_link_targets(&text) {
        if let Some(name) = Path::new(&target).file_name().and_then(|n| n.to_str()) {
            out.insert(name.to_string());
        }
    }
    out
}

/// The link targets inside `](...)` spans of a markdown fragment.
fn markdown_link_targets(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b']' && bytes[i + 1] == b'(' {
            if let Some(end) = text[i + 2..].find(')') {
                let target = &text[i + 2..i + 2 + end];
                // Strip any "#fragment" or title after a space.
                let target = target.split(['#', ' ']).next().unwrap_or(target);
                out.push(target.to_string());
                i = i + 2 + end + 1;
                continue;
            }
        }
        i += 1;
    }
    out
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .to_string()
}
