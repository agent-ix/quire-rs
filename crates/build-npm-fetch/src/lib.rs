//! Shared `npm pack` fetch-and-unpack build-time helper (PLAT-901).
//!
//! Both quire-rs's root `build.rs` (`@agent-ix/semantic-core`,
//! `@agent-ix/semantic-schema`) and `spec-objects-architecture-fixture`'s
//! `build.rs` (`@agent-ix/spec-objects-architecture`) need the same
//! sequence: `npm pack <package>@<version>` into a scratch dir, find the
//! emitted `.tgz`, unpack it with `tar xzf`, and skip the whole thing on a
//! later `cargo build`/`cargo test` against the same `target/` (a build
//! script only reruns when `build.rs` itself changes, so this only matters
//! within one `cargo clean`). This crate is a `[build-dependencies]`-only
//! helper — never a normal runtime dependency of anything — so there is
//! exactly one copy of this logic instead of one per build script.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Why `try_fetch_npm_package` failed. Carries enough detail to reproduce
/// [`fetch_npm_package`]'s historical panic messages verbatim, and enough
/// for a caller that tolerates a missing package to explain itself via
/// `cargo:warning` instead.
#[derive(Debug)]
pub enum FetchError {
    /// `npm` itself could not be spawned (not on `PATH`, permissions, ...).
    Spawn {
        package: String,
        version: String,
        source: std::io::Error,
    },
    /// `npm pack` ran and exited non-zero — most commonly the package or
    /// version does not exist on the configured registry (a 404).
    PackFailed { package: String, version: String },
    /// `npm pack` exited zero but wrote no `.tgz` where one was expected.
    NoTarball { pack_dir: PathBuf },
    /// `tar` itself could not be spawned.
    TarSpawn {
        tarball: PathBuf,
        source: std::io::Error,
    },
    /// `tar` ran and exited non-zero.
    TarFailed { tarball: PathBuf },
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::Spawn {
                package,
                version,
                source,
            } => write!(
                f,
                "run `npm pack {package}@{version}` (build-time network access to \
                 fetch the published package): {source}"
            ),
            FetchError::PackFailed { package, version } => {
                write!(f, "npm pack {package}@{version} failed")
            }
            FetchError::NoTarball { pack_dir } => {
                write!(f, "npm pack produced no .tgz in {}", pack_dir.display())
            }
            FetchError::TarSpawn { tarball, source } => {
                write!(f, "run `tar` to unpack {}: {source}", tarball.display())
            }
            FetchError::TarFailed { tarball } => {
                write!(f, "tar xzf {} failed", tarball.display())
            }
        }
    }
}

impl std::error::Error for FetchError {}

/// Fetch `<package>@<version>` via `npm pack` (network access, verified by
/// npm's own registry integrity check) into `<out_dir>/npm-packages/<slug>/
/// <version>`, unpack it, and return the path to its unpacked `package/`
/// directory.
///
/// Idempotent per `out_dir`: a `.fetched` marker — written only after both
/// `npm pack` and `tar xzf` succeed — skips re-fetching on a later build
/// against the same `target/`/`CARGO_TARGET_DIR`. The `.tgz` and the
/// scratch directory `npm pack` wrote it into are deleted once the tarball
/// is unpacked; only the unpacked `package/` directory this function
/// returns is kept under `out_dir`.
///
/// Panics on any failure — this is the entry point for callers for whom a
/// missing package is a fatal build error (root `build.rs`'s
/// `@agent-ix/semantic-core`/`@agent-ix/semantic-schema` fetches, which CI
/// authenticates to GitHub Packages and which must exist for the build to
/// mean anything). A caller that can legitimately tolerate the package being
/// unreachable (an optional cross-repo fixture) should call
/// [`try_fetch_npm_package`] instead and decide for itself.
pub fn fetch_npm_package(package: &str, version: &str, out_dir: &Path) -> PathBuf {
    try_fetch_npm_package(package, version, out_dir).unwrap_or_else(|e| panic!("{e}"))
}

/// The fallible core of [`fetch_npm_package`]: identical behavior, but a
/// `npm pack`/`tar` failure (package or version not present on the
/// configured registry, spawn failure, corrupt tarball) is returned as a
/// [`FetchError`] instead of panicking the build. Local environment
/// failures unrelated to the package itself (can't create a scratch
/// directory, can't write the idempotency marker) still panic: those are
/// not "the package is absent", they are "this build can't do I/O", which
/// no caller can meaningfully tolerate.
pub fn try_fetch_npm_package(
    package: &str,
    version: &str,
    out_dir: &Path,
) -> Result<PathBuf, FetchError> {
    let slug = package.trim_start_matches('@').replace('/', "__");
    let dest = out_dir.join("npm-packages").join(&slug).join(version);
    let unpacked = dest.join("package");
    let marker = dest.join(".fetched");

    if !marker.is_file() {
        fs::create_dir_all(&dest).unwrap_or_else(|e| panic!("create {}: {e}", dest.display()));

        let pack_dir = out_dir.join("npm-pack-tmp").join(&slug).join(version);
        fs::create_dir_all(&pack_dir)
            .unwrap_or_else(|e| panic!("create {}: {e}", pack_dir.display()));

        let status = Command::new("npm")
            .arg("pack")
            .arg(format!("{package}@{version}"))
            .arg("--pack-destination")
            .arg(&pack_dir)
            .status()
            .map_err(|source| FetchError::Spawn {
                package: package.to_string(),
                version: version.to_string(),
                source,
            })?;
        if !status.success() {
            return Err(FetchError::PackFailed {
                package: package.to_string(),
                version: version.to_string(),
            });
        }

        let tarball = fs::read_dir(&pack_dir)
            .unwrap_or_else(|e| panic!("read_dir {}: {e}", pack_dir.display()))
            .filter_map(Result::ok)
            .map(|e| e.path())
            .find(|p| p.extension().and_then(|e| e.to_str()) == Some("tgz"))
            .ok_or_else(|| FetchError::NoTarball {
                pack_dir: pack_dir.clone(),
            })?;

        if unpacked.exists() {
            fs::remove_dir_all(&unpacked)
                .unwrap_or_else(|e| panic!("remove stale {}: {e}", unpacked.display()));
        }
        let status = Command::new("tar")
            .arg("xzf")
            .arg(&tarball)
            .arg("-C")
            .arg(&dest)
            .status()
            .map_err(|source| FetchError::TarSpawn {
                tarball: tarball.clone(),
                source,
            })?;
        if !status.success() {
            return Err(FetchError::TarFailed { tarball });
        }

        // OUT_DIR hygiene: the `.tgz` and the scratch dir `npm pack` wrote
        // it into are never needed again once unpacked — delete them so
        // only the unpacked destination directory remains.
        fs::remove_dir_all(&pack_dir)
            .unwrap_or_else(|e| panic!("remove {}: {e}", pack_dir.display()));

        fs::write(&marker, b"")
            .unwrap_or_else(|e| panic!("write marker {}: {e}", marker.display()));
    }

    Ok(unpacked)
}
