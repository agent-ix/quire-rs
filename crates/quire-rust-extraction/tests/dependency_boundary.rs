//! Dependency-boundary gate (PLAT-843), copied from `filament-ide-rs`'s
//! `crates/filament-code-extraction/tests/extraction.rs`
//! (`fr_072_ac_5_tree_sitter_is_reachable_only_through_the_quire_code_rs_pin`).
//!
//! This is the mechanism that stops a second tree-sitter parser reappearing
//! in this workspace — the exact duplication PLAT-843 exists to end. It was
//! deliberately **observed red** before being trusted (see the PR body): a
//! temporary direct `tree-sitter` dependency was added to the root package,
//! confirmed to fail this test, then reverted. **Observed red a second time**
//! after being strengthened to scope by `workspace_members` instead of two
//! hardcoded names (review F2) — same technique, same result, recorded in the
//! PR body rather than repeated here.

#[test]
fn tree_sitter_is_reachable_only_through_the_quire_code_parse_pin() {
    let metadata = cargo_metadata();
    let packages = metadata["packages"].as_array().expect("packages");
    // The in-scope set is *every current workspace member*, read from
    // `workspace_members` rather than hardcoded by name (review F2): a gate
    // that names "quire-rs" and "quire-rust-extraction" literally checks
    // nothing about a third workspace member added later, which is the same
    // defect class as a criterion too weak to fail. `workspace_members` is a
    // list of package ids; cross-reference against each package's own `id`
    // rather than parsing the id string, so this holds across cargo's id
    // format (`pkg#ver` / `pkg@ver` all resolve the same way here).
    let workspace_members: std::collections::HashSet<&str> = metadata["workspace_members"]
        .as_array()
        .expect("workspace_members")
        .iter()
        .map(|id| id.as_str().expect("workspace member id is a string"))
        .collect();
    assert!(
        workspace_members.len() >= 2,
        "expected at least quire-rs and quire-rust-extraction as workspace members, got {}",
        workspace_members.len()
    );

    for package in packages {
        let id = package["id"].as_str().unwrap_or_default();
        // Only this workspace's own crates are in scope; third-party
        // packages (including `quire-code-parse` itself, and the
        // `quire-code-rs` package that pins it) are entitled to their own
        // dependencies.
        if !workspace_members.contains(id) {
            continue;
        }
        let name = package["name"].as_str().unwrap_or_default();
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

/// FR-051-AC-10 (CR-177): the grammar version is part of what "an identical
/// tree, at a pinned grammar version" holds constant, and `rev =
/// "57b83ba0..."` only pins `quire-code-rs`'s own *source* — that repo's
/// `Cargo.toml` declares `tree-sitter = "0.26"` and `tree-sitter-rust =
/// "0.24.2"` as **caret ranges**, so the grammar version this workspace
/// actually builds against lives only in *this* repo's own `Cargo.lock`,
/// moved silently by a bare `cargo update` with no manifest edit, no `rev`
/// change, no ADR, and nothing here to fail (spec review F1). This is the
/// compiled assertion `FR-051-AC-10`'s note now cites in place of
/// `NFR-009` — that policy does not cover `tree-sitter*` at all
/// (`scripts/audits/check_dep_pins.sh`'s table names `minijinja`,
/// `jsonschema`, `serde_yaml`, `serde_json`, `indexmap` only).
///
/// Asserts the exact **locked** version (`cargo metadata` reports the
/// resolved version from `Cargo.lock`, never the manifest's caret range),
/// so a `cargo update` that moves the grammar fails here with a name and a
/// number, and updating the pin is a reviewed, visible diff to this
/// assertion rather than a transitive-dependency change nothing catches.
#[test]
fn the_locked_grammar_version_is_asserted_not_only_pinned_by_source_rev() {
    let metadata = cargo_metadata();
    let packages = metadata["packages"].as_array().expect("packages");

    let locked_version_of = |name: &str| -> String {
        packages
            .iter()
            .find(|package| package["name"].as_str() == Some(name))
            .unwrap_or_else(|| panic!("{name} is not in the resolved dependency graph"))["version"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    };

    assert_eq!(
        locked_version_of("tree-sitter-rust"),
        "0.24.2",
        "the locked tree-sitter-rust grammar moved — this is the input FR-051-AC-10's \
         identity depends on; review the node-kind/field-name surface this adapter reads \
         before updating this expected version, then update it deliberately"
    );
    assert_eq!(
        locked_version_of("tree-sitter"),
        "0.26.13",
        "the locked tree-sitter runtime moved; review before updating this expected version"
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
