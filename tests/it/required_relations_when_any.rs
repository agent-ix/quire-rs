//! FR-058-AC-23..28 — the `when_any` edge-condition selector on a required
//! relation. Kinds, verbs and directions all come from manifests written here
//! (FR-058-CON-4).

use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;
use quire_rs::grammar::{GrammarSeverityLevel, GrammarSeverityMap};
use quire_rs::{validate_bundle_at, BundlePosture, BundleReport, Registry};

const KINDS: [&str; 4] = ["Ctl", "Haz", "Est", "Req"];

const WHEN_ANY: &str = "    when_any:\n      - { verb: protects, direction: outgoing, other: [Haz] }\n      - { verb: assumes, direction: incoming, other: [Est] }\n";

fn tmpdir(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "quire_reqrel_when_any_{tag}_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("mkdir");
    p
}

fn write(root: &Path, rel: &str, body: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(path, body).expect("write");
}

/// A module with one relation: `Ctl` documents need an incoming `satisfies`
/// edge from a `Req`, narrowed by `extra` (the `when_any`/`where` lines).
fn module_with(tag: &str, extra: &str) -> PathBuf {
    let dir = tmpdir(&format!("mod_{tag}"));
    let mut archetypes = String::new();
    for kind in KINDS {
        write(
            &dir,
            &format!("schemas/{kind}.schema.json"),
            r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object"}"#,
        );
        archetypes.push_str(&format!(
            "- kind: {kind}\n  name: {kind}\n  frontmatter_schema_ref: schemas/{kind}.schema.json\n"
        ));
    }
    write(
        &dir,
        "manifest.yaml",
        &format!(
            "name: when-any-{tag}\nversion: 0.1.0\ndescription: d\narchetypes:\n{archetypes}traceability:\n  required_relations:\n  - name: ctl-has-req\n    from: Ctl\n{extra}    edges: [satisfies]\n    to: [Req]\n    direction: incoming\n    check: ctl-without-req\n"
        ),
    );
    dir
}

fn doc(id: &str, kind: &str, extra_fm: &str, edges: &[(&str, &str)]) -> String {
    let relationships = if edges.is_empty() {
        String::new()
    } else {
        let rows: String = edges
            .iter()
            .map(|(verb, target)| format!("  - target: \"{target}\"\n    type: \"{verb}\"\n"))
            .collect();
        format!("relationships:\n{rows}")
    };
    format!("---\nid: {id}\ntype: {kind}\ntitle: T\n{extra_fm}{relationships}---\n\n## Description\n\nText.\n")
}

fn put(root: &Path, id: &str, kind: &str, edges: &[(&str, &str)]) {
    write(root, &format!("{id}.md"), &doc(id, kind, "", edges));
}

fn reported(root: &Path, registry: &Registry) -> Vec<String> {
    let report = validate_bundle_at(root, registry, BundlePosture::Okf);
    ids(&report, "ctl-without-req")
}

fn ids(report: &BundleReport, reason: &str) -> Vec<String> {
    let mut v: Vec<String> = report
        .errors
        .iter()
        .chain(report.warnings.iter())
        .filter(|f| f.reason == reason)
        .map(|f| f.message.split('\'').nth(1).unwrap_or("").to_string())
        .collect();
    v.sort();
    v
}

#[trace("FR-058-AC-23")]
#[test]
fn tc_when_any_reports_only_in_scope_documents_lacking_the_edge() {
    let dir = module_with("23", WHEN_ANY);
    let registry = Registry::load_module(&dir).expect("load");
    let root = tmpdir("23");
    put(&root, "H-1", "Haz", &[]);
    put(&root, "E-1", "Est", &[("assumes", "C-2")]);
    put(&root, "R-1", "Req", &[("satisfies", "C-6")]);
    // First condition only, second only, both, and the satisfied one.
    put(&root, "C-1", "Ctl", &[("protects", "H-1")]);
    put(&root, "C-2", "Ctl", &[]);
    put(&root, "C-3", "Ctl", &[("protects", "H-1")]);
    put(&root, "E-3", "Est", &[("assumes", "C-3")]);
    put(&root, "C-6", "Ctl", &[("protects", "H-1")]);
    assert_eq!(reported(&root, &registry), ["C-1", "C-2", "C-3"]);
    fs::remove_dir_all(&root).ok();
    fs::remove_dir_all(&dir).ok();
}

#[trace("FR-058-AC-24")]
#[test]
fn tc_a_document_in_scope_for_no_condition_is_never_reported() {
    let dir = module_with("24", WHEN_ANY);
    let registry = Registry::load_module(&dir).expect("load");
    let root = tmpdir("24");
    put(&root, "H-1", "Haz", &[]);
    put(&root, "R-1", "Req", &[("satisfies", "C-5")]);
    put(&root, "C-1", "Ctl", &[]); // no edge at all
    put(&root, "C-2", "Ctl", &[("assumes", "H-1")]); // wrong verb direction: outgoing assumes
    put(&root, "C-3", "Ctl", &[("protects", "R-1")]); // other end outside `other`
    put(&root, "C-4", "Ctl", &[("protects", "H-404")]); // dangling
    put(&root, "C-5", "Ctl", &[]); // satisfied, out of scope
    put(&root, "E-1", "Est", &[("protects", "C-1")]); // incoming protects, not outgoing
    assert_eq!(reported(&root, &registry), Vec::<String>::new());
    fs::remove_dir_all(&root).ok();
    fs::remove_dir_all(&dir).ok();
}

