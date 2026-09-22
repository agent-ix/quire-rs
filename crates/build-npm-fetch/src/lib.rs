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

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
pub fn fetch_npm_package(package: &str, version: &str, out_dir: &Path) -> PathBuf {
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
            .unwrap_or_else(|e| {
                panic!(
                    "run `npm pack {package}@{version}` (build-time network access to \
                     fetch the published package): {e}"
                )
            });
        assert!(status.success(), "npm pack {package}@{version} failed");

        let tarball = fs::read_dir(&pack_dir)
            .unwrap_or_else(|e| panic!("read_dir {}: {e}", pack_dir.display()))
            .filter_map(Result::ok)
            .map(|e| e.path())
            .find(|p| p.extension().and_then(|e| e.to_str()) == Some("tgz"))
            .unwrap_or_else(|| panic!("npm pack produced no .tgz in {}", pack_dir.display()));

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
            .unwrap_or_else(|e| panic!("run `tar` to unpack {}: {e}", tarball.display()));
        assert!(status.success(), "tar xzf {} failed", tarball.display());

        // OUT_DIR hygiene: the `.tgz` and the scratch dir `npm pack` wrote
        // it into are never needed again once unpacked — delete them so
        // only the unpacked destination directory remains.
        fs::remove_dir_all(&pack_dir)
            .unwrap_or_else(|e| panic!("remove {}: {e}", pack_dir.display()));

        fs::write(&marker, b"")
            .unwrap_or_else(|e| panic!("write marker {}: {e}", marker.display()));
    }

    unpacked
}
