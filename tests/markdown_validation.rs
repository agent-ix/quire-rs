//! Gate G7 integration: markdown structural validation (FR-032/FR-035)
//! against the real loaded `spec-artifacts-iso` FR archetype.
//!
//! A conformant FR document validates; mutations (missing / placeholder /
//! frontmatter / duplicate-heading) each fail with a line-numbered,
//! reasoned diagnostic. This is the single engine entry point
//! (`validate_document`); no surface re-implements the logic.

use std::collections::BTreeMap;
use std::path::Path;

use quire_rs::validate_document::ValidationReason;
use quire_rs::Registry;

fn iso_registry() -> Registry {
    let module = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules/iso");
    Registry::load_module(&module).expect("load iso module")
}

/// A conformant FR markdown document: valid frontmatter + all required
/// sections (Description, Specification, Acceptance Criteria,
/// Dependencies) populated with substantive content, headings unique
/// per level.
const CONFORMANT_FR: &str = "---\n\
id: FR-901\n\
title: \"A conformant requirement\"\n\
type: FR\n\
---\n\
# [FR-901] A conformant requirement\n\
\n\
## Description\n\
The system SHALL preserve byte-exact content across a parse round-trip.\n\
\n\
## Specification\n\
On parse, the engine retains every byte of the section body verbatim.\n\
\n\
## Acceptance Criteria\n\
\n\
| ID | Criteria | Verification |\n\
|----|----------|--------------|\n\
| FR-901-AC-1 | Round-trip is byte-identical | Integration Test |\n\
\n\
## Dependencies\n\
\n\
- **Upstream**: none\n\
- **Downstream**: none\n";

// TC-528 (FR-032-AC-1): a conformant FR validates.
#[test]
fn conformant_fr_validates() {
    let r = iso_registry();
    let fr = r.archetype("FR").expect("FR archetype");
    let result = quire_rs::validate_document(fr, CONFORMANT_FR);
    assert!(result.is_valid, "expected valid, got: {:?}", result.errors);
    assert!(result.errors.is_empty());
}

// TC-529 (FR-032-AC-2): removing a required section fails with reason
// `missing`, naming the archetype + section.
#[test]
fn missing_required_section_fails() {
    let r = iso_registry();
    let fr = r.archetype("FR").expect("FR archetype");
    // Drop the whole "## Specification" section.
    let mutated = CONFORMANT_FR.replace(
        "## Specification\n\
On parse, the engine retains every byte of the section body verbatim.\n\n",
        "",
    );
    let result = quire_rs::validate_document(fr, &mutated);
    assert!(!result.is_valid);
    let e = result
        .errors
        .iter()
        .find(|e| e.reason == ValidationReason::Missing)
        .expect("a missing-section diagnostic");
    assert!(e.message.contains("FR"), "{}", e.message);
    assert!(e.message.contains("Specification"), "{}", e.message);
}

// TC-530 (FR-032-AC-3): a placeholder-only required section fails with
// reason `placeholder` even though the frontmatter is valid.
#[test]
fn placeholder_section_fails() {
    let r = iso_registry();
    let fr = r.archetype("FR").expect("FR archetype");
    let mutated = CONFORMANT_FR.replace(
        "On parse, the engine retains every byte of the section body verbatim.",
        "TODO",
    );
    let result = quire_rs::validate_document(fr, &mutated);
    assert!(!result.is_valid);
    assert!(result
        .errors
        .iter()
        .any(|e| e.reason == ValidationReason::Placeholder));
}

// TC-531 (FR-032-AC-4): a frontmatter-schema violation fails with reason
// `frontmatter`, independent of body structure.
#[test]
fn frontmatter_violation_fails() {
    let r = iso_registry();
    let fr = r.archetype("FR").expect("FR archetype");
    // `type: NFR` violates the FR `const: "FR"`.
    let mutated = CONFORMANT_FR.replace("type: FR", "type: NFR");
    let result = quire_rs::validate_document(fr, &mutated);
    assert!(!result.is_valid);
    assert!(result
        .errors
        .iter()
        .any(|e| e.reason == ValidationReason::Frontmatter));
}

// TC-562 (FR-033-AC-4, registry path): an archetype that carries an
// `assert` facet (AC table `columns` + interpolated `id_pattern`),
// loaded via `Registry`, is wired manifest → load → `validate_document`.
// This proves the assert path travels the whole pipeline, not just the
// unit-level `evaluate_assert`. A mis-prefixed AC id fails with reason
// `assert`; the conformant document passes.
const ASSERT_MANIFEST: &str = r#"name: assert-mod
artifact_types:
- name: FR
  frontmatter_schema_ref: schemas/fr.schema.json
  body_extraction:
    yield_pattern:
      match:
        acceptance_criteria:
          from: table_row
          under_section: Acceptance Criteria
          required: true
          assert:
            columns: [ID, Criteria, Verification]
            id_column: ID
            id_pattern: '^{id}-AC-\d+$'
