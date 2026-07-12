//! Static boundary audit for the canonical Filament extraction module
//! (NFR-020-AC-1 / FR-045-CON-1/3, TC-690).
//!
//! The engine SHALL be a pure document-semantics boundary: it must not reach
//! into persistence, IPC, network/auth, CloudManager sync, workspace watchers,
//! or embeddings. This asserts that statically over the module source so the
//! constraint cannot silently regress.

const FILAMENT_SRC: &str = include_str!("../src/filament.rs");

/// Tokens that would indicate the extraction boundary reaching into a runtime
/// service surface it must stay free of.
const FORBIDDEN: &[&str] = &[
    "PGlite",
    "pglite",
    "Electron",
    "electron",
    "reqwest",
    "CloudManager",
    "cloud_manager",
    "workspace_watch",
    "embedding",
    "Embedding",
    "std::net",
    "TcpStream",
    "std::fs",
    "tokio",
];

#[test]
fn tc690_extraction_module_has_no_forbidden_runtime_surface() {
    for needle in FORBIDDEN {
        assert!(
            !FILAMENT_SRC.contains(needle),
            "src/filament.rs must not reference {needle:?} (NFR-020-AC-1 / FR-045-CON-1..3): \
             the extraction boundary stays free of persistence, IPC, network/auth, \
             CloudManager sync, watchers, and embeddings"
        );
    }
}
