//! The published `@agent-ix/spec-objects-safety` schemas used by downstream
//! integration tests. `build.rs` fetches the package at a pinned version and
//! places the complete unmodified schema document set under
//! `OUT_DIR/module/schemas`; no schema bytes are authored or copied into this
//! repository.

use std::path::Path;

/// The pinned published module version fetched by `build.rs`.
pub const VERSION: &str = env!("SPEC_OBJECTS_SAFETY_VERSION");

/// The fetched module directory. Check [`is_available`] before using this path.
fn module_dir() -> &'static Path {
    Path::new(concat!(env!("OUT_DIR"), "/module"))
}

/// Directory containing every authentic schema document from the fetched
/// safety module, including the sibling documents referenced by its roots.
pub fn schemas_dir() -> std::path::PathBuf {
    module_dir().join("schemas")
}

/// Whether the published package was fetched successfully.
pub fn is_available() -> bool {
    env!("SPEC_OBJECTS_SAFETY_AVAILABLE") == "true"
}

/// Why the package could not be fetched, if it is unavailable.
pub fn unavailable_reason() -> Option<&'static str> {
    if is_available() {
        None
    } else {
        Some(env!("SPEC_OBJECTS_SAFETY_UNAVAILABLE_REASON"))
    }
}
