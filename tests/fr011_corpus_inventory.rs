use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
struct Inventory {
    modules: Vec<ModuleEntry>,
    object_types: Vec<ObjectTypeEntry>,
    documents: Vec<FileEntry>,
    unmapped_documents: Vec<String>,
    generated_inputs: Vec<GeneratedInput>,
    files: Vec<FileEntry>,
}

#[derive(Deserialize)]
struct ModuleEntry {
    name: String,
    source_repository: String,
    source_commit: String,
    manifest_sha256: String,
    object_type_count: usize,
    body_extraction_count: usize,
    schema_closure: Vec<String>,
    schema_external_refs: Vec<SchemaExternalRef>,
}

#[derive(Deserialize)]
struct ObjectTypeEntry {
    module: String,
    name: String,
    module_qualified_key: String,
    dsl_sha256_canonical_json: String,
    fixtures: Vec<String>,
    fixture_status: String,
    missing_fixture: bool,
    missing_fixture_reason: Option<String>,
    generated_fixture: Option<String>,
    has_exercisable_input: bool,
}

#[derive(Deserialize)]
struct GeneratedInput {
    module: String,
    name: String,
    module_qualified_key: String,
    source_repository: String,
    source_commit: String,
    source_manifest_path: String,
    source_manifest_sha256: String,
    dsl_sha256_canonical_json: String,
    origin: String,
    path: String,
    content_sha256: String,
    file_sha256: String,
    locators: Vec<String>,
}

#[derive(Deserialize)]
struct SchemaExternalRef {
    source_file: String,
    uri: String,
}

#[derive(Deserialize)]
struct FileEntry {
    path: String,
    source_path: String,
    sha256: String,
    role: String,
}

const CORPUS_ROOT: &str = "tests/fixtures/fr011-body-extraction";

