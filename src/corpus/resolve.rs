//! Intra-spec reference resolution (FR-026).
//!
//! Harvests edge stubs from two already-parsed sources — frontmatter
//! `relationships` arrays and `ix://` body links — unifies + dedups
//! them, and resolves each target against the corpus id index. A
//! target present in the loaded set is [`Resolution::Resolved`];
//! anything else (including a target that lives only in a *different*
//! spec) is [`Resolution::Dangling`]. Resolution is O(edges) — one hash
//! lookup per stub — and never reaches outside the loaded set
//! (StR-006-AC-4).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Component, Path, PathBuf};

use regex::Regex;

use crate::corpus::spec::artifact_key;
use crate::corpus::walk::LoadedDocument;
use crate::corpus::ArtifactId;
use crate::diagnostic::Diagnostic;

/// A resolved or dangling reference between two artifacts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    /// Source artifact id (the document the reference was harvested from).
    pub source: ArtifactId,
    /// Target artifact id (last `/`-segment of an `ix://` URI, or a bare id).
    pub target: ArtifactId,
    /// Edge type — the `relationships` entry's `type`, or `references`
    /// for an `ix://` body link.
    pub edge_type: String,
    /// Whether the target is present in the loaded set.
    pub resolution: Resolution,
}

/// Whether an [`Edge`]'s target was found in the corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    Resolved,
    Dangling,
}

/// Output of [`resolve`]: the edge set + forward/reverse indices +
/// dangling diagnostics.
pub(crate) struct ResolveOutput {
    pub edges: Vec<Edge>,
    /// source id → edge slots (includes dangling).
    pub outgoing: BTreeMap<ArtifactId, Vec<usize>>,
    /// target id → edge slots (resolved only — dangling has no target doc).
    pub incoming: BTreeMap<ArtifactId, Vec<usize>>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Resolve every reference in `documents` against `by_id`. Deterministic
/// (stubs are collected into a sorted set before edges are built).
pub(crate) fn resolve(
    documents: &[LoadedDocument],
    by_id: &HashMap<ArtifactId, usize>,
) -> ResolveOutput {
    let ix_link = ix_link_regex();
    let md_link = md_link_regex();

    // Path→id index over the loaded set: the normalized on-disk path of
    // each document maps to its artifact id. Internal relative-path links
    // (ADR 0007) resolve through this (FR-026-AC-9).
    let by_path = build_path_index(documents);

    // Harvest into a sorted, deduplicated stub set: identical
    // (source, target, type) triples from any source collapse to one
    // (FR-026-AC-8/AC-11); same-pair-different-type stay distinct.
    let mut stubs: BTreeSet<(ArtifactId, ArtifactId, String)> = BTreeSet::new();
    for doc in documents {
        let source = artifact_key(doc);
        for (target_raw, edge_type) in harvest_frontmatter(doc) {
            stubs.insert((
                source.clone(),
                extract_target_id(&target_raw).to_string(),
                edge_type,
            ));
        }
        for target_raw in harvest_body_links(doc, ix_link) {
            stubs.insert((
                source.clone(),
                extract_target_id(&target_raw).to_string(),
                "references".to_string(),
            ));
        }
        // Internal relative-path links (ADR 0007). Navigation documents
        // (`index.md`/`log.md`) are excluded as a source so their
        // wall-to-wall contents links do not flood the graph (FR-026-AC-10).
        if !is_nav_doc(&doc.path) {
            for target in harvest_body_relative_links(doc, md_link, &by_path) {
                stubs.insert((source.clone(), target, "references".to_string()));
            }
        }
    }

    let mut edges = Vec::with_capacity(stubs.len());
    let mut outgoing: BTreeMap<ArtifactId, Vec<usize>> = BTreeMap::new();
    let mut incoming: BTreeMap<ArtifactId, Vec<usize>> = BTreeMap::new();
    let mut diagnostics = Vec::new();

    for (source, target, edge_type) in stubs {
        let resolution = if by_id.contains_key(&target) {
            Resolution::Resolved
        } else {
            Resolution::Dangling
        };
        let idx = edges.len();
        outgoing.entry(source.clone()).or_default().push(idx);
        match resolution {
            Resolution::Resolved => {
                incoming.entry(target.clone()).or_default().push(idx);
            }
            Resolution::Dangling => diagnostics.push(Diagnostic::DanglingReference {
                source: source.clone(),
                target: target.clone(),
                edge_type: edge_type.clone(),
            }),
        }
        edges.push(Edge {
            source,
            target,
            edge_type,
            resolution,
        });
    }

    ResolveOutput {
        edges,
        outgoing,
        incoming,
        diagnostics,
    }
}

/// Harvest *all* edge stubs (frontmatter + body `ix://` links) for a
/// single document, dedup'd. Returns `(target_id, edge_type)` pairs
/// where `target_id` is already reduced via [`extract_target_id`].
/// This is the public single-doc shape used by the Python binding
/// (`quire.harvest_edges`) — corpus-level resolution still goes
/// through [`resolve`].
pub fn harvest_edges(doc: &LoadedDocument) -> Vec<(String, String)> {
    let ix_link = ix_link_regex();
    let mut out: BTreeSet<(String, String)> = BTreeSet::new();
    for (target_raw, edge_type) in harvest_frontmatter(doc) {
        out.insert((extract_target_id(&target_raw).to_string(), edge_type));
    }
    for target_raw in harvest_body_links(doc, ix_link) {
        out.insert((
            extract_target_id(&target_raw).to_string(),
            "references".to_string(),
        ));
    }
    out.into_iter().collect()
}

/// `(target, edge_type)` pairs from the document's frontmatter
/// `relationships` array. Entries missing a `target` are skipped;
/// entries missing a `type` default to `references`.
fn harvest_frontmatter(doc: &LoadedDocument) -> Vec<(String, String)> {
    // Header tier only (CR-047): resolution stays eager with zero body parses.
    let Some(fm) = doc.frontmatter() else {
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
            Some((target.to_string(), edge_type.to_string()))
        })
        .collect()
}

