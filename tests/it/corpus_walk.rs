//! Repository-walk behavior (FR-024).

use ix_trace_rs::trace;

#[cfg(unix)]
#[trace("FR-024-AC-5")]
#[test]
fn symlink_loop_is_reported_without_being_followed() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("create repository root");
    let root = temp.path().join("spec");
    std::fs::create_dir_all(root.join("nested")).expect("create nested directory");
    std::fs::write(
        root.join("NOTE-001.md"),
        "---\nid: NOTE-001\ntype: note\n---\n# Note\n",
    )
    .expect("write document");
    let link = root.join("nested/back-to-root");
    symlink(&root, &link).expect("create loop to an ancestor");

    let loaded = quire_rs::load_repo(&root);
    assert_eq!(loaded.documents.len(), 1);
    assert_eq!(
        loaded
            .diagnostics
            .iter()
            .filter(|diagnostic| matches!(diagnostic, quire_rs::Diagnostic::SymlinkLoop { path } if path == &link))
            .count(),
        1,
        "the loop is skipped and reported once: {:#?}",
        loaded.diagnostics
    );
}