"#;

fn assert_registry() -> Registry {
    let mut schemas = BTreeMap::new();
    schemas.insert(
        "schemas/fr.schema.json".to_string(),
        r#"{"type":"object","required":["id"],"properties":{"id":{"type":"string"}}}"#.to_string(),
    );
    Registry::from_inline_parts(ASSERT_MANIFEST.as_bytes(), &schemas)
        .expect("inline module with assert facet loads")
}

const ASSERT_DOC_OK: &str = "---\n\
id: FR-700\n\
---\n\
## Acceptance Criteria\n\
\n\
| ID | Criteria | Verification |\n\
|----|----------|--------------|\n\
| FR-700-AC-1 | First criterion | Integration Test |\n\
| FR-700-AC-2 | Second criterion | Unit Test |\n";

#[test]
fn registry_assert_path_passes_conformant() {
    let r = assert_registry();
    let fr = r.archetype("FR").expect("FR archetype loaded with assert");
    assert!(
        fr.body_extraction().is_some(),
        "assert facet must survive manifest load"
    );
    let result = quire_rs::validate_document(fr, ASSERT_DOC_OK);
    assert!(result.is_valid, "expected valid, got: {:?}", result.errors);
}

#[test]
fn registry_assert_path_fails_on_bad_id_prefix() {
    let r = assert_registry();
    let fr = r.archetype("FR").expect("FR archetype");
    // An AC id that does not match `^{id}-AC-\d+$`.
    let mutated = ASSERT_DOC_OK.replace("FR-700-AC-2", "WRONG-AC-2");
    let result = quire_rs::validate_document(fr, &mutated);
    assert!(!result.is_valid);
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.reason == ValidationReason::Assert),
        "expected an assert failure, got: {:?}",
        result.errors
    );
}

#[test]
fn registry_assert_path_fails_on_wrong_columns() {
    let r = assert_registry();
    let fr = r.archetype("FR").expect("FR archetype");
    // Rename a column header so `columns` assert no longer matches.
    let mutated = ASSERT_DOC_OK.replace(
        "| ID | Criteria | Verification |",
        "| ID | Desc | Verification |",
    );
    let result = quire_rs::validate_document(fr, &mutated);
    assert!(!result.is_valid);
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.reason == ValidationReason::Assert),
        "expected an assert failure, got: {:?}",
        result.errors
    );
}

// ── FR-003 (spec-artifacts-iso): the generic master-requirements archetype ──
//
// The root spec.md (type: master-requirements) validates against the
// ninth registered archetype: a bespoke frontmatter schema (no id/title; kebab
// component_type) plus the canonical body (H1 title + Purpose/Scope/System
// Overview/Requirements Architecture/References). Proves a level-1 heading
// locator and a frontmatter-only-shape archetype travel manifest → load →
// validate_document.

const CONFORMANT_MASTER: &str = "---\n\
type: master-requirements\n\
name: example-service\n\
org: agent-ix\n\
component_type: fastapi-service\n\
---\n\
# Master Requirements Specification\n\
\n\
## Purpose\n\
This document specifies the requirements for the example service so that\n\
implementers and consumers share one authoritative definition of done.\n\
\n\
## Scope\n\
### In Scope\n\
- The service's public API and the behaviour each requirement pins down.\n\
\n\
## System Overview\n\
### System Description\n\
A concise description of the service and its place in the wider system.\n\
\n\
## Requirements Architecture\n\
The requirement classes that make up this specification and how they trace.\n\
\n\
## References\n\
- ISO/IEC/IEEE 29148 — Requirements engineering.\n";

// FR-003-AC-3: a conformant master spec validates.
#[test]
fn conformant_master_requirements_validates() {
    let r = iso_registry();
    let a = r
        .archetype("master-requirements")
        .expect("master-requirements archetype");
    let result = quire_rs::validate_document(a, CONFORMANT_MASTER);
    assert!(result.is_valid, "expected valid, got: {:?}", result.errors);
}

// FR-003-AC-4: a missing component_type fails with reason `frontmatter`.
#[test]
fn master_requirements_missing_component_type_fails() {
    let r = iso_registry();
    let a = r.archetype("master-requirements").expect("archetype");
    let mutated = CONFORMANT_MASTER.replace("component_type: fastapi-service\n", "");
    let result = quire_rs::validate_document(a, &mutated);
    assert!(!result.is_valid);
    assert!(result
        .errors
        .iter()
        .any(|e| e.reason == ValidationReason::Frontmatter));
}

