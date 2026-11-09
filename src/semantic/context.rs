//! Extraction context: the module block, the document's identity, and the
//! `BundleIndex` every surface supplies explicitly (FR-070 Inputs).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::contract::SemanticModule;
use super::model::{DeclaredSections, DeclaredTables};
use super::record::RecordSchema;
use super::relations::{RelationSchema, RelationVocabulary};
use crate::extract::dsl::ExtractionDsl;

/// One resolvable declaration in the bundle: its artifact `id` and every
/// name a `Type` cell may use for it (id, title, frontmatter `name`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BundleEntry {
    pub id: String,
    #[serde(default)]
    pub names: Vec<String>,
}

/// One artifact of the bundle with its frontmatter `object` type, if any,
/// and the operation names it declares under `## Operations`: what a
/// relationship `Target` (FR-076 Inputs) and an allocation `<id>/<member>`
/// source (FR-075 Inputs) resolve against.
///
/// `operations` is required, not `#[serde(default)]`: a caller that omits it
/// fails to deserialize rather than silently supplying "no operations" for
/// every artifact, which would make every allocation operation source
/// refuse. Every quire-rs-owned construction site must supply it explicitly
/// from `clauses::declared_operation_names` — the `### <name>` headings under
/// `## Operations`, **not** `extract_operations`'s own filtered output
/// (`OperationsOutcome.operations`, `None` for the whole document once any
/// one operation's body has an error). `BundleIndex::from_documents` (corpus
/// mode) and the self-reference path in `model.rs` both use
/// `declared_operation_names` for the same reason: a heading elsewhere in
/// the section having a malformed body must not make a well-formed
/// operation's name disappear from this list. No `Default` derive:
/// `..Default::default()` would silently skip `operations` the same way
/// `#[serde(default)]` would.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleArtifact {
    pub id: String,
    #[serde(default)]
    pub object: Option<String>,
    pub operations: Vec<String>,
}

/// One artifact of a package a module imports and a loaded corpus provides:
/// the owning `package` (`<org>/<repo>`), the artifact `id`, every name a
/// unit or dimension token may use for it, and its frontmatter `object` type
/// (FR-070 Inputs).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportedArtifact {
    pub package: String,
    pub id: String,
    #[serde(default)]
    pub names: Vec<String>,
    #[serde(default)]
    pub object: Option<String>,
}

/// The bundle-wide name index type resolution reads (FR-070). An empty
/// index is an explicit state (`no-bundle-index`), never a default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BundleIndex {
    /// `<org>/<repo>` of the bundle; resolved identities are minted under it.
    #[serde(default)]
    pub package: String,
    #[serde(default)]
    pub objects: Vec<BundleEntry>,
    #[serde(default)]
    pub enumerations: Vec<BundleEntry>,
    /// Imported package → exported type names, from the loaded modules.
    #[serde(default)]
    pub imports: BTreeMap<String, Vec<String>>,
    /// Every artifact `id` of the bundle, typed or not (FR-076).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<BundleArtifact>,
    /// The artifacts of every imported package a loaded corpus provides;
    /// absent when the corpus loads none.
    #[serde(
        default,
        rename = "importedArtifacts",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub imported_artifacts: Vec<ImportedArtifact>,
}

