//! Bundle validation postures (OKF) + index-completeness.
//!
//! Strict = our archetype-conformant posture; Okf = permissive
//! foreign-bundle reading. `type` is required in BOTH (the
//! "untyped corpus doc is an error, not a warning" fix); unknown types,
//! broken `ix://` links, and index gaps degrade to warnings only under Okf.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use quire_rs::{validate_bundle_at, BundlePosture, Registry};

fn bundle_registry() -> Registry {
    let module = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules/bundle");
    Registry::load_module(&module).expect("load bundle test module")
}

fn tmpdir(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "quire_bundle_{tag}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&p).unwrap();
    p
}

fn write(dir: &Path, rel: &str, body: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

fn note(id: &str, body: &str) -> String {
    format!("---\nid: {id}\ntype: note\n---\n# {id}\n{body}\n")
}

/// Strict: a document with no `type` is a hard error (was a non-fatal
/// `UntypedArtifact` warning before this change).
// TC-600, FR-038-AC-1: under Strict an untyped document is a hard error.
#[test]
fn strict_untyped_doc_is_error() {
    let root = tmpdir("strict_untyped");
    write(
        &root,
        "NOTE-001.md",
        "---\nid: NOTE-001\n---\n# note\nbody\n",
    );
    let report = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Strict);
    assert!(!report.is_valid());
    assert!(report
        .errors
        .iter()
        .any(|f| f.reason == "frontmatter" && f.message.contains("type")));
}

/// Okf: `type` is still required — an untyped doc is an error even under
/// the permissive posture.
// TC-601, FR-038-AC-2: and the permissive posture does not soften it.
#[test]
fn okf_untyped_doc_is_still_error() {
    let root = tmpdir("okf_untyped");
    write(
        &root,
        "NOTE-001.md",
        "---\nid: NOTE-001\n---\n# note\nbody\n",
    );
    let report = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Okf);
    assert!(!report.is_valid());
    assert!(report.errors.iter().any(|f| f.reason == "frontmatter"));
}

/// Okf tolerates an unknown type and a dangling `ix://` reference as
/// warnings; Strict rejects the unknown type.
// TC-602, FR-038-AC-3: what Okf does soften — unknown type, dangling ref.
#[test]
fn okf_tolerates_unknown_type_and_broken_links() {
    let root = tmpdir("okf_tolerant");
    write(
        &root,
        "X-1.md",
        "---\nid: X-1\ntype: weird\n---\n# x\nsee [missing](ix://o/r/MISSING)\n",
    );

    let okf = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Okf);
    assert!(okf.is_valid(), "okf errors: {:?}", okf.errors);
    assert!(okf.warnings.iter().any(|f| f.reason == "unknown-type"));
    assert!(okf
        .warnings
        .iter()
        .any(|f| f.reason == "dangling-reference"));

    let strict = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Strict);
    assert!(!strict.is_valid());
    assert!(strict.errors.iter().any(|f| f.reason == "unknown-type"));
    assert!(strict
        .errors
        .iter()
        .any(|f| f.reason == "dangling-reference"));
}

/// A typed, archetype-conformant bundle whose index lists every sibling +
/// carries `okf_version` is valid under Strict.
// TC-603, FR-038-AC-4: the conformant bundle passes Strict end to end.
#[test]
fn strict_conformant_bundle_with_complete_index_is_valid() {
    let root = tmpdir("strict_ok");
    write(&root, "NOTE-001.md", &note("NOTE-001", "body"));
    write(
        &root,
        "index.md",
        "---\ntype: index\ntitle: Root\nokf_version: \"0.1\"\n---\n# Root\n\n## Contents\n\n* [NOTE-001](./NOTE-001.md)\n",
    );
    write(
        &root,
        "log.md",
        "---\ntype: log\n---\n# Log\n\n## History\n\n* 2026-06-16 created\n",
    );

    let report = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Strict);
    assert!(report.is_valid(), "errors: {:?}", report.errors);
}