// This checks pinned corpus integrity and generated input substrates only.
// It does not assert ExtractionResult values or provide FR-011-AC-5 evidence.
#[test]
fn fr011_input_inventory_matches_manifests_schemas_and_fixture_documents() {
    let corpus_root = Path::new(env!("CARGO_MANIFEST_DIR")).join(CORPUS_ROOT);
    let inventory: Inventory = serde_json::from_str(
        &fs::read_to_string(corpus_root.join("inventory.json")).expect("read inventory"),
    )
    .expect("parse inventory");

    let listed_files = inventory
        .files
        .iter()
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        listed_files.len(),
        inventory.files.len(),
        "duplicate file path"
    );
    for file in &inventory.files {
        let bytes = fs::read(corpus_root.join(&file.path)).expect("read corpus file");
        assert_eq!(sha256(&bytes), file.sha256, "file hash: {}", file.path);
        match file.role.as_str() {
            "manifest" => assert!(file.source_path.ends_with("/manifest.yaml")),
            "schema" => assert!(file.source_path.contains("/schemas/")),
            "document" => {
                assert!(file.source_path.starts_with("tests/fixtures/"));
                assert!(file.source_path.ends_with(".md"));
            }
            "generated_document" => assert!(file.source_path.starts_with("generated-from:")),
            role => panic!("unknown inventory role {role}"),
        }
    }

    let mut actual_files = BTreeSet::new();
    collect_files(
        &corpus_root.join("modules"),
        &corpus_root,
        &mut actual_files,
    );
    assert_eq!(
        actual_files, listed_files,
        "corpus files must be inventoried exactly"
    );

    let source_document_paths = inventory
        .documents
        .iter()
        .map(|document| document.path.clone())
        .collect::<BTreeSet<_>>();
    let indexed_source_documents = inventory
        .files
        .iter()
        .filter(|file| file.role == "document")
        .map(|file| file.path.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(source_document_paths, indexed_source_documents);

    let source_fixture_paths = inventory
        .object_types
        .iter()
        .flat_map(|entry| entry.fixtures.iter().cloned())
        .collect::<BTreeSet<_>>();
    let expected_unmapped = source_document_paths
        .difference(&source_fixture_paths)
        .cloned()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        expected_unmapped,
        inventory.unmapped_documents.iter().cloned().collect()
    );

    let generated_by_key = inventory
        .generated_inputs
        .iter()
        .map(|generated| {
            assert_eq!(
                generated.module_qualified_key,
                format!("{}::{}", generated.module, generated.name)
            );
            assert_eq!(generated.origin, "generated-input-only");
            assert_eq!(generated.source_commit.len(), 40);
            assert!(is_sha256(&generated.source_manifest_sha256));
            assert!(is_sha256(&generated.dsl_sha256_canonical_json));
            assert!(is_sha256(&generated.content_sha256));
            assert!(is_sha256(&generated.file_sha256));
            let path = corpus_root.join(&generated.path);
            let source = fs::read_to_string(&path).expect("read generated input");
            assert_eq!(
                sha256(source.as_bytes()),
                generated.file_sha256,
                "{}",
                generated.path
            );
            verify_generated_metadata(generated, &source);
            (
                (generated.module.as_str(), generated.name.as_str()),
                generated,
            )
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(generated_by_key.len(), inventory.generated_inputs.len());
    assert_eq!(
        inventory.generated_inputs.len(),
        24,
        "generated input count"
    );

    let entries_by_key = inventory
        .object_types
        .iter()
        .map(|entry| {
            assert_eq!(
                entry.module_qualified_key,
                format!("{}::{}", entry.module, entry.name)
            );
            ((entry.module.as_str(), entry.name.as_str()), entry)
        })
        .collect::<BTreeMap<_, _>>();
    assert_eq!(entries_by_key.len(), inventory.object_types.len());
    assert_eq!(inventory.object_types.len(), 72, "pinned DSL count");

    let mut all_schema_ids = BTreeSet::new();
    for file in inventory.files.iter().filter(|file| file.role == "schema") {
        let value: JsonValue =
            serde_json::from_slice(&fs::read(corpus_root.join(&file.path)).expect("read schema"))
                .expect("parse schema JSON");
        collect_json_values(&value, "$id", &mut all_schema_ids);
    }

    for module in &inventory.modules {
        assert_eq!(
            module.source_repository,
            format!("agent-ix/spec-objects-{}", module.name)
        );
        assert_eq!(module.source_commit.len(), 40);
        assert_eq!(module.object_type_count, module.body_extraction_count);
        assert!(is_sha256(&module.manifest_sha256));
        let module_root = corpus_root.join("modules").join(&module.name);
        let package_name = format!("spec_objects_{}", module.name);
        let manifest_path = module_root.join("manifest.yaml");
        let manifest_bytes = fs::read(&manifest_path).expect("read pinned manifest");
        assert_eq!(sha256(&manifest_bytes), module.manifest_sha256);
        assert!(inventory.files.iter().any(|file| {
            file.path == format!("modules/{}/manifest.yaml", module.name)
                && file.source_path == format!("{package_name}/manifest.yaml")
                && file.sha256 == module.manifest_sha256
        }));

        let manifest: YamlValue = serde_yaml::from_slice(&manifest_bytes).expect("parse manifest");
        let object_types = manifest
            .get("object_types")
            .and_then(YamlValue::as_sequence)
            .expect("manifest object_types sequence");
        assert_eq!(
            object_types.len(),
            module.object_type_count,
            "{}",
            module.name
        );
        let mut declared_names = BTreeSet::new();
        for object_type in object_types {
            let name = object_type
                .get("name")
                .and_then(YamlValue::as_str)
                .expect("object type name");
            assert!(
                declared_names.insert(name),
                "duplicate DSL name: {}::{name}",
                module.name
            );
            let dsl = object_type
                .get("body_extraction")
                .expect("body_extraction DSL");
            let canonical_dsl = serde_json::to_vec(&serde_json::to_value(dsl).expect("DSL JSON"))
                .expect("canonical DSL JSON");
            let dsl_hash = sha256(&canonical_dsl);
            let key = (module.name.as_str(), name);
            let entry = entries_by_key
                .get(&key)
                .unwrap_or_else(|| panic!("inventory missing {}::{name}", module.name));
            assert_eq!(
                entry.dsl_sha256_canonical_json, dsl_hash,
                "DSL hash: {}::{name}",
                module.name
            );
            assert_eq!(entry.fixtures.is_empty(), entry.missing_fixture);
            assert_eq!(entry.fixture_status == "missing", entry.missing_fixture);
            assert_eq!(entry.missing_fixture, entry.generated_fixture.is_some());
            assert_eq!(
                entry.has_exercisable_input,
                !entry.fixtures.is_empty() || entry.generated_fixture.is_some()
            );
            if entry.missing_fixture {
                assert!(entry.missing_fixture_reason.is_some());
            } else {
                assert!(entry.missing_fixture_reason.is_none());
            }
            for fixture in &entry.fixtures {
                verify_fixture_mapping(entry, fixture, &inventory.documents, &corpus_root);
            }
            if let Some(generated_path) = &entry.generated_fixture {
                let generated = generated_by_key.get(&key).unwrap_or_else(|| {
                    panic!("missing generated input for {}::{name}", module.name)
                });
                assert_eq!(&generated.path, generated_path);
                assert_eq!(generated.source_repository, module.source_repository);
                assert_eq!(generated.source_commit, module.source_commit);
                assert_eq!(
                    generated.source_manifest_path,
                    format!("spec_objects_{}/manifest.yaml", module.name)
                );
                assert_eq!(generated.source_manifest_sha256, module.manifest_sha256);
                assert_eq!(generated.dsl_sha256_canonical_json, dsl_hash);
                verify_locator_substrate(generated, dsl, &corpus_root);
            }
        }
        assert_eq!(declared_names.len(), module.object_type_count);

        let schema_files = module
            .schema_closure
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let indexed_schema_files = inventory
            .files
            .iter()
            .filter(|file| {
                file.role == "schema" && file.path.starts_with(&format!("modules/{}/", module.name))
            })
            .map(|file| file.path.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            schema_files, indexed_schema_files,
            "schema closure: {}",
            module.name
        );
        let module_prefix = format!("modules/{}/", module.name);
        for schema_file in &schema_files {
            let source_suffix = schema_file
                .strip_prefix(&module_prefix)
                .expect("schema path is module-qualified");
            assert!(inventory.files.iter().any(|file| {
                file.path == *schema_file
                    && file.source_path == format!("{package_name}/{source_suffix}")
            }));
        }
        for schema_ref in collect_manifest_schema_refs(&manifest) {
            assert!(
                module_root.join(schema_ref).is_file(),
                "unresolved manifest schema ref {}::{schema_ref}",
                module.name
            );
        }

        let mut external_schema_refs = BTreeSet::new();
        for schema_path in &module.schema_closure {
            let schema: JsonValue = serde_json::from_slice(
                &fs::read(corpus_root.join(schema_path)).expect("read schema"),
            )
            .expect("parse schema JSON");
            let mut refs = BTreeSet::new();
            collect_json_values(&schema, "$ref", &mut refs);
            for reference in refs {
                let base = reference.split('#').next().unwrap_or_default();
                if base.is_empty() {
                    continue;
                }
                if base.contains("://") {
                    if !all_schema_ids.contains(base) {
                        external_schema_refs.insert((schema_path.clone(), reference));
                    }
                    continue;
                }
                let source_schema = corpus_root.join(schema_path);
                let relative_target = source_schema.parent().expect("schema parent").join(base);
                let module_target = module_root.join(base);
                assert!(
                    relative_target.is_file() || module_target.is_file(),
                    "unresolved local schema reference {} in {schema_path}",
                    reference
                );
            }
        }
        let recorded_external = module
            .schema_external_refs
            .iter()
            .map(|reference| (reference.source_file.clone(), reference.uri.clone()))
            .collect::<BTreeSet<_>>();
        assert_eq!(
            external_schema_refs, recorded_external,
            "external schema gaps: {}",
            module.name
        );
    }

    assert_eq!(
        inventory
            .object_types
            .iter()
            .filter(|entry| entry.missing_fixture)
            .count(),
        24,
        "source fixture gaps remain explicit"
    );
    assert_eq!(
        inventory
            .object_types
            .iter()
            .filter(|entry| entry.has_exercisable_input)
            .count(),
        72,
        "every pinned DSL has a source or generated input"
    );
}

fn collect_files(directory: &Path, root: &Path, files: &mut BTreeSet<String>) {
    for entry in fs::read_dir(directory).expect("read corpus directory") {
        let path = entry.expect("read corpus directory entry").path();
        if path.is_dir() {
            collect_files(&path, root, files);
        } else {
            files.insert(
                path.strip_prefix(root)
                    .expect("corpus path is under root")
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
}

fn verify_fixture_mapping(
    entry: &ObjectTypeEntry,
    fixture: &str,
    documents: &[FileEntry],
    root: &Path,
) {
    let document = documents
        .iter()
        .find(|document| document.path == fixture)
        .unwrap_or_else(|| panic!("fixture {fixture} is absent from document inventory"));
    let suffix = fixture
        .strip_prefix(&format!("modules/{}/documents/", entry.module))
        .expect("fixture path is module-qualified");
    assert_eq!(document.source_path, format!("tests/fixtures/{suffix}"));
    let source = fs::read_to_string(root.join(fixture)).expect("read fixture document");
    let frontmatter: YamlValue =
        serde_yaml::from_str(frontmatter(&source).expect("mapped fixture frontmatter"))
            .expect("parse fixture frontmatter");
    let object_name = frontmatter
        .get("object")
        .or_else(|| frontmatter.get("type"))
        .and_then(YamlValue::as_str)
        .expect("mapped fixture object or type");
    assert_eq!(object_name, entry.name, "fixture mapping {fixture}");
}

fn verify_generated_metadata(generated: &GeneratedInput, source: &str) {
    let comments = source
        .lines()
        .filter_map(|line| line.strip_prefix("# corpus "))
        .collect::<Vec<_>>();
    for expected in [
        format!("origin=generated-input-only"),
        format!("module={}", generated.module),
        format!("object_type={}", generated.name),
        format!("source_repository={}", generated.source_repository),
        format!("source_commit={}", generated.source_commit),
        format!("source_manifest_path={}", generated.source_manifest_path),
        format!(
            "source_manifest_sha256={}",
            generated.source_manifest_sha256
        ),
        format!(
            "dsl_sha256_canonical_json={}",
            generated.dsl_sha256_canonical_json
        ),
        format!("content_sha256={}", generated.content_sha256),
    ] {
        assert!(
            comments.contains(&expected.as_str()),
            "missing provenance {expected}"
        );
    }
    assert!(
        !generated.locators.is_empty(),
        "generated input records locator checks"
    );

    let rest = source
        .strip_prefix("---\n")
        .expect("frontmatter starts document");
    let end = rest.find("\n---\n").expect("frontmatter terminator");
    let payload_frontmatter = rest[..end]
        .lines()
        .filter(|line| !line.starts_with("# corpus "))
        .collect::<Vec<_>>()
        .join("\n");
    let payload = format!("---\n{payload_frontmatter}\n---\n{}", &rest[end + 5..]);
    assert_eq!(
        sha256(payload.as_bytes()),
        generated.content_sha256,
        "payload hash"
    );
}

fn verify_locator_substrate(generated: &GeneratedInput, dsl: &YamlValue, root: &Path) {
    let source = fs::read_to_string(root.join(&generated.path)).expect("read generated document");
    let frontmatter: YamlValue =
        serde_yaml::from_str(frontmatter(&source).expect("generated frontmatter"))
            .expect("parse generated frontmatter");
    let locators = dsl
        .get("yield_pattern")
        .and_then(|pattern| pattern.get("match"))
        .and_then(YamlValue::as_mapping)
        .expect("single-yield match DSL");
    let mut checked = Vec::new();
    for (field, locator) in locators {
        let field = field.as_str().expect("DSL field name");
        let primitive = locator
            .get("from")
            .and_then(YamlValue::as_str)
            .expect("locator primitive");
        match primitive {
            "frontmatter_field" => {
                let path = locator
                    .get("path")
                    .and_then(YamlValue::as_sequence)
                    .expect("frontmatter path");
                let value = path.iter().fold(Some(&frontmatter), |current, part| {
                    let part = part.as_str()?;
                    current?.get(part)
                });
                let value = value.unwrap_or_else(|| {
                    panic!(
                        "missing frontmatter substrate for {} field {field}",
                        generated.module_qualified_key
                    )
                });
                assert!(
                    value_is_concrete(value),
                    "placeholder frontmatter value: {field}"
                );
                if let Some(pattern) = locator.get("regex").and_then(YamlValue::as_str) {
                    let value = value.as_str().expect("regex locator produces text");
                    assert!(
                        regex::Regex::new(pattern)
                            .expect("valid source regex")
                            .is_match(value),
                        "frontmatter value does not satisfy `{pattern}`: {field}"
                    );
                }
                checked.push(format!("frontmatter_field:{field}"));
            }
            "section_body" => {
                let heading = locator
                    .get("after_heading")
                    .and_then(YamlValue::as_str)
                    .expect("section locator heading");
                let section = section_after_heading(&source, heading).unwrap_or_else(|| {
                    panic!(
                        "missing section substrate: {} / {heading}",
                        generated.module_qualified_key
                    )
                });
                assert!(
                    !section.trim().is_empty(),
                    "empty section substrate: {field}"
                );
                assert!(
                    !section.to_ascii_lowercase().contains("placeholder"),
                    "placeholder section substrate: {field}"
                );
                checked.push(format!("section_body:{field}"));
            }
            "code_block" => {
                let heading = locator
                    .get("after_heading")
                    .and_then(YamlValue::as_str)
                    .expect("code block heading");
                let language = locator
                    .get("language")
                    .and_then(YamlValue::as_str)
                    .unwrap_or_default();
                let section = section_after_heading(&source, heading).unwrap_or_else(|| {
                    panic!(
                        "missing code-block section: {} / {heading}",
                        generated.module_qualified_key
                    )
                });
                assert!(
                    section.contains(&format!("```{language}")),
                    "missing scoped {language} fence for {field}"
                );
                checked.push(format!("code_block:{field}:{language}"));
            }
            "table_row" => {
                let heading = locator
                    .get("under_section")
                    .and_then(YamlValue::as_str)
                    .expect("table locator section");
                let section = section_after_heading(&source, heading).unwrap_or_else(|| {
                    panic!(
                        "missing table section: {} / {heading}",
                        generated.module_qualified_key
                    )
                });
                let headers = locator
                    .get("assert")
                    .and_then(|assertion| assertion.get("columns"))
                    .and_then(YamlValue::as_sequence)
                    .expect("required table columns")
                    .iter()
                    .filter_map(YamlValue::as_str)
                    .collect::<BTreeSet<_>>();
                let table_header = section
                    .lines()
                    .find(|line| line.trim().starts_with('|'))
                    .expect("generated table header");
                let actual_headers = table_header
                    .split('|')
                    .map(str::trim)
                    .filter(|cell| !cell.is_empty())
                    .collect::<BTreeSet<_>>();
                assert!(
                    headers.is_subset(&actual_headers),
                    "missing asserted table columns for {field}"
                );
                checked.push(format!("table_row:{field}"));
            }
            other => panic!("unexpected generated locator primitive {other}"),
        }
    }
    checked.sort();
    assert_eq!(
        checked, generated.locators,
        "inventory locator checks: {}",
        generated.module_qualified_key
    );
}

fn section_after_heading(document: &str, title: &str) -> Option<String> {
    let mut lines = document.lines();
    while let Some(line) = lines.next() {
        if line.trim() == format!("## {title}") {
            let remaining = lines.collect::<Vec<_>>().join("\n");
            let end = remaining
                .lines()
                .position(|line| line.starts_with('#'))
                .unwrap_or_else(|| remaining.lines().count());
            return Some(remaining.lines().take(end).collect::<Vec<_>>().join("\n"));
        }
    }
    None
}

fn value_is_concrete(value: &YamlValue) -> bool {
    match value {
        YamlValue::String(value) => {
            let normalized = value.to_ascii_lowercase();
            !value.trim().is_empty()
                && !value.contains('<')
                && !normalized.contains("todo")
                && !normalized.contains("placeholder")
                && !value.contains("{{")
        }
        YamlValue::Sequence(values) => !values.is_empty() && values.iter().all(value_is_concrete),
        YamlValue::Mapping(values) => !values.is_empty() && values.values().all(value_is_concrete),
        _ => !value.is_null(),
    }
}

fn frontmatter(document: &str) -> Option<&str> {
    let rest = document.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

fn collect_manifest_schema_refs(value: &YamlValue) -> Vec<&str> {
    fn visit<'a>(value: &'a YamlValue, refs: &mut Vec<&'a str>) {
        match value {
            YamlValue::Mapping(mapping) => {
                for (key, value) in mapping {
                    if key.as_str().is_some_and(|key| key.ends_with("schema_ref")) {
                        if let Some(path) = value.as_str() {
                            refs.push(path);
                        }
                    }
                    visit(value, refs);
                }
            }
            YamlValue::Sequence(sequence) => {
                for value in sequence {
                    visit(value, refs);
                }
            }
            _ => {}
        }
    }

    let mut refs = Vec::new();
    visit(value, &mut refs);
    refs.sort_unstable();
    refs.dedup();
    refs
}

fn collect_json_values(value: &JsonValue, key: &str, values: &mut BTreeSet<String>) {
    match value {
        JsonValue::Object(object) => {
            for (name, child) in object {
                if name == key {
                    if let Some(value) = child.as_str() {
                        values.insert(value.to_owned());
                    }
                }
                collect_json_values(child, key, values);
            }
        }
        JsonValue::Array(array) => {
            for child in array {
                collect_json_values(child, key, values);
            }
        }
        _ => {}
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
