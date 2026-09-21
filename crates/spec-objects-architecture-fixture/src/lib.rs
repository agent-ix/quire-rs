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

/// The published version this crate resolves. Keep in sync with the
/// `VERSION` constant in `build.rs` and with FR-075-AC-13's prose.
pub const VERSION: &str = "0.7.0";

/// Directory containing the fetched module's `manifest.yaml`, `schemas/`,
/// and `skeletons/`, laid out exactly as the npm package publishes them.
pub fn module_dir() -> &'static Path {
    Path::new(concat!(env!("OUT_DIR"), "/module"))
}
