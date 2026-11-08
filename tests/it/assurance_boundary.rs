//! Static FR-067/FR-068 purity and compatibility boundaries.

use std::fs;
use std::path::PathBuf;

use ix_trace_rs::trace;
use serde_json::Value;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[trace("TC-1099", "FR-068-AC-9", "FR-068-CON-1")]
#[test]
fn tc1099_exporter_uses_only_authoritative_record_surfaces() {
    let source = fs::read_to_string(repo().join("src/assurance.rs")).expect("assurance source");
    for forbidden in [
        "extract_frontmatter(",
        "parse_document(",
        "parse_body(",
        "harvest_edges(",
        "std::process::Command",
        "TcpStream",
        "reqwest",
        "fs::write",
    ] {
        assert!(
            !source.contains(forbidden),
            "assurance exporter crossed forbidden boundary `{forbidden}`"
        );
    }
    for required in [
        "spec.documents()",
        "spec.edges()",
        "crate::obligation::derive",
        "extraction.symbols",
        "graph.verifies",
        "graph.implements",
    ] {
        assert!(
            source.contains(required),
            "assurance exporter stopped using authoritative surface `{required}`"
        );
    }
}

#[trace("TC-1090", "FR-067-AC-7", "FR-067-CON-4", "FR-055-AC-7")]
#[test]
fn tc1090_existing_v1_contracts_remain_assurance_version_free() {
    for name in ["coverage-v1.schema.json", "properties-v1.schema.json"] {
        let schema: Value = serde_json::from_str(
            &fs::read_to_string(repo().join("schemas/output").join(name)).expect("schema"),
        )
        .expect("schema JSON");
        let serialized = serde_json::to_string(&schema["properties"]).expect("properties JSON");
        for key in ["format", "format_version", "schema_version"] {
            assert!(
                !serialized.contains(&format!("\"{key}\"")),
                "{name} unexpectedly declares assurance key `{key}`"
            );
        }
    }
}
