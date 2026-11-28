use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use ix_trace_rs::trace;
use quire_rs::extract::{dsl::ExtractionDsl, extract};
use quire_rs::parse_document;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;

const CORPUS_ROOT: &str = "tests/fixtures/fr011-body-extraction";
const REFERENCE_COMMIT: &str = "310d6744bd8fa90f261d888c0287818aa9308d8e";

#[derive(Deserialize)]
struct InputEntry {
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
struct ReferenceRecords {
    oracle_commit: String,
    input_count: usize,
    records: BTreeMap<String, Vec<JsonValue>>,
}

#[trace("FR-011-AC-5")]
#[test]
fn tier2_records_match_the_pinned_independent_python_reference_for_all_72_types() {
    let corpus_root = Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS_ROOT);
    let inputs: BTreeMap<String, InputEntry> = serde_json::from_slice(
        &fs::read(corpus_root.join("parity-inputs.json")).expect("read parity inputs"),
    )
    .expect("parse parity inputs");
    let reference: ReferenceRecords = serde_json::from_slice(
        &fs::read(corpus_root.join("python-reference-records.json"))
            .expect("read reference outputs"),
    )
    .expect("parse reference outputs");

    assert_eq!(reference.oracle_commit, REFERENCE_COMMIT);
    assert_eq!(
        inputs.len(),
        72,
        "every pinned body-extraction DSL has input"
    );
    assert_eq!(reference.input_count, inputs.len());
    assert_eq!(reference.records.len(), inputs.len());

    for (qualified_name, input) in &inputs {
        let input_path = corpus_root.join(&input.path);
        let source = fs::read(&input_path).expect("read parity document");
        assert_eq!(
            sha256(&source),
            input.sha256,
            "input hash: {qualified_name}"
        );
        let document = parse_document(std::str::from_utf8(&source).expect("UTF-8 document"));

        let (module, name) = qualified_name
            .split_once("::")
            .expect("module-qualified object type");
        let manifest: YamlValue = serde_yaml::from_slice(
            &fs::read(
                corpus_root
                    .join("modules")
                    .join(module)
                    .join("manifest.yaml"),
            )
            .expect("read pinned module manifest"),
        )
        .expect("parse pinned module manifest");
        let object_types = manifest
            .get("object_types")
            .and_then(YamlValue::as_sequence)
            .expect("manifest object_types sequence");
        let object_type = object_types
            .iter()
            .find(|object_type| object_type.get("name").and_then(YamlValue::as_str) == Some(name))
            .unwrap_or_else(|| panic!("missing DSL: {qualified_name}"));
        let dsl: ExtractionDsl = serde_yaml::from_value(
            object_type
                .get("body_extraction")
                .expect("body_extraction DSL")
                .clone(),
        )
        .expect("parse body_extraction DSL");
        let actual = extract(&document, &dsl)
            .unwrap_or_else(|error| panic!("extract {qualified_name}: {error}"));
        let expected = reference
            .records
            .get(qualified_name)
            .unwrap_or_else(|| panic!("missing oracle output: {qualified_name}"));
        assert_eq!(
            serde_json::to_value(actual.records).expect("serialize extracted records"),
            serde_json::to_value(expected).expect("serialize expected records"),
            "Tier 2 output differs from Python reference for {qualified_name}"
        );
    }
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    format!("{:x}", Sha256::digest(bytes))
}
