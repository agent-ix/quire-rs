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

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::corpus::resolve::Resolution;
use crate::corpus::spec::Spec;
use crate::corpus::walk::LoadedDocument;
use crate::grammar::{GrammarSeverity, GrammarSeverityLevel, GrammarSeverityMap};
use crate::query::section;
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

/// The corpus check packs (FR-057). A pack is the `<pack>` half of the
/// `grammar_severity` registry key; the `<check>` half is the finding's existing
/// `reason` token, which this FR deliberately leaves unchanged — it is the
/// machine surface `quire validate` already prints.
///
/// Document-level results bridged into [`BundleReport`] (schema errors,
/// `unknown-type`, missing `type`) carry **no** pack and are not registrable:
/// mapping them would let a module switch off schema validation under a
/// severity key (FR-057-CON-1).
pub mod pack {
    /// Bundle structure — FR-038. Frontmatter presence and index completeness.
    pub const BUNDLE: &str = "bundle";
    /// Reference resolution — FR-026.
    pub const REFS: &str = "refs";
    /// Edge-target vocabulary — FR-040.
    pub const EDGES: &str = "edges";
    /// Declared traceability references — FR-049.
    pub const TRACE: &str = "trace";
}

/// One bundle-level finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleFinding {
    /// Document or directory the finding concerns.
    pub path: PathBuf,
    /// Human-readable message.
    pub message: String,
    /// Stable machine-readable reason token.
    ///
    /// `Cow` rather than `&'static str` (FR-058): every reason the engine emits
    /// is a literal, but a **module-declared** required-relation names its own
    /// check, and that token is owned by the manifest. Leaking it to obtain a
    /// `&'static str` would grow the heap on every `validate_bundle` call — a
    /// corpus sweep calls it once per repository.
    pub reason: Cow<'static, str>,
    /// Check pack this finding belongs to, or `None` for a document-level
    /// result bridged in from schema validation (FR-057-CON-1). With `reason`
    /// it forms the `<pack>:<check>` severity-registry key.
    pub pack: Option<&'static str>,
    /// The severity that was applied (FR-057-AC-7), so a surface renders the
    /// configured level instead of inferring it from which vector it landed in.
    pub severity: GrammarSeverity,
}

impl BundleFinding {
    /// A finding from a check pack, before severity is resolved.
    pub(crate) fn in_pack(
        pack: &'static str,
        reason: impl Into<Cow<'static, str>>,
        path: PathBuf,
        message: String,
    ) -> Self {
        Self {
            path,
            message,
            reason: reason.into(),
            pack: Some(pack),
            severity: GrammarSeverity::Warning,
        }
    }

    /// A document-level result bridged into the bundle report. Not registrable
    /// (FR-057-CON-1), so it carries no pack.
    pub(crate) fn bridged(
        reason: impl Into<Cow<'static, str>>,
        path: PathBuf,
        message: String,
    ) -> Self {
        Self {
            path,
            message,
            reason: reason.into(),
            pack: None,
            severity: GrammarSeverity::Error,
        }
    }

    /// The `<pack>:<check>` severity-registry key, when this finding is
    /// registrable at all.
    pub fn severity_key(&self) -> Option<String> {
        self.pack
            .map(|pack| crate::grammar::severity_key(pack, &self.reason))
    }
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

    /// Push a document-level result bridged in from schema validation. Not
    /// registrable (FR-057-CON-1).
    pub(crate) fn bridged(&mut self, severity: GrammarSeverity, finding: BundleFinding) {
        match severity {
            GrammarSeverity::Error => self.errors.push(finding),
            GrammarSeverity::Warning => self.warnings.push(finding),
        }
    }

