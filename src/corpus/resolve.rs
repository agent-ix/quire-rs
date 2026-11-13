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

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::ops::Range;
use std::path::{Component, Path, PathBuf};

use regex::Regex;

use crate::corpus::spec::{artifact_key, fold_identity};
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
    /// Sorted, deduplicated `<org>/<repo>` authorities of the `ix://`
    /// spellings of a target that did not resolve in this corpus. Empty for
    /// resolved edges and for targets also spelled as a bare id. Lets a
    /// consumer tell a foreign-repo reference from a broken local one. In a
    /// corpus whose members declare identities, an authority naming a loaded
    /// member is not listed: that repository is visible, so a miss there is
    /// genuinely dangling.
    pub authorities: Vec<String>,
    /// Identity of the source document's member, when it declares one.
    pub source_repo: Option<String>,
    /// Identity of the resolved target's member, when resolved and declared.
    pub target_repo: Option<String>,
    /// Sorted, deduplicated spellings of the target as authored, so a report
    /// can name a dangling reference exactly.
    pub references: Vec<String>,
    /// Corpus slot of the source document.
    pub(crate) source_slot: usize,
    /// Corpus slot of the resolved target document.
    pub(crate) target_slot: Option<usize>,
}

/// Whether an [`Edge`]'s target was found in the corpus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    Resolved,
    Dangling,
}

/// What [`resolve`] reads: the corpus indices built before resolution.
pub(crate) struct Index<'a> {
    pub documents: &'a [LoadedDocument],
    /// Document slot → member.
    pub member_of: &'a [usize],
    pub member_count: usize,
    /// Per member: artifact key → slot.
    pub ids: &'a [HashMap<ArtifactId, usize>],
    /// ASCII-folded member identity → member.
    pub identities: &'a BTreeMap<String, usize>,
    /// Whether members declare identities. When they do an `ix://<org>/<repo>/`
    /// target is looked up in the member with that identity; when they do not
    /// (a one-member corpus) it resolves lexically by its trailing id.
    pub identity_mode: bool,
    /// Per member: its declared identity as given.
    pub declared: &'a [Option<std::sync::Arc<str>>],
}

