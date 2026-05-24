//! Non-fatal load + render diagnostics.
//!
//! Diagnostics are advisory: the operation that emits them completes,
//! and the caller decides whether any specific diagnostic should be
//! treated as fatal (e.g. via `Registry::load_strict`). The variant set
//! is `#[non_exhaustive]` so new advisory categories can be added
//! without breaking match arms downstream.
//!
//! Task 022 will expand this with render-side variants (template
//! warnings, deprecation notes, etc.). Task 006 introduces the load-
//! side variants: duplicate module name, duplicate archetype, missing
//! manifest `name`, and the path-resolution diagnostics produced by
//! `loader::paths`.

use std::path::PathBuf;

/// One advisory note produced by a loader / render call.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Diagnostic {
    /// Two manifests at different paths declared the same `name`. The
    /// first-loaded module wins; the others are shadowed but still
    /// loaded for inspection via `Registry::archetype_in_module`.
    DuplicateModuleName { name: String, paths: Vec<PathBuf> },
    /// Two modules contributed an archetype with the same `name`. The
    /// first-loaded module wins. (FR-014-AC-2.)
    DuplicateArchetype { name: String, modules: Vec<String> },
    /// A `manifest.yaml` did not declare a `name`; the loader fell back
    /// to the parent directory's basename. (FR-014-AC-7.)
    ManifestMissingName { path: PathBuf, derived_name: String },
    /// A search-path entry exists but is a file, not a directory.
    SearchPathNotADirectory { path: PathBuf },
    /// A search-path entry doesn't exist on disk. (Empty modules are
    /// valid — this is informational.)
    SearchPathMissing { path: PathBuf },
    /// A search-path entry could not be canonicalized (permission
    /// denied, broken symlink).
    SearchPathUnreadable { path: PathBuf, reason: String },
    /// A symlink cycle was detected and broken during the walk.
    SymlinkLoop { path: PathBuf },
}

/// Tag for the family of issue a [`Diagnostic`] reports — useful for
/// log routing and aggregation without `match`ing every variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    DuplicateModuleName,
    DuplicateArchetype,
    ManifestMissingName,
    SearchPath,
    SymlinkLoop,
}

impl Diagnostic {
    /// Coarse classification of this diagnostic.
    pub fn kind(&self) -> DiagnosticKind {
        match self {
            Self::DuplicateModuleName { .. } => DiagnosticKind::DuplicateModuleName,
            Self::DuplicateArchetype { .. } => DiagnosticKind::DuplicateArchetype,
            Self::ManifestMissingName { .. } => DiagnosticKind::ManifestMissingName,
            Self::SearchPathNotADirectory { .. }
            | Self::SearchPathMissing { .. }
            | Self::SearchPathUnreadable { .. } => DiagnosticKind::SearchPath,
            Self::SymlinkLoop { .. } => DiagnosticKind::SymlinkLoop,
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateModuleName { name, paths } => {
                write!(
                    f,
                    "DuplicateModuleName: '{}' declared at {} path(s); first-wins",
                    name,
                    paths.len()
                )
            }
            Self::DuplicateArchetype { name, modules } => {
                write!(
                    f,
                    "DuplicateArchetype: '{}' contributed by modules {:?}; first-wins",
                    name, modules
                )
            }
            Self::ManifestMissingName { path, derived_name } => {
                write!(
                    f,
                    "ManifestMissingName at {}: using parent-dir name '{}'",
                    path.display(),
                    derived_name
                )
            }
            Self::SearchPathNotADirectory { path } => {
                write!(f, "SearchPathNotADirectory: {} is a file", path.display())
            }
            Self::SearchPathMissing { path } => {
                write!(f, "SearchPathMissing: {}", path.display())
            }
            Self::SearchPathUnreadable { path, reason } => {
                write!(f, "SearchPathUnreadable {}: {}", path.display(), reason)
            }
            Self::SymlinkLoop { path } => {
                write!(f, "SymlinkLoop broken at {}", path.display())
            }
        }
    }
}

/// Append-only collector of diagnostics. Avoids the borrow-checker
/// dance of passing `&mut Vec<Diagnostic>` through many call sites.
#[derive(Debug, Default)]
pub struct Diagnostics {
    items: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, d: Diagnostic) {
        self.items.push(d);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.items.iter()
    }

    pub fn into_vec(self) -> Vec<Diagnostic> {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl Extend<Diagnostic> for Diagnostics {
    fn extend<I: IntoIterator<Item = Diagnostic>>(&mut self, iter: I) {
        self.items.extend(iter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_strings_are_non_empty_and_named() {
        let cases = vec![
            Diagnostic::DuplicateModuleName {
                name: "iso".into(),
                paths: vec![PathBuf::from("/a"), PathBuf::from("/b")],
            },
            Diagnostic::DuplicateArchetype {
                name: "fr".into(),
                modules: vec!["a".into(), "b".into()],
            },
            Diagnostic::ManifestMissingName {
                path: PathBuf::from("/x/manifest.yaml"),
                derived_name: "x".into(),
            },
            Diagnostic::SearchPathNotADirectory {
                path: PathBuf::from("/etc/passwd"),
            },
            Diagnostic::SearchPathMissing {
                path: PathBuf::from("/nope"),
            },
            Diagnostic::SearchPathUnreadable {
                path: PathBuf::from("/locked"),
                reason: "permission denied".into(),
            },
            Diagnostic::SymlinkLoop {
                path: PathBuf::from("/loopy"),
            },
        ];
        for d in cases {
            let s = d.to_string();
            assert!(!s.is_empty(), "empty: {:?}", d);
        }
    }

    #[test]
    fn collector_extends() {
        let mut c = Diagnostics::new();
        c.push(Diagnostic::SymlinkLoop {
            path: PathBuf::from("/a"),
        });
        c.extend([Diagnostic::SymlinkLoop {
            path: PathBuf::from("/b"),
        }]);
        assert_eq!(c.len(), 2);
    }
}
