//! Unlinked-reference detection (FR-039).
//!
//! Per ADR 0007, intra-bundle references are authored as relative-path
//! links so the graph is read from explicit links, never from a runtime
//! prose scan. This module finds bare artifact-id tokens in prose that are
//! *not* links and, where the parent id resolves to exactly one in-bundle
//! artifact, emits the exact relative-path link an autofix would splice in.
//!
//! Detection is advisory — it never blocks anything. Each token is sorted
//! into one of three buckets:
//!
//! - **Auto-fix** — prose / table / inline-code token whose parent id
//!   resolves to exactly one *other* loaded artifact. Carries the
//!   `suggested_link` to splice over `byte_span`.
//! - **Warn-only** — parent id resolves to nothing in-bundle (`Unresolved`)
//!   or to more than one artifact (`Ambiguous`). No suggestion.
//! - **Ignore** (no finding) — inside a fenced code block, in frontmatter,
//!   already inside a Markdown link, or a self-reference (parent id == the
//!   document's own id).

use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};

use regex::Regex;

use crate::corpus::resolve::normalize_lexical;
use crate::corpus::spec::artifact_key;
use crate::corpus::walk::LoadedDocument;
use crate::corpus::{ArtifactId, Spec};

/// One bare artifact-id token that should be a link (FR-039).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnlinkedReference {
    /// Document the token was found in.
    pub path: PathBuf,
    /// That document's own artifact id.
    pub source: ArtifactId,
    /// The matched id token, e.g. `"FR-008"` or `"FR-008-CON-4"`.
    pub token: String,
    /// Byte span in the document's source text (`doc.raw`) the autofix
    /// would replace — the token itself, or the whole inline-code span
    /// (backticks included) when the token sits in one.
    pub byte_span: Range<usize>,
    /// How the token is classified.
    pub fix: UnlinkedFix,
}

/// Whether (and how) an [`UnlinkedReference`] can be auto-fixed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnlinkedFix {
    /// Parent id resolves to exactly one in-bundle artifact;
    /// `suggested_link` is the exact Markdown to splice over `byte_span`.
    AutoFix { suggested_link: String },
    /// Not safely fixable. No suggestion is offered.
    WarnOnly { reason: UnlinkedReason },
}

/// Why an [`UnlinkedReference`] is warn-only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnlinkedReason {
    /// Parent id is absent from the loaded set (likely cross-repo, a typo,
    /// or not yet written).
    Unresolved,
    /// Parent id maps to more than one loaded document.
    Ambiguous,
}

/// Detect every unlinked artifact-id reference across the corpus (FR-039).
/// Deterministic: results are sorted by `(path, byte_span.start)`.
pub fn unlinked_references(spec: &Spec) -> Vec<UnlinkedReference> {
    let docs = &spec.inner.documents;

    // id → (count, first path). Documents are path-sorted, so the first
    // insert is the first-wins owner (mirrors `by_id`).
    let mut id_count: HashMap<&str, usize> = HashMap::new();
    let mut id_path: HashMap<&str, &Path> = HashMap::new();
    for d in docs {
        if !d.id.is_empty() {
            *id_count.entry(d.id.as_str()).or_insert(0) += 1;
            id_path
                .entry(d.id.as_str())
                .or_insert_with(|| d.path.as_path());
        }
    }

    let token_re = token_regex();
    let code_re = code_span_regex();
    let link_re = link_regex();

    let mut out = Vec::new();
    for doc in docs {
        scan_document(
            doc, &token_re, &code_re, &link_re, &id_count, &id_path, &mut out,
        );
    }
    out.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.byte_span.start.cmp(&b.byte_span.start))
    });
    out
}