/// Raw `ix://` URIs found in the document body, matched against the
/// [`ix_link_regex`] grammar.
///
/// One rule the regex cannot express (the `regex` crate has no lookahead):
/// **a match immediately followed by `/` is discarded.** A trailing slash means
/// the next segment failed the grammar, so the URI is truncated rather than
/// complete — `ix://org/repo/...` would otherwise match its first two segments
/// and mint an edge to `repo`, which is not what the author wrote (CR-067).
fn harvest_body_links(doc: &LoadedDocument, re: &Regex) -> Vec<String> {
    let raw = doc.raw();
    re.find_iter(raw)
        .filter(|m| raw.as_bytes().get(m.end()) != Some(&b'/'))
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Resolve internal relative-path links (`[text](./FR-002-….md)`) in the
/// document body against the corpus path index (FR-026-AC-9). A link whose
/// normalized destination matches a loaded document yields that document's
/// id; one that matches nothing yields the raw destination string, so it
/// resolves to `Dangling` downstream like an absent `ix://` target.
fn harvest_body_relative_links(
    doc: &LoadedDocument,
    re: &Regex,
    by_path: &HashMap<PathBuf, ArtifactId>,
) -> Vec<String> {
    let base = doc.path.parent().unwrap_or_else(|| Path::new(""));
    re.captures_iter(doc.raw())
        .filter_map(|c| c.get(1).map(|m| m.as_str()))
        .filter_map(|dest| {
            let path_part = dest.split('#').next().unwrap_or(dest);
            if !is_relative_md_dest(path_part) {
                return None;
            }
            let normalized = normalize_lexical(&base.join(path_part));
            Some(match by_path.get(&normalized) {
                Some(id) => id.clone(),
                None => dest.to_string(),
            })
        })
        .collect()
}

/// True when a Markdown link destination is an internal relative path to a
/// markdown artifact (not an `ix://`/`http(s)`/`mailto:` URI, not a bare
/// in-document `#anchor`).
fn is_relative_md_dest(dest: &str) -> bool {
    !dest.is_empty()
        && !dest.contains("://")
        && !dest.starts_with('#')
        && !dest.starts_with("mailto:")
        && !dest.starts_with("tel:")
        && dest.ends_with(".md")
}

/// `index.md` / `log.md` are navigation documents, not reference sources.
fn is_nav_doc(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|n| n.to_str()),
        Some("index.md") | Some("log.md")
    )
}

