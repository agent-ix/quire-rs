//! Load-time + extract-time non-fatal diagnostics.
//!
//! v0.2 scope: data shape only. The v0.1 `Diagnostics` collector +
//! `by_kind` filter formalism (FR-017) was stripped during the
//! spec-refinement audit because it had no INPUT.md basis. Consumers
//! that need to inspect load warnings get a `&[Diagnostic]` slice
//! directly off `Registry::diagnostics()` and `ExtractionResult.diagnostics`.

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
