//! Gate G7 integration: markdown structural validation (FR-032/FR-035)
//! against the real loaded `spec-artifacts-iso` FR archetype.
//!
//! A conformant FR document validates; mutations (missing / placeholder /
//! frontmatter / duplicate-heading) each fail with a line-numbered,
//! reasoned diagnostic. This is the single engine entry point
//! (`validate_document`); no surface re-implements the logic.

use std::path::Path;

use quire_rs::validate_document::ValidationReason;
use quire_rs::Registry;

fn iso_registry() -> Registry {
    let module = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/render_parity/modules/iso");
    Registry::load_module(&module).expect("load iso module")
}

/// A conformant FR markdown document: valid frontmatter + all required
/// sections (Description, Specification, Acceptance Criteria,
/// Dependencies) populated with substantive content, headings unique
/// per level.
const CONFORMANT_FR: &str = "---\n\
id: FR-901\n\
title: \"A conformant requirement\"\n\
artifact_type: FR\n\
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
    // `artifact_type: NFR` violates the FR `const: "FR"`.
    let mutated = CONFORMANT_FR.replace("artifact_type: FR", "artifact_type: NFR");
    let result = quire_rs::validate_document(fr, &mutated);
    assert!(!result.is_valid);
    assert!(result
        .errors
        .iter()
        .any(|e| e.reason == ValidationReason::Frontmatter));
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