/// Normalized on-disk path → artifact id for every loaded document.
fn build_path_index(documents: &[LoadedDocument]) -> HashMap<PathBuf, ArtifactId> {
    documents
        .iter()
        .map(|d| (normalize_lexical(&d.path), artifact_key(d)))
        .collect()
}

/// Lexically normalize a path — collapse `.` and `..` segments without any
/// filesystem access (resolution stays I/O-free and deterministic,
/// StR-006-AC-4 / NFR-006).
pub(crate) fn normalize_lexical(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Compiled once (CR-072). Both link regexes were rebuilt on every call, and
/// [`harvest_edges`] is the **per-document** surface the Python binding exposes,
/// so a consumer walking N documents paid N compilations. Measured: **148µs to
/// compile against 4.8µs to scan a 20-line document — 31× the work it enables**.
/// Pre-existing rather than introduced by the CR-067 grammar (the blacklist it
/// replaced compiled in 161µs, slightly slower), and found by the Wave A review.
///
/// `OnceLock` here follows `declared_tables.rs` and is a named exemption in
/// `scripts/audits/check_no_shared_mutable.sh`: idempotent deterministic init,
/// outside the FR-024 parallel region.
fn md_link_regex() -> &'static Regex {
    // Match a Markdown inline link's destination: `[text](dest)`, where
    // `dest` runs up to the closing paren or whitespace. `ix://` and other
    // URI/anchor destinations are filtered out by `is_relative_md_dest`.
    static R: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r#"\[[^\]]*\]\(([^)\s]+)\)"#).expect("static md-link regex is valid")
    })
}

/// The artifact id a reference target points at: the last `/`-segment
/// of an `ix://` URI, or a bare id unchanged (FR-026-AC-6).
pub(crate) fn extract_target_id(target: &str) -> &str {
    let rest = target.strip_prefix("ix://").unwrap_or(target);
    rest.rsplit('/').next().unwrap_or(rest)
}

