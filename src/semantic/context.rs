//! Extraction context: the module block, the document's identity, and the
//! `BundleIndex` every surface supplies explicitly (FR-070 Inputs).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::contract::SemanticModule;

/// One resolvable declaration in the bundle: its artifact `id` and every
/// name a `Type` cell may use for it (id, title, frontmatter `name`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BundleEntry {
    pub id: String,
    #[serde(default)]
    pub names: Vec<String>,
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
}

impl BundleIndex {
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty() && self.enumerations.is_empty() && self.imports.is_empty()
    }

    /// Build the corpus-mode index from loaded documents (FR-070 Inputs):
    /// every document with a frontmatter `object` is an object whose names
    /// are its `id`, `title`, and `name` when present; documents whose
    /// `object` is `enumeration` are also enumerations. `imports` come from
    /// the loaded modules' `exports`, keyed by package.
    pub fn from_documents<'a>(
        package: &str,
        documents: impl Iterator<Item = &'a serde_json::Map<String, serde_json::Value>>,
        modules: impl Iterator<Item = &'a SemanticModule>,
    ) -> Self {
        let mut index = Self {
            package: package.to_string(),
            ..Self::default()
        };
        for fm in documents {
            let Some(object) = fm.get("object").and_then(|v| v.as_str()) else {
                continue;
            };
            let Some(id) = fm.get("id").and_then(|v| v.as_str()) else {
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
    pub bundle: BundleIndex,
}

impl SemanticContext {
    pub fn new(module: SemanticModule, path: impl Into<String>, bundle: BundleIndex) -> Self {
        Self {
            module,
            path: path.into(),
            source_identity: None,
            bundle,
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
