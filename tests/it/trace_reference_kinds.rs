//! FR-049-AC-10..18 and FR-057-AC-11 — kind-typed document references
//! (TC-2240..TC-2247).

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::grammar::{GrammarSeverityLevel, GrammarSeverityMap};
use quire_rs::{validate_bundle_at, BundleFinding, BundlePosture, BundleReport, Registry};

fn fixture_module(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("traceability")
        .join(name)
}

fn tmpdir(suffix: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "quire-rs-trace-kinds-{}-{suffix}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("mkdir");
    p
}

fn write(root: &Path, rel: &str, body: &str) {
    fs::write(root.join(rel), body).expect("write");
}

/// A bundle with targets L-1 (Left) and R-1 (Right) and a link document
/// carrying `rows` of (kind, target) under a `Class` / `Target` table.
fn bundle(suffix: &str, rows: &[(&str, &str)]) -> PathBuf {
    let root = tmpdir(suffix);
    write(
        &root,
        "left.md",
        "---\nid: LD-001\ntype: Left\ntitle: Left\n---\n\n## Items\n\n| Item |\n|------|\n| L-1 |\n",
    );
    write(
        &root,
        "right.md",
        "---\nid: RD-001\ntype: Right\ntitle: Right\n---\n\n## Items\n\n| Item |\n|------|\n| R-1 |\n",
    );
    let mut links = String::from(
        "---\nid: LK-001\ntype: Link\ntitle: Links\n---\n\n## Links\n\n| Link | Class | Target |\n|------|-------|--------|\n",
    );
    for (n, (kind, target)) in rows.iter().enumerate() {
        links.push_str(&format!("| K-{n} | {kind} | {target} |\n"));
    }
    write(&root, "links.md", &links);
    root
}

fn registry() -> Registry {
    Registry::load_module(&fixture_module("kinds")).expect("load module")
}

fn all(report: &BundleReport) -> Vec<&BundleFinding> {
    report.errors.iter().chain(report.warnings.iter()).collect()
}

fn with_reason<'a>(report: &'a BundleReport, reason: &str) -> Vec<&'a BundleFinding> {
    all(report)
        .into_iter()
        .filter(|f| f.reason == reason)
        .collect()
}

fn count(report: &BundleReport, reason: &str) -> usize {
    with_reason(report, reason).len()
}

