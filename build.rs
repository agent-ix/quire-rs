//! Fetches the `@agent-ix/semantic-core` JSON Schema bundle for every
//! version `src/semantic/embedded.rs` embeds (FR-069 Inputs) from the
//! published npm package, and writes a generated Rust module under
//! `OUT_DIR` that `include!`s the fetched bytes.
//!
//! This is the crate's only network access, and it happens at build time,
//! never at runtime: the compiled binary never reaches the network
//! (FR-013, FR-069-CON-1). `cargo` only reruns this script when `build.rs`
//! itself changes, so the fetch happens once per `target/` (or
//! `CARGO_TARGET_DIR`), not on every `cargo build`/`cargo test`.
//!
//! Nothing here is a copy of `@agent-ix/semantic-core`'s bytes committed to
//! this repository — see `schemas/vendored/PROVENANCE.json` for the one
//! file that still is, and why.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Every semantic-core version this crate embeds, ascending. Keep in sync
/// with the versions a `semantic` block may declare (FR-069-AC-2/AC-5).
const SEMANTIC_CORE_VERSIONS: &[&str] = &["0.1.0", "0.2.0"];

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR set by cargo"));

    let mut versions_body = String::new();
    let mut consts_body = String::new();
    let mut match_arms = String::new();

    for version in SEMANTIC_CORE_VERSIONS {
        let files = fetch_semantic_core(version, &out_dir);
        let const_name = format!("SEMANTIC_CORE_{}", version.replace('.', "_"));
        versions_body.push_str(&format!("\"{version}\", "));
        consts_body.push_str(&format!(
            "/// `(file name, bytes)` for every schema of semantic-core {version}, sorted by\n\
             /// name; `toolchain.json` is provenance, not a schema, and is excluded.\n\
             pub const {const_name}: &[(&str, &str)] = &[\n"
        ));
        for name in &files {
            let path = out_dir
                .join("semantic-core")
                .join(version)
                .join(name)
                .canonicalize()
                .unwrap_or_else(|e| panic!("canonicalize {name} for {version}: {e}"));
            let path = path.to_str().expect("OUT_DIR path is valid UTF-8");
            consts_body.push_str(&format!("    (\"{name}\", include_str!(r#\"{path}\"#)),\n"));
        }
        consts_body.push_str("];\n\n");
        match_arms.push_str(&format!("        \"{version}\" => Some({const_name}),\n"));
    }

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
         }}\n"
    );
    fs::write(out_dir.join("embedded_semantic_core.rs"), generated)
        .expect("write generated embedded_semantic_core.rs");
}

/// Fetch `@agent-ix/semantic-core@<version>` via `npm pack` (network access,
/// verified by npm's own registry integrity check) into `out_dir`, and
/// return the sorted schema file names (excluding `toolchain.json`).
fn fetch_semantic_core(version: &str, out_dir: &Path) -> Vec<String> {
    let dest = out_dir.join("semantic-core").join(version);
    let marker = dest.join("toolchain.json");
    if !marker.is_file() {
        fs::create_dir_all(&dest).unwrap_or_else(|e| panic!("create {}: {e}", dest.display()));

        let pack_dir = out_dir.join("semantic-core-pack").join(version);
        fs::create_dir_all(&pack_dir)
            .unwrap_or_else(|e| panic!("create {}: {e}", pack_dir.display()));

        let status = Command::new("npm")
            .arg("pack")
            .arg(format!("@agent-ix/semantic-core@{version}"))
            .arg("--pack-destination")
            .arg(&pack_dir)
            .status()
            .expect(
                "run `npm pack` (build-time network access to fetch the published \
                 @agent-ix/semantic-core package, FR-069 Inputs)",
            );
        assert!(
            status.success(),
            "npm pack @agent-ix/semantic-core@{version} failed"
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

        let src = pack_dir.join("package").join("generated");
        let json_schema = src.join("json-schema");
        for entry in fs::read_dir(&json_schema)
            .unwrap_or_else(|e| panic!("read_dir {}: {e}", json_schema.display()))
        {
            let entry = entry.expect("readable dir entry");
            fs::copy(entry.path(), dest.join(entry.file_name()))
                .unwrap_or_else(|e| panic!("copy {}: {e}", entry.path().display()));
        }
        fs::copy(src.join("toolchain.json"), &marker)
            .unwrap_or_else(|e| panic!("copy toolchain.json for {version}: {e}"));
    }

    let mut names: Vec<String> = fs::read_dir(&dest)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dest.display()))
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".json") && n != "toolchain.json")
        .collect();
    names.sort();
    names
}
