//! Shared test-only helpers (PLAT-906). `#[path = "support/mod.rs"] mod
//! support;` in each integration test file that needs one, rather than
//! duplicating it per file — Cargo does not auto-discover this file as its
//! own test binary because it lives under a subdirectory of `tests/`.
#![allow(
    dead_code,
    reason = "not every integration test file uses every helper"
)]

use std::path::PathBuf;
use std::process::Command;

/// The `tests/fixtures/semantic-module` directory of the pinned `quoin`
/// dependency checkout, located via `cargo metadata`.
///
/// quire-rs stops holding its own copy of quoin's golden semantic-module
/// fixtures under `tests/fixtures/semantic/quoin/` (PLAT-906): Cargo checks
/// out the *entire* `quoin` repository for the `quoin-finding-types`
/// dev-dependency (see the `Cargo.toml` comment on that entry), and
/// `cargo metadata`'s `packages[].manifest_path` names that checkout — the
/// same mechanism `crates/quire-rust-extraction`'s own
/// `tests/dependency_boundary.rs` already uses to inspect the resolved
/// graph. `quoin-finding-types` itself is never called; it exists only to
/// give Cargo a package name to materialise the checkout under.
pub fn quoin_fixtures() -> PathBuf {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--manifest-path"])
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .output()
        .expect("cargo metadata must run");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("cargo metadata emits JSON");
    let package = metadata["packages"]
        .as_array()
        .expect("cargo metadata emits a packages array")
        .iter()
        .find(|package| package["name"] == "quoin-finding-types")
        .expect("quoin-finding-types is not in the resolved dependency graph");
    let manifest_path = PathBuf::from(
        package["manifest_path"]
            .as_str()
            .expect("cargo metadata reports manifest_path as a string"),
    );
    // manifest_path = <checkout>/rust/crates/quoin-finding-types/Cargo.toml
    manifest_path
        .parent()
        .and_then(std::path::Path::parent)
        .and_then(std::path::Path::parent)
        .and_then(std::path::Path::parent)
        .unwrap_or_else(|| {
            panic!(
                "unexpected manifest_path shape: {}",
                manifest_path.display()
            )
        })
        .join("tests/fixtures/semantic-module")
}
