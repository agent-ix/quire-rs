//! Materialises `common.schema.json` and the semantic-core 0.2.0 bundle from
//! the pinned `filament-core-data` dependency into `OUT_DIR`, so
//! `src/semantic/vendored.rs` can `include_str!` them without a committed
//! copy under `schemas/vendored/` (PLAT-906).
//!
//! Cargo checks out the *entire* `filament-core-data` repository for a git
//! dependency; `cargo metadata`'s `packages[].manifest_path` names that
//! checkout (the same mechanism `crates/quire-rust-extraction`'s own
//! `tests/dependency_boundary.rs` already uses to inspect the resolved
//! graph). This build script walks up from the dependency crate's manifest
//! to the checkout root and copies the two source trees verbatim — no byte
//! is transformed, so the digests `tests/semantic_baseline.rs` asserts stay
//! the pinned ones.
//!
//! `schemas/vendored/semantic-core/0.1.0/` is **not** touched here: no
//! commit of `filament-core-data` carries both a Cargo package and the
//! 0.1.0-era schema bytes (the workspace was added in the same commit that
//! bumped semantic-core to 0.2.0), so that vintage has no dependency edge to
//! resolve through. See the PLAT-906 PR description.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The dependency this build script exists to locate. Never `use`d as a
/// library — see the `Cargo.toml` comment on the `[build-dependencies]`
/// entry — so its only trace in this crate is its name here and in the
/// manifest.
const FCD_PACKAGE: &str = "agent-ix-semantic-ir";

/// Every semantic-core 0.2.0 schema file `src/semantic/vendored.rs` embeds,
/// sorted by name; kept in lock-step with that file's own list.
/// `toolchain.json` is provenance, not a schema, and stays excluded (matching
/// the bundle-digest rule `tests/semantic_baseline.rs` checks against).
const SEMANTIC_CORE_0_2_0_FILES: &[&str] = &[
    "ClauseLanguage.json",
    "ClauseRef.json",
    "ConstraintDecl.json",
    "ConstraintKeyword.json",
    "DecimalPolicy.json",
    "DefaultDecl.json",
    "DefaultKind.json",
    "EdgeCategory.json",
    "EnumValue.json",
    "EnumValuesConstraint.json",
    "ExclusiveMaxConstraint.json",
    "ExclusiveMinConstraint.json",
    "FieldDecl.json",
    "FormatConstraint.json",
    "Identifier.json",
    "KernelScalar.json",
    "MaxConstraint.json",
    "MaxLengthConstraint.json",
    "MinConstraint.json",
    "MinLengthConstraint.json",
    "Multiplicity.json",
    "NonEmptyConstraint.json",
    "OperationDecl.json",
    "PatternConstraint.json",
    "RelationDecl.json",
    "SemanticId.json",
    "SourceLocus.json",
    "TypeRef.json",
    "UniqueConstraint.json",
    "UnitSymbol.json",
];

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");

    let manifest_dir = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("cargo always sets CARGO_MANIFEST_DIR"),
    );
    let out_dir = PathBuf::from(
        env::var_os("OUT_DIR").expect("cargo always sets OUT_DIR for a build script"),
    );
    let cargo = env::var_os("CARGO").expect("cargo always sets CARGO");

    let checkout_root = fcd_checkout_root(&cargo, &manifest_dir);

    let common_schema = checkout_root.join("schema/semantic/v1/common.schema.json");
    copy_or_panic(&common_schema, &out_dir.join("common.schema.json"));

    let core_dir = checkout_root.join("packages/semantic-core/generated/json-schema");
    let dest_dir = out_dir.join("semantic-core-0.2.0");
    std::fs::create_dir_all(&dest_dir)
        .unwrap_or_else(|error| panic!("creating {}: {error}", dest_dir.display()));
    for name in SEMANTIC_CORE_0_2_0_FILES {
        copy_or_panic(&core_dir.join(name), &dest_dir.join(name));
    }
}

/// The checkout root of the pinned `filament-core-data` git dependency, found
/// via `cargo metadata`.
fn fcd_checkout_root(cargo: &std::ffi::OsStr, manifest_dir: &Path) -> PathBuf {
    let output = Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--manifest-path"])
        .arg(manifest_dir.join("Cargo.toml"))
        .output()
        .expect("cargo metadata must run");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("cargo metadata emits JSON");
    let packages = metadata["packages"]
        .as_array()
        .expect("cargo metadata emits a packages array");
    let package = packages
        .iter()
        .find(|package| package["name"] == FCD_PACKAGE)
        .unwrap_or_else(|| panic!("{FCD_PACKAGE} is not in the resolved dependency graph"));
    let manifest_path = PathBuf::from(
        package["manifest_path"]
            .as_str()
            .expect("cargo metadata reports manifest_path as a string"),
    );
    // manifest_path = <checkout>/crates/semantic-ir/Cargo.toml
    manifest_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .unwrap_or_else(|| {
            panic!(
                "unexpected manifest_path shape: {}",
                manifest_path.display()
            )
        })
        .to_path_buf()
}

fn copy_or_panic(source: &Path, destination: &Path) {
    println!("cargo:rerun-if-changed={}", source.display());
    std::fs::copy(source, destination).unwrap_or_else(|error| {
        panic!(
            "copying {} to {}: {error}",
            source.display(),
            destination.display()
        )
    });
}
