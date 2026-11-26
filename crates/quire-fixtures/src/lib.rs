//! Shared integration fixtures: Markdown-to-semantic-core mapping cases and
//! authentic published module schemas for downstream tests.
//!
//! **Ownership, not authorship.** quire-rs is the parser that defines the
//! forms these fixtures exercise (FR-070/FR-071/FR-072/FR-074/FR-076); quoin
//! is a downstream consumer that already depends on the quire-rs engine as a
//! pinned git dependency. Before this crate existed, quoin's own copy of
//! this exact fixture set (`agent-ix/quoin` FR-071/FR-072/FR-074/FR-104) was
//! duplicated byte-for-byte into `quire-rs`'s own test tree — an
//! engine-input fixture set copied from its own downstream consumer, and a
//! dependency cycle since quoin already depends on quire-rs as a crate
//! (PLAT-901). The fix is not a second, independently-authored fixture set:
//! that produces a near-copy every time, because both repos have to
//! exercise the same forms. The fix is a **move**: the content living here
//! is quoin's original, unedited — same ids, same field names, same
//! vocabulary — relocated to the repo that owns the format it exercises, so
//! quoin can resolve it as a dependency at the same rev that already governs
//! its quire-rs engine version, instead of holding a copy that drifts.
//!
//! This crate carries only path/read helpers over its local `fixtures/`
//! directory and the authentic safety module fetched by its companion
//! fixture crate. Both `quire-rs`'s own tests and downstream quoin tests can
//! read the same inputs without maintaining a second schema copy.

use std::path::{Path, PathBuf};

use serde_json::Value;

/// This crate's own directory, resolved at compile time — works whether
/// `quire-fixtures` is a workspace path dependency (quire-rs's own tests) or
/// a pinned git dependency checked out into cargo's git cache (quoin).
fn crate_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// The fixture tree root: `fixtures/`.
pub fn fixtures_root() -> PathBuf {
    crate_root().join("fixtures")
}

/// `fixtures/mapping` — the table/fence/legacy/operations/relationships set.
pub fn mapping_dir() -> PathBuf {
    fixtures_root().join("mapping")
}

/// `fixtures/module-ok` — a valid `semantic` block, one reference-form
/// export and one inline `data_schema`. Callers that mutate a manifest for a
/// scenario copy this directory to a scratch location first; it is never
/// edited in place.
pub fn module_ok_dir() -> PathBuf {
    fixtures_root().join("module-ok")
}

/// `fixtures/record-sections` — the quoin FR-121/FR-122/FR-123 record-sections
/// cases (`cases.json`) and the small record schemas they extract against
/// (`schemas/`).
pub fn record_sections_dir() -> PathBuf {
    fixtures_root().join("record-sections")
}

/// One `fixtures/record-sections` file's text.
pub fn record_sections_fixture(name: &str) -> String {
    std::fs::read_to_string(record_sections_dir().join(name))
        .unwrap_or_else(|e| panic!("record-sections fixture {name}: {e}"))
}

/// One `fixtures/record-sections` file, parsed as JSON.
pub fn record_sections_json(name: &str) -> Value {
    serde_json::from_str(&record_sections_fixture(name))
        .unwrap_or_else(|e| panic!("record-sections fixture {name}: invalid JSON: {e}"))
}

/// Directory containing the authentic fetched safety schemas. Check
/// [`safety_module_available`] first.
pub fn safety_schemas_dir() -> PathBuf {
    spec_objects_safety_fixture::schemas_dir()
}

/// Whether the published safety module was fetched by its fixture crate.
pub fn safety_module_available() -> bool {
    spec_objects_safety_fixture::is_available()
}

/// Why the safety module fixture is unavailable, if its npm fetch failed.
pub fn safety_module_unavailable_reason() -> Option<&'static str> {
    spec_objects_safety_fixture::unavailable_reason()
}

/// One fixture file's text, from `fixtures/mapping`.
pub fn mapping_fixture(name: &str) -> String {
    std::fs::read_to_string(mapping_dir().join(name))
        .unwrap_or_else(|e| panic!("mapping fixture {name}: {e}"))
}

/// One `fixtures/mapping` fixture file, parsed as JSON.
pub fn mapping_json(name: &str) -> Value {
    serde_json::from_str(&mapping_fixture(name))
        .unwrap_or_else(|e| panic!("mapping fixture {name}: invalid JSON: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_dir_exists_and_is_nonempty() {
        let dir = mapping_dir();
        assert!(dir.is_dir(), "{dir:?}");
        assert!(std::fs::read_dir(&dir).unwrap().next().is_some());
    }

    #[test]
    fn module_ok_dir_has_the_expected_layout() {
        let dir = module_ok_dir();
        assert!(dir.join("manifest.yaml").is_file());
        assert!(dir.join("schemas").join("Entity.json").is_file());
        assert!(dir.join("skeletons").is_dir());
    }

    #[test]
    fn mapping_json_parses_a_known_fixture() {
        let value = mapping_json("relationships.expected.json");
        assert!(value.is_object());
    }

    #[test]
    fn safety_fixture_exposes_authentic_record_schemas() {
        if !safety_module_available() {
            eprintln!(
                "SKIPPED: published @agent-ix/spec-objects-safety@{} unavailable: {}",
                spec_objects_safety_fixture::VERSION,
                safety_module_unavailable_reason().unwrap_or("unknown fetch failure")
            );
            return;
        }

        let schemas = safety_schemas_dir();
        for name in [
            "HazardAssessment.json",
            "FailureAnalysis.json",
            "Hazard.json",
        ] {
            let path = schemas.join(name);
            let contents = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("authentic safety schema {}: {e}", path.display()));
            let schema: Value = serde_json::from_str(&contents)
                .unwrap_or_else(|e| panic!("invalid safety schema {}: {e}", path.display()));
            assert!(
                schema.is_object(),
                "{} must be a JSON object schema",
                path.display()
            );
        }
    }
}
