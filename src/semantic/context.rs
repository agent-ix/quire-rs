//! Extraction context: the module block, the document's identity, and the
//! `BundleIndex` every surface supplies explicitly (FR-070 Inputs).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::contract::SemanticModule;
use super::model::DeclaredTables;
use super::relations::RelationVocabulary;
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
/// from the same `extract_operations` output the artifact's own extraction
/// uses (`BundleIndex::from_documents` uses the shared
/// `clauses::declared_operation_names` scan for the same reason). No
/// `Default` derive: `..Default::default()` would silently skip
/// `operations` the same way `#[serde(default)]` would.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleArtifact {
    pub id: String,
    #[serde(default)]
    pub object: Option<String>,
    pub operations: Vec<String>,
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
}

impl BundleIndex {
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty() && self.enumerations.is_empty() && self.imports.is_empty()
    }

    /// Build the corpus-mode index from loaded documents (FR-070 Inputs):
    /// every document with an `id` is an artifact with its `object` type
    /// and the operation names its `raw` text declares under
    /// `## Operations` (FR-075 Inputs), via the same scan
    /// `extract_operations` runs (`clauses::declared_operation_names`);
    /// every document with a frontmatter `object` is an object whose names
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
            let entry = BundleEntry {
                id: id.to_string(),
                names,
            };
            if object == "enumeration" {
                index.enumerations.push(entry.clone());
            }
            index.objects.push(entry);
        }
        for module in modules {
            index
                .imports
                .insert(module.package.clone(), module.exports.clone());
        }
        index
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
    /// The edge registry and object-type facts relationship rows are
    /// checked against (FR-076 Inputs); `None` when the surface supplies
    /// none, which extracts under `no-relation-vocabulary`.
    pub relation_vocabulary: Option<RelationVocabulary>,
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
            relation_vocabulary: None,
        }
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
