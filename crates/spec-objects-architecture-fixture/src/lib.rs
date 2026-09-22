//! The real, published `@agent-ix/spec-objects-architecture` module
//! (FR-075-AC-13, TC-1872/TC-1873), resolved through the package manager at
//! build time instead of vendored into this repository.
//!
//! `build.rs` fetches the package via `npm pack` and unpacks its
//! `manifest.yaml`, `schemas/`, and `skeletons/` into `OUT_DIR/module/`.
//! `module_dir()` just points at that directory — no bytes here are
//! authored or copied, so the tests that load this module assert against
//! the real thing, not a stand-in built to match the tool's own output.

use std::path::Path;

/// The published version this crate resolves. `build.rs` is the single
/// source of truth (its own `VERSION` constant, matching FR-075-AC-13's
/// prose); it emits `SPEC_OBJECTS_ARCHITECTURE_VERSION` via
/// `cargo:rustc-env`, and this reads it back rather than repeating the
/// literal.
pub const VERSION: &str = env!("SPEC_OBJECTS_ARCHITECTURE_VERSION");

/// Directory containing the fetched module's `manifest.yaml`, `schemas/`,
/// and `skeletons/`, laid out exactly as the npm package publishes them.
pub fn module_dir() -> &'static Path {
    Path::new(concat!(env!("OUT_DIR"), "/module"))
}
