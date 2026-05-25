//! Load-time + extract-time non-fatal diagnostics.
//!
//! Consumers inspect load warnings via the `&[Diagnostic]` slice
//! exposed on [`crate::registry::Registry::diagnostics`] and
//! [`crate::extract::ExtractionResult::diagnostics`].

use std::path::PathBuf;

/// One advisory note from a loader or extract call.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Diagnostic {
    DuplicateModuleName {
        name: String,
        paths: Vec<PathBuf>,
    },
    DuplicateArchetype {
        name: String,
        modules: Vec<String>,
    },
    ManifestMissingName {
        path: PathBuf,
        derived_name: String,
    },
    SearchPathNotADirectory {
        path: PathBuf,
    },
    SearchPathMissing {
        path: PathBuf,
    },
    SearchPathUnreadable {
        path: PathBuf,
        reason: String,
    },
    SymlinkLoop {
        path: PathBuf,
    },
    IterateRootMissing {
        path: Vec<String>,
    },
    FallbackLocatorUsed {
        key: String,
        position: usize,
        locator: String,
    },
    /// A document loaded by `load_repo` has no frontmatter `uuid`
    /// (CR-002). Non-fatal — the document still loads; its durable
    /// catalog id is simply absent until quire authors one.
    MissingUuid {
        path: PathBuf,
    },
    /// A file discovered by `load_repo` could not be read (missing,
    /// permission denied, or not valid UTF-8). Non-fatal: the file is
    /// skipped and the rest of the repo loads (FR-024-AC-2).
    DocumentUnreadable {
        path: PathBuf,
        reason: String,
    },
    /// Two or more loaded documents share the same artifact key
    /// (FR-025-AC-3). First occurrence wins for lookup; the duplicate
    /// is recorded here. Construction does not fail.
    DuplicateArtifactId {
        id: String,
        paths: Vec<PathBuf>,
    },
    /// A reference (frontmatter `relationships` entry or `ix://` body
    /// link) whose target id is absent from the loaded set (FR-026-AC-3).
    /// Non-fatal — resolution never reaches outside the corpus.
    DanglingReference {
        source: String,
        target: String,
        edge_type: String,
    },
    /// A loaded document has no frontmatter `type`/`artifact_type`
    /// (FR-027-AC-9). It is never returned by `by_type` and is reachable
    /// only via `by_id`. Non-fatal.
    UntypedArtifact {
        id: String,
        path: PathBuf,
    },
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateModuleName { name, paths } => write!(
                f,
                "DuplicateModuleName: '{}' declared at {} path(s); first-wins",
                name,
                paths.len()
            ),
            Self::DuplicateArchetype { name, modules } => write!(
                f,
                "DuplicateArchetype: '{}' contributed by modules {:?}; first-wins",
                name, modules
            ),
            Self::ManifestMissingName { path, derived_name } => write!(
                f,
                "ManifestMissingName at {}: using parent-dir name '{}'",
                path.display(),
                derived_name
            ),
            Self::SearchPathNotADirectory { path } => {
                write!(f, "SearchPathNotADirectory: {} is a file", path.display())
            }
            Self::SearchPathMissing { path } => {
                write!(f, "SearchPathMissing: {}", path.display())
            }
            Self::SearchPathUnreadable { path, reason } => {
                write!(f, "SearchPathUnreadable {}: {}", path.display(), reason)
            }
            Self::SymlinkLoop { path } => write!(f, "SymlinkLoop broken at {}", path.display()),
            Self::IterateRootMissing { path } => {
                write!(f, "IterateRootMissing: section_path {:?} not found", path)
            }
            Self::FallbackLocatorUsed {
                key,
                position,
                locator,
            } => write!(
                f,
                "FallbackLocatorUsed: key '{key}' resolved via fallback position {position} ({locator})"
            ),
            Self::MissingUuid { path } => {
                write!(f, "MissingUuid: {} has no frontmatter uuid", path.display())
            }
            Self::DocumentUnreadable { path, reason } => {
                write!(f, "DocumentUnreadable {}: {}", path.display(), reason)
            }
            Self::DuplicateArtifactId { id, paths } => write!(
                f,
                "DuplicateArtifactId: '{}' at {} path(s); first-wins",
                id,
                paths.len()
            ),
            Self::DanglingReference {
                source,
                target,
                edge_type,
            } => write!(
                f,
                "DanglingReference: {source} --{edge_type}--> {target} (target not in loaded set)"
            ),
            Self::UntypedArtifact { id, path } => write!(
                f,
                "UntypedArtifact: '{}' ({}) has no type/artifact_type field",
                id,
                path.display()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_strings_carry_variant_name_and_identifier() {
        let cases: Vec<(Diagnostic, &[&str])> = vec![
            (
                Diagnostic::DuplicateModuleName {
                    name: "iso".into(),
                    paths: vec![PathBuf::from("/a"), PathBuf::from("/b")],
                },
                &["DuplicateModuleName", "iso"],
            ),
            (
                Diagnostic::DuplicateArchetype {
                    name: "fr".into(),
                    modules: vec!["mod-a".into(), "mod-b".into()],
                },
                &["DuplicateArchetype", "fr", "mod-a", "mod-b"],
            ),
            (
                Diagnostic::ManifestMissingName {
                    path: PathBuf::from("/x/manifest.yaml"),
                    derived_name: "x".into(),
                },
                &["ManifestMissingName", "/x/manifest.yaml", "x"],
            ),
            (
                Diagnostic::SearchPathNotADirectory {
                    path: PathBuf::from("/etc/passwd"),
                },
                &["SearchPathNotADirectory", "/etc/passwd"],
            ),
            (
                Diagnostic::SearchPathMissing {
                    path: PathBuf::from("/nope"),
                },
                &["SearchPathMissing", "/nope"],
            ),
            (
                Diagnostic::SearchPathUnreadable {
                    path: PathBuf::from("/locked"),
                    reason: "permission denied".into(),
                },
                &["SearchPathUnreadable", "/locked", "permission denied"],
            ),
            (
                Diagnostic::SymlinkLoop {
                    path: PathBuf::from("/loopy"),
                },
                &["SymlinkLoop", "/loopy"],
            ),
            (
                Diagnostic::IterateRootMissing {
                    path: vec!["Algorithms".into(), "Sort".into()],
                },
                &["IterateRootMissing", "Algorithms", "Sort"],
            ),
            (
                Diagnostic::FallbackLocatorUsed {
                    key: "k".into(),
                    position: 2,
                    locator: "heading".into(),
                },
                &["FallbackLocatorUsed", "k", "2", "heading"],
            ),
            (
                Diagnostic::MissingUuid {
                    path: PathBuf::from("/spec/functional/FR-099.md"),
                },
                &["MissingUuid", "/spec/functional/FR-099.md"],
            ),
            (
                Diagnostic::DocumentUnreadable {
                    path: PathBuf::from("/spec/bad.md"),
                    reason: "stream did not contain valid UTF-8".into(),
                },
                &["DocumentUnreadable", "/spec/bad.md", "valid UTF-8"],
            ),
            (
                Diagnostic::DuplicateArtifactId {
                    id: "FR-023".into(),
                    paths: vec![PathBuf::from("/a/FR-023.md"), PathBuf::from("/b/FR-023.md")],
                },
                &["DuplicateArtifactId", "FR-023"],
            ),
            (
                Diagnostic::DanglingReference {
                    source: "FR-023".into(),
                    target: "StR-099".into(),
                    edge_type: "implements".into(),
                },
                &["DanglingReference", "FR-023", "StR-099", "implements"],
            ),
            (
                Diagnostic::UntypedArtifact {
                    id: "X-1".into(),
                    path: PathBuf::from("/spec/x.md"),
                },
                &["UntypedArtifact", "X-1", "/spec/x.md"],
            ),
        ];
        for (d, needles) in cases {
            let s = d.to_string();
            for needle in needles {
                assert!(s.contains(needle), "{d:?} Display missing '{needle}': {s}");
            }
        }
    }

    #[test]
    fn diagnostic_satisfies_trait_bounds() {
        fn assert_full<T: Send + Sync + std::fmt::Debug + Clone + Eq>() {}
        assert_full::<Diagnostic>();
    }
}
