//! Fetch the published safety module so downstream tests can use its actual schemas.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use build_npm_fetch::FetchError;

const VERSION: &str = "0.9.0";
const REGISTRY: &str = "https://registry.npmjs.org";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rustc-env=SPEC_OBJECTS_SAFETY_VERSION={VERSION}");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR set by cargo"));

    match fetch_spec_objects_safety(VERSION, &out_dir) {
        Ok(()) => {
            println!("cargo:rustc-env=SPEC_OBJECTS_SAFETY_AVAILABLE=true");
            println!("cargo:rustc-env=SPEC_OBJECTS_SAFETY_UNAVAILABLE_REASON=");
        }
        Err(reason) => {
            let reason = reason.replace(['\n', '\r'], " ");
            println!(
                "cargo:warning=spec-objects-safety-fixture: could not fetch \
                 @agent-ix/spec-objects-safety@{VERSION} ({reason}); tests that need its \
                 published schemas will skip when the registry is unavailable."
            );
            println!("cargo:rustc-env=SPEC_OBJECTS_SAFETY_AVAILABLE=false");
            println!("cargo:rustc-env=SPEC_OBJECTS_SAFETY_UNAVAILABLE_REASON={reason}");
        }
    }
}

fn fetch_spec_objects_safety(version: &str, out_dir: &Path) -> Result<(), String> {
    let dest = out_dir.join("module");
    let marker = out_dir.join(".module-fetched");
    let identity = format!("version={version}\nregistry={REGISTRY}\n");
    if build_npm_fetch::cache_marker_matches(&marker, &identity) {
        return Ok(());
    }

    build_npm_fetch::clear_npm_package_cache("@agent-ix/spec-objects-safety", version, out_dir)
        .unwrap_or_else(|e| panic!("clear stale npm package cache: {e}"));

    let package_dir = match build_npm_fetch::try_fetch_npm_package_from_registry(
        "@agent-ix/spec-objects-safety",
        version,
        REGISTRY,
        out_dir,
    ) {
        Ok(dir) => dir,
        Err(e @ (FetchError::Spawn { .. } | FetchError::PackFailed { .. })) => {
            return Err(e.to_string());
        }
        Err(e) => panic!("{e}"),
    };

    if dest.exists() {
        fs::remove_dir_all(&dest)
            .unwrap_or_else(|e| panic!("remove stale {}: {e}", dest.display()));
    }
    let schemas = package_dir.join("schemas");
    fs::create_dir_all(dest.join("schemas"))
        .unwrap_or_else(|e| panic!("create {}: {e}", dest.join("schemas").display()));
    for name in [
        "HazardAssessment.json",
        "FailureAnalysis.json",
        "Hazard.json",
    ] {
        fs::copy(schemas.join(name), dest.join("schemas").join(name))
            .unwrap_or_else(|e| panic!("copy authentic safety schema {name}: {e}"));
    }

    let _ = fs::remove_dir_all(out_dir.join("npm-packages"));
    fs::write(&marker, identity)
        .unwrap_or_else(|e| panic!("write marker {}: {e}", marker.display()));
    Ok(())
}
