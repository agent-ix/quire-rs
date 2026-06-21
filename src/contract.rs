//! Archetype input contract + authoring skeleton (FR-029, recast by
//! ADR 0004).
//!
//! With direct-markdown authoring there is no required render template,
//! so the per-archetype input contract is a **skeleton/example** derived
//! from the `frontmatter_schema_ref` + the `body_extraction` asserts
//! (FR-033) — the structure an author fills and `validate_document`
//! (FR-032) checks. The contract is derived from the **loaded module**
//! (manifest + schema), never inferred from rendered markdown.
//!
//! [`input_contract_for`] returns deterministic, JSON-serializable data;
//! [`InputContract::skeleton`] emits a markdown scaffold for an authoring
//! agent (`/specify`).

use serde::Serialize;
use serde_json::Value;

use crate::error::QuireError;
use crate::extract::dsl::ExtractionDsl;
use crate::extract::locator::{LocatorAssert, LocatorKind, LocatorPrimitive};
use crate::loader::compile::CompiledArchetype;
use crate::registry::Registry;

/// The shape of one required body element in the contract — derived from
/// a `body_extraction` locator + its `assert` facet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContractSection {
    /// The DSL key (the field the locator populates).
    pub key: String,
    /// The section heading the locator addresses by name, or `None` when
    /// the locator does not pin a concrete section (unresolved mapping).
    pub heading: Option<String>,
    /// Required heading level, when the assert pins one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,
    /// The kind of body element (`section_body`, `table_row`, …).
    pub kind: String,
    /// Whether the locator declares `required: true`.
    pub required: bool,
    /// Exact table column headers the assert requires, in order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub columns: Option<Vec<String>>,
    /// Minimum table rows the assert requires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_rows: Option<usize>,
    /// Minimum list items the assert requires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<usize>,
    /// The id pattern (possibly with `{field}` tokens) the assert pins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_pattern: Option<String>,
    /// The content-match pattern (possibly with `{field}` tokens) the
    /// assert pins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<String>,
}

/// One allowed relationship verb in the resolved (artifact ∪ object)
/// vocabulary, presented to the authoring agent (FR-040-AC-11).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContractRelationship {
    /// The edge verb.
    pub verb: String,
    /// The verb's category from the merged `edge_types` registry, when
    /// declared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// The verb's description from the merged `edge_types` registry, when
    /// declared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Allowed target tokens (concrete object-type names, role names, or
    /// `"*"`).
    pub targets: Vec<String>,
}

/// The per-archetype input contract (FR-029 recast).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InputContract {
    /// Archetype name.
    pub archetype: String,
    /// The composed `object:` archetype name, when the contract was built
    /// for a specific object (FR-040-AC-11).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// Frontmatter JSON Schema (FR-003), or `null` when the archetype
    /// declares none.
    pub frontmatter_schema: Value,
    /// Required body structure, in manifest (`match`) order.
    pub sections: Vec<ContractSection>,
    /// Resolved relationship vocabulary (artifact ∪ object), verb-sorted
    /// (FR-040-AC-11).
    pub relationships: Vec<ContractRelationship>,
    /// Unresolved-mapping diagnostics (FR-029-AC-6) — never silently
    /// dropped sections.
    pub diagnostics: Vec<String>,
}