/// The `ix://` URI grammar (CR-067):
///
/// ```text
/// ix-uri   = "ix://" segment ( "/" segment )+ ( "#" fragment )?
/// segment  = URI-legal characters, at least one of them alphanumeric
/// ```
///
/// Two segments minimum, because `ix://agent-ix/workflow-service` — a
/// repo-level reference — is a real and common form. The last segment is
/// **not** required to look like an artifact id: `ix://agent-ix/ecaz/`​`master-requirements`
/// and `ix://agent-ix/ecaz/spire-partition-object-header` reference declared
/// objects, not `XX-000` ids, and both are legitimate.
///
/// This replaced a blacklist (`ix://[^\s)\]>"']+`) that did not treat a
/// backtick as a delimiter, so prose naming the protocol — a bare
/// `` `ix://` `` — matched as ``ix://` `` and minted a reference whose target
/// was the closing backtick (agent-ix/quire-rs#89). Stating which characters a
/// URI *may* contain, rather than guessing which ones end it, also rejects the
/// documentation templates the blacklist accepted: `ix://{code}`,
/// `ix://<org>/<repo>/…`, and an `ix://([^)]+)` regex quoted in prose.
///
/// Backticks and fenced blocks are **not** consulted. A well-formed `ix://` URI
/// is a reference to another artifact wherever it appears; a code span is
/// typography. (FR-039 already takes the same position from the other side — it
/// converts a backticked artifact id *into* a link.)
/// Compiled once — see [`md_link_regex`] for the measurement (CR-072).
fn ix_link_regex() -> &'static Regex {
    // At least one alphanumeric, written inline because the `regex` crate has
    // no lookahead. This is what stops `...` and `--` from being segments.
    const SEG: &str = r"[A-Za-z0-9._~@%+-]*[A-Za-z0-9][A-Za-z0-9._~@%+-]*";
    static R: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    R.get_or_init(|| {
        Regex::new(&format!(r"ix://{SEG}(?:/{SEG})+(?:#[A-Za-z0-9._~-]+)?"))
            .expect("static ix-link regex is valid")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // TC-491 / FR-026-AC-6: target-id extraction is a pure function.
    #[test]
    fn extract_target_id_handles_uri_and_bare() {
        assert_eq!(
            extract_target_id("ix://agent-ix/quire-rs/spec/functional/FR-021"),
            "FR-021"
        );
        assert_eq!(extract_target_id("FR-021"), "FR-021");
        assert_eq!(extract_target_id("ix://x/StR-001"), "StR-001");
    }

    #[test]
    fn ix_regex_extracts_from_markdown_link_and_autolink() {
        let re = ix_link_regex();
        let body = "see [the FR](ix://o/r/spec/functional/FR-021) and <ix://o/r/StR-001>.";
        let hits: Vec<_> = re.find_iter(body).map(|m| m.as_str()).collect();
        assert_eq!(
            hits,
            vec!["ix://o/r/spec/functional/FR-021", "ix://o/r/StR-001"]
        );
    }

    use crate::parser::parse_document;
    use std::path::PathBuf;

    fn loaded(id: &str, frontmatter_extra: &str, body: &str) -> LoadedDocument {
        let text = format!("---\nid: {id}\n{frontmatter_extra}---\n{body}");
        LoadedDocument::from_parsed(
            PathBuf::from(format!("{id}.md")),
            id.to_string(),
            None,
            parse_document(&text),
        )
    }

    fn index(docs: &[LoadedDocument]) -> HashMap<ArtifactId, usize> {
        docs.iter()
            .enumerate()
            .map(|(i, d)| (artifact_key(d), i))
            .collect()
    }

    // TC-486 / FR-026-AC-1: frontmatter relationship to a present id -> Resolved.
    #[test]
    fn frontmatter_relationship_resolves() {
        let docs = vec![
            loaded(
                "FR-023",
                "relationships:\n  - target: \"ix://o/r/spec/stakeholder/StR-005\"\n    type: implements\n",
                "# body\n",
            ),
            loaded("StR-005", "", "# need\n"),
        ];
        let out = resolve(&docs, &index(&docs));
        let edge = out.edges.iter().find(|e| e.source == "FR-023").unwrap();
        assert_eq!(edge.target, "StR-005");
        assert_eq!(edge.edge_type, "implements");
        assert_eq!(edge.resolution, Resolution::Resolved);
    }

    // TC-487 / FR-026-AC-2: ix:// body link to a present id -> Resolved, type "references".
    #[test]
    fn body_link_resolves() {
        let docs = vec![
            loaded(
                "FR-023",
                "",
                "see [it](ix://o/r/spec/stakeholder/StR-005)\n",
            ),
            loaded("StR-005", "", "# need\n"),
        ];
        let out = resolve(&docs, &index(&docs));
        let edge = out.edges.iter().find(|e| e.target == "StR-005").unwrap();
        assert_eq!(edge.edge_type, "references");
        assert_eq!(edge.resolution, Resolution::Resolved);
    }

    // TC-488 / FR-026-AC-3: absent target -> Dangling + diagnostic; no failure.
    #[test]
    fn absent_target_is_dangling() {
        let docs = vec![loaded(
            "FR-023",
            "relationships:\n  - target: \"StR-999\"\n    type: implements\n",
            "# body\n",
        )];
        let out = resolve(&docs, &index(&docs));
        let edge = &out.edges[0];
        assert_eq!(edge.target, "StR-999");
        assert_eq!(edge.resolution, Resolution::Dangling);
        assert_eq!(
            out.diagnostics
                .iter()
                .filter(|d| matches!(d, Diagnostic::DanglingReference { .. }))
                .count(),
            1
        );
    }

    // TC-489 / FR-026-AC-4: cross-spec target (absent from loaded set) -> Dangling.
    #[test]
    fn cross_spec_target_is_dangling_not_resolved() {
        // Target lives only in a *different* spec — not in this index.
        let docs = vec![loaded(
            "FR-023",
            "relationships:\n  - target: \"ix://other-org/other-repo/spec/stakeholder/StR-001\"\n    type: requires\n",
            "# body\n",
        )];
        let out = resolve(&docs, &index(&docs));
        assert_eq!(out.edges[0].target, "StR-001");
        assert_eq!(out.edges[0].resolution, Resolution::Dangling);
    }

    // TC-490 / FR-026-AC-5: a Resolved edge is in both outgoing(src) and incoming(tgt).
    #[test]
    fn resolved_edge_is_bidirectional() {
        let docs = vec![
            loaded(
                "FR-023",
                "relationships:\n  - target: \"StR-005\"\n    type: implements\n",
                "# body\n",
            ),
            loaded("StR-005", "", "# need\n"),
        ];
        let out = resolve(&docs, &index(&docs));
        let from = out.outgoing.get("FR-023").unwrap();
        let to = out.incoming.get("StR-005").unwrap();
        assert_eq!(from.len(), 1);
        assert_eq!(to.len(), 1);
        assert_eq!(from[0], to[0]); // same edge slot
    }

    // TC-501 / FR-026-AC-8: identical (src,target,type) from both sources -> one edge;
    // same pair, different type -> two.
    #[test]
    fn dedup_collapses_identical_triples() {
        // frontmatter `references` StR-005 AND a body ix:// link to StR-005
        // (also type `references`) => one edge.
        let docs = vec![
            loaded(
                "FR-023",
                "relationships:\n  - target: \"StR-005\"\n    type: references\n",
                "see [it](ix://o/r/spec/stakeholder/StR-005)\n",
            ),
            loaded("StR-005", "", "# need\n"),
        ];
        let out = resolve(&docs, &index(&docs));
        let to_str005: Vec<_> = out
            .edges
            .iter()
            .filter(|e| e.source == "FR-023" && e.target == "StR-005")
            .collect();
        assert_eq!(to_str005.len(), 1, "identical triples should dedup");

        // Different type from each source => two distinct edges.
        let docs2 = vec![
            loaded(
                "FR-024",
                "relationships:\n  - target: \"StR-005\"\n    type: implements\n",
                "see [it](ix://o/r/spec/stakeholder/StR-005)\n",
            ),
            loaded("StR-005", "", "# need\n"),
        ];
        let out2 = resolve(&docs2, &index(&docs2));
        let types: BTreeSet<_> = out2
            .edges
            .iter()
            .filter(|e| e.source == "FR-024" && e.target == "StR-005")
            .map(|e| e.edge_type.clone())
            .collect();
        assert_eq!(
            types,
            BTreeSet::from(["implements".to_string(), "references".to_string()])
        );
    }

    /// A document at an explicit path with a slug-suffixed filename (id != slug),
    /// so relative-path resolution is exercised independently of the file slug.
    fn loaded_at(path: &str, id: &str, body: &str) -> LoadedDocument {
        let text = format!("---\nid: {id}\n---\n{body}");
        LoadedDocument::from_parsed(
            PathBuf::from(path),
            id.to_string(),
            None,
            parse_document(&text),
        )
    }

    // TC-620 / FR-026-AC-9: relative-path body link resolves via the path index
    // (independent of link text and file slug); an unmatched relative link dangles.
    #[test]
    fn relative_path_link_resolves_and_dangles() {
        let docs = vec![
            loaded_at(
                "spec/functional/FR-001-foo.md",
                "FR-001",
                "see [the schema](./FR-002-graph-edges.md) and [up](../stakeholder/StR-005-need.md)\n",
            ),
            loaded_at("spec/functional/FR-002-graph-edges.md", "FR-002", "# schema\n"),
            loaded_at("spec/stakeholder/StR-005-need.md", "StR-005", "# need\n"),
        ];
        let out = resolve(&docs, &index(&docs));
        let to_fr002 = out
            .edges
            .iter()
            .find(|e| e.source == "FR-001" && e.target == "FR-002")
            .unwrap();
        assert_eq!(to_fr002.edge_type, "references");
        assert_eq!(to_fr002.resolution, Resolution::Resolved);
        // `../` segment resolves across directories.
        assert!(out
            .edges
            .iter()
            .any(|e| e.target == "StR-005" && e.resolution == Resolution::Resolved));

        // A relative link to a path not in the loaded set -> Dangling.
        let docs2 = vec![loaded_at(
            "spec/functional/FR-001-foo.md",
            "FR-001",
            "see [missing](./FR-999-missing.md)\n",
        )];
        let out2 = resolve(&docs2, &index(&docs2));
        assert_eq!(out2.edges.len(), 1);
        assert_eq!(out2.edges[0].resolution, Resolution::Dangling);
    }

    // TC-621 / FR-026-AC-10: relative links in index.md/log.md are not harvested;
    // the same link in an ordinary artifact is.
    #[test]
    fn nav_documents_excluded_as_relative_source() {
        let docs = vec![
            loaded_at(
                "spec/functional/index.md",
                "",
                "* [FR-002](./FR-002-graph-edges.md)\n",
            ),
            loaded_at(
                "spec/functional/log.md",
                "",
                "* changed [FR-002](./FR-002-graph-edges.md)\n",
            ),
            loaded_at(
                "spec/functional/FR-001-foo.md",
                "FR-001",
                "see [it](./FR-002-graph-edges.md)\n",
            ),
            loaded_at(
                "spec/functional/FR-002-graph-edges.md",
                "FR-002",
                "# schema\n",
            ),
        ];
        let out = resolve(&docs, &index(&docs));
        let refs_to_fr002: Vec<_> = out
            .edges
            .iter()
            .filter(|e| e.target == "FR-002" && e.edge_type == "references")
            .collect();
        // Only the ordinary FR-001 document contributes the edge.
        assert_eq!(refs_to_fr002.len(), 1);
        assert_eq!(refs_to_fr002[0].source, "FR-001");
    }

    // TC-622 / FR-026-AC-11: identical (source, target, references) from a
    // relative-path link and an ix:// link / frontmatter entry dedups to one.
    #[test]
    fn relative_and_ix_link_dedup_to_one() {
        let docs = vec![
            loaded_at(
                "spec/functional/FR-001-foo.md",
                "FR-001",
                "rel [a](./FR-002-graph-edges.md) and ix [b](ix://o/r/FR-002)\n",
            ),
            loaded_at(
                "spec/functional/FR-002-graph-edges.md",
                "FR-002",
                "# schema\n",
            ),
        ];
        let out = resolve(&docs, &index(&docs));
        let edges: Vec<_> = out
            .edges
            .iter()
            .filter(|e| e.source == "FR-001" && e.target == "FR-002")
            .collect();
        assert_eq!(
            edges.len(),
            1,
            "relative + ix:// to same target should dedup"
        );
        assert_eq!(edges[0].edge_type, "references");
    }

    // TC-880 / FR-026-AC-12 (CR-067): every `ix://` shape the ecosystem
    // actually authors still matches. The counts are occurrences across the
    // 237 `~/dev` spec bundles at the time of the change — this test exists so
    // that a future tightening of the grammar has to argue with real usage
    // rather than with an invented example.
    #[test]
    fn tc880_grammar_accepts_every_authored_shape() {
        let re = ix_link_regex();
        let hit = |s: &str| re.find(s).map(|m| m.as_str().to_string());

        // org/repo/ID — 5,080 occurrences, the dominant form.
        assert_eq!(
            hit("ix://agent-ix/ecaz/FR-048").as_deref(),
            Some("ix://agent-ix/ecaz/FR-048")
        );
        // org/repo/spec/class/ID — 540, the form FR-026-AC-6 documents.
        assert_eq!(
            hit("ix://agent-ix/quire-rs/spec/functional/FR-021").as_deref(),
            Some("ix://agent-ix/quire-rs/spec/functional/FR-021")
        );
        // org/repo — 225. A repo-level reference is why the minimum is two
        // segments and not three.
        assert_eq!(
            hit("ix://agent-ix/workflow-service").as_deref(),
            Some("ix://agent-ix/workflow-service")
        );
        // org/repo/spec/class/subdir/ID — 107 (`ix-ui`/`ix-cli` nest by area).
        assert_eq!(
            hit("ix://agent-ix/ix-ui/spec/functional/cli/FR-001").as_deref(),
            Some("ix://agent-ix/ix-ui/spec/functional/cli/FR-001")
        );
        // A target that is a declared object slug, not an `XX-000` id — 55 +
        // 20 occurrences. "The last segment must look like FR-001" would be a
        // wrong rule, and this is the evidence.
        assert_eq!(
            hit("ix://agent-ix/spec-artifacts-iso/master-requirements").as_deref(),
            Some("ix://agent-ix/spec-artifacts-iso/master-requirements")
        );
        assert_eq!(
            hit("ix://agent-ix/ecaz/spire-partition-object-header").as_deref(),
            Some("ix://agent-ix/ecaz/spire-partition-object-header")
        );
        // Underscored object-type form, and a non-`agent-ix` authority.
        assert_eq!(
            hit("ix://agent-ix/identity/aggregate_root/User").as_deref(),
            Some("ix://agent-ix/identity/aggregate_root/User")
        );
        assert_eq!(
            hit("ix://npm/react-router-dom").as_deref(),
            Some("ix://npm/react-router-dom")
        );
        // A `#fragment` is part of the URI.
        assert_eq!(
            hit("ix://agent-ix/ecaz/spire-leaf-v2#segment_tuple").as_deref(),
            Some("ix://agent-ix/ecaz/spire-leaf-v2#segment_tuple")
        );
        // Closing delimiters still end the URI, in every authored wrapper.
        assert_eq!(
            hit("see [the FR](ix://o/r/spec/functional/FR-021) here").as_deref(),
            Some("ix://o/r/spec/functional/FR-021")
        );
        assert_eq!(
            hit("<ix://o/r/StR-001>.").as_deref(),
            Some("ix://o/r/StR-001")
        );
        // And — the whole point of the revision — a well-formed URI inside a
        // code span or a fenced block is still a reference. Backticks are
        // typography, not semantics.
        assert_eq!(
            hit("write `ix://o/r/FR-002` like this").as_deref(),
            Some("ix://o/r/FR-002")
        );
        assert_eq!(
            hit("```\nix://o/r/FR-003\n```").as_deref(),
            Some("ix://o/r/FR-003")
        );
    }

    // TC-881 / FR-026-AC-13 (CR-067, agent-ix/quire-rs#89): the bare protocol
    // and every malformed form the corpus contains mint nothing. The reported
    // defect is the first case: `` `ix://` `` matched as ``ix://` `` and the
    // harvested target was the closing backtick.
    #[test]
    fn tc881_grammar_rejects_the_bare_protocol_and_placeholders() {
        let re = ix_link_regex();
        for malformed in [
            "A broken `ix://` link is tolerated as a warning.", // 158 occurrences
            "the ix:// scheme",
            "ix://",
            "ix://agent-ix",                       // one segment
            "ix://<org>/<repo>/spec/<class>/<ID>", // 11, doc template
            "ix://{code}",                         // 5, placeholder
            "matched by `ix://([^)]+)` in prose",  // a regex, not a URI
        ] {
            assert_eq!(
                re.find(malformed).map(|m| m.as_str()),
                None,
                "must mint nothing: {malformed:?}"
            );
        }

        // A truncated URI is discarded whole rather than matching its first
        // two segments — `ix://org/repo/...` is not a reference to `repo`.
        let docs = vec![loaded("FR-023", "", "see ix://agent-ix/quire-rs/...\n")];
        let out = resolve(&docs, &index(&docs));
        assert!(
            out.edges.is_empty(),
            "a truncated URI must mint nothing, got {:?}",
            out.edges
        );

        // End to end on the reported reproduction: a matrix whose prose
        // documents the link format mints no edge and no diagnostic.
        let docs = vec![loaded(
            "TM-009",
            "",
            "A broken `ix://` link is tolerated as a warning.\n",
        )];
        let out = resolve(&docs, &index(&docs));
        assert!(out.edges.is_empty(), "got {:?}", out.edges);
        assert!(out.diagnostics.is_empty(), "got {:?}", out.diagnostics);
    }

    // TC-882 / FR-026-CON-1 (CR-067): `harvest_edges` — the single-document
    // surface behind the Python binding — reads the same grammar as corpus
    // resolution, because both go through `harvest_body_links`.
    #[test]
    fn tc882_single_doc_harvest_reads_the_same_grammar() {
        let doc = loaded(
            "FR-023",
            "relationships:\n  - target: \"StR-005\"\n    type: implements\n",
            "The `ix://` form is external. See `ix://o/r/spec/stakeholder/StR-007`.\n",
        );
        assert_eq!(
            harvest_edges(&doc),
            vec![
                ("StR-005".to_string(), "implements".to_string()),
                ("StR-007".to_string(), "references".to_string()),
            ],
            "the bare protocol is dropped; the backticked URI is kept"
        );
    }
    // TC-897 (FR-026-AC-14, CR-071): every clause of the relative-destination
    // filter is load-bearing, checked one exclusion at a time.
    //
    // Found by the agent-ix/quoin#48 mutation pilot: `cargo mutants` scoped to
    // FR-026's traced files turned each `&&` in `is_relative_md_dest` into `||`
    // and **no test failed**. The AC already said it — "non-relative
    // destinations (`http(s)://`, `mailto:`, `ix://`, bare in-document
    // `#anchor`) are not relative-path stubs and are ignored by this source" —
    // so this is the half of AC-9 the suite asserted in prose and nowhere else.
    // TC-620 covers the positive case and the dangling case; nothing covered
    // the exclusions.
    #[test]
    fn tc897_every_relative_destination_exclusion_is_load_bearing() {
        // The one accepted shape.
        assert!(is_relative_md_dest("./FR-002-graph-edges.md"));
        assert!(is_relative_md_dest("../stakeholder/StR-005-need.md"));

        // Each exclusion, alone. Any of these flipping to `||` makes one of
        // these pass.
        for excluded in [
            "",                              // empty
            "https://example.com/a.md",      // scheme
            "ix://o/r/FR-002",               // the external form (no .md either)
            "ix://o/r/FR-002.md",            // scheme even with a .md tail
            "#anchor",                       // in-document anchor
            "mailto:someone@example.com.md", // mailto, .md tail and all
            "tel:+15551234.md",              // tel, likewise
            "./FR-002-graph-edges.txt",      // not markdown
            "./FR-002-graph-edges",          // no extension
        ] {
            assert!(
                !is_relative_md_dest(excluded),
                "{excluded:?} must not be harvested as a relative-path stub"
            );
        }
    }

    // TC-897 (FR-026-AC-14, CR-071): and end to end — a document whose only
    // links are excluded destinations mints no edge at all.
    #[test]
    fn tc897_excluded_destinations_mint_no_edges() {
        let docs = vec![loaded_at(
            "spec/functional/FR-001-foo.md",
            "FR-001",
            "[web](https://example.com/a.md) [anchor](#section) \
             [mail](mailto:a@b.com.md) [phone](tel:+15551234.md) \
             [plain](./notes.txt)\n",
        )];
        let out = resolve(&docs, &index(&docs));
        assert!(
            out.edges.is_empty(),
            "no excluded destination may mint an edge, got {:?}",
            out.edges
        );
    }
}
