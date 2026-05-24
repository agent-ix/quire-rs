//! Search-path resolution (FR-013 path section).
//!
//! The loader resolves archetype roots from, in priority order:
//!
//! 1. Explicit `Registry::load_from(paths)` argument.
//! 2. `IX_SCHEMA_PATH` env var — colon-separated PATH-style list.
//! 3. Default: `~/.ix/schemas/`.
//!
//! Each path is tilde-expanded (leading `~/` or `~` only), canonicalized
//! when possible, and de-duplicated. Missing / file-not-directory /
//! permission-denied entries surface as warning diagnostics, not fatal
//! errors (FR-013-AC-10).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Outcome of resolving a single search-path entry.
#[derive(Debug)]
pub enum PathDiagnostic {
    /// Entry is usable.
    Ok(PathBuf),
    /// Entry doesn't exist on disk.
    Missing(PathBuf),
    /// Entry exists but is a file, not a directory.
    NotADirectory(PathBuf),
    /// Could not canonicalize (permission denied, broken link).
    Unreadable { path: PathBuf, reason: String },
}

impl PathDiagnostic {
    /// Return the underlying path regardless of outcome.
    pub fn path(&self) -> &Path {
        match self {
            Self::Ok(p)
            | Self::Missing(p)
            | Self::NotADirectory(p)
            | Self::Unreadable { path: p, .. } => p,
        }
    }

    /// `true` if the path can be walked.
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }
}

/// Expand a leading `~` or `~/` to the user's home directory. No
/// mid-path tilde expansion; non-tilde entries are returned unchanged.
pub fn expand_tilde<P: AsRef<Path>>(path: P) -> PathBuf {
    let p = path.as_ref();
    let s: &str = match p.to_str() {
        Some(s) => s,
        None => return p.to_path_buf(),
    };
    let home = match home_dir() {
        Some(h) => h,
        None => return p.to_path_buf(),
    };
    if s == "~" {
        return home;
    }
    if let Some(rest) = s.strip_prefix("~/") {
        return home.join(rest);
    }
    p.to_path_buf()
}

/// Resolve `IX_SCHEMA_PATH` (colon-separated, no env-var-in-value
/// expansion) plus optional explicit overrides.
///
/// `explicit` wins when non-empty; otherwise `IX_SCHEMA_PATH` is read;
/// otherwise the default `~/.ix/schemas/` is returned.
pub fn resolve_search_paths(
    explicit: &[&Path],
    env_value: Option<OsString>,
) -> Vec<PathDiagnostic> {
    let candidates: Vec<PathBuf> = if !explicit.is_empty() {
        explicit.iter().map(|p| p.to_path_buf()).collect()
    } else if let Some(env) = env_value {
        split_colon_paths(&env)
    } else {
        match home_dir() {
            Some(h) => vec![h.join(".ix").join("schemas")],
            None => Vec::new(),
        }
    };

    let mut diags: Vec<PathDiagnostic> = Vec::new();
    let mut seen_canonical: Vec<PathBuf> = Vec::new();
    for cand in candidates {
        let expanded = expand_tilde(&cand);
        let diag = classify(&expanded);
        if let PathDiagnostic::Ok(ref canon) = diag {
            if seen_canonical.iter().any(|p| p == canon) {
                continue; // FR-013-AC-8: dedup canonical paths
            }
            seen_canonical.push(canon.clone());
        }
        diags.push(diag);
    }
    diags
}

/// Decide whether a path is usable, and canonicalize it when possible.
fn classify(path: &Path) -> PathDiagnostic {
    if !path.exists() {
        return PathDiagnostic::Missing(path.to_path_buf());
    }
    if !path.is_dir() {
        return PathDiagnostic::NotADirectory(path.to_path_buf());
    }
    match std::fs::canonicalize(path) {
        Ok(p) => PathDiagnostic::Ok(p),
        Err(e) => PathDiagnostic::Unreadable {
            path: path.to_path_buf(),
            reason: e.to_string(),
        },
    }
}

/// Split a colon-separated path string into `PathBuf`s. Empty entries
/// (consecutive `:`s or leading/trailing) are dropped.
fn split_colon_paths(s: &OsString) -> Vec<PathBuf> {
    let s = s.to_string_lossy();
    s.split(':')
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .collect()
}

/// Cross-platform home-dir lookup. Falls back to `HOME` / `USERPROFILE`
/// env vars without pulling in a separate crate.
pub fn home_dir() -> Option<PathBuf> {
    if let Some(h) = std::env::var_os("HOME") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    if let Some(h) = std::env::var_os("USERPROFILE") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn expand_tilde_root() {
        let h = home_dir().expect("HOME set");
        assert_eq!(expand_tilde("~"), h);
    }

    #[test]
    fn expand_tilde_subpath() {
        let h = home_dir().expect("HOME set");
        assert_eq!(expand_tilde("~/foo/bar"), h.join("foo/bar"));
    }

    #[test]
    fn expand_tilde_no_op_for_absolute() {
        assert_eq!(expand_tilde("/etc/passwd"), PathBuf::from("/etc/passwd"));
    }

    #[test]
    fn expand_tilde_no_op_for_mid_path() {
        assert_eq!(expand_tilde("/foo/~bar"), PathBuf::from("/foo/~bar"));
    }

    #[test]
    fn split_colon_paths_drops_empties() {
        let v = split_colon_paths(&OsString::from("/a::/b:/c:"));
        assert_eq!(
            v,
            vec![
                PathBuf::from("/a"),
                PathBuf::from("/b"),
                PathBuf::from("/c")
            ]
        );
    }

    #[test]
    fn resolve_with_explicit_wins_over_env() {
        let tmp = env::temp_dir();
        let diags = resolve_search_paths(&[&tmp], Some(OsString::from("/nonexistent")));
        assert_eq!(diags.len(), 1);
        assert!(diags[0].is_ok());
    }

    #[test]
    fn missing_path_classified_as_missing_not_error() {
        let diags = resolve_search_paths(&[Path::new("/definitely/does/not/exist/quire-rs")], None);
        assert!(matches!(diags[0], PathDiagnostic::Missing(_)));
    }

    #[test]
    fn duplicate_canonical_paths_dedupe() {
        let tmp = env::temp_dir();
        let s = format!("{}:{}", tmp.display(), tmp.display());
        let diags = resolve_search_paths(&[], Some(OsString::from(s)));
        let oks: Vec<_> = diags.iter().filter(|d| d.is_ok()).collect();
        assert_eq!(oks.len(), 1);
    }

    #[test]
    fn file_not_directory_is_diagnosed_not_fatal() {
        let mut path = env::temp_dir();
        path.push(format!("quire-rs-test-file-{}.tmp", std::process::id()));
        std::fs::write(&path, b"hi").expect("write");
        let diags = resolve_search_paths(&[&path], None);
        assert!(matches!(diags[0], PathDiagnostic::NotADirectory(_)));
        let _ = std::fs::remove_file(&path);
    }
}