// FR-003-AC-5: a non-kebab component_type fails the pattern (frontmatter).
#[test]
fn master_requirements_non_kebab_component_type_fails() {
    let r = iso_registry();
    let a = r.archetype("master-requirements").expect("archetype");
    let mutated = CONFORMANT_MASTER.replace(
        "component_type: fastapi-service",
        "component_type: \"Fast API Service\"",
    );
    let result = quire_rs::validate_document(a, &mutated);
    assert!(!result.is_valid);
    assert!(result
        .errors
        .iter()
        .any(|e| e.reason == ValidationReason::Frontmatter));
}

// FR-003-AC-6: dropping the H1 title fails with reason `missing` (level-1
// heading locator).
#[test]
fn master_requirements_missing_h1_title_fails() {
    let r = iso_registry();
    let a = r.archetype("master-requirements").expect("archetype");
    let mutated =
        CONFORMANT_MASTER.replace("# Master Requirements Specification", "# example-service");
    let result = quire_rs::validate_document(a, &mutated);
    assert!(!result.is_valid);
    let e = result
        .errors
        .iter()
        .find(|e| e.reason == ValidationReason::Missing)
        .expect("a missing diagnostic for the H1 title");
    assert!(e.message.contains("title"), "{}", e.message);
}

// FR-003-AC-6: dropping a required canonical section fails with `missing`.
#[test]
fn master_requirements_missing_section_fails() {
    let r = iso_registry();
    let a = r.archetype("master-requirements").expect("archetype");
    let mutated = CONFORMANT_MASTER.replace(
        "## References\n- ISO/IEC/IEEE 29148 — Requirements engineering.\n",
        "",
    );
    let result = quire_rs::validate_document(a, &mutated);
    assert!(!result.is_valid);
    let e = result
        .errors
        .iter()
        .find(|e| e.reason == ValidationReason::Missing)
        .expect("a missing-section diagnostic");
    assert!(e.message.contains("References"), "{}", e.message);
}

// FR-003-AC-7: optional/extra sections (Domain Model, Security Model) are
// accepted — the contract asserts required structure, it does not forbid extras.
#[test]
fn master_requirements_optional_sections_accepted() {
    let r = iso_registry();
    let a = r.archetype("master-requirements").expect("archetype");
    let doc = format!(
        "{CONFORMANT_MASTER}\n## Domain Model\nEntities and invariants.\n\
\n## Security Model\nActor roles and trust boundaries.\n"
    );
    let result = quire_rs::validate_document(a, &doc);
    assert!(result.is_valid, "expected valid, got: {:?}", result.errors);
}

// FR-003 + FR-010: ISO section numbering is decorative. A master spec whose
// canonical sections carry `## N. Name` prefixes validates the same as the
// bare `## Name` skeleton — the `from: heading` locator normalizes the number
// prefix consistently with `section_body`/`after_heading`.
#[test]
fn master_requirements_tolerates_numbered_headings() {
    let r = iso_registry();
    let a = r.archetype("master-requirements").expect("archetype");
    let numbered = "---\n\
type: master-requirements\n\
name: x\n\
org: agent-ix\n\
component_type: fastapi-service\n\
---\n\
# Master Requirements Specification\n\
\n\
## 1. Purpose\n\
Numbered purpose prose, substantive.\n\
\n\
## 2. Scope\n\
### 2.1 In Scope\n\
- stuff\n\
\n\
## 3. System Overview\n\
### 3.1 System Description\n\
desc\n\
\n\
## 4. Requirements Architecture\n\
classes\n\
\n\
## 5. References\n\
- ISO 29148\n";
    let result = quire_rs::validate_document(a, numbered);
    assert!(result.is_valid, "expected valid, got: {:?}", result.errors);
}

// FR-035 (TC-544/547): a duplicate heading at the same level fails with
// reason `duplicate-heading`, line-numbered at the second heading.
#[test]
fn duplicate_heading_fails_with_line() {
    let r = iso_registry();
    let fr = r.archetype("FR").expect("FR archetype");
    // Append a second "## Description" heading.
    let mutated = format!("{CONFORMANT_FR}\n## Description\nA second one.\n");
    let result = quire_rs::validate_document(fr, &mutated);
    assert!(!result.is_valid);
    let e = result
        .errors
        .iter()
        .find(|e| e.reason == ValidationReason::DuplicateHeading)
        .expect("duplicate-heading diagnostic");
    assert!(e.message.contains("Description"), "{}", e.message);
    assert!(e.line.is_some(), "duplicate must be line-numbered");
}