    /// Route a pack finding, honouring the module's `<pack>:<check>` entry when
    /// there is one (FR-057).
    ///
    /// `unconfigured` is the tier the check uses when the registry says nothing.
    /// It is **not** a blanket `warning`: FR-048-AC-4's default would silently
    /// downgrade every corpus check that hard-errors under `Strict` today, and
    /// turning a failing build green is not a severity mechanism's job. Each
    /// caller passes the tier it had before FR-057 (FR-057-AC-4); a pack added
    /// after FR-057 passes `Warning`.
    pub(crate) fn route(
        &mut self,
        severity: &GrammarSeverityMap,
        unconfigured: GrammarSeverity,
        mut finding: BundleFinding,
    ) {
        let key = finding
            .severity_key()
            .expect("route() is for pack findings; bridged ones use bridged()");
        // Deliberately not `grammar::severity_level`: that function bakes in
        // FR-048-AC-4's warning default, and this surface needs "absent" to mean
        // "whatever this check did before", not "warning".
        match severity.get(&key).copied() {
            Some(GrammarSeverityLevel::Off) => {}
            Some(GrammarSeverityLevel::Warning) => {
                finding.severity = GrammarSeverity::Warning;
                self.warnings.push(finding);
            }
            Some(GrammarSeverityLevel::Error) => {
                finding.severity = GrammarSeverity::Error;
                self.errors.push(finding);
            }
            None => {
                finding.severity = unconfigured;
                match unconfigured {
                    GrammarSeverity::Error => self.errors.push(finding),
                    GrammarSeverity::Warning => self.warnings.push(finding),
                }
            }
        }
    }
}

/// The tier a posture-routed check uses when the registry says nothing: a hard
/// error under `Strict`, a warning under `Okf` — the behaviour these checks had
/// before FR-057 (FR-057-AC-4).
pub(crate) fn posture_tier(posture: BundlePosture) -> GrammarSeverity {
    match posture {
        BundlePosture::Strict => GrammarSeverity::Error,
        BundlePosture::Okf => GrammarSeverity::Warning,
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
    let mut report = BundleReport::default();
    // FR-057: the merged `<pack>:<check>` registry, already carrying whatever a
    // surface layered over the module map via `Registry::with_grammar_severity`
    // — which is how `quire validate --severity` reaches corpus checks without
    // any change to the CLI (FR-048-AC-5).
    let severity = registry.grammar_severity();

    // CR-048: a frontmatter-less markdown file under the document root is a
    // warning naming the path — in BOTH postures (never an error: the file
    // is not a document, so nothing structural can be wrong with it as one).
    // Walk diagnostics live on `Spec`, not in this report, so the finding is
    // bridged here or `quire validate` would never show it.
    for diag in spec.diagnostics() {
        if let crate::diagnostic::Diagnostic::DocumentWithoutFrontmatter { path, malformed } = diag
        {
            // The two flavors carry distinct reasons (CR-051): a consumer
            // triaging the machine surface acts differently on "this file was
            // never meant to be a document" than on "someone wrote a
            // frontmatter block and it does not parse as a mapping". The
            // engine already distinguishes them (`malformed: bool` on the
            // diagnostic); dropping that at this boundary made the second
            // indistinguishable from the first.
            let (message, reason) = if *malformed {
                (
                    "frontmatter block is not a YAML mapping; file is not a document and was not loaded",
                    "malformed-frontmatter",
                )
            } else {
                (
                    "no frontmatter block; file is not a document and was not loaded",
                    "no-frontmatter",
                )
            };
            // Fixed warning tier in both postures (CR-048) — the file is not a
            // document, so nothing structural can be wrong with it as one.
            report.route(
                severity,
                GrammarSeverity::Warning,
                BundleFinding::in_pack(pack::BUNDLE, reason, path.clone(), message.to_string()),
            );
        }
    }

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
            report.route(
                severity,
                posture_tier(posture),
                BundleFinding::in_pack(
                    pack::REFS,
                    "dangling-reference",
                    PathBuf::from(&edge.source),
                    format!(
                        "dangling reference '{}' (edge '{}') has no target in the bundle",
                        edge.target, edge.edge_type
                    ),
                ),
            );
        }
    }

    // Tier-2 edge-target validation (FR-040-AC-9). `spec.inner.edges` is
    // already sorted by `(source, target, type)` (resolver collects into a
    // BTreeSet), so findings come out deterministically. Warn-tier always
    // (FR-040-AC-10) — never degraded to a Strict error this revision.
    validate_edge_targets(spec, registry, severity, &mut report);

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

    // FR-058: upward-trace completeness — the declared edges every document of
    // a kind must have, in either direction, plus any declared acyclic verb.
    // A no-op when the module declares neither.
    crate::corpus::required_relations::validate_required_relations(
        spec,
        registry,
        posture,
        reference_root,
        &mut report,
    );

    check_index_completeness(spec, posture, severity, document_root, &mut report);

    report
}