/// An index.md missing a sibling artifact is an index-completeness error
/// under Strict, a warning under Okf.
// TC-604, FR-038-AC-5: a missing sibling is an error, then a warning.
#[test]
fn index_incompleteness_is_error_strict_warning_okf() {
    let root = tmpdir("index_incomplete");
    write(&root, "NOTE-001.md", &note("NOTE-001", "body"));
    write(&root, "NOTE-002.md", &note("NOTE-002", "body"));
    // index lists only NOTE-001.
    write(
        &root,
        "index.md",
        "---\ntype: index\nokf_version: \"0.1\"\n---\n# Root\n\n## Contents\n\n* [NOTE-001](./NOTE-001.md)\n",
    );

    let strict = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Strict);
    assert!(strict
        .errors
        .iter()
        .any(|f| f.reason == "index-incomplete" && f.message.contains("NOTE-002")));

    let okf = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Okf);
    assert!(okf
        .warnings
        .iter()
        .any(|f| f.reason == "index-incomplete" && f.message.contains("NOTE-002")));
}

/// The bundle-root index.md must declare `okf_version`.
// TC-605, FR-038-AC-6: the bundle root must declare `okf_version`.
#[test]
fn root_index_missing_okf_version_is_flagged() {
    let root = tmpdir("no_okf_version");
    write(&root, "NOTE-001.md", &note("NOTE-001", "body"));
    write(
        &root,
        "index.md",
        "---\ntype: index\n---\n# Root\n\n## Contents\n\n* [NOTE-001](./NOTE-001.md)\n",
    );

    let report = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Strict);
    assert!(report
        .errors
        .iter()
        .any(|f| f.reason == "index-okf-version"));
}

/// A subdirectory index need not carry `okf_version` (only the root does).
// TC-606, FR-038-AC-6: a subdirectory index does not.
#[test]
fn subdir_index_does_not_require_okf_version() {
    let root = tmpdir("subdir_index");
    write(&root, "NOTE-000.md", &note("NOTE-000", "body"));
    write(
        &root,
        "index.md",
        "---\ntype: index\nokf_version: \"0.1\"\n---\n# Root\n\n## Contents\n\n* [NOTE-000](./NOTE-000.md)\n* [sub](./sub/index.md)\n",
    );
    write(&root, "sub/NOTE-001.md", &note("NOTE-001", "body"));
    write(
        &root,
        "sub/index.md",
        "---\ntype: index\n---\n# Sub\n\n## Contents\n\n* [NOTE-001](./NOTE-001.md)\n",
    );

    let report = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Strict);
    assert!(report.is_valid(), "errors: {:?}", report.errors);
}

/// A mistyped optional `description` is caught by the base concept contract
/// under Strict.
// TC-607, FR-038-AC-7: the base-concept contract still applies to a known type.
#[test]
fn strict_rejects_mistyped_description() {
    let root = tmpdir("mistyped_desc");
    write(
        &root,
        "NOTE-001.md",
        "---\nid: NOTE-001\ntype: note\ndescription: 7\n---\n# note\nbody\n",
    );
    let report = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Strict);
    assert!(!report.is_valid());
    assert!(report
        .errors
        .iter()
        .any(|f| f.message.contains("description")));
}

/// Build an inline registry with a `concept` artifact type plus object
/// types carrying roles + allowed_links, for the Tier-2 edge-target test
/// (FR-040-AC-9).
fn edge_target_registry() -> Registry {
    let manifest = br#"
name: edge-target-test
artifact_types:
- name: concept
  frontmatter_schema_ref: schemas/concept.schema.json
object_types:
- name: api_endpoint
  allowed_links:
    exposes: [domain-object]
- name: entity
  roles: [domain-object]
- name: data_schema
edge_types:
  exposes: { description: surface, category: realization }
roles:
  domain-object: { description: a business-model type }
"#;
    let mut schemas = std::collections::BTreeMap::new();
    schemas.insert(
        "schemas/concept.schema.json".to_string(),
        r#"{"type":"object","required":["id"],"properties":{"id":{"type":"string"}}}"#.to_string(),
    );
    Registry::from_inline_parts(manifest, &schemas).expect("inline registry")
}