/// Output of [`resolve`]: the edge set + forward/reverse indices +
/// dangling diagnostics.
pub(crate) struct ResolveOutput {
    /// Edges grouped by source member, sorted within a member.
    pub edges: Vec<Edge>,
    /// Per member: its range in `edges`.
    pub edge_spans: Vec<Range<usize>>,
    /// source slot → edge slots (includes dangling).
    pub outgoing: BTreeMap<usize, Vec<usize>>,
    /// target slot → edge slots (resolved only — dangling has no target doc).
    pub incoming: BTreeMap<usize, Vec<usize>>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Sort key of a harvested reference: source member, source id, target id,
/// edge type, then the resolved target's slot (`None` while dangling). Two
/// members holding one id stay distinct targets.
type StubKey = (usize, ArtifactId, ArtifactId, String, Option<usize>);

/// What the spellings that collapsed into one stub carried.
struct StubInfo {
    /// Unloaded foreign authorities of the spellings; `None` once a spelling
    /// claims the target visible (a bare id, or an authority naming a loaded
    /// member).
    authorities: Option<BTreeSet<String>>,
    /// The spellings as authored.
    references: BTreeSet<String>,
}

/// Resolve every reference in `ix.documents`. Deterministic (stubs are
/// collected into a sorted map before edges are built).
pub(crate) fn resolve(ix: &Index<'_>) -> ResolveOutput {
    let ix_link = ix_link_regex();
    let md_link = md_link_regex();

    // Path→id index per member: the normalized on-disk path of each document
    // maps to its artifact id. Internal relative-path links (ADR 0007) resolve
    // through the source's own member (FR-026-AC-9, FR-085-AC-8).
    let by_path = build_path_index(ix);

    // Harvest into a sorted, deduplicated stub map: identical
    // (source, target, type) triples from any source collapse to one
    // (FR-026-AC-8/AC-11); same-pair-different-type stay distinct.
    let mut stubs: BTreeMap<StubKey, StubInfo> = BTreeMap::new();
    for (slot, doc) in ix.documents.iter().enumerate() {
        let member = ix.member_of[slot];
        let source = artifact_key(doc);
        let mut add = |reference: Reference, edge_type: String| {
            let (target, target_slot, authority) = locate(ix, member, &reference);
            let info = StubInfo {
                authorities: authority.map(|a| BTreeSet::from([a])),
                references: BTreeSet::from([reference.raw.clone()]),
            };
            match stubs.entry((member, source.clone(), target, edge_type, target_slot)) {
                Entry::Vacant(v) => {
                    v.insert(info);
                }
                // A spelling that claims the target visible wins; foreign
                // spellings accumulate every authority.
                Entry::Occupied(mut o) => {
                    let existing = o.get_mut();
                    existing.authorities = match (existing.authorities.take(), info.authorities) {
                        (Some(mut a), Some(b)) => {
                            a.extend(b);
                            Some(a)
                        }
                        _ => None,
                    };
                    existing.references.extend(info.references);
                }
            }
        };
        for (target_raw, edge_type) in harvest_frontmatter(doc) {
            add(Reference::authored(target_raw), edge_type);
        }
        for target_raw in harvest_body_links(doc, ix_link) {
            add(Reference::authored(target_raw), "references".to_string());
        }
        // Internal relative-path links (ADR 0007). Navigation documents
        // (`index.md`) is excluded as a source so its contents links do not
        // flood the graph (FR-026-AC-10).
        if !is_nav_doc(&doc.path) {
            for target in harvest_body_relative_links(doc, md_link, &by_path[member]) {
                // Already an artifact key or a full relative path: no
                // reduction to a last segment, no authority.
                add(Reference::local(target), "references".to_string());
            }
        }
    }

    let mut edges = Vec::with_capacity(stubs.len());
    let mut edge_spans: Vec<Range<usize>> = vec![0..0; ix.member_count];
    let mut outgoing: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    let mut incoming: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let identity_of = |member: usize| ix.declared[member].as_deref().map(str::to_string);

    for ((member, source, target, edge_type, target_slot), info) in stubs {
        let resolution = if target_slot.is_some() {
            Resolution::Resolved
        } else {
            Resolution::Dangling
        };
        let idx = edges.len();
        let source_slot = ix.ids[member][&source];
        outgoing.entry(source_slot).or_default().push(idx);
        match target_slot {
            Some(slot) => incoming.entry(slot).or_default().push(idx),
            None => diagnostics.push(Diagnostic::DanglingReference {
                source: source.clone(),
                target: target.clone(),
                edge_type: edge_type.clone(),
            }),
        }
        let span = &mut edge_spans[member];
        if span.start == span.end {
            span.start = idx;
        }
        span.end = idx + 1;
        edges.push(Edge {
            source,
            target,
            edge_type,
            resolution,
            authorities: info.authorities.into_iter().flatten().collect(),
            source_repo: identity_of(member),
            target_repo: target_slot.and_then(|slot| identity_of(ix.member_of[slot])),
            references: info.references.into_iter().collect(),
            source_slot,
            target_slot,
        });
    }

    ResolveOutput {
        edges,
        edge_spans,
        outgoing,
        incoming,
        diagnostics,
    }
}

/// A harvested target spelling.
struct Reference {
    /// The target exactly as authored.
    raw: String,
    /// Whether it is an `ix://`/bare reference to reduce to a trailing id and
    /// read an authority from; relative-path links are already ids.
    authored: bool,
}

impl Reference {
    fn authored(raw: String) -> Self {
        Self {
            raw,
            authored: true,
        }
    }