/// The frontmatter `object:` value of a loaded document, when present and
/// a non-empty string.
fn document_object(doc: &LoadedDocument) -> Option<&str> {
    doc.frontmatter()?
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
fn validate_edge_targets(
    spec: &Spec,
    registry: &Registry,
    severity: &GrammarSeverityMap,
    report: &mut BundleReport,
) {
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
            let Some(source_type) = source_doc.concept_type() else {
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
                // verb), per FR-041-AC-4. Warn tier always (FR-040-AC-10).
                report.route(
                    severity,
                    GrammarSeverity::Warning,
                    BundleFinding::in_pack(
                        pack::EDGES,
                        "disallowed-edge-target",
                        PathBuf::from(&edge.source),
                        format!(
                            "edge '{}' from '{}' targets '{}' (object type '{}'), which satisfies none of {:?}",
                            edge.edge_type, edge.source, edge.target, target_obj_name, targets
                        ),
                    ),
                );
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
    let fm = doc.frontmatter().cloned().unwrap_or_default();

    // `type` is required + non-empty in BOTH postures — this is the
    // "untyped corpus doc is an error, not a warning" fix.
    match doc.concept_type() {
        None | Some("") => {
            report.bridged(
                GrammarSeverity::Error,
                BundleFinding::bridged(
                    "frontmatter",
                    doc.path.clone(),
                    "frontmatter is missing the required, non-empty 'type'".to_string(),
                ),
            );
            return;
        }
        Some(_) => {}
    }

    match posture {
        BundlePosture::Strict => {
            // Full base concept contract (typing of description/tags too).
            for err in crate::concept::validate_base_concept(&fm) {
                report.bridged(
                    GrammarSeverity::Error,
                    BundleFinding::bridged(err.reason.as_str(), doc.path.clone(), err.message),
                );
            }

            let ty = doc.concept_type().unwrap_or_default();
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
                        doc.raw(),
                        lexicon,
                    );
                    for err in result.errors {
                        report.bridged(
                            GrammarSeverity::Error,
                            BundleFinding::bridged(
                                err.reason.as_str(),
                                doc.path.clone(),
                                err.message,
                            ),
                        );
                    }
                    for warn in result.warnings {
                        report.bridged(
                            GrammarSeverity::Warning,
                            BundleFinding::bridged(
                                warn.reason.as_str(),
                                doc.path.clone(),
                                warn.message,
                            ),
                        );
                    }
                }
                None => report.bridged(
                    GrammarSeverity::Error,
                    BundleFinding::bridged(
                        "unknown-type",
                        doc.path.clone(),
                        format!("unknown type '{ty}' (no archetype registered for it)"),
                    ),
                ),
            }
        }
        BundlePosture::Okf => {
            // Permissive: unknown types are tolerated as a warning; the
            // archetype body contract is not enforced for foreign bundles.
            let ty = doc.concept_type().unwrap_or_default();
            if registry.archetype(ty).is_none() {
                report.bridged(
                    GrammarSeverity::Warning,
                    BundleFinding::bridged(
                        "unknown-type",
                        doc.path.clone(),
                        format!("unknown type '{ty}' tolerated under OKF posture"),
                    ),
                );
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
    severity: &GrammarSeverityMap,
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
                .frontmatter()
                .and_then(|fm| fm.get("okf_version"))
                .is_some();
            if !has_okf {
                report.route(
                    severity,
                    posture_tier(posture),
                    BundleFinding::in_pack(
                        pack::BUNDLE,
                        "index-okf-version",
                        index_doc.path.clone(),
                        "root index.md is missing 'okf_version' in frontmatter".to_string(),
                    ),
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
                report.route(
                    severity,
                    posture_tier(posture),
                    BundleFinding::in_pack(
                        pack::BUNDLE,
                        "index-incomplete",
                        index_doc.path.clone(),
                        format!("index.md does not list sibling artifact '{name}'"),
                    ),
                );
            }
        }
    }
}

/// Basenames of the files linked from an index document's `## Contents`
/// section. Falls back to the whole document when no `Contents` heading is
/// present (a permissive read).
fn contents_basenames(index_doc: &LoadedDocument) -> BTreeSet<String> {
    let text = section(index_doc.body(), "Contents")
        .map(|s| s.content.clone())
        .unwrap_or_else(|| index_doc.raw().to_string());

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