/// TC-642, FR-040-AC-9: a resolved edge whose target object type satisfies
/// no token in the verb's target list yields a `disallowed-edge-target`
/// warning; the same verb to a target carrying the required role passes —
/// across object-type boundaries. Warn-tier: the bundle stays valid.
#[test]
fn tc642_disallowed_edge_target_is_warning() {
    let root = tmpdir("edge_target");
    // Source api_endpoint `exposes` two targets: an entity (role
    // domain-object → OK) and a data_schema (no role → flagged).
    write(
        &root,
        "API-1.md",
        "---\nid: API-1\ntype: concept\nobject: api_endpoint\nrelationships:\n- target: ix://o/r/ENT-1\n  type: exposes\n- target: ix://o/r/DS-1\n  type: exposes\n---\n# API-1\nbody\n",
    );
    write(
        &root,
        "ENT-1.md",
        "---\nid: ENT-1\ntype: concept\nobject: entity\n---\n# ENT-1\nbody\n",
    );
    write(
        &root,
        "DS-1.md",
        "---\nid: DS-1\ntype: concept\nobject: data_schema\n---\n# DS-1\nbody\n",
    );

    let report = validate_bundle_at(&root, &edge_target_registry(), BundlePosture::Okf);
    let target_warnings: Vec<_> = report
        .warnings
        .iter()
        .filter(|f| f.reason == "disallowed-edge-target")
        .collect();
    assert_eq!(
        target_warnings.len(),
        1,
        "exactly one disallowed target, got {:?}",
        report.warnings
    );
    assert!(
        target_warnings[0].message.contains("DS-1")
            && target_warnings[0].message.contains("data_schema"),
        "names the offending target + object type: {}",
        target_warnings[0].message
    );
    // Warn-tier: disallowed edge targets never invalidate the bundle.
    assert!(report.is_valid(), "errors: {:?}", report.errors);
}

/// TC-820, FR-024-AC-12: the CR-048 walk→bundle bridge. A frontmatter-less
/// file and a malformed-frontmatter file under the document root each become
/// exactly one `BundleReport` **warning** naming the path — in both postures,
/// never an error — and the two flavors carry distinct machine reasons so a
/// consumer can triage "not meant to be a document" apart from "someone wrote
/// a front block and it does not parse" (CR-051).
#[test]
fn tc820_frontmatter_less_files_bridge_into_bundle_warnings() {
    for posture in [BundlePosture::Strict, BundlePosture::Okf] {
        let root = tmpdir("fm_bridge");
        write(&root, "NOTE-001.md", &note("NOTE-001", "real document"));
        write(&root, "draft.md", "# draft\n\nno front block at all.\n");
        // A complete fence block that is valid YAML but not a mapping.
        write(&root, "listy.md", "---\n- a\n- b\n---\n# listy\nbody\n");

        let report = validate_bundle_at(&root, &bundle_registry(), posture);

        let absent: Vec<_> = report
            .warnings
            .iter()
            .filter(|f| f.reason == "no-frontmatter")
            .collect();
        let malformed: Vec<_> = report
            .warnings
            .iter()
            .filter(|f| f.reason == "malformed-frontmatter")
            .collect();

        assert_eq!(
            absent.len(),
            1,
            "{posture:?}: exactly one no-frontmatter warning, got {:?}",
            report.warnings
        );
        assert_eq!(
            malformed.len(),
            1,
            "{posture:?}: the malformed flavor is distinguishable, got {:?}",
            report.warnings
        );
        assert!(
            absent[0].path.ends_with("draft.md"),
            "{posture:?}: names the path, got {:?}",
            absent[0].path
        );
        assert!(
            malformed[0].path.ends_with("listy.md"),
            "{posture:?}: names the path, got {:?}",
            malformed[0].path
        );
        assert!(
            malformed[0].message.contains("not a YAML mapping"),
            "{posture:?}: human message states the flavor too: {}",
            malformed[0].message
        );

        // Never an error, in either posture — the file is not a document, so
        // nothing structural can be wrong with it as one. The bundle stays
        // valid, which is what the CLI's exit code reads.
        assert!(
            !report
                .errors
                .iter()
                .any(|f| f.reason == "no-frontmatter" || f.reason == "malformed-frontmatter"),
            "{posture:?}: bridged as warnings only, errors: {:?}",
            report.errors
        );
        assert!(
            report.is_valid(),
            "{posture:?}: exit code unchanged, errors: {:?}",
            report.errors
        );

        fs::remove_dir_all(&root).ok();
    }
}

