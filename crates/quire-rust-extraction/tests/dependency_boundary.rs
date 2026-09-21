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

    // `deny.toml` sets `multiple-versions = "allow"` (this workspace does not
    // forbid two majors of a crate coexisting). `.find(...)` would silently
    // assert against whichever copy it reaches first while the build links
    // the other, so this collects every match and requires there be exactly
    // one before comparing — a second major entering the graph fails here
    // with a name and a count, not a version string that happens to still
    // match one of the copies.
    let locked_version_of = |name: &str| -> String {
        let matches: Vec<&str> = packages
            .iter()
            .filter(|package| package["name"].as_str() == Some(name))
            .map(|package| package["version"].as_str().unwrap_or_default())
            .collect();
        match matches.as_slice() {
            [] => panic!("{name} is not in the resolved dependency graph"),
            [version] => version.to_string(),
            multiple => panic!(
                "{name} resolved to {} versions ({multiple:?}), not exactly one — \
                 multiple-versions is allowed in deny.toml, so this assertion cannot \
                 assume `find` reaches the same copy the build links",
                multiple.len()
            ),
        }
    };

    assert_eq!(
        locked_version_of("tree-sitter-rust"),
        "0.24.2",
        "the locked tree-sitter-rust grammar moved — this is the input FR-051-AC-10's \
         identity depends on; review the node-kind/field-name surface this adapter reads \
         before updating this expected version, then update it deliberately"
    );
    // PLAT-851: the boundary crate widened to carry python/typescript alongside
    // rust — a gate widened to cover a new case and not extended to assert it
    // has a blind spot for exactly the case it was widened for (the same
    // defect class this campaign has repeatedly hit: a check that keeps
    // passing for reasons unrelated to what it claims to verify). Neither
    // grammar has an adapter yet (that's PLAT-868/869), but the lockfile
    // already resolves both — `quire-code-parse`'s own optional dependencies
    // are present in `Cargo.lock` regardless of which Cargo feature currently
    // activates them, since cargo resolves versions for every declared
    // optional dependency up front to keep the lock stable across feature
    // combinations.
    assert_eq!(
        locked_version_of("tree-sitter-python"),
        "0.25.0",
        "the locked tree-sitter-python grammar moved; review before the Python AST port relies \
         on it, then update this expected version deliberately"
    );
    assert_eq!(
        locked_version_of("tree-sitter-typescript"),
        "0.23.2",
        "the locked tree-sitter-typescript grammar moved; review before the TypeScript AST port \
         relies on it, then update this expected version deliberately"
    );
    assert_eq!(
        locked_version_of("tree-sitter"),
        "0.26.13",
        "the locked tree-sitter runtime moved; review before updating this expected version"
    );
}

fn cargo_metadata() -> serde_json::Value {
    // Run from the workspace root, not this crate's own manifest, so the
    // graph includes the root `quire-rs` package too. `--all-features`
    // (PLAT-851) is *mechanically necessary* for the second test below to
    // even run — it is not a stricter scope for the reachability check
    // above, which review corrected: `cargo metadata`'s `package["dependencies"]`
    // array is the *manifest-declared* dependency list, not filtered by
    // active features, so the reachability test above already saw
    // `tree-sitter-python`/`tree-sitter-typescript` as optional dependencies
    // of `quire-code-parse` (and would already have caught a direct
    // `tree-sitter` dependency added behind `python-symbols`) with or without
    // this flag; `quire-rs`'s own dependency count is identical either way.
    // What `--all-features` actually controls is `cargo metadata`'s top-level
    // `packages` array, which *is* filtered to the requested feature set: the
    // root package's default features activate only `rust-symbols`, so plain
    // `cargo metadata` here would resolve `tree-sitter-python`/
    // `tree-sitter-typescript` into `Cargo.lock` (every optional dependency
    // is locked regardless of activation — see that test's own doc comment)
    // but leave them out of `packages`. Without `--all-features`,
    // `the_locked_grammar_version_is_asserted_...` below would panic "not in
    // the resolved dependency graph" for both new grammars despite them
    // being perfectly resolvable.
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
