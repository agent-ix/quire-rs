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
//! other document; only `README.md`/`tests.md` are skipped at walk time.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::corpus::resolve::Resolution;
use crate::corpus::spec::Spec;
use crate::corpus::walk::LoadedDocument;
use crate::query::{concept_type, section};
use crate::registry::Registry;

/// Files that are structural, not artifacts, for index-completeness.
const NON_ARTIFACT_FILES: &[&str] = &["index.md", "log.md", "README.md", "tests.md"];

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
    fn degradable(&mut self, posture: BundlePosture, finding: BundleFinding) {
        match posture {
            BundlePosture::Strict => self.errors.push(finding),
            BundlePosture::Okf => self.warnings.push(finding),
        }
    }
}

/// Load `root` as a corpus and validate it under `posture`.
pub fn validate_bundle_at(
    root: &Path,
    registry: &Registry,
    posture: BundlePosture,
) -> BundleReport {
    let spec = Spec::from_path(root);
    validate_bundle(&spec, registry, posture, root)
}

/// Validate an already-loaded `spec` under `posture`. `root` identifies the
/// bundle root (used to locate the root `index.md`).
pub fn validate_bundle(
    spec: &Spec,
    registry: &Registry,
    posture: BundlePosture,
    root: &Path,
) -> BundleReport {
    let mut report = BundleReport::default();

    for doc in &spec.inner.documents {
        validate_concept(doc, registry, posture, &mut report);
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

    check_index_completeness(spec, posture, root, &mut report);

    report
}

/// Per-document checks: base concept contract + (Strict only) full
/// archetype validation against a known type.
fn validate_concept(
    doc: &LoadedDocument,
    registry: &Registry,
    posture: BundlePosture,
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
                    let result = crate::validate_document(archetype, &doc.doc.raw);
                    for err in result.errors {
                        report.errors.push(BundleFinding {
                            path: doc.path.clone(),
                            message: err.message,
                            reason: err.reason.as_str(),
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
