//! Fetches JSON Schema bytes from published `@agent-ix` npm packages —
//! `@agent-ix/semantic-core` (every version `src/semantic/embedded.rs`
//! embeds, FR-069 Inputs) and `@agent-ix/semantic-schema`
//! (`module-manifest.schema.json`) — and writes a generated Rust module
//! under `OUT_DIR` that `include!`/`include_str!`s the fetched bytes.
//!
//! This is the crate's only network access, and it happens at build time,
//! never at runtime: the compiled binary never reaches the network
//! (FR-013, FR-069-CON-1). `cargo` only reruns this script when `build.rs`
//! itself changes, so the fetch happens once per `target/` (or
//! `CARGO_TARGET_DIR`), not on every `cargo build`/`cargo test`.
//!
//! Nothing here is a copy of another repository's bytes committed to this
//! repository — every schema embedded by `src/semantic/embedded.rs` is
//! fetched fresh from its published package by this script.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Every semantic-core version this crate embeds, ascending. Keep in sync
/// with the versions a `semantic` block may declare (FR-069-AC-2/AC-5).
const SEMANTIC_CORE_VERSIONS: &[&str] = &["0.1.0", "0.2.0"];

/// The `@agent-ix/semantic-schema` version `MODULE_MANIFEST_SCHEMA` is
/// fetched from.
const SEMANTIC_SCHEMA_VERSION: &str = "0.1.0";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR set by cargo"));

    let mut versions_body = String::new();
    let mut consts_body = String::new();
    let mut match_arms = String::new();

    for version in SEMANTIC_CORE_VERSIONS {
        let package_dir = fetch_npm_package("@agent-ix/semantic-core", version, &out_dir);
        let json_schema_dir = package_dir.join("generated").join("json-schema");
        let mut names: Vec<String> = fs::read_dir(&json_schema_dir)
            .unwrap_or_else(|e| panic!("read_dir {}: {e}", json_schema_dir.display()))
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".json"))
            .collect();
        names.sort();

        let const_name = format!("SEMANTIC_CORE_{}", version.replace('.', "_"));
        versions_body.push_str(&format!("\"{version}\", "));
        consts_body.push_str(&format!(
            "/// `(file name, bytes)` for every schema of semantic-core {version}, sorted by\n\
             /// name.\n\
             pub const {const_name}: &[(&str, &str)] = &[\n"
        ));
        for name in &names {
            let path = json_schema_dir
                .join(name)
                .canonicalize()
                .unwrap_or_else(|e| panic!("canonicalize {name} for {version}: {e}"));
            let path = path.to_str().expect("OUT_DIR path is valid UTF-8");
            consts_body.push_str(&format!("    (\"{name}\", include_str!(r#\"{path}\"#)),\n"));
        }
        consts_body.push_str("];\n\n");
        match_arms.push_str(&format!("        \"{version}\" => Some({const_name}),\n"));
    }

    let schema_package_dir = fetch_npm_package(
        "@agent-ix/semantic-schema",
        SEMANTIC_SCHEMA_VERSION,
        &out_dir,
    );
    let module_manifest_path = schema_package_dir
        .join("semantic")
        .join("v1")
        .join("module-manifest.schema.json")
        .canonicalize()
        .unwrap_or_else(|e| panic!("canonicalize module-manifest.schema.json: {e}"));
    let module_manifest_path = module_manifest_path
        .to_str()
        .expect("OUT_DIR path is valid UTF-8");

    let generated = format!(
        "/// Semantic-core versions with an embedded bundle, ascending.\n\
         pub const SEMANTIC_CORE_VERSIONS: &[&str] = &[{versions_body}];\n\n\
         {consts_body}\
         /// The embedded bundle for `version`, if any.\n\
         pub fn semantic_core_bundle(version: &str) -> Option<&'static [(&'static str, &'static str)]> {{\n\
         \x20   match version {{\n\
         {match_arms}\
         \x20       _ => None,\n\
         \x20   }}\n\
         }}\n\n\
         /// `@agent-ix/semantic-schema`'s `semantic/v1/module-manifest.schema.json`,\n\
         /// fetched at build time — never a copy committed to this repository.\n\
         pub const MODULE_MANIFEST_SCHEMA: &str = include_str!(r#\"{module_manifest_path}\"#);\n"
    );
    fs::write(out_dir.join("embedded_schemas.rs"), generated)
        .expect("write generated embedded_schemas.rs");
}

/// Fetch `<package>@<version>` via `npm pack` (network access, verified by
/// npm's own registry integrity check) into `<out_dir>`, unpack it, and
/// return the path to its unpacked `package/` directory. Idempotent per
/// `out_dir`: a `.fetched` marker skips re-fetching on a later
/// `cargo build`/`cargo test` against the same `target/` (or
/// `CARGO_TARGET_DIR`).
fn fetch_npm_package(package: &str, version: &str, out_dir: &Path) -> PathBuf {
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
                     fetch the published package, FR-069 Inputs): {e}"
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

        fs::write(&marker, b"")
            .unwrap_or_else(|e| panic!("write marker {}: {e}", marker.display()));
    }

    unpacked
}
