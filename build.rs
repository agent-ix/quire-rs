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
//!
//! The `npm pack` -> find `.tgz` -> `tar xzf` -> idempotency-marker sequence
//! itself lives in the `build-npm-fetch` crate (PLAT-901), shared with
//! `crates/spec-objects-architecture-fixture/build.rs` so there is exactly
//! one copy of that logic.

use std::env;
use std::fs;
use std::path::PathBuf;

/// Every semantic-core version this crate embeds, ascending. Keep in sync
/// with the versions a `semantic` block may declare (FR-069-AC-2/AC-5).
const SEMANTIC_CORE_VERSIONS: &[&str] = &["0.3.0"];

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
        let package_dir =
            build_npm_fetch::fetch_npm_package("@agent-ix/semantic-core", version, &out_dir);
        let json_schema_dir = package_dir.join("generated").join("json-schema");
        let mut names: Vec<String> = fs::read_dir(&json_schema_dir)
            .unwrap_or_else(|e| panic!("read_dir {}: {e}", json_schema_dir.display()))
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            // `toolchain.json` is provenance, not a schema, and is excluded
            // (the same rule `origin/main`'s old `vendored.rs` documented).
            .filter(|n| n.ends_with(".json") && n != "toolchain.json")
            .collect();
        names.sort();

        let const_name = format!("SEMANTIC_CORE_{}", version.replace('.', "_"));
        versions_body.push_str(&format!("{version:?}, "));
        consts_body.push_str(&format!(
            "/// `(file name, bytes)` for every schema of semantic-core {version}, sorted by\n\
             /// name; `toolchain.json` is provenance, not a schema, and is excluded.\n\
             pub const {const_name}: &[(&str, &str)] = &[\n"
        ));
        for name in &names {
            let path = json_schema_dir
                .join(name)
                .canonicalize()
                .unwrap_or_else(|e| panic!("canonicalize {name} for {version}: {e}"));
            let path = path.to_str().expect("OUT_DIR path is valid UTF-8");
            consts_body.push_str(&format!("    ({name:?}, include_str!(r#\"{path}\"#)),\n"));
        }
        consts_body.push_str("];\n\n");
        match_arms.push_str(&format!("        {version:?} => Some({const_name}),\n"));
    }

    let schema_package_dir = build_npm_fetch::fetch_npm_package(
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