impl BundleIndex {
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
            && self.enumerations.is_empty()
            && self.imports.is_empty()
            && self.imported_artifacts.is_empty()
    }

    /// Build the corpus-mode index from loaded documents (FR-070 Inputs):
    /// every document with an `id` is an artifact with its `object` type
    /// and the operation names its `raw` text declares under
    /// `## Operations` (FR-075 Inputs), via `clauses::declared_operation_names`
    /// — every `### <name>` heading, regardless of whether that operation's
    /// own body has an error; every document with a frontmatter `object` is
    /// an object whose names
    /// are its `id`, `title`, and `name` when present; documents whose
    /// `object` is `enumeration` are also enumerations. `imports` come from
    /// the loaded modules' `exports`, keyed by package.
    pub fn from_documents<'a>(
        package: &str,
        documents: impl Iterator<Item = (&'a serde_json::Map<String, serde_json::Value>, &'a str)>,
        modules: impl Iterator<Item = &'a SemanticModule>,
    ) -> Self {
        let mut index = Self {
            package: package.to_string(),
            ..Self::default()
        };
        for (fm, raw) in documents {
            let Some(id) = fm.get("id").and_then(|v| v.as_str()) else {
                continue;
            };
            let object = fm.get("object").and_then(|v| v.as_str());
            index.artifacts.push(BundleArtifact {
                id: id.to_string(),
                object: object.map(str::to_string),
                operations: super::clauses::declared_operation_names(raw),
            });
            let Some(object) = object else {
                continue;
            };
            let mut names = vec![id.to_string()];
            for key in ["title", "name"] {
                if let Some(v) = fm.get(key).and_then(|v| v.as_str()) {
                    if !names.iter().any(|n| n == v) {
                        names.push(v.to_string());
                    }
                }
            }
            // The same declaration authored in two forms (an artifact and its
            // alternate-form counterpart) shares one `id`: one entry, with the
            // union of the names.
            if object == "enumeration" {
                merge_entry(&mut index.enumerations, id, names.clone());
            }
            merge_entry(&mut index.objects, id, names);
        }
        for module in modules {
            index
                .imports
                .insert(module.package.clone(), module.exports.clone());
        }
        index
    }
}

/// Add `names` to the entry for `id`, creating it when absent.
fn merge_entry(entries: &mut Vec<BundleEntry>, id: &str, names: Vec<String>) {
    match entries.iter_mut().find(|e| e.id == id) {
        Some(entry) => {
            for name in names {
                if !entry.names.contains(&name) {
                    entry.names.push(name);
                }
            }
        }
        None => entries.push(BundleEntry {
            id: id.to_string(),
            names,
        }),
    }
}

/// The object type's construct `references` (FR-075 Inputs): each model
/// member (`transitions`, `steps`, ...) mapped to the role tokens a
/// declaration it names must carry.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConstructReferences(BTreeMap<String, Vec<String>>);

impl ConstructReferences {
    pub fn new(references: BTreeMap<String, Vec<String>>) -> Self {
        Self(references)
    }

    /// The `references` of a raw `construct` declaration. `None` when the
    /// construct declares no `references`, or declares one that is not a map
    /// of member to role list: its shape is owned by filament-core-data, and
    /// a construct that does not state one admits through the next tier.
    pub fn from_construct(construct: &serde_json::Value) -> Option<Self> {
        serde_json::from_value(construct.get("references")?.clone()).ok()
    }

    /// The role tokens the construct declares for `member`.
    pub(crate) fn roles(&self, member: &str) -> Option<&[String]> {
        self.0.get(member).map(Vec::as_slice)
    }
}

/// Everything extraction needs besides the document (FR-070/FR-071 Inputs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticContext {
    pub module: SemanticModule,
    /// Corpus-relative path of the document; the `path` of every span.
    pub path: String,
    /// `ix://<org>/<repo>/spec` of the document's repository; `None` lets
    /// FR-071 default it with an advisory.
    pub source_identity: Option<String>,
    /// Scope directory name, used for the defaulted identity
    /// `ix://local/<scope>/spec` when `source_identity` is absent.
    pub scope: Option<String>,
    pub bundle: BundleIndex,
    /// The object type's `table_row` locators (FR-075 table gating).
    pub(crate) declared_tables: DeclaredTables,
    /// The object type's raw `section_body` locators.
    pub(crate) declared_sections: DeclaredSections,
    /// The edge registry and object-type facts relationship rows are
    /// checked against (FR-076 Inputs); `None` when the surface supplies
    /// none, which extracts under `no-relation-vocabulary`.
    pub relation_vocabulary: Option<RelationVocabulary>,
    /// The object type's construct `references`, read by the FR-075
    /// declaration-reference admission; `None` skips the construct tier.
    pub construct_references: Option<ConstructReferences>,
    /// What the object type's resolved `data_schema` declares that FR-076
    /// typed relation keys read; `None` reads every row into `relations`.
    pub(crate) relation_schema: Option<RelationSchema>,
    /// The record schema FR-081 reads record-key sections against; `None`
    /// reads no record key.
    pub(crate) record_schema: Option<RecordSchema>,
    /// The extracting object type's name; scopes the keys quoin FR-123 lists
    /// as unmapped. Falls back to the relation vocabulary's.
    pub object_type: Option<String>,
}