    fn local(raw: String) -> Self {
        Self {
            raw,
            authored: false,
        }
    }
}

/// Where a reference harvested in `member` points: its target id, the target
/// document's slot when it resolves, and the unloaded foreign authority to
/// record when it does not.
fn locate(
    ix: &Index<'_>,
    member: usize,
    reference: &Reference,
) -> (ArtifactId, Option<usize>, Option<String>) {
    let (id, authority) = if reference.authored {
        (
            extract_target_id(&reference.raw),
            ix_authority(&reference.raw),
        )
    } else {
        (reference.raw.as_str(), None)
    };
    // The member a target is looked up in. Without declared identities there
    // is one member and the authority plays no part (FR-085-AC-10); with them,
    // an authority picks the member declaring it, and everything else is the
    // source's own member (FR-085-AC-8).
    let (target_member, authority) = match (ix.identity_mode, authority) {
        (true, Some(authority)) => match ix.identities.get(&fold_identity(&authority)) {
            Some(&named) => (Some(named), None),
            None => (None, Some(authority)),
        },
        (true, None) => (Some(member), None),
        (false, authority) => (Some(0), authority),
    };
    let slot = target_member.and_then(|m| ix.ids[m].get(id).copied());
    // A resolved target has no foreign authority to record.
    let authority = if slot.is_some() { None } else { authority };
    (id.to_string(), slot, authority)
}

/// Harvest *all* edge stubs (frontmatter + body `ix://` links) for a
/// single document, dedup'd. Returns `(target_id, edge_type)` pairs
/// where `target_id` is already reduced via `extract_target_id`.
/// This is the public single-doc shape used by the Python binding
/// (`quire.harvest_edges`) — corpus-level resolution still goes
/// through `resolve`.
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

/// `index.md` is a navigation document, not a reference source.
fn is_nav_doc(path: &Path) -> bool {
    path.file_name().and_then(|n| n.to_str()) == Some("index.md")
}

/// Per member: normalized on-disk path → artifact id for every loaded document.
fn build_path_index(ix: &Index<'_>) -> Vec<HashMap<PathBuf, ArtifactId>> {
    let mut by_path = vec![HashMap::new(); ix.member_count];
    for (slot, doc) in ix.documents.iter().enumerate() {
        by_path[ix.member_of[slot]].insert(normalize_lexical(&doc.path), artifact_key(doc));
    }
    by_path
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

/// The `<org>/<repo>` authority of an `ix://` URI: its first two path
/// segments, kept as written (comparison to a package is ASCII
/// case-insensitive, FR-038-AC-10). Only a URI with a third segment names a repo
/// that way; `ix://<org>/<id>` has no repo segment, so it has no authority.
/// `None` for a bare id.
fn ix_authority(target: &str) -> Option<String> {
    let rest = target.strip_prefix("ix://")?;
    let mut segments = rest.split('/').filter(|s| !s.is_empty());
    let (org, repo) = (segments.next()?, segments.next()?);
    segments.next()?;
    Some(format!("{org}/{repo}"))
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
    use ix_trace_rs::trace;

    // target-id extraction is a pure function.
    #[trace("FR-026-AC-6")]
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

    /// Resolve `docs` as a one-member corpus declaring no identity.
    fn run(docs: &[LoadedDocument]) -> ResolveOutput {
        let ids = vec![docs
            .iter()
            .enumerate()
            .rev()
            .map(|(i, d)| (artifact_key(d), i))
            .collect::<HashMap<_, _>>()];
        resolve(&Index {
            documents: docs,
            member_of: &vec![0; docs.len()],
            member_count: 1,
            ids: &ids,
            identities: &BTreeMap::new(),
            identity_mode: false,
            declared: &[None],
        })
    }

    // frontmatter relationship to a present id -> Resolved.
    #[trace("FR-026-AC-1")]
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
        let out = run(&docs);
        let edge = out.edges.iter().find(|e| e.source == "FR-023").unwrap();
        assert_eq!(edge.target, "StR-005");
        assert_eq!(edge.edge_type, "implements");
        assert_eq!(edge.resolution, Resolution::Resolved);
    }

    // ix:// body link to a present id -> Resolved, type "references".
    #[trace("FR-026-AC-2")]
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
        let out = run(&docs);
        let edge = out.edges.iter().find(|e| e.target == "StR-005").unwrap();
        assert_eq!(edge.edge_type, "references");
        assert_eq!(edge.resolution, Resolution::Resolved);
    }

    // absent target -> Dangling + diagnostic; no failure.
    #[trace("FR-026-AC-3")]
    #[test]
    fn absent_target_is_dangling() {
        let docs = vec![loaded(
            "FR-023",
            "relationships:\n  - target: \"StR-999\"\n    type: implements\n",
            "# body\n",
        )];
        let out = run(&docs);
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

    // cross-spec target (absent from loaded set) -> Dangling.
    #[trace("FR-026-AC-4")]
    #[test]
    fn cross_spec_target_is_dangling_not_resolved() {
        // Target lives only in a *different* spec — not in this index.
        let docs = vec![loaded(
            "FR-023",
            "relationships:\n  - target: \"ix://other-org/other-repo/spec/stakeholder/StR-001\"\n    type: requires\n",
            "# body\n",
        )];
        let out = run(&docs);
        assert_eq!(out.edges[0].target, "StR-001");
        assert_eq!(out.edges[0].resolution, Resolution::Dangling);
    }

    // AC-11: one absent target spelled as a bare id and as a
    // foreign ix:// link is one edge, one diagnostic, and the bare spelling
    // keeps it local (no authority).
    #[trace("FR-026-AC-8")]
    #[test]
    fn spellings_of_one_absent_target_dedup_without_splitting_on_authority() {
        let docs = vec![loaded(
            "FR-023",
            "relationships:\n  - target: \"StR-999\"\n    type: references\n",
            "see [x](ix://o/r/spec/StR-999)\n",
        )];
        let out = run(&docs);
        assert_eq!(out.edges.len(), 1);
        assert_eq!(out.diagnostics.len(), 1);
        assert!(out.edges[0].authorities.is_empty());
    }

    #[test]
    fn ix_authority_needs_org_repo_and_a_target_segment() {
        assert_eq!(ix_authority("ix://o/r/spec/StR-1").as_deref(), Some("o/r"));
        assert_eq!(ix_authority("ix://o/r/StR-1").as_deref(), Some("o/r"));
        assert_eq!(ix_authority("ix://o/StR-1"), None);
        assert_eq!(ix_authority("StR-1"), None);
    }

    // a Resolved edge is in both outgoing(src) and incoming(tgt).
    #[trace("FR-026-AC-5")]
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
        let out = run(&docs);
        let from = out.outgoing.get(&0).unwrap();
        let to = out.incoming.get(&1).unwrap();
        assert_eq!(from.len(), 1);
        assert_eq!(to.len(), 1);
        assert_eq!(from[0], to[0]); // same edge slot
    }

