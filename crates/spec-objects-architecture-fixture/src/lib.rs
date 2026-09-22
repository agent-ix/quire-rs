//! The real, published `@agent-ix/spec-objects-architecture` module
//! (FR-075-AC-13, TC-1872/TC-1873), resolved through the package manager at
//! build time instead of vendored into this repository.
//!
//! `build.rs` fetches the package via `npm pack` and unpacks its
//! `manifest.yaml`, `schemas/`, and `skeletons/` into `OUT_DIR/module/`.
//! `module_dir()` just points at that directory — no bytes here are
//! authored or copied, so the tests that load this module assert against
//! the real thing, not a stand-in built to match the tool's own output.
//!
//! The fetch is allowed to fail (agent-ix/quire-rs#488: the package isn't
//! published anywhere CI can reach it) — `build.rs` doesn't panic on that,
//! it surfaces it here via [`is_available`]/[`unavailable_reason`]. A test
//! that needs the real module must check [`is_available`] first; calling
//! [`module_dir`] when it's `false` returns a path with nothing fetched
//! into it.

use std::path::Path;

/// The published version this crate resolves. `build.rs` is the single
/// source of truth (its own `VERSION` constant, matching FR-075-AC-13's
/// prose); it emits `SPEC_OBJECTS_ARCHITECTURE_VERSION` via
/// `cargo:rustc-env`, and this reads it back rather than repeating the
/// literal.
pub const VERSION: &str = env!("SPEC_OBJECTS_ARCHITECTURE_VERSION");

/// Directory containing the fetched module's `manifest.yaml`, `schemas/`,
/// and `skeletons/`, laid out exactly as the npm package publishes them.
///
/// Populated only when [`is_available`] is `true` — check that first.
pub fn module_dir() -> &'static Path {
    Path::new(concat!(env!("OUT_DIR"), "/module"))
}

/// Whether `build.rs` actually fetched the module. `false` when
/// `@agent-ix/spec-objects-architecture@`[`VERSION`] wasn't reachable at
/// the configured npm registry — see [`unavailable_reason`] for why, and
/// agent-ix/quire-rs#488 for the tracked gap.
pub fn is_available() -> bool {
    env!("SPEC_OBJECTS_ARCHITECTURE_AVAILABLE") == "true"
}

/// Why the module wasn't fetched, when [`is_available`] is `false`.
/// `None` when it was fetched successfully.
pub fn unavailable_reason() -> Option<&'static str> {
    if is_available() {
        None
    } else {
        Some(env!("SPEC_OBJECTS_ARCHITECTURE_UNAVAILABLE_REASON"))
    }
}