impl SemanticContext {
    pub fn new(module: SemanticModule, path: impl Into<String>, bundle: BundleIndex) -> Self {
        Self {
            module,
            path: path.into(),
            source_identity: None,
            scope: None,
            bundle,
            declared_tables: DeclaredTables::default(),
            declared_sections: DeclaredSections::default(),
            relation_vocabulary: None,
            construct_references: None,
            relation_schema: None,
            record_schema: None,
            object_type: None,
        }
    }

    /// Admit FR-075 declaration references through the object type's
    /// construct `references`.
    pub fn with_construct_references(mut self, references: ConstructReferences) -> Self {
        self.construct_references = Some(references);
        self
    }

    /// Lower FR-076 rows into the typed relation keys `data_schema` declares.
    pub fn with_data_schema(mut self, data_schema: &serde_json::Value) -> Self {
        self.relation_schema = Some(RelationSchema::from_data_schema(data_schema));
        self
    }

    /// Read record-key sections (FR-081) against the object type's resolved
    /// `data_schema` `root`, following its `$ref`s through `documents` and the
    /// embedded semantic-core bundle.
    pub fn with_record_schema(
        mut self,
        root: &serde_json::Value,
        documents: &BTreeMap<String, serde_json::Value>,
    ) -> Self {
        self.record_schema = Some(RecordSchema::new(root, documents));
        self
    }

    /// Name the extracting object type.
    pub fn with_object_type(mut self, object_type: impl Into<String>) -> Self {
        self.object_type = Some(object_type.into());
        self
    }

    /// The extracting object type's name, when a surface supplies it.
    pub(crate) fn object_type_name(&self) -> Option<&str> {
        self.object_type.as_deref().or_else(|| {
            self.relation_vocabulary
                .as_ref()
                .and_then(|v| v.object_type())
        })
    }

    /// Check relationship rows against `vocabulary` (FR-076).
    pub fn with_relation_vocabulary(mut self, vocabulary: RelationVocabulary) -> Self {
        self.relation_vocabulary = Some(vocabulary);
        self
    }

    /// Gate FR-075 model tables on the object type's typed `body_extraction`:
    /// each `table_row` locator it declares admits one model table
    /// (FR-075 Inputs). Without it no model table is extracted.
    pub fn with_body_extraction(mut self, dsl: &ExtractionDsl) -> Self {
        self.declared_tables = DeclaredTables::from_dsl(dsl);
        self.declared_sections = DeclaredSections::from_dsl(dsl);
        self
    }

    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// The span identity: the caller's, or the documented default.
    pub fn resolved_source_identity(&self) -> (String, bool) {
        match &self.source_identity {
            Some(id) => (id.clone(), false),
            None => (
                format!(
                    "ix://local/{}/spec",
                    self.scope.as_deref().unwrap_or("scope")
                ),
                true,
            ),
        }
    }

    pub fn with_source_identity(mut self, identity: impl Into<String>) -> Self {
        self.source_identity = Some(identity.into());
        self
    }

    /// `<org>/<repo>` under which resolved and placeholder identities are
    /// minted: the bundle's package when set, else the module's.
    pub fn identity_package(&self) -> &str {
        if self.bundle.package.is_empty() {
            &self.module.package
        } else {
            &self.bundle.package
        }
    }
}