// ── FR-057: per-check corpus severity ──────────────────────────────────
//
// The knob P2's new corpus checks (agent-ix/quire-rs#85, #162) need in order to
// ship advisory and be tuned per repository. Exercised through
// `validate_bundle_at` against a bundle on disk with a `Registry` carrying the
// merged map — the same path `quire validate --severity` takes.

use quire_rs::corpus::validate::pack;
use quire_rs::grammar::{
    merge_severity_overrides, severity_key, GrammarSeverityLevel, GrammarSeverityMap,
};
use quire_rs::{BundleFinding, BundleReport, GrammarSeverity};

/// The bundle test module with a `<pack>:<check>` severity map installed, the
/// way `Registry::with_grammar_severity` installs one for a surface.
fn severity_registry(entries: &[(&str, GrammarSeverityLevel)]) -> Registry {
    let mut map = GrammarSeverityMap::new();
    for (key, level) in entries {
        map.insert((*key).to_string(), *level);
    }
    bundle_registry().with_grammar_severity(map)
}

fn count(findings: &[BundleFinding], reason: &str) -> usize {
    findings.iter().filter(|f| f.reason == reason).count()
}

fn all(report: &BundleReport) -> impl Iterator<Item = &BundleFinding> {
    report.errors.iter().chain(report.warnings.iter())
}

/// A bundle whose only fault is one dangling `ix://` reference.
fn one_dangling_ref(tag: &str) -> PathBuf {
    let root = tmpdir(tag);
    write(
        &root,
        "NOTE-001.md",
        &note("NOTE-001", "see [missing](ix://o/r/MISSING)"),
    );
    root
}

// TC-883 (FR-057-AC-1/AC-2/AC-3): a mapped key promotes, demotes, and
// suppresses a corpus check — the three states posture alone could not reach.
#[test]
fn tc883_registry_promotes_demotes_and_suppresses() {
    let root = one_dangling_ref("883");
    const KEY: &str = "refs:dangling-reference";

    // Unconfigured under Okf: one warning, bundle valid.
    let base = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Okf);
    assert!(base.is_valid(), "errors: {:?}", base.errors);
    assert_eq!(count(&base.warnings, "dangling-reference"), 1);

    // AC-1 — `error` promotes it out of Okf's warning tier.
    let promoted = validate_bundle_at(
        &root,
        &severity_registry(&[(KEY, GrammarSeverityLevel::Error)]),
        BundlePosture::Okf,
    );
    assert!(!promoted.is_valid(), "error must fail the bundle");
    assert_eq!(count(&promoted.errors, "dangling-reference"), 1);
    assert_eq!(count(&promoted.warnings, "dangling-reference"), 0);

    // AC-2 — `warning` demotes what Strict makes a hard error, and the bundle
    // becomes valid. The lever works in both directions.
    let strict_base = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Strict);
    assert!(!strict_base.is_valid(), "Strict errors by default");
    assert_eq!(count(&strict_base.errors, "dangling-reference"), 1);

    let demoted = validate_bundle_at(
        &root,
        &severity_registry(&[(KEY, GrammarSeverityLevel::Warning)]),
        BundlePosture::Strict,
    );
    assert_eq!(count(&demoted.errors, "dangling-reference"), 0);
    assert_eq!(count(&demoted.warnings, "dangling-reference"), 1);
    assert!(demoted.is_valid(), "errors: {:?}", demoted.errors);

    // AC-3 — `off` records nothing at all, in either posture.
    for posture in [BundlePosture::Strict, BundlePosture::Okf] {
        let off = validate_bundle_at(
            &root,
            &severity_registry(&[(KEY, GrammarSeverityLevel::Off)]),
            posture,
        );
        assert_eq!(count(&off.errors, "dangling-reference"), 0, "{posture:?}");
        assert_eq!(count(&off.warnings, "dangling-reference"), 0, "{posture:?}");
    }

    fs::remove_dir_all(&root).ok();
}

