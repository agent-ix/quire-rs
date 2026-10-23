//! FR-050-AC-52: the ratchet run refuses to collect without an attestation.

use std::path::Path;
use std::process::Command;

use ix_trace_rs::trace;

#[trace("FR-050-AC-52")]
#[test]
fn quire_bench_refuses_to_collect_without_a_verification_stack() {
    let output = Command::new(env!("CARGO_BIN_EXE_quire-bench"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8_lossy(&output.stderr).trim(),
        "bench: --verification-stack is required to collect"
    );
}

#[trace("FR-050-AC-52")]
#[test]
fn make_bench_targets_refuse_without_verification_stack() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for target in ["bench", "bench-update"] {
        let output = Command::new("make")
            .arg("-C")
            .arg(&root)
            .arg("--no-print-directory")
            .arg(target)
            .arg("VERIFICATION_STACK=")
            .output()
            .unwrap();
        assert!(!output.status.success(), "{target}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("VERIFICATION_STACK= is required"),
            "{target}: {stdout}"
        );
    }
}