#[allow(clippy::too_many_arguments)]
fn scan_document(
    doc: &LoadedDocument,
    token_re: &Regex,
    code_re: &Regex,
    link_re: &Regex,
    id_count: &HashMap<&str, usize>,
    id_path: &HashMap<&str, &Path>,
    out: &mut Vec<UnlinkedReference>,
) {
    let raw = doc.raw();
    let own_id = doc.id.as_str();
    let source = artifact_key(doc);

    let mut offset = 0usize;
    let mut first_line = true;
    let mut in_frontmatter = false;
    let mut fence: Option<char> = None;

    for line in raw.split_inclusive('\n') {
        let line_start = offset;
        offset += line.len();
        let content = line.strip_suffix('\n').unwrap_or(line);
        let trimmed_end = content.trim_end();

        // Leading frontmatter: only when the very first line is a `---`
        // fence. Everything up to the closing `---` is skipped.
        if first_line {
            first_line = false;
            if trimmed_end == "---" {
                in_frontmatter = true;
                continue;
            }
        }
        if in_frontmatter {
            if trimmed_end == "---" {
                in_frontmatter = false;
            }
            continue;
        }

        // Fenced code blocks: skip the markers and everything between.
        let marker = fence_marker(content);
        if let Some(open) = fence {
            if marker == Some(open) {
                fence = None;
            }
            continue;
        } else if let Some(open) = marker {
            fence = Some(open);
            continue;
        }

        let link_regions: Vec<Range<usize>> =
            link_re.find_iter(content).map(|m| m.range()).collect();
        let code_regions: Vec<Range<usize>> =
            code_re.find_iter(content).map(|m| m.range()).collect();

        // Candidate token ranges, so we can tell how many sit in one
        // inline-code span. A span holding more than one token (e.g.
        // `` `FR-012/FR-013` ``) can't become a single link, and converting
        // each token would emit overlapping whole-span fixes that corrupt on
        // apply — so such tokens are skipped entirely (FR-039).
        let token_ranges: Vec<Range<usize>> =
            token_re.find_iter(content).map(|m| m.range()).collect();

        for m in token_re.find_iter(content) {
            let tr = m.range();
            // Already inside a Markdown link (text or destination): ignore.
            if link_regions
                .iter()
                .any(|r| r.start <= tr.start && tr.end <= r.end)
            {
                continue;
            }
            let token = m.as_str();
            let parent = parent_id(token);
            // Self-reference (own id / own AC/CON rows / H1): ignore.
            if parent == own_id {
                continue;
            }
            // An inline-code token is replaced span-and-all (backticks gone).
            let code_span = code_regions
                .iter()
                .find(|r| r.start <= tr.start && tr.end <= r.end);
            if let Some(cr) = code_span {
                // Multiple candidate tokens share this code span -> skip.
                if token_ranges
                    .iter()
                    .filter(|t| cr.start <= t.start && t.end <= cr.end)
                    .count()
                    > 1
                {
                    continue;
                }
            }
            let local = code_span.cloned().unwrap_or(tr.clone());
            let byte_span = (line_start + local.start)..(line_start + local.end);

            let fix = match id_count.get(parent).copied().unwrap_or(0) {
                0 => UnlinkedFix::WarnOnly {
                    reason: UnlinkedReason::Unresolved,
                },
                1 => {
                    let target = id_path.get(parent).expect("count==1 implies a path");
                    let dest = relative_dest(&doc.path, target);
                    UnlinkedFix::AutoFix {
                        suggested_link: format!("[{token}]({dest})"),
                    }
                }
                _ => UnlinkedFix::WarnOnly {
                    reason: UnlinkedReason::Ambiguous,
                },
            };

            out.push(UnlinkedReference {
                path: doc.path.clone(),
                source: source.clone(),
                token: token.to_string(),
                byte_span,
                fix,
            });
        }
    }
}

/// The fence-marker char (`` ` `` or `~`) if `line` opens/closes a fenced
/// code block, else `None`.
fn fence_marker(line: &str) -> Option<char> {
    let t = line.trim_start();
    if t.starts_with("```") {
        Some('`')
    } else if t.starts_with("~~~") {
        Some('~')
    } else {
        None
    }
}

/// The parent artifact id of a token — the id with any trailing
/// `-AC-<n>` / `-CON-<n>` / `-VC-<n>` suffix stripped (`FR-008-CON-4` →
/// `FR-008`, `StR-001-VC-2` → `StR-001`).
///
/// `-VC-` is StR's validation-criterion kind (CR-020, spec-artifacts-iso#9).
/// Without it a bare `StR-001-VC-2` in prose resolves to no document and
/// `quire fix` offers no link.
fn parent_id(token: &str) -> &str {
    const SUB_ID_KINDS: [&str; 3] = ["-AC-", "-CON-", "-VC-"];
    match SUB_ID_KINDS.iter().filter_map(|k| token.find(k)).min() {
        Some(idx) => &token[..idx],
        None => token,
    }
}