impl InputContract {
    /// Deterministic JSON serialization (FR-029-AC-4). Object keys are
    /// emitted by `serde_json`'s sorted-key `Map`; arrays preserve order.
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).expect("InputContract is always serializable")
    }

    /// Emit a markdown authoring skeleton (heading scaffold + literal
    /// table headers + contract comments + placeholders) derived from the
    /// same contract (FR-029, ADR 0004). Deterministic.
    pub fn skeleton(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("<!-- {} authoring skeleton -->\n", self.archetype));
        out.push_str("<!-- Fill every section below; `quire validate` checks the structure. -->\n");
        out.push_str(
            "---\n# frontmatter: populate per the archetype's frontmatter schema\n---\n\n",
        );

        // Relationships block (FR-040-AC-11): the resolved (artifact ∪
        // object) edge vocabulary an author may declare in the frontmatter
        // `relationships:` array.
        if !self.relationships.is_empty() {
            let scope = match &self.object {
                Some(o) => format!("{} + object:{}", self.archetype, o),
                None => self.archetype.clone(),
            };
            out.push_str(&format!(
                "<!-- Relationships (frontmatter `relationships:` array, typed `ix://` edges).\n     Allowed for {scope}:\n"
            ));
            for r in &self.relationships {
                let cat = r
                    .category
                    .as_deref()
                    .map(|c| format!(" ({c})"))
                    .unwrap_or_default();
                let desc = r
                    .description
                    .as_deref()
                    .map(|d| format!(" — {d}"))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "       {} → {}{cat}{desc}\n",
                    r.verb,
                    r.targets.join(", ")
                ));
            }
            out.push_str("-->\n\n");
        }

        for s in &self.sections {
            let level = s.level.unwrap_or(2) as usize;
            let hashes = "#".repeat(level.max(1));
            match s.heading.as_deref() {
                Some(heading) => out.push_str(&format!("{hashes} {heading}\n\n")),
                None => {
                    out.push_str(&format!(
                        "<!-- unresolved: '{}' has no concrete heading; see diagnostics -->\n\n",
                        s.key
                    ));
                    continue;
                }
            }
            match s.kind.as_str() {
                "table_row" => {
                    if let Some(cols) = &s.columns {
                        out.push_str(&format!("| {} |\n", cols.join(" | ")));
                        out.push_str(&format!(
                            "| {} |\n",
                            cols.iter().map(|_| "---").collect::<Vec<_>>().join(" | ")
                        ));
                        out.push_str(&format!(
                            "| {} |\n\n",
                            cols.iter()
                                .map(|_| "<!-- fill -->")
                                .collect::<Vec<_>>()
                                .join(" | ")
                        ));
                    } else {
                        out.push_str("<!-- table: fill rows -->\n\n");
                    }
                }
                "list_item" => out.push_str("- <!-- fill -->\n\n"),
                _ => out.push_str("<!-- fill -->\n\n"),
            }
        }
        out
    }
}

/// Build the input contract for `archetype` in `registry` (FR-029).
///
/// Derived from the loaded module's frontmatter schema + `body_extraction`
/// locators/asserts — never from rendered markdown. Returns
/// `UnknownArchetype` when `archetype` is not registered (FR-029-AC-5).
pub fn input_contract_for(
    registry: &Registry,
    archetype: &str,
) -> Result<InputContract, QuireError> {
    input_contract_for_object(registry, archetype, None)
}

/// Build the input contract for `archetype` composed with an optional
/// `object:` archetype (FR-040-AC-11).
///
/// The body sections are the artifact archetype's (the `object:` layer
/// asserts body structure separately at validation time); the
/// Relationships vocabulary is the **resolved union** of the artifact and
/// object `allowed_links`, annotated with each verb's registry category +
/// description. `object = Some(name)` for an unknown name contributes no
/// object vocabulary (artifact axis alone). Returns `UnknownArchetype`
/// only when the *artifact* `archetype` is unregistered (FR-029-AC-5).
pub fn input_contract_for_object(
    registry: &Registry,
    archetype: &str,
    object: Option<&str>,
) -> Result<InputContract, QuireError> {
    let arch = registry
        .archetype(archetype)
        .ok_or_else(|| QuireError::UnknownArchetype {
            name: archetype.to_string(),
        })?;

    let frontmatter_schema = arch
        .frontmatter_schema
        .as_ref()
        .map(|s| s.as_ref().clone())
        .unwrap_or(Value::Null);

    let mut sections: Vec<ContractSection> = Vec::new();
    let mut diagnostics: Vec<String> = Vec::new();
    if let Some(dsl) = arch.body_extraction() {
        build_sections(arch, dsl, &mut sections, &mut diagnostics);
    }

    let object_arch = object.and_then(|o| registry.archetype(o));
    let relationships = build_relationships(registry, arch, object_arch);

    Ok(InputContract {
        archetype: arch.name.clone(),
        object: object.map(str::to_string),
        frontmatter_schema,
        sections,
        relationships,
        diagnostics,
    })
}