    // identical (src,target,type) from both sources -> one edge;
    // same pair, different type -> two.
    #[trace("FR-026-AC-8")]
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
        let out = run(&docs);
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
        let out2 = run(&docs2);
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

    // relative-path body link resolves via the path index
    // (independent of link text and file slug); an unmatched relative link dangles.
    #[trace("FR-026-AC-9")]
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
        let out = run(&docs);
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
        let out2 = run(&docs2);
        assert_eq!(out2.edges.len(), 1);
        assert_eq!(out2.edges[0].resolution, Resolution::Dangling);
    }

    // a relative link to an id-less (path-keyed) document
    // resolves to its path key, and an unresolved nested path is reported whole.
    #[trace("FR-026-AC-9")]
    #[test]
    fn relative_link_to_id_less_doc_resolves_and_nested_miss_keeps_full_path() {
        let idless = LoadedDocument::from_parsed(
            PathBuf::from("spec/sub/b.md"),
            String::new(),
            None,
            parse_document("# b\n"),
        );
        let docs = vec![
            loaded_at(
                "spec/a.md",
                "A-1",
                "see [b](sub/b.md) and [gone](sub/gone.md)\n",
            ),
            idless,
        ];
        let out = run(&docs);
        let hit = out
            .edges
            .iter()
            .find(|e| e.target == "spec/sub/b.md")
            .unwrap();
        assert_eq!(hit.resolution, Resolution::Resolved);
        let miss = out
            .edges
            .iter()
            .find(|e| e.target == "sub/gone.md")
            .unwrap();
        assert_eq!(miss.resolution, Resolution::Dangling);
        assert_eq!(out.edges.len(), 2);
    }

    // relative links in index.md are not harvested;
    // the same link in an ordinary artifact is.
    #[trace("FR-026-AC-10")]
    #[test]
    fn nav_documents_excluded_as_relative_source() {
        let docs = vec![
            loaded_at(
                "spec/functional/index.md",
                "",
                "* [FR-002](./FR-002-graph-edges.md)\n",
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
        let out = run(&docs);
        let refs_to_fr002: Vec<_> = out
            .edges
            .iter()
            .filter(|e| e.target == "FR-002" && e.edge_type == "references")
            .collect();
        // Only the ordinary FR-001 document contributes the edge.
        assert_eq!(refs_to_fr002.len(), 1);
        assert_eq!(refs_to_fr002[0].source, "FR-001");
    }

    // identical (source, target, references) from a
    // relative-path link and an ix:// link / frontmatter entry dedups to one.
    #[trace("FR-026-AC-11")]
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
        let out = run(&docs);
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

    // CR-067: every `ix://` shape the ecosystem
    // actually authors still matches. The counts are occurrences across the
    // 237 `~/dev` spec bundles at the time of the change — this test exists so
    // that a future tightening of the grammar has to argue with real usage
    // rather than with an invented example.
    #[trace("FR-026-AC-12")]
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

    // CR-067, agent-ix/quire-rs#89: the bare protocol
    // and every malformed form the corpus contains mint nothing. The reported
    // defect is the first case: `` `ix://` `` matched as ``ix://` `` and the
    // harvested target was the closing backtick.
    #[trace("FR-026-AC-13")]
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
        let out = run(&docs);
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
        let out = run(&docs);
        assert!(out.edges.is_empty(), "got {:?}", out.edges);
        assert!(out.diagnostics.is_empty(), "got {:?}", out.diagnostics);
    }

    // FR-026-CON-1, CR-067: `harvest_edges` — the single-document
    // surface behind the Python binding — reads the same grammar as corpus
    // resolution, because both go through `harvest_body_links`.
    #[trace("FR-026-CON-1")]
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
    #[trace("FR-026-AC-14")]
    // every clause of the relative-destination (CR-071)
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

    #[trace("FR-026-AC-14")]
    // and end to end — a document whose only (CR-071)
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
        let out = run(&docs);
        assert!(
            out.edges.is_empty(),
            "no excluded destination may mint an edge, got {:?}",
            out.edges
        );
    }
}