/// A relative-path link destination from `from_file`'s directory to
/// `to_file`, prefixed `./` when they share a directory.
fn relative_dest(from_file: &Path, to_file: &Path) -> String {
    let from_dir = from_file.parent().unwrap_or_else(|| Path::new(""));
    let base = normalize_lexical(from_dir);
    let target = normalize_lexical(to_file);
    let base_c: Vec<_> = base.components().collect();
    let targ_c: Vec<_> = target.components().collect();
    let common = base_c
        .iter()
        .zip(&targ_c)
        .take_while(|(a, b)| a == b)
        .count();

    let mut parts: Vec<String> = Vec::new();
    for _ in common..base_c.len() {
        parts.push("..".to_string());
    }
    for c in &targ_c[common..] {
        parts.push(c.as_os_str().to_string_lossy().into_owned());
    }
    let joined = parts.join("/");
    if joined.is_empty() {
        ".".to_string()
    } else if parts[0] == ".." {
        joined
    } else {
        format!("./{joined}")
    }
}

fn token_regex() -> Regex {
    // Only the known artifact-id prefixes — NOT a generic `[A-Z]{2,4}-\d+`,
    // which over-matches standards/notes like `ISO-8601`, `IMPL-4`, `CR-002`
    // and bare sub-ids like `CON-1`/`AC-2`. A bare `-AC-`/`-CON-`/`-VC-`
    // suffix is only matched as part of a parent artifact id
    // (`FR-008-CON-4`, `StR-001-VC-2`). `StR` is intentionally mixed-case.
    //
    // The sub-id kinds must stay in sync with `parent_id`: a kind stripped
    // there but not matched here truncates the token, so the autofix would
    // link `StR-001` and leave a dangling `-VC-2` behind (CR-020).
    Regex::new(r"\b(?:FR|NFR|StR|US|IT|TC)-[0-9]+(?:-(?:AC|CON|VC)-[0-9]+)?\b")
        .expect("static token regex is valid")
}

fn code_span_regex() -> Regex {
    Regex::new(r"`[^`\n]+`").expect("static code-span regex is valid")
}

