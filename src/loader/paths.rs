//! Search-path resolution (FR-013 path section).
//!
//! The loader resolves archetype roots as:
//!
//! 1. An explicit `Registry::load_from(paths)` argument, which wins outright.
//! 2. Otherwise the roots of `IX_FILAMENT_MODULES_PATH`, then those of the
//!    legacy `IX_SCHEMA_PATH` alias (both are read), followed by
//! 3. the default `$IX_HOME/filament/modules/`, which the environment roots
//!    extend rather than replace.
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

/// Resolve the search-path env value (colon-separated, no env-var-in-value
/// expansion) plus optional explicit overrides.
///
/// `explicit` wins when non-empty; otherwise the caller-supplied `env_value`
/// (see [`module_path_env`]) roots are read, followed by the default
/// `$IX_HOME/filament/modules/` ([`default_module_root`]), which the env value
/// extends rather than replaces.
pub fn resolve_search_paths(
    explicit: &[&Path],
    env_value: Option<OsString>,
) -> Vec<PathDiagnostic> {
    let candidates: Vec<PathBuf> = if !explicit.is_empty() {
        explicit.iter().map(|p| p.to_path_buf()).collect()
    } else {
        let mut roots = env_value
            .map(|e| split_search_paths(&e))
            .unwrap_or_default();
        roots.extend(default_module_root());
        roots
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

/// Split a platform-separated path string into `PathBuf`s. Empty entries
/// (consecutive `:`s or leading/trailing) are dropped.
fn split_search_paths(s: &OsString) -> Vec<PathBuf> {
    std::env::split_paths(s)
        .filter(|p| !p.as_os_str().is_empty())
        .collect()
}

/// The default module search root: `$IX_HOME/filament/modules`, or
/// `~/.ix/filament/modules` when `IX_HOME` is unset or empty.
///
/// quire-rs reads installed Filament modules from here — the same directory
/// `quoin` materializes its default module set into, and `quoin` honours
/// `IX_HOME` the same way. Returns `None` when neither `IX_HOME` nor a home
/// directory can be determined.
pub fn default_module_root() -> Option<PathBuf> {
    module_root_under(std::env::var_os("IX_HOME"), home_dir())
}

/// Pure form of [`default_module_root`]: a non-empty `ix_home` is the config
/// root; otherwise it is `<home>/.ix`.
pub fn module_root_under(ix_home: Option<OsString>, home: Option<PathBuf>) -> Option<PathBuf> {
    let root = match ix_home {
        Some(h) if !h.is_empty() => PathBuf::from(h),
        _ => home?.join(".ix"),
    };
    Some(root.join("filament").join("modules"))
}

/// Select the module-search env value: the roots of `IX_FILAMENT_MODULES_PATH`
/// followed by those of `IX_SCHEMA_PATH` (both are honoured, as `quire-cli`
/// does).
pub fn module_path_env(filament: Option<OsString>, schema: Option<OsString>) -> Option<OsString> {
    let roots: Vec<PathBuf> = [filament, schema]
        .iter()
        .flatten()
        .flat_map(split_search_paths)
        .collect();
    if roots.is_empty() {
        return None;
    }
    // Every piece came out of `split_paths`, so it holds no separator and
    // `join_paths` cannot fail.
    std::env::join_paths(roots).ok()
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
    use ix_trace_rs::trace;
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
    fn default_module_root_points_at_filament_modules() {
        if std::env::var_os("IX_HOME").is_some_and(|v| !v.is_empty()) {
            return;
        }
        let h = home_dir().expect("HOME set");
        assert_eq!(
            default_module_root().expect("home"),
            h.join(".ix").join("filament").join("modules")
        );
    }

    #[test]
    fn ix_home_overrides_home_for_module_root() {
        assert_eq!(
            module_root_under(Some(OsString::from("/x/fake")), Some(PathBuf::from("/h"))),
            Some(PathBuf::from("/x/fake/filament/modules"))
        );
        assert_eq!(
            module_root_under(Some(OsString::new()), Some(PathBuf::from("/h"))),
            Some(PathBuf::from("/h/.ix/filament/modules"))
        );
        assert_eq!(
            module_root_under(None, Some(PathBuf::from("/h"))),
            Some(PathBuf::from("/h/.ix/filament/modules"))
        );
        assert_eq!(module_root_under(None, None), None);
        assert_eq!(
            module_root_under(Some(OsString::from("/x")), None),
            Some(PathBuf::from("/x/filament/modules"))
        );
    }

    #[test]
    fn module_path_env_unions_both_variables() {
        // Multi-root values keep every root, in variable order.
        let multi = std::env::join_paths(["/a", "/b"]).unwrap();
        assert_eq!(
            module_path_env(Some(multi), Some(OsString::from("/c"))),
            std::env::join_paths(["/a", "/b", "/c"]).ok()
        );
        assert_eq!(
            module_path_env(Some(OsString::from("/a")), Some(OsString::from("/b"))),
            std::env::join_paths(["/a", "/b"]).ok()
        );
        assert_eq!(
            module_path_env(None, Some(OsString::from("/b"))),
            Some(OsString::from("/b"))
        );
        assert_eq!(module_path_env(None, None), None);
    }

    #[trace("TC-2050", "FR-013-AC-18")]
    // PLAT-106: env roots extend the default install root, as quire-cli does.
    #[test]
    fn env_roots_keep_default_root() {
        let a = env::temp_dir().join(format!("quire-rs-roots-a-{}", std::process::id()));
        let b = env::temp_dir().join(format!("quire-rs-roots-b-{}", std::process::id()));
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        let env_value = module_path_env(
            Some(a.clone().into_os_string()),
            Some(b.clone().into_os_string()),
        );
        let diags = resolve_search_paths(&[], env_value);
        let paths: Vec<PathBuf> = diags
            .iter()
            .map(|d| match d {
                PathDiagnostic::Ok(p) => p.clone(),
                PathDiagnostic::Missing(p) | PathDiagnostic::NotADirectory(p) => p.clone(),
                PathDiagnostic::Unreadable { path, .. } => path.clone(),
            })
            .collect();
        let _ = std::fs::remove_dir_all(&a);
        let _ = std::fs::remove_dir_all(&b);
        assert_eq!(paths[0], std::fs::canonicalize(&a).unwrap_or(a));
        assert_eq!(paths[1], std::fs::canonicalize(&b).unwrap_or(b));
        assert_eq!(paths.get(2), default_module_root().as_ref());
    }

    #[test]
    fn split_search_paths_drops_empties() {
        let v = split_search_paths(&OsString::from("/a::/b:/c:"));
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
        let canon = std::fs::canonicalize(&tmp).unwrap();
        let hits = diags
            .iter()
            .filter(|d| matches!(d, PathDiagnostic::Ok(p) if *p == canon))
            .count();
        assert_eq!(hits, 1);
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
