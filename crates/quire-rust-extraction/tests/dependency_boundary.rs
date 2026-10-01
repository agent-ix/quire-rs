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
//!
//! PLAT-851 widened this crate to carry `python`/`typescript` alongside
//! `rust`. Adding a temporary direct `tree-sitter-python` dependency and
//! watching the reachability test above fail would prove nothing about that
//! widening — review caught this: `package["dependencies"]` is manifest-
//! derived, not feature-filtered, so that test's prefix match and
//! `workspace_members` scoping are unchanged by PLAT-851 and would have
//! failed identically before it. The assertions PLAT-851 actually added are
//! the two new `locked_version_of` equality checks below; those were
//! observed red by perturbing one expected version string, confirming the
//! failure named the mismatch, then reverting (see the PR body) — the
//! evidence that actually matches what PLAT-851 changed.

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
            "--all-features",
        ])
        .output()
        .expect("cargo metadata must run");
    assert!(output.status.success(), "cargo metadata failed");
    serde_json::from_slice(&output.stdout).expect("cargo metadata emits JSON")
}