/// Build the resolved relationship vocabulary (artifact ∪ object),
/// verb-sorted, annotating each verb with its `edge_types` registry
/// category + description when declared (FR-040-AC-11).
fn build_relationships(
    registry: &Registry,
    artifact: &CompiledArchetype,
    object: Option<&CompiledArchetype>,
) -> Vec<ContractRelationship> {
    let resolved = registry.resolve_allowed_links(artifact, object);
    resolved
        .into_iter()
        .map(|(verb, targets)| {
            let def = registry.edge_types().get(&verb);
            ContractRelationship {
                category: def.map(|d| edge_category_str(d.category).to_string()),
                description: def.map(|d| d.description.clone()),
                verb,
                targets,
            }
        })
        .collect()
}

/// Stable lowercase token for an [`crate::vocab::EdgeCategory`].
fn edge_category_str(category: crate::vocab::EdgeCategory) -> &'static str {
    use crate::vocab::EdgeCategory::*;
    match category {
        Structural => "structural",
        Behavioral => "behavioral",
        Dataflow => "dataflow",
        Dependency => "dependency",
        Realization => "realization",
        Governance => "governance",
        Traceability => "traceability",
    }
}

fn build_sections(
    arch: &CompiledArchetype,
    dsl: &ExtractionDsl,
    sections: &mut Vec<ContractSection>,
    diagnostics: &mut Vec<String>,
) {
    let Some(map) = &dsl.yield_pattern.r#match else {
        return;
    };
    for (key, locator) in map {
        let primitive = locator.canonical();
        let section = contract_section(&arch.name, key, primitive, locator.required(), diagnostics);
        sections.push(section);
    }
}

fn contract_section(
    archetype: &str,
    key: &str,
    primitive: &LocatorPrimitive,
    required: bool,
    diagnostics: &mut Vec<String>,
) -> ContractSection {
    let heading = section_name(primitive);
    if heading.is_none() && required {
        // FR-029-AC-6: still include the section; flag the unresolved
        // mapping with an actionable diagnostic.
        diagnostics.push(format!(
            "[{archetype}] required '{key}' ({}) does not address a concrete section heading; \
             the authoring skeleton cannot scaffold it",
            primitive.describe()
        ));
    }
    let assert = primitive.assert();
    ContractSection {
        key: key.to_string(),
        heading,
        level: assert.and_then(|a| a.level),
        kind: primitive.kind().as_str().to_string(),
        required,
        columns: assert.and_then(|a: &LocatorAssert| a.columns.clone()),
        min_rows: assert.and_then(|a| a.min_rows),
        min_items: assert.and_then(|a| a.min_items),
        id_pattern: assert.and_then(|a| a.id_pattern.clone()),
        matches: assert.and_then(|a| a.matches.clone()),
    }
}

/// The concrete section heading a locator addresses by name, if any.
fn section_name(primitive: &LocatorPrimitive) -> Option<String> {
    match primitive {
        LocatorPrimitive::SectionBody { after_heading, .. } => Some(after_heading.clone()),
        LocatorPrimitive::TableRow {
            under_section: Some(s),
            ..
        }
        | LocatorPrimitive::ListItem {
            under_section: Some(s),
            ..
        }
        | LocatorPrimitive::CodeBlock {
            under_section: Some(s),
            ..
        } => Some(s.clone()),
        LocatorPrimitive::Heading { path: Some(p), .. } => p.last().cloned(),
        _ => None,
    }
}

