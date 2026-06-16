//! Real-document parser sweep.
//!
//! Walks the actual markdown spec corpora at sibling-repo paths
//! (`~/dev/spec-artifacts-iso/spec/`, `~/dev/spec-artifacts-app/spec/`,
//! `~/dev/spec-artifacts-process/spec/`) plus this repo's own
//! `spec/` tree, and asserts the parser:
//!
//! 1. Never panics on any real-world document (FR-005-AC-4 dimension
//!    that the proptest covers for synthetic input — this hits it
//!    with input the parser was actually designed for).
//! 2. Always produces a byte-exact round-trip: stitching the parsed
//!    sections back into a body equals the post-frontmatter body
//!    (FR-008-AC-3 against real inputs, not just proptest-generated).
//! 3. Slug IDs are well-formed `<slug>-L<line>` strings (FR-009).
//! 4. Frontmatter that survived `extract_frontmatter` is a JSON
//!    Object — never null, never a scalar (FR-006 invariant).
//! 5. For documents with a top-level `id:` frontmatter field, the
//!    field is a string (sanity check — every artifact in the
//!    corpus declares its id this way).
//!
//! Each sibling corpus is **optional**: if the directory isn't
//! present (CI on a fresh host won't have them), the test skips
//! that root and proceeds. The quire-rs `spec/` tree is always
//! present.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use quire_rs::{extract_frontmatter, parse_document, QuireDocument};

fn candidate_roots() -> Vec<PathBuf> {
    // Prefer the canonical sibling paths. Skip silently when absent.
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    let mut roots = vec![
        // Always present in this repo.
        Path::new(env!("CARGO_MANIFEST_DIR")).join("spec"),
    ];
    for s in [
        "dev/spec-artifacts-iso/spec",
        "dev/spec-artifacts-app/spec",
        "dev/spec-artifacts-process/spec",
    ] {
        let p = home.join(s);
        if p.is_dir() {
            roots.push(p);
        }
    }
    roots
}

fn walk_markdown(root: &Path) -> Vec<PathBuf> {
    fn rec(path: &Path, out: &mut Vec<PathBuf>) {
        let entries = match std::fs::read_dir(path) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if matches!(name, "node_modules" | "target" | ".git" | ".agent") {
                    continue;
                }
                rec(&p, out);
            } else if p.extension().is_some_and(|e| e == "md") {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    rec(root, &mut out);
    out.sort();
    out
}

/// Byte-exact stitch: preamble + (heading line + content)+ should
/// reproduce the parsed body. Re-implements the proptest invariant
/// against real input.
fn reconstruct(body: &str, doc: &QuireDocument) -> String {
    // Walk flat headings via the doc's tree.
    fn flatten<'a>(s: &'a quire_rs::QuireSection, out: &mut Vec<&'a quire_rs::QuireSection>) {
        out.push(s);
        for c in &s.children {
            flatten(c, out);
        }
    }
    let mut flat: Vec<&quire_rs::QuireSection> = Vec::new();
    for s in &doc.sections {
        flatten(s, &mut flat);
    }
    flat.sort_by_key(|s| s.start_line);

    let lines: Vec<&str> = body.split('\n').collect();
    if flat.is_empty() {
        return body.to_string();
    }
    let first: usize = flat[0].start_line;
    let mut out = String::with_capacity(body.len());
    if first > 0 {
        out.push_str(&lines[..first].join("\n"));
        // No trailing '\n' added — the trailing-\n of the preamble
        // belongs to the heading-line boundary; content slice
        // includes it.
        out.push('\n');
    }
    for s in &flat {
        out.push_str(lines[s.start_line]);
        out.push('\n');
        out.push_str(&s.content);
    }
    // If body ended without trailing \n the slice already reflects
    // that; the only edge is when the body had a trailing empty line
    // (final \n). The slice includes everything up to the next
    // heading, so the final section's content already covers it.
    out
}

