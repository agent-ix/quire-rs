//! Fetches the published `@agent-ix/spec-objects-architecture` npm package
//! (FR-075-AC-13) via `npm pack` and extracts its `manifest.yaml`,
//! `schemas/`, and `skeletons/` into `OUT_DIR/module/`.
//!
//! Same mechanism quire-rs's own root `build.rs` already uses for
//! `@agent-ix/semantic-core`: build-time network access, cached under
//! `OUT_DIR` (so it happens once per `target/`/`CARGO_TARGET_DIR`, not on
//! every build), never at runtime. Nothing here is a copy of the package's
//! bytes committed to this repository — this crate asserts a real module's
//! schemas, resolved through the package manager, the way TC-1872/TC-1873
//! (FR-075-AC-13) require: the tests read a real module, not an
//! authored-minimal stand-in.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Keep in sync with FR-075-AC-13's cited version.
const VERSION: &str = "0.7.0";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR set by cargo"));
    fetch_spec_objects_architecture(VERSION, &out_dir);
}

/// Fetch `@agent-ix/spec-objects-architecture@<version>` via `npm pack`
/// (network access, verified by npm's own registry integrity check) into
/// `out_dir/module`, idempotently (a `manifest.yaml` marker skips a
/// re-fetch).
fn fetch_spec_objects_architecture(version: &str, out_dir: &Path) {
    let dest = out_dir.join("module");
    let marker = dest.join("manifest.yaml");
    if marker.is_file() {
        return;
    }
    fs::create_dir_all(&dest).unwrap_or_else(|e| panic!("create {}: {e}", dest.display()));

    let pack_dir = out_dir.join("pack");
    fs::create_dir_all(&pack_dir).unwrap_or_else(|e| panic!("create {}: {e}", pack_dir.display()));

    let status = Command::new("npm")
        .arg("pack")
        .arg(format!("@agent-ix/spec-objects-architecture@{version}"))
        .arg("--pack-destination")
        .arg(&pack_dir)
        .status()
        .expect(
            "run `npm pack` (build-time network access to fetch the published \
             @agent-ix/spec-objects-architecture package, FR-075-AC-13)",
        );
    assert!(
        status.success(),
        "npm pack @agent-ix/spec-objects-architecture@{version} failed"
    );

    let tarball = fs::read_dir(&pack_dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", pack_dir.display()))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.extension().and_then(|e| e.to_str()) == Some("tgz"))
        .unwrap_or_else(|| panic!("npm pack produced no .tgz in {}", pack_dir.display()));

    let status = Command::new("tar")
        .arg("xzf")
        .arg(&tarball)
        .arg("-C")
        .arg(&pack_dir)
        .status()
        .expect("run `tar` to unpack the fetched npm package");
    assert!(status.success(), "tar xzf {} failed", tarball.display());

    let src = pack_dir.join("package");
    let name = "manifest.yaml";
    fs::copy(src.join(name), dest.join(name)).unwrap_or_else(|e| panic!("copy {name}: {e}"));
    for dir in ["schemas", "skeletons"] {
        copy_dir(&src.join(dir), &dest.join(dir));
    }
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|e| panic!("create {}: {e}", to.display()));
    for entry in fs::read_dir(from).unwrap_or_else(|e| panic!("read_dir {}: {e}", from.display())) {
        let entry = entry.expect("readable dir entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("file type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target)
                .unwrap_or_else(|e| panic!("copy {}: {e}", entry.path().display()));
        }
    }
}
