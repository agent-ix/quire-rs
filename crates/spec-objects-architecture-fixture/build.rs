//! Fetches the published `@agent-ix/spec-objects-architecture` npm package
//! (FR-075-AC-13) via `npm pack` and extracts its `manifest.yaml`,
//! `schemas/`, and `skeletons/` into `OUT_DIR/module/`.
//!
//! The `npm pack` -> find `.tgz` -> `tar xzf` sequence itself lives in the
//! `build-npm-fetch` crate (PLAT-901), shared with quire-rs's own root
//! `build.rs` (`@agent-ix/semantic-core`/`@agent-ix/semantic-schema`) so
//! there is exactly one copy of that logic: build-time network access,
//! cached under `OUT_DIR` (so it happens once per `target/`/
//! `CARGO_TARGET_DIR`, not on every build), never at runtime. Nothing here
//! is a copy of the package's bytes committed to this repository — this
//! crate asserts a real module's schemas, resolved through the package
//! manager, the way TC-1872/TC-1873 (FR-075-AC-13) require: the tests read
//! a real module, not an authored-minimal stand-in.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Keep in sync with FR-075-AC-13's cited version and with `src/lib.rs`'s
/// `VERSION`, which reads this same value back via `env!` at compile time —
/// there is only one literal.
const VERSION: &str = "0.7.0";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=SPEC_OBJECTS_ARCHITECTURE_VERSION={VERSION}");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR set by cargo"));
    fetch_spec_objects_architecture(VERSION, &out_dir);
}

/// Fetch `@agent-ix/spec-objects-architecture@<version>` and copy its
/// `manifest.yaml`, `schemas/`, and `skeletons/` into `out_dir/module`.
///
/// Idempotent per `out_dir`: `marker` is written **last**, only after every
/// copy below has succeeded, and is what a later build checks. An
/// interruption between copies must not leave a marker that makes a later
/// build short-circuit on an incomplete `module/` — this mirrors how root
/// `build.rs`'s own `.fetched` marker (in `build-npm-fetch`) is written
/// only once its own fetch-and-unpack has fully succeeded.
fn fetch_spec_objects_architecture(version: &str, out_dir: &Path) {
    let dest = out_dir.join("module");
    let marker = out_dir.join(".module-fetched");
    if marker.is_file() {
        return;
    }

    let package_dir =
        build_npm_fetch::fetch_npm_package("@agent-ix/spec-objects-architecture", version, out_dir);

    if dest.exists() {
        fs::remove_dir_all(&dest)
            .unwrap_or_else(|e| panic!("remove stale {}: {e}", dest.display()));
    }
    fs::create_dir_all(&dest).unwrap_or_else(|e| panic!("create {}: {e}", dest.display()));

    let name = "manifest.yaml";
    fs::copy(package_dir.join(name), dest.join(name))
        .unwrap_or_else(|e| panic!("copy {name}: {e}"));
    for dir in ["schemas", "skeletons"] {
        copy_dir(&package_dir.join(dir), &dest.join(dir));
    }

    // OUT_DIR hygiene: `package_dir` (the unpacked npm-pack tree) is
    // intermediate once its files are copied into `dest` — keep only the
    // final destination directory.
    let npm_packages_dir = out_dir.join("npm-packages");
    let _ = fs::remove_dir_all(&npm_packages_dir);

    // Written last: everything above succeeded, so `dest` is complete.
    fs::write(&marker, b"").unwrap_or_else(|e| panic!("write marker {}: {e}", marker.display()));
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