// TC-884, FR-057-AC-4: with no entry, every check keeps the exact tier it had
// before FR-057 — checked per check, not in aggregate. FR-048-AC-4's blanket
// `warning` default deliberately does NOT apply here: it would silently
// downgrade every corpus check that hard-errors under Strict today.
#[test]
fn tc884_unconfigured_checks_keep_their_prior_tier() {
    // Posture-routed: error under Strict, warning under Okf.
    let root = tmpdir("884_posture");
    write(
        &root,
        "index.md",
        "---\nid: IDX\ntype: index\n---\n# Index\n\n## Contents\n",
    );
    write(
        &root,
        "NOTE-001.md",
        &note("NOTE-001", "see [missing](ix://o/r/MISSING)"),
    );
    for (posture, in_errors) in [(BundlePosture::Strict, true), (BundlePosture::Okf, false)] {
        let r = validate_bundle_at(&root, &bundle_registry(), posture);
        let (hit, miss) = if in_errors {
            (&r.errors, &r.warnings)
        } else {
            (&r.warnings, &r.errors)
        };
        for reason in [
            "dangling-reference",
            "index-incomplete",
            "index-okf-version",
        ] {
            assert_eq!(count(hit, reason), 1, "{posture:?}: {reason} tier moved");
            assert_eq!(count(miss, reason), 0, "{posture:?}: {reason} tier moved");
        }
    }
    fs::remove_dir_all(&root).ok();

    // Fixed warning tier in both postures — never promoted by Strict.
    let root = tmpdir("884_fixed");
    write(&root, "NOTE-001.md", &note("NOTE-001", "fine"));
    write(&root, "loose.md", "no frontmatter at all\n");
    write(&root, "listy.md", "---\n- a\n- b\n---\n# listy\n");
    for posture in [BundlePosture::Strict, BundlePosture::Okf] {
        let r = validate_bundle_at(&root, &bundle_registry(), posture);
        for reason in ["no-frontmatter", "malformed-frontmatter"] {
            assert_eq!(count(&r.warnings, reason), 1, "{posture:?}: {reason}");
            assert_eq!(count(&r.errors, reason), 0, "{posture:?}: {reason}");
        }
    }
    fs::remove_dir_all(&root).ok();
}

// TC-885, FR-057-AC-5: overrides layered the way `quire validate --severity`
// layers them reach corpus checks, and a CLI entry beats a module entry for the
// same key. This is `apply_severity_overrides` in quire-cli, verbatim.
#[test]
fn tc885_cli_shaped_overrides_reach_corpus_checks() {
    let root = one_dangling_ref("885");
    const KEY: &str = "refs:dangling-reference";

    // Module declares `error`…
    let module = severity_registry(&[(KEY, GrammarSeverityLevel::Error)]);
    assert!(!validate_bundle_at(&root, &module, BundlePosture::Okf).is_valid());

    // …and the CLI turns it off for this run.
    let merged =
        merge_severity_overrides(module.grammar_severity(), [format!("{KEY}=off").as_str()])
            .expect("well-formed entry");
    let scoped = module.with_grammar_severity(merged);
    let r = validate_bundle_at(&root, &scoped, BundlePosture::Okf);
    assert_eq!(count(&r.errors, "dangling-reference"), 0);
    assert_eq!(count(&r.warnings, "dangling-reference"), 0);
    assert!(r.is_valid());

    // The module registry it was derived from is untouched.
    assert!(!validate_bundle_at(&root, &module, BundlePosture::Okf).is_valid());

    fs::remove_dir_all(&root).ok();
}