#[trace("FR-058-AC-25")]
#[test]
fn tc_when_any_composes_with_where_and_exclude_by_intersection() {
    let dir = module_with(
        "25",
        &format!(
            "{WHEN_ANY}    where: {{ field: class, values: [safety] }}\n    exclude: [\"legacy/**\"]\n"
        ),
    );
    let registry = Registry::load_module(&dir).expect("load");
    let root = tmpdir("25");
    put(&root, "H-1", "Haz", &[]);
    let with_class = |id: &str, class: &str| {
        doc(
            id,
            "Ctl",
            &format!("class: {class}\n"),
            &[("protects", "H-1")],
        )
    };
    write(&root, "C-1.md", &with_class("C-1", "safety"));
    write(&root, "C-2.md", &with_class("C-2", "other"));
    write(&root, "legacy/C-3.md", &with_class("C-3", "safety"));
    assert_eq!(reported(&root, &registry), ["C-1"]);
    fs::remove_dir_all(&root).ok();
    fs::remove_dir_all(&dir).ok();
}

#[trace("FR-058-AC-26")]
#[test]
fn tc_a_malformed_when_any_fails_module_load_naming_relation_and_when_any() {
    let bad = [
        "    when_any: protects\n",
        "    when_any: null\n",
        "    when_any: []\n",
        "    when_any: [protects]\n",
        "    when_any: [{ direction: outgoing, other: [Haz] }]\n",
        "    when_any: [{ verb: \" \", direction: outgoing, other: [Haz] }]\n",
        "    when_any: [{ verb: protects, other: [Haz] }]\n",
        "    when_any: [{ verb: protects, direction: either, other: [Haz] }]\n",
        "    when_any: [{ verb: protects, direction: 3, other: [Haz] }]\n",
        "    when_any: [{ verb: protects, direction: outgoing }]\n",
        "    when_any: [{ verb: protects, direction: outgoing, other: [] }]\n",
        "    when_any: [{ verb: protects, direction: outgoing, other: Haz }]\n",
        "    when_any: [{ verb: protects, direction: outgoing, other: [Haz, \" \"] }]\n",
        "    when_any: [{ verb: protects, direction: outgoing, other: [Haz], extra: 1 }]\n",
    ];
    for (n, yaml) in bad.iter().enumerate() {
        let dir = module_with(&format!("26_{n}"), yaml);
        let err = Registry::load_module(&dir)
            .err()
            .unwrap_or_else(|| panic!("{yaml:?} must be rejected"))
            .to_string();
        assert!(
            err.contains("'ctl-has-req'") && err.contains("`when_any`"),
            "{yaml:?} -> {err}"
        );
        fs::remove_dir_all(&dir).ok();
    }
}

#[trace("FR-058-AC-27")]
#[test]
fn tc_without_when_any_every_from_document_is_obliged_and_the_key_tunes_severity() {
    let root = tmpdir("27");
    put(&root, "H-1", "Haz", &[]);
    put(&root, "C-1", "Ctl", &[("protects", "H-1")]);
    put(&root, "C-2", "Ctl", &[]);

    let plain = module_with("27a", "");
    let registry = Registry::load_module(&plain).expect("load");
    assert_eq!(reported(&root, &registry), ["C-1", "C-2"]);

    let scoped = module_with("27b", WHEN_ANY);
    let registry = Registry::load_module(&scoped).expect("load");
    let first = reported(&root, &registry);
    assert_eq!(first, ["C-1"]);
    assert_eq!(first, reported(&root, &registry));

    let mut map = GrammarSeverityMap::new();
    map.insert("trace:ctl-without-req".into(), GrammarSeverityLevel::Error);
    let registry = registry.with_grammar_severity(map);
    let report = validate_bundle_at(&root, &registry, BundlePosture::Okf);
    assert!(report.errors.iter().any(|f| f.reason == "ctl-without-req"));
    fs::remove_dir_all(&root).ok();
    fs::remove_dir_all(&plain).ok();
    fs::remove_dir_all(&scoped).ok();
}

#[trace("FR-058-AC-28")]
#[test]
fn tc_a_when_any_naming_a_dead_other_kind_reports_itself() {
    let dir = module_with(
        "28",
        "    when_any:\n      - { verb: protects, direction: outgoing, other: [Hazz] }\n",
    );
    let registry = Registry::load_module(&dir).expect("load");
    let root = tmpdir("28");
    put(&root, "C-1", "Ctl", &[]);
    let report = validate_bundle_at(&root, &registry, BundlePosture::Okf);
    let dead: Vec<_> = report
        .errors
        .iter()
        .chain(report.warnings.iter())
        .filter(|f| f.reason == "undeclared-relation-vocabulary")
        .collect();
    assert_eq!(dead.len(), 1, "{dead:?}");
    assert!(dead[0].message.contains("'ctl-has-req'") && dead[0].message.contains("'Hazz'"));
    fs::remove_dir_all(&root).ok();
    fs::remove_dir_all(&dir).ok();
}