fn link_regex() -> Regex {
    Regex::new(r"\[[^\]]*\]\([^)]*\)").expect("static link regex is valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus::walk::RepoLoad;
    use crate::parser::parse_document;
    use ix_trace_rs::trace;

    fn doc_at(path: &str, id: &str, body: &str) -> LoadedDocument {
        let text = format!("---\nid: {id}\ntype: FR\n---\n{body}");
        LoadedDocument::from_parsed(
            PathBuf::from(path),
            id.to_string(),
            None,
            parse_document(&text),
        )
    }

    fn spec_of(docs: Vec<LoadedDocument>) -> Spec {
        Spec::from_repo(RepoLoad {
            documents: docs,
            diagnostics: Vec::new(),
        })
    }

    fn autofix(r: &UnlinkedReference) -> Option<&str> {
        match &r.fix {
            UnlinkedFix::AutoFix { suggested_link } => Some(suggested_link.as_str()),
            _ => None,
        }
    }

    // TC-623 / FR-039-AC-1: bare id in prose -> one AutoFix with token span +
    // suggested relative-path link.
    #[test]
    fn bare_id_in_prose_is_autofix() {
        let spec = spec_of(vec![
            doc_at(
                "spec/functional/FR-001-foo.md",
                "FR-001",
                "See FR-008 for the schema.\n",
            ),
            doc_at("spec/functional/FR-008-byte-exact.md", "FR-008", "# x\n"),
        ]);
        let refs = unlinked_references(&spec);
        let r = refs.iter().find(|r| r.token == "FR-008").unwrap();
        assert_eq!(autofix(r), Some("[FR-008](./FR-008-byte-exact.md)"));
        // Span covers exactly the token text.
        let raw = spec.inner.documents[0].raw();
        assert_eq!(&raw[r.byte_span.clone()], "FR-008");
    }

    // TC-624 / FR-039-AC-2: sub-id links to the PARENT file, label = full token.
    #[test]
    fn sub_id_links_to_parent_file() {
        let spec = spec_of(vec![
            doc_at(
                "spec/functional/FR-001-foo.md",
                "FR-001",
                "per FR-008-CON-4 no self-edges\n",
            ),
            doc_at("spec/functional/FR-008-byte-exact.md", "FR-008", "# x\n"),
        ]);
        let refs = unlinked_references(&spec);
        let r = refs.iter().find(|r| r.token == "FR-008-CON-4").unwrap();
        assert_eq!(autofix(r), Some("[FR-008-CON-4](./FR-008-byte-exact.md)"));
    }

    // TC-625 / FR-039-AC-3: inline-code token -> span covers backticks, link is
    // backtick-free.
    #[test]
    fn inline_code_token_converted() {
        let spec = spec_of(vec![
            doc_at(
                "spec/functional/FR-001-foo.md",
                "FR-001",
                "see `FR-008` here\n",
            ),
            doc_at("spec/functional/FR-008-byte-exact.md", "FR-008", "# x\n"),
        ]);
        let refs = unlinked_references(&spec);
        let r = refs.iter().find(|r| r.token == "FR-008").unwrap();
        let raw = spec.inner.documents[0].raw();
        assert_eq!(&raw[r.byte_span.clone()], "`FR-008`");
        assert_eq!(autofix(r), Some("[FR-008](./FR-008-byte-exact.md)"));
    }

    // TC-632 / FR-039-AC-10: a code span holding >1 artifact token is skipped
    // entirely (converting each would emit overlapping whole-span fixes that
    // corrupt on apply); a single-token code span still converts.
    #[test]
    fn multi_token_code_span_skipped() {
        let spec = spec_of(vec![
            doc_at(
                "spec/functional/FR-001-foo.md",
                "FR-001",
                "see `FR-008/FR-009` together\n",
            ),
            doc_at("spec/functional/FR-008-byte-exact.md", "FR-008", "# x\n"),
            doc_at("spec/functional/FR-009-baz.md", "FR-009", "# y\n"),
        ]);
        let refs = unlinked_references(&spec);
        assert!(
            refs.iter()
                .all(|r| r.token != "FR-008" && r.token != "FR-009"),
            "multi-token code span must produce no fixes"
        );

        // A single-token code span still converts (regression guard).
        let spec2 = spec_of(vec![
            doc_at(
                "spec/functional/FR-001-foo.md",
                "FR-001",
                "see `FR-008` alone\n",
            ),
            doc_at("spec/functional/FR-008-byte-exact.md", "FR-008", "# x\n"),
        ]);
        assert!(unlinked_references(&spec2)
            .iter()
            .any(|r| r.token == "FR-008"));
    }

    // TC-626 / FR-039-AC-4: fenced block + frontmatter tokens yield no finding.
    #[test]
    fn fenced_and_frontmatter_ignored() {
        let spec = spec_of(vec![
            doc_at(
                "spec/functional/FR-001-foo.md",
                "FR-001",
                "```\nFR-008 in code\n```\nclean prose\n",
            ),
            doc_at("spec/functional/FR-008-byte-exact.md", "FR-008", "# x\n"),
        ]);
        let refs = unlinked_references(&spec);
        assert!(refs.iter().all(|r| r.token != "FR-008"));
    }

    // TC-627 / FR-039-AC-5: already-linked tokens yield nothing (idempotence).
    #[test]
    fn already_linked_ignored() {
        let spec = spec_of(vec![
            doc_at(
                "spec/functional/FR-001-foo.md",
                "FR-001",
                "see [FR-008](./FR-008-byte-exact.md) and [x](ix://o/r/FR-008)\n",
            ),
            doc_at("spec/functional/FR-008-byte-exact.md", "FR-008", "# x\n"),
        ]);
        let refs = unlinked_references(&spec);
        assert!(refs.iter().all(|r| r.token != "FR-008"));
    }

    // TC-628 / FR-039-AC-6: self-references skipped; cross-references kept.
    #[test]
    fn self_reference_skipped() {
        let spec = spec_of(vec![
            doc_at(
                "spec/functional/FR-024-containers.md",
                "FR-024",
                "# [FR-024] Containers\n\n| FR-024-AC-1 | does X |\n\nrefs FR-008 too\n",
            ),
            doc_at("spec/functional/FR-008-byte-exact.md", "FR-008", "# x\n"),
        ]);
        let refs = unlinked_references(&spec);
        assert!(refs.iter().all(|r| parent_id(&r.token) != "FR-024"));
        assert!(refs.iter().any(|r| r.token == "FR-008"));
    }

    // TC-629 / FR-039-AC-7: unresolved parent -> WarnOnly{Unresolved}, no link.
    #[test]
    fn unresolved_is_warn_only() {
        let spec = spec_of(vec![doc_at(
            "spec/functional/FR-001-foo.md",
            "FR-001",
            "see FR-900 elsewhere\n",
        )]);
        let refs = unlinked_references(&spec);
        let r = refs.iter().find(|r| r.token == "FR-900").unwrap();
        assert_eq!(
            r.fix,
            UnlinkedFix::WarnOnly {
                reason: UnlinkedReason::Unresolved
            }
        );
    }

    // TC-630 / FR-039-AC-8: duplicate parent id -> WarnOnly{Ambiguous}.
    #[test]
    fn ambiguous_is_warn_only() {
        let spec = spec_of(vec![
            doc_at(
                "spec/functional/FR-001-foo.md",
                "FR-001",
                "see FR-008 dup\n",
            ),
            doc_at("spec/a/FR-008-byte-exact.md", "FR-008", "# x\n"),
            doc_at("spec/b/FR-008-other.md", "FR-008", "# y\n"),
        ]);
        let refs = unlinked_references(&spec);
        let r = refs.iter().find(|r| r.token == "FR-008").unwrap();
        assert_eq!(
            r.fix,
            UnlinkedFix::WarnOnly {
                reason: UnlinkedReason::Ambiguous
            }
        );
    }

    // TC-631 / FR-039-AC-9: results sorted by (path, span.start), stable.
    #[test]
    fn results_sorted_and_stable() {
        let docs = vec![
            doc_at(
                "spec/functional/FR-002-bar.md",
                "FR-002",
                "FR-008 then FR-008 again\n",
            ),
            doc_at("spec/functional/FR-001-foo.md", "FR-001", "FR-008 once\n"),
            doc_at("spec/functional/FR-008-byte-exact.md", "FR-008", "# x\n"),
        ];
        let spec = spec_of(docs.clone());
        let refs = unlinked_references(&spec);
        for w in refs.windows(2) {
            let a = (&w[0].path, w[0].byte_span.start);
            let b = (&w[1].path, w[1].byte_span.start);
            assert!(a <= b, "results must be sorted by (path, span.start)");
        }
        // Stable across a re-run.
        assert_eq!(unlinked_references(&spec_of(docs)), refs);
    }

    #[trace("TC-765", "FR-039-AC-11")]
    // a sub-id token resolves to its parent (CR-020)
    // document for every declared sub-id kind. `-VC-` is StR's
    // validation-criterion kind (spec-artifacts-iso#9); without it a bare
    // `StR-001-VC-2` in prose resolves to nothing and `quire fix` offers no
    // link — the same regression `-AC-`/`-CON-` would have had.
    #[test]
    fn tc765_sub_id_kinds_resolve_to_their_parent() {
        assert_eq!(parent_id("FR-008-AC-3"), "FR-008");
        assert_eq!(parent_id("FR-008-CON-4"), "FR-008");
        assert_eq!(parent_id("StR-001-VC-2"), "StR-001");
        // A plain artifact id is returned unchanged.
        assert_eq!(parent_id("StR-001"), "StR-001");
        // Nothing resembling a sub-id kind is stripped.
        assert_eq!(parent_id("FR-008-VECTOR-1"), "FR-008-VECTOR-1");

        // End to end: a bare `StR-001-VC-2` in an FR's prose is reported as an
        // unlinked reference to the StR that owns it.
        let docs = vec![
            doc_at(
                "spec/functional/FR-001-foo.md",
                "FR-001",
                "Satisfies StR-001-VC-2 at the import boundary.\n",
            ),
            doc_at("spec/stakeholder/StR-001-need.md", "StR-001", "# x\n"),
        ];
        let refs = unlinked_references(&spec_of(docs));
        assert!(
            refs.iter().any(|r| r.token == "StR-001-VC-2"),
            "a -VC- sub-id must resolve to its StR: {refs:?}"
        );
    }
}