// TC-886 (FR-057-AC-7/AC-8/AC-9): the finding carries the level that was
// applied; the `reason` token consumers match on is unchanged; and every pack
// finding has a key `--severity` would accept — asserted over what the engine
// emits rather than a hardcoded list, so a new pack cannot ship unregistrable.
#[test]
fn tc886_findings_carry_severity_and_a_wellformed_key() {
    let root = tmpdir("886");
    write(
        &root,
        "index.md",
        "---\nid: IDX\ntype: index\n---\n# Index\n\n## Contents\n",
    );
    write(
        &root,
        "NOTE-001.md",
        &note("NOTE-001", "see [missing](ix://o/r/MISSING)"),
    );
    write(&root, "loose.md", "no frontmatter at all\n");

    let report = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Okf);

    // AC-8: the tokens are byte-identical to their pre-FR-057 values.
    let reasons: BTreeSet<&str> = all(&report).map(|f| f.reason.as_ref()).collect();
    for expected in [
        "dangling-reference",
        "index-incomplete",
        "index-okf-version",
        "no-frontmatter",
    ] {
        assert!(reasons.contains(expected), "missing reason {expected}");
    }

    // AC-7: severity matches the vector the finding landed in.
    for f in report.errors.iter() {
        assert_eq!(f.severity, GrammarSeverity::Error, "{f:?}");
    }
    for f in report.warnings.iter() {
        assert_eq!(f.severity, GrammarSeverity::Warning, "{f:?}");
    }

    // AC-9: every pack finding is registrable, and its pack is a declared one.
    let known = [pack::BUNDLE, pack::REFS, pack::EDGES, pack::TRACE];
    let mut packed = 0usize;
    for f in all(&report) {
        let Some(p) = f.pack else { continue };
        packed += 1;
        assert!(known.contains(&p), "undeclared pack {p:?} on {f:?}");
        let key = f.severity_key().expect("a packed finding has a key");
        assert_eq!(key, severity_key(p, &f.reason));
        assert!(
            quire_rs::grammar::is_severity_key(&key),
            "`--severity {key}=off` would be rejected"
        );
    }
    assert!(packed >= 4, "expected several pack findings, got {packed}");

    fs::remove_dir_all(&root).ok();
}

// TC-887, FR-057-AC-10: order is a property of the bundle, not of the map
// (NFR-006) — findings come out in the same order with and without a registry,
// and identically across runs.
#[test]
fn tc887_severity_does_not_perturb_order() {
    let root = tmpdir("887");
    write(
        &root,
        "index.md",
        "---\nid: IDX\ntype: index\n---\n# Index\n\n## Contents\n",
    );
    for id in ["NOTE-003", "NOTE-001", "NOTE-002"] {
        write(
            &root,
            &format!("{id}.md"),
            &note(id, "see [missing](ix://o/r/MISSING)"),
        );
    }

    let shape = |r: &BundleReport| -> Vec<(String, PathBuf)> {
        all(r)
            .map(|f| (f.reason.to_string(), f.path.clone()))
            .collect()
    };

    let plain = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Okf);
    let mapped = validate_bundle_at(
        &root,
        &severity_registry(&[("refs:dangling-reference", GrammarSeverityLevel::Warning)]),
        BundlePosture::Okf,
    );
    assert_eq!(shape(&plain), shape(&mapped), "a map must not reorder");

    let again = validate_bundle_at(&root, &bundle_registry(), BundlePosture::Okf);
    assert_eq!(shape(&plain), shape(&again), "repeated runs must agree");

    fs::remove_dir_all(&root).ok();
}

// TC-888, FR-057-CON-1: document-level results bridged into the report are NOT
// registrable. A module that could map `unknown-type: off` would be switching
// off schema validation under a severity key, which is a different decision.
#[test]
fn tc888_bridged_document_results_are_not_registrable() {
    let root = tmpdir("888");
    write(
        &root,
        "X-1.md",
        "---\nid: X-1\ntype: weird\n---\n# x\nbody\n",
    );
    write(&root, "Y-1.md", "---\nid: Y-1\n---\n# y\nbody\n");

    // Even with both keys mapped `off`, the findings stand.
    let registry = severity_registry(&[
        ("bundle:unknown-type", GrammarSeverityLevel::Off),
        ("bundle:frontmatter", GrammarSeverityLevel::Off),
    ]);
    let report = validate_bundle_at(&root, &registry, BundlePosture::Strict);
    assert!(!report.is_valid());
    assert_eq!(count(&report.errors, "unknown-type"), 1);
    assert_eq!(count(&report.errors, "frontmatter"), 1);

    // And they carry no pack, so no key could ever address them.
    for f in all(&report).filter(|f| f.reason == "unknown-type" || f.reason == "frontmatter") {
        assert_eq!(f.pack, None, "{f:?} must not be registrable");
        assert_eq!(f.severity_key(), None, "{f:?} must have no key");
    }

    fs::remove_dir_all(&root).ok();
}
