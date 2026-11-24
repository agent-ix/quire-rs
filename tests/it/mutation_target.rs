//! Checks the local mutation target's command boundary without running mutants.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Output};

use ix_trace_rs::trace;

fn invoke_mutation_target(status: i32) -> (tempfile::TempDir, Output, String) {
    let temp = tempfile::tempdir().expect("temporary directory");
    let fake_cargo = temp.path().join("fake-cargo");
    let capture = temp.path().join("invocation.txt");
    let output_dir = temp.path().join("mutation-output");
    fs::write(
        &fake_cargo,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$CAPTURE_FILE\"\nprintf 'CARGO_BUILD_JOBS=%s\\n' \"$CARGO_BUILD_JOBS\" >> \"$CAPTURE_FILE\"\nexit \"$FAKE_CARGO_STATUS\"\n",
    )
    .expect("write fake cargo executable");
    fs::set_permissions(&fake_cargo, fs::Permissions::from_mode(0o755))
        .expect("make fake cargo executable");

    let output = Command::new("make")
        .args(["--no-print-directory", "mutants"])
        .arg(format!("CARGO={}", fake_cargo.display()))
        .arg(format!("MUTANTS_OUTPUT_DIR={}", output_dir.display()))
        .env("CARGO_BUILD_JOBS", "3")
        .env("CAPTURE_FILE", &capture)
        .env("FAKE_CARGO_STATUS", status.to_string())
        .output()
        .expect("run make against fake cargo");
    let captured = fs::read_to_string(capture).expect("captured cargo invocation");
    (temp, output, captured)
}

#[test]
#[trace("NFR-013-AC-2")]
fn local_mutation_target_delegates_to_behavioral_scoped_cargo_mutants() {
    let (_temp, output, captured) = invoke_mutation_target(0);
    assert!(
        output.status.success(),
        "make failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let args: Vec<_> = captured.lines().collect();
    for expected in [
        "mutants",
        "--jobs=3",
        "--file",
        "src/parser/**",
        "src/extract/**",
        "src/corpus/resolve.rs",
        "--cargo-arg=--locked",
    ] {
        assert!(args.contains(&expected), "missing {expected:?} in {args:?}");
    }
    assert!(!args.contains(&"--check"), "compile-only mode was passed");
    assert!(
        !args.contains(&"--in-place"),
        "serial in-place mode was passed"
    );
    assert!(args.contains(&"CARGO_BUILD_JOBS=1"));
}

#[test]
#[trace("NFR-013-AC-5")]
fn local_mutation_target_propagates_a_nonzero_cargo_mutants_result() {
    let (_temp, output, _captured) = invoke_mutation_target(37);
    assert!(!output.status.success(), "make hid fake cargo's failure");
}