#[trace("TC-2240", "FR-049-AC-10", "FR-049-AC-11")]
#[test]
fn tc2240_kind_subset_decides_mismatch_and_dangling_stays_separate() {
    let root = bundle(
        "2240",
        &[("north", "L-1"), ("north", "R-1"), ("south", "L-9")],
    );
    let report = validate_bundle_at(&root, &registry(), BundlePosture::Strict);
    let mismatch = with_reason(&report, "reference-kind-mismatch");
    assert_eq!(mismatch.len(), 1, "{:?}", all(&report));
    let message = &mismatch[0].message;
    assert!(mismatch[0].path.ends_with("links.md"));
    for part in ["R-1", "(row 'K-1')", "north", "lefty", "links"] {
        assert!(message.contains(part), "{part} missing from {message}");
    }
    let dangling = with_reason(&report, "dangling-trace-reference");
    assert_eq!(dangling.len(), 1, "{:?}", all(&report));
    assert!(dangling[0].message.contains("L-9"));
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-2241", "FR-049-AC-12")]
#[test]
fn tc2241_mismatch_is_error_under_strict_warning_under_okf() {
    let root = bundle("2241", &[("north", "R-1")]);
    let strict = validate_bundle_at(&root, &registry(), BundlePosture::Strict);
    assert!(strict
        .errors
        .iter()
        .any(|f| f.reason == "reference-kind-mismatch"));
    let okf = validate_bundle_at(&root, &registry(), BundlePosture::Okf);
    assert!(okf
        .warnings
        .iter()
        .any(|f| f.reason == "reference-kind-mismatch"));
    assert!(!okf
        .errors
        .iter()
        .any(|f| f.reason == "reference-kind-mismatch"));
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-2242", "FR-049-AC-13")]
#[test]
fn tc2242_unknown_empty_or_missing_kind_checks_the_union_only() {
    let root = bundle(
        "2242",
        &[
            ("", "R-1"),
            ("east", "L-1"),
            ("  NoRtH ", "L-1"),
            ("east", "R-7"),
        ],
    );
    let report = validate_bundle_at(&root, &registry(), BundlePosture::Strict);
    assert_eq!(
        count(&report, "reference-kind-mismatch"),
        0,
        "{:?}",
        all(&report)
    );
    let dangling = with_reason(&report, "dangling-trace-reference");
    assert_eq!(dangling.len(), 1, "{:?}", all(&report));
    assert!(dangling[0].message.contains("R-7"));
    fs::remove_dir_all(&root).ok();

    // A table lacking the kind column: union only.
    let root = bundle("2242b", &[]);
    write(
        &root,
        "links.md",
        "---\nid: LK-001\ntype: Link\ntitle: Links\n---\n\n## Links\n\n| Link | Target |\n|------|--------|\n| K-0 | R-1 |\n| K-1 | R-8 |\n",
    );
    let report = validate_bundle_at(&root, &registry(), BundlePosture::Strict);
    assert_eq!(count(&report, "reference-kind-mismatch"), 0);
    assert_eq!(count(&report, "dangling-trace-reference"), 1);
    fs::remove_dir_all(&root).ok();

    // Case fold: matching kinds in other case give a real mismatch.
    let root = bundle("2242c", &[(" NORTH", "R-1")]);
    let report = validate_bundle_at(&root, &registry(), BundlePosture::Strict);
    assert_eq!(count(&report, "reference-kind-mismatch"), 1);
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-2243", "FR-049-AC-14")]
#[test]
fn tc2243_empty_targets_yield_nothing_and_multi_id_cells_split() {
    let root = bundle(
        "2243",
        &[
            ("north", "-"),
            ("south", "-"),
            ("east", "-"),
            ("north", "L-1, R-1"),
        ],
    );
    write(
        &root,
        "links.md",
        &fs::read_to_string(root.join("links.md"))
            .expect("read")
            .replace("| K-1 | south | - |", "| K-1 | south |  |"),
    );
    let report = validate_bundle_at(&root, &registry(), BundlePosture::Strict);
    let mismatch = with_reason(&report, "reference-kind-mismatch");
    assert_eq!(mismatch.len(), 1, "{:?}", all(&report));
    assert!(mismatch[0].message.contains("R-1"));
    assert!(!mismatch[0].message.contains("L-1"));
    assert_eq!(count(&report, "dangling-trace-reference"), 0);
    fs::remove_dir_all(&root).ok();
}

fn manifest_with(kind_lines: &str) -> String {
    let base = fs::read_to_string(fixture_module("kinds").join("manifest.yaml")).expect("read");
    let start = base.find("    kind_column:").expect("kind block");
    format!("{}{kind_lines}", &base[..start])
}

#[trace("TC-2244", "FR-049-AC-15")]
#[test]
fn tc2244_malformed_kind_declarations_fail_module_load() {
    let cases: &[(&str, &str)] = &[
        ("    kind_column: Class\n", "without `kind_targets`"),
        ("    kind_targets:\n      north: [lefty]\n", "without `kind_column`"),
        (
            "    kind_column: ' '\n    kind_targets:\n      north: [lefty]\n",
            "empty `kind_column`",
        ),
        (
            "    kind_column: target\n    kind_targets:\n      north: [lefty]\n",
            "`kind_column` equal to its `column`",
        ),
        ("    kind_column: Class\n    kind_targets: {}\n", "empty `kind_targets`"),
        (
            "    kind_column: Class\n    kind_targets:\n      ' ': [lefty]\n",
            "empty kind key",
        ),
        (
            "    kind_column: Class\n    kind_targets:\n      north: [lefty]\n      North: [righty]\n",
            "duplicate kind key",
        ),
        (
            "    kind_column: Class\n    kind_targets:\n      north: [lefty]\n      north: [righty]\n",
            "duplicate kind key 'north'",
        ),
        (
            "    kind_column: Class\n    kind_targets:\n      north: []\n",
            "lists no targets for kind 'north'",
        ),
        (
            "    kind_column: Class\n    kind_targets:\n      north: [elsewhere]\n",
            "'elsewhere'",
        ),
    ];
    for (lines, expected) in cases {
        let root = tmpdir("2244");
        write(&root, "manifest.yaml", &manifest_with(lines));
        let error = Registry::load_module(&root)
            .expect_err(expected)
            .to_string();
        assert!(error.contains(expected), "{expected}: {error}");
        assert!(error.contains("'links'"), "names the declaration: {error}");
        fs::remove_dir_all(&root).ok();
    }
}

#[trace("TC-2245", "FR-049-AC-16", "FR-049-AC-17")]
#[test]
fn tc2245_untyped_declarations_unchanged_and_findings_deterministic() {
    // Without the kind fields the same cross-kind id is clean.
    let root = bundle("2245", &[("north", "R-1")]);
    let untyped = tmpdir("2245m");
    write(&untyped, "manifest.yaml", &manifest_with(""));
    let plain = Registry::load_module(&untyped).expect("load");
    let report = validate_bundle_at(&root, &plain, BundlePosture::Strict);
    assert_eq!(count(&report, "reference-kind-mismatch"), 0);
    assert_eq!(count(&report, "dangling-trace-reference"), 0);
    fs::remove_dir_all(&untyped).ok();
    fs::remove_dir_all(&root).ok();

    // Repeated validation: identical, ordered findings.
    let root = bundle(
        "2245b",
        &[
            ("south", "L-1"),
            ("north", "R-1"),
            ("north", "L-9"),
            ("south", "R-9"),
        ],
    );
    let registry = registry();
    let first = validate_bundle_at(&root, &registry, BundlePosture::Strict);
    let second = validate_bundle_at(&root, &registry, BundlePosture::Strict);
    let view = |r: &BundleReport| -> Vec<(String, String)> {
        r.errors
            .iter()
            .map(|f| (f.reason.to_string(), f.message.clone()))
            .collect()
    };
    assert_eq!(view(&first), view(&second));
    let ids: Vec<String> = view(&first)
        .into_iter()
        .map(|(_, message)| message.split('\'').nth(1).unwrap_or_default().to_string())
        .collect();
    assert_eq!(ids, ["L-1", "L-9", "R-1", "R-9"], "ordered by id");
    assert_eq!(view(&first).len(), 4);
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-2246", "FR-057-AC-11")]
#[test]
fn tc2246_kind_mismatch_is_registrable_beside_dangling() {
    let root = bundle("2246", &[("north", "R-1"), ("north", "L-9")]);
    for (posture, unconfigured) in [(BundlePosture::Strict, true), (BundlePosture::Okf, false)] {
        let base = validate_bundle_at(&root, &registry(), posture);
        let tier_errors = |r: &BundleReport| {
            r.errors
                .iter()
                .filter(|f| f.reason == "reference-kind-mismatch")
                .count()
        };
        assert_eq!(tier_errors(&base) == 1, unconfigured);
        assert_eq!(count(&base, "reference-kind-mismatch"), 1);

        for (level, errors, warnings) in [
            (GrammarSeverityLevel::Error, 1, 0),
            (GrammarSeverityLevel::Warning, 0, 1),
            (GrammarSeverityLevel::Off, 0, 0),
        ] {
            let mut map = GrammarSeverityMap::new();
            map.insert("trace:reference-kind-mismatch".to_string(), level);
            let report = validate_bundle_at(&root, &registry().with_grammar_severity(map), posture);
            let e = report
                .errors
                .iter()
                .filter(|f| f.reason == "reference-kind-mismatch")
                .count();
            let w = report
                .warnings
                .iter()
                .filter(|f| f.reason == "reference-kind-mismatch")
                .count();
            assert_eq!((e, w), (errors, warnings), "{posture:?} {level:?}");
            assert_eq!(
                count(&report, "dangling-trace-reference"),
                count(&base, "dangling-trace-reference"),
                "sibling count unchanged"
            );
        }
    }
    fs::remove_dir_all(&root).ok();
}

#[trace("TC-2247", "FR-049-AC-18")]
#[test]
fn tc2247_missing_kind_header_warns_once_under_both_postures() {
    let root = bundle("2247", &[]);
    write(
        &root,
        "links.md",
        "---\nid: LK-001\ntype: Link\ntitle: Links\n---\n\n## Links\n\n| Link | Target |\n|------|--------|\n| K-0 | R-1 |\n| K-1 | L-1 |\n",
    );
    for posture in [BundlePosture::Strict, BundlePosture::Okf] {
        let report = validate_bundle_at(&root, &registry(), posture);
        let found: Vec<_> = report
            .warnings
            .iter()
            .filter(|f| f.reason == "kind-column-matches-nothing")
            .collect();
        assert_eq!(found.len(), 1, "{posture:?}: {:?}", all(&report));
        assert!(found[0].path.ends_with("links.md"));
        assert!(found[0].message.contains("links"));
        assert!(!report
            .errors
            .iter()
            .any(|f| f.reason == "kind-column-matches-nothing"));
    }
    let mut map = GrammarSeverityMap::new();
    map.insert(
        "trace:kind-column-matches-nothing".to_string(),
        GrammarSeverityLevel::Off,
    );
    let off = validate_bundle_at(
        &root,
        &registry().with_grammar_severity(map),
        BundlePosture::Strict,
    );
    assert_eq!(count(&off, "kind-column-matches-nothing"), 0);
    fs::remove_dir_all(&root).ok();

    // Per table (FR-049-AC-18): a table with the headers and zero data rows
    // warns, and two such tables in one document warn twice.
    let root = bundle("2247z", &[]);
    write(
        &root,
        "links.md",
        "---\nid: LK-001\ntype: Link\ntitle: Links\n---\n\n## Links\n\n| Link | Target |\n|------|--------|\n",
    );
    let report = validate_bundle_at(&root, &registry(), BundlePosture::Strict);
    assert_eq!(count(&report, "kind-column-matches-nothing"), 1);
    fs::remove_dir_all(&root).ok();

    let root = bundle("2247t", &[]);
    write(
        &root,
        "links.md",
        "---\nid: LK-001\ntype: Link\ntitle: Links\n---\n\n## Links\n\n| Link | Target |\n|------|--------|\n| K-0 | R-1 |\n\n## Links\n\n| Link | Target |\n|------|--------|\n| K-1 | L-1 |\n",
    );
    let report = validate_bundle_at(&root, &registry(), BundlePosture::Strict);
    assert_eq!(
        count(&report, "kind-column-matches-nothing"),
        2,
        "{:?}",
        all(&report)
    );
    fs::remove_dir_all(&root).ok();

    // Both headers, and neither header, yield none.
    let root = bundle("2247b", &[("north", "L-1")]);
    let report = validate_bundle_at(&root, &registry(), BundlePosture::Strict);
    assert_eq!(count(&report, "kind-column-matches-nothing"), 0);
    write(
        &root,
        "links.md",
        "---\nid: LK-001\ntype: Link\ntitle: Links\n---\n\n## Links\n\n| Link | Note |\n|------|------|\n| K-0 | x |\n",
    );
    let report = validate_bundle_at(&root, &registry(), BundlePosture::Strict);
    assert_eq!(count(&report, "kind-column-matches-nothing"), 0);
    fs::remove_dir_all(&root).ok();
}