/// The locator kinds that address a body section by name (used by the
/// skeleton emitter to decide scaffolding).
pub fn addresses_section(kind: LocatorKind) -> bool {
    matches!(
        kind,
        LocatorKind::SectionBody
            | LocatorKind::TableRow
            | LocatorKind::ListItem
            | LocatorKind::CodeBlock
            | LocatorKind::Heading
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    fn iso_module() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/modules/iso")
    }

    fn iso_registry() -> Registry {
        Registry::load_module(&iso_module()).expect("load iso")
    }

    // TC-548 (FR-029-AC-1): FR contract has frontmatter schema + the four
    // required sections (derived from body_extraction).
    #[test]
    fn tc548_fr_contract() {
        let r = iso_registry();
        let c = input_contract_for(&r, "FR").expect("FR contract");
        assert_eq!(c.archetype, "FR");
        assert_eq!(c.frontmatter_schema["type"], "object");
        let headings: Vec<&str> = c
            .sections
            .iter()
            .filter_map(|s| s.heading.as_deref())
            .collect();
        for required in [
            "Description",
            "Specification",
            "Acceptance Criteria",
            "Dependencies",
        ] {
            assert!(
                headings.contains(&required),
                "missing {required}: {headings:?}"
            );
        }
    }

    // TC-549 (FR-029-AC-2): NFR contract feeds Scope, Measurement and
    // Evaluation, Verification.
    #[test]
    fn tc549_nfr_contract() {
        let r = iso_registry();
        let c = input_contract_for(&r, "NFR").expect("NFR contract");
        let headings: Vec<&str> = c
            .sections
            .iter()
            .filter_map(|s| s.heading.as_deref())
            .collect();
        for required in ["Scope", "Measurement and Evaluation", "Verification"] {
            assert!(
                headings.contains(&required),
                "missing {required}: {headings:?}"
            );
        }
    }

    // TC-550 (FR-029-AC-3): every iso archetype's contract contains each
    // body_extraction section exactly once, in manifest (match) order.
    #[test]
    fn tc550_every_archetype_sections_in_manifest_order() {
        let r = iso_registry();
        for name in ["FR", "NFR", "StR", "US", "IT", "TC", "AC", "CON"] {
            let c = input_contract_for(&r, name).expect("contract");
            // Compare contract heading order to the manifest match order.
            let arch = r.archetype(name).expect("arch");
            let dsl = arch.body_extraction().expect("body_extraction");
            let expected: Vec<String> = dsl
                .yield_pattern
                .r#match
                .as_ref()
                .expect("match")
                .iter()
                .map(|(k, _)| k.clone())
                .collect();
            let got: Vec<String> = c.sections.iter().map(|s| s.key.clone()).collect();
            assert_eq!(got, expected, "section order mismatch for {name}");
            // Each key appears exactly once.
            let mut sorted = got.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), got.len(), "duplicate section in {name}");
        }
    }

    // TC-551 (FR-029-AC-4): JSON serialization is byte-identical across
    // repeated calls.
    #[test]
    fn tc551_contract_json_is_byte_stable() {
        let r = iso_registry();
        let a = input_contract_for(&r, "FR").unwrap();
        let b = input_contract_for(&r, "FR").unwrap();
        let ja = serde_json::to_string(&a.to_json()).unwrap();
        let jb = serde_json::to_string(&b.to_json()).unwrap();
        assert_eq!(ja, jb);
        // And stable across an independent reload of the same module.
        let r2 = iso_registry();
        let c = input_contract_for(&r2, "FR").unwrap();
        assert_eq!(ja, serde_json::to_string(&c.to_json()).unwrap());
    }

    // TC-552 (FR-029-AC-5): unknown archetype → UnknownArchetype.
    #[test]
    fn tc552_unknown_archetype_errors() {
        let r = iso_registry();
        let err = input_contract_for(&r, "nonexistent").expect_err("unknown");
        assert!(matches!(err, QuireError::UnknownArchetype { .. }));
    }

    // TC-553 (FR-029-AC-6): a required locator that cannot map to a
    // concrete section still appears in the contract, with an
    // unresolved-mapping diagnostic.
    #[test]
    fn tc553_unresolved_mapping_still_included() {
        let parent =
            std::env::temp_dir().join(format!("quire-rs-contract-{}-unmapped", std::process::id()));
        let root = parent.join("mod");
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(&root).unwrap();
        // A `heading` locator selecting by level only addresses no
        // concrete section heading.
        fs::write(
            root.join("manifest.yaml"),
            r#"
name: mod
object_types:
- name: thing
  body_extraction:
    yield_pattern:
      match:
        any_section:
          from: heading
          level: 2
          required: true
"#,
        )
        .unwrap();
        let r = Registry::load_from(&[&parent]).expect("load");
        let c = input_contract_for(&r, "thing").expect("contract");
        // The section is present (not omitted)…
        assert!(c.sections.iter().any(|s| s.key == "any_section"));
        let s = c.sections.iter().find(|s| s.key == "any_section").unwrap();
        assert!(s.heading.is_none(), "no concrete heading");
        // …and an unresolved-mapping diagnostic names the archetype +
        // section.
        assert!(
            c.diagnostics
                .iter()
                .any(|d| d.contains("thing") && d.contains("any_section")),
            "diagnostics: {:?}",
            c.diagnostics
        );
        let _ = fs::remove_dir_all(&parent);
    }

    /// Inline registry with FR + aggregate_root allowed_links + edge_types
    /// for the Relationships-block test (FR-040-AC-11).
    fn rel_registry() -> Registry {
        let manifest = br#"
name: rel-test
artifact_types:
- name: FR
  frontmatter_schema_ref: schemas/fr.schema.json
  allowed_links: [references]
object_types:
- name: aggregate_root
  allowed_links:
    emits: [event]
    aggregates: [entity]
edge_types:
  references: { description: loose linkage, category: traceability }
  emits: { description: emits a domain event, category: dataflow }
  aggregates: { description: owns member entities, category: structural }
"#;
        let mut schemas = std::collections::BTreeMap::new();
        schemas.insert(
            "schemas/fr.schema.json".to_string(),
            r#"{"type":"object"}"#.to_string(),
        );
        Registry::from_inline_parts(manifest, &schemas).expect("inline registry")
    }

    // TC-645 (FR-040-AC-11): input_contract_for_object with an object
    // renders a Relationships block listing each resolved verb with its
    // category + description + targets; without object, only the artifact
    // vocabulary is listed.
    #[test]
    fn tc645_skeleton_renders_relationships_block() {
        let r = rel_registry();
        // With object: union of FR (references) + aggregate_root (emits,
        // aggregates).
        let composed = input_contract_for_object(&r, "FR", Some("aggregate_root")).unwrap();
        let verbs: Vec<&str> = composed
            .relationships
            .iter()
            .map(|x| x.verb.as_str())
            .collect();
        assert_eq!(
            verbs,
            vec!["aggregates", "emits", "references"],
            "verb-sorted union"
        );
        let sk = composed.skeleton();
        assert!(sk.contains("FR + object:aggregate_root"));
        assert!(
            sk.contains("emits → event (dataflow) — emits a domain event"),
            "{sk}"
        );
        assert!(sk.contains("aggregates → entity (structural)"), "{sk}");

        // Without object: only the artifact axis (references).
        let artifact_only = input_contract_for(&r, "FR").unwrap();
        let verbs2: Vec<&str> = artifact_only
            .relationships
            .iter()
            .map(|x| x.verb.as_str())
            .collect();
        assert_eq!(verbs2, vec!["references"]);
        assert!(artifact_only.object.is_none());
        let sk2 = artifact_only.skeleton();
        assert!(
            !sk2.contains("emits"),
            "no object verbs without object: {sk2}"
        );
    }

    // The skeleton emitter scaffolds headings + literal table headers,
    // is deterministic, and never emits friendly defaults like TODO.
    #[test]
    fn skeleton_scaffolds_headings_and_tables() {
        let r = iso_registry();
        let c = input_contract_for(&r, "FR").unwrap();
        let s1 = c.skeleton();
        let s2 = c.skeleton();
        assert_eq!(s1, s2, "skeleton must be deterministic");
        assert!(s1.contains("## Description"));
        assert!(s1.contains("## Specification"));
        assert!(!s1.contains("TODO"), "skeleton must not seed TODO defaults");
    }
}
