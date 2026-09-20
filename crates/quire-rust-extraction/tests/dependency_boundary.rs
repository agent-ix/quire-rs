//! Dependency-boundary gate (PLAT-843), copied from `filament-ide-rs`'s
//! `crates/filament-code-extraction/tests/extraction.rs`
//! (`fr_072_ac_5_tree_sitter_is_reachable_only_through_the_quire_code_rs_pin`).
//!
//! This is the mechanism that stops a second tree-sitter parser reappearing
//! in this workspace — the exact duplication PLAT-843 exists to end. It was
//! deliberately **observed red** before being trusted (see the PR body): a
//! temporary direct `tree-sitter` dependency was added to the root package,
//! confirmed to fail this test, then reverted.

#[test]
fn tree_sitter_is_reachable_only_through_the_quire_code_parse_pin() {
    let metadata = cargo_metadata();
    let packages = metadata["packages"].as_array().expect("packages");

    for package in packages {
        let name = package["name"].as_str().unwrap_or_default();
        // Only this workspace's own crates are in scope; third-party
        // packages (including `quire-code-parse` itself, and the
        // `quire-code-rs` package that pins it) are entitled to their own
        // dependencies.
        if name != "quire-rs" && name != "quire-rust-extraction" {
            continue;
        }
        for dependency in package["dependencies"].as_array().expect("dependencies") {
            let dep_name = dependency["name"].as_str().unwrap_or_default();
            assert!(
                !dep_name.starts_with("tree-sitter"),
                "crate `{name}` depends on `{dep_name}` directly; the \
                 tree-sitter boundary must live behind the quire-code-parse pin \
                 in crates/quire-rust-extraction"
            );
        }
    }

    // ...and the pin really is the path by which it is reachable at all.
    let quire_code_parse = packages
        .iter()
        .find(|package| package["name"].as_str() == Some("quire-code-parse"))
        .expect("quire-code-parse is pinned by crates/quire-rust-extraction");
    assert!(
        quire_code_parse["dependencies"]
            .as_array()
            .expect("dependencies")
            .iter()
            .any(|dependency| dependency["name"]
                .as_str()
                .unwrap_or_default()
                .starts_with("tree-sitter")),
        "the pin is what owns the tree-sitter dependency"
    );
}

fn cargo_metadata() -> serde_json::Value {
    // Run from the workspace root, not this crate's own manifest, so the
    // graph includes the root `quire-rs` package too.
    let manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.toml");
    let output = std::process::Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--manifest-path",
            manifest,
        ])
        .output()
        .expect("cargo metadata must run");
    assert!(output.status.success(), "cargo metadata failed");
    serde_json::from_slice(&output.stdout).expect("cargo metadata emits JSON")
}