#[test]
fn parse_real_documents_never_panics_and_rounds_trip() {
    let roots = candidate_roots();
    let mut total = 0usize;
    let mut per_root: BTreeMap<PathBuf, usize> = BTreeMap::new();
    let mut roundtrip_failures: Vec<(PathBuf, String)> = Vec::new();
    let mut slug_failures: Vec<(PathBuf, String)> = Vec::new();
    let mut fm_failures: Vec<(PathBuf, String)> = Vec::new();

    for root in &roots {
        let docs = walk_markdown(root);
        per_root.insert(root.clone(), docs.len());
        for path in docs {
            let bytes = match std::fs::read(&path) {
                Ok(b) => b,
                Err(_) => continue,
            };
            let s = match std::str::from_utf8(&bytes) {
                Ok(s) => s.to_string(),
                Err(_) => continue, // skip non-utf8 blobs (none expected)
            };
            total += 1;

            // (1) no panic
            let doc = parse_document(&s);

            // (4) frontmatter shape — already enforced by the type:
            // `Option<Map<String, Value>>`. Sanity-check the id field
            // when present, per (5).
            if let Some(fm) = &doc.frontmatter {
                if let Some(id) = fm.get("id") {
                    if !id.is_string() {
                        fm_failures
                            .push((path.clone(), format!("frontmatter.id is non-string: {id}")));
                    }
                }
            }

            // (3) every section's id must look like `<slug>-L<line>`.
            verify_slug_ids(&doc.sections, &path, &mut slug_failures);

            // (2) round-trip stitch on the post-frontmatter body.
            let fm_result = extract_frontmatter(&s);
            let body = fm_result.body;
            let stitched = reconstruct(&body, &doc);
            if stitched != body {
                let (a_excerpt, b_excerpt) = first_diff_excerpt(&stitched, &body);
                roundtrip_failures.push((
                    path.clone(),
                    format!(
                        "stitch mismatch:\n--- expected ---\n{b_excerpt}\n--- got ---\n{a_excerpt}"
                    ),
                ));
            }
        }
    }

    println!(
        "real-doc sweep: {} document(s) across {} root(s)",
        total,
        roots.len()
    );
    for (r, n) in &per_root {
        println!("  {} → {} doc(s)", r.display(), n);
    }

    assert!(
        total >= 10,
        "expected to find ≥10 real spec docs (got {total})"
    );
    assert!(
        slug_failures.is_empty(),
        "slug-id shape failures:\n{}",
        format_failures(&slug_failures)
    );
    assert!(
        fm_failures.is_empty(),
        "frontmatter shape failures:\n{}",
        format_failures(&fm_failures)
    );
    assert!(
        roundtrip_failures.is_empty(),
        "round-trip failures:\n{}",
        format_failures(&roundtrip_failures)
    );
}

fn verify_slug_ids(
    sections: &[quire_rs::QuireSection],
    path: &Path,
    failures: &mut Vec<(PathBuf, String)>,
) {
    for s in sections {
        // Format: "<slug>-L<line>" where slug ∈ [a-z0-9-]* and line is digits.
        // slug may be empty (degenerate heading like "## !!!") — that's
        // FR-009-AC-7 ("-L<n>").
        let Some(idx) = s.id.rfind("-L") else {
            failures.push((path.to_path_buf(), format!("missing -L suffix: {}", s.id)));
            continue;
        };
        let line_part = &s.id[idx + 2..];
        if line_part.is_empty() || !line_part.chars().all(|c| c.is_ascii_digit()) {
            failures.push((
                path.to_path_buf(),
                format!("non-numeric line suffix in id: {}", s.id),
            ));
            continue;
        }
        let parsed: usize = line_part.parse().expect("digits");
        if parsed != s.start_line {
            failures.push((
                path.to_path_buf(),
                format!(
                    "id line suffix ({parsed}) != start_line ({}): {}",
                    s.start_line, s.id
                ),
            ));
        }
        let slug = &s.id[..idx];
        for c in slug.chars() {
            if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                failures.push((
                    path.to_path_buf(),
                    format!("non-[a-z0-9-] in slug: {} (offending: {c:?})", s.id),
                ));
                break;
            }
        }
        verify_slug_ids(&s.children, path, failures);
    }
}

fn first_diff_excerpt(a: &str, b: &str) -> (String, String) {
    let common = a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count();
    let start = common.saturating_sub(40);
    let end_a = (common + 80).min(a.len());
    let end_b = (common + 80).min(b.len());
    (
        format!("…{}…", &a[start..end_a].replace('\n', "⏎")),
        format!("…{}…", &b[start..end_b].replace('\n', "⏎")),
    )
}

fn format_failures(v: &[(PathBuf, String)]) -> String {
    v.iter()
        .take(10)
        .map(|(p, m)| format!("  {}: {m}", p.display()))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn parse_real_document_with_frontmatter_extracts_typed_fields() {
    // Pin one well-known artifact and assert specific fields the
    // ISO spec promises every artifact must carry.
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("spec")
        .join("functional")
        .join("FR-001-render-dispatch.md");
    let s = std::fs::read_to_string(&path).expect("FR-001 exists");
    let doc = parse_document(&s);
    let fm = doc.frontmatter.as_ref().expect("FR-001 has frontmatter");
    assert_eq!(
        fm.get("id").and_then(|v| v.as_str()),
        Some("FR-001"),
        "frontmatter.id"
    );
    assert_eq!(
        fm.get("type").and_then(|v| v.as_str()),
        Some("FR"),
        "frontmatter.type"
    );
    assert!(
        !doc.sections.is_empty(),
        "FR-001 is expected to have ## sections"
    );
    // Section IDs are stable: heading "Behavior" maps to `behavior-L<line>`.
    let has_behavior = doc
        .sections
        .iter()
        .any(|s| s.id.starts_with("behavior-L") && s.heading == "Behavior");
    assert!(has_behavior, "FR-001 should expose a 'Behavior' section");
}
