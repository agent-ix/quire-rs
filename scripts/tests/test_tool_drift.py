from __future__ import annotations

import pathlib
import shutil
import subprocess

import pytest

ROOT = pathlib.Path(__file__).resolve().parents[2]


def fixture(tmp_path: pathlib.Path) -> pathlib.Path:
    root = tmp_path / "repo"
    (root / "scripts/audits").mkdir(parents=True)
    (root / "bench").mkdir()
    (root / ".github/workflows").mkdir(parents=True)
    (root / "requirements").mkdir()
    (root / "quality").mkdir()
    shutil.copy(ROOT / "scripts/audits/check_tool_drift.sh", root / "scripts/audits")
    shutil.copy(ROOT / "scripts/check_engine.py", root / "scripts")
    shutil.copy(ROOT / "scripts/export_measurements.py", root / "scripts")
    shutil.copy(ROOT / "scripts/validate_spec.py", root / "scripts")
    shutil.copy(ROOT / "bench/manifest.json", root / "bench")
    shutil.copy(ROOT / "quality/validation-stack-lock.json", root / "quality")
    shutil.copy(ROOT / "quality/spec-validation-exclusions.json", root / "quality")
    (root / "examples").mkdir()
    shutil.copy(ROOT / "examples/spec_validate.rs", root / "examples")
    (root / "rust-toolchain.toml").write_text('[toolchain]\nchannel = "1.94.1"\n')
    (root / "Cargo.toml").write_text('[package]\nversion = "0.46.0"\n')
    (root / "Makefile").write_text(
        "BENCH_MODULE ?= ../spec-artifacts-process/spec_artifacts_process\n"
        "VALIDATION_PROCESS_ROOT ?= ../spec-artifacts-process\n"
        "VALIDATION_ISO_ROOT ?= ../spec-artifacts-iso\n"
        "validate:\n\tpython3 scripts/validate_spec.py \\\n"
        '\t\t--process-root "$(VALIDATION_PROCESS_ROOT)" \\\n'
        '\t\t--iso-root "$(VALIDATION_ISO_ROOT)"\n'
        "test:\n\t$(CARGO) test --locked\n"
    )
    (root / "requirements/ci.txt").write_text(
        "helper==1.2.3 --hash=sha256:" + "a" * 64 + "\n"
    )
    (root / ".github/workflows/ci.yml").write_text(
        "jobs:\n  test:\n    runs-on: ubuntu-24.04\n    steps:\n"
        "      - uses: actions/checkout@" + "b" * 40 + " # v4\n"
        "      - uses: actions/checkout@" + "b" * 40 + " # v4\n"
        "        with:\n"
        "          repository: agent-ix/spec-artifacts-process\n"
        "          ref: e6ea5151b59a55d7ce0d43f1581cbe276f750e04\n"
        "      - uses: actions/checkout@" + "b" * 40 + " # v4\n"
        "        with:\n"
        "          repository: agent-ix/spec-artifacts-iso\n"
        "          ref: a60ee12d735976081849f60a38d603fb5494b015\n"
        "      - run: >-\n"
        "          python3 scripts/validate_spec.py\n"
        "          --process-root .ci/spec-artifacts-process\n"
        "          --iso-root .ci/spec-artifacts-iso\n"
    )
    return root


def run(root: pathlib.Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["bash", str(root / "scripts/audits/check_tool_drift.sh")],
        text=True,
        capture_output=True,
        check=False,
    )


def mutate(path: pathlib.Path, old: str, new: str) -> None:
    text = path.read_text()
    assert old in text
    path.write_text(text.replace(old, new, 1))


def test_valid_exact_stack_passes(tmp_path: pathlib.Path) -> None:
    assert run(fixture(tmp_path)).returncode == 0


@pytest.mark.parametrize(
    ("relative", "old", "new", "reason"),
    [
        ("rust-toolchain.toml", "1.94.1", "stable", "exact x.y.z"),
        (".github/workflows/ci.yml", "b" * 40, "v4", "full SHA"),
        (".github/workflows/ci.yml", "ubuntu-24.04", "ubuntu-latest", "latest"),
        ("Makefile", "test --locked", "test", "--locked"),
        ("requirements/ci.txt", " --hash=sha256:" + "a" * 64, "", "hash-locked"),
        ("Cargo.toml", "0.46.0", "0.33.0", "0.46.0"),
        (
            "scripts/check_engine.py",
            '        "--locked",\n',
            "",
            "programmatic Cargo build is not --locked",
        ),
        (
            "scripts/export_measurements.py",
            "build_engine(consumer, release=True)",
            "build_engine(consumer, release=False)",
            "canonical release profile",
        ),
        (
            "scripts/export_measurements.py",
            'value.get("buildProfile") != "release"',
            'value.get("ignoredProfile") != "release"',
            "attested release profile",
        ),
        (
            "scripts/export_measurements.py",
            'allowed_overlay_paths=("spec/evidence/measurements",)',
            'allowed_overlay_paths=("",)',
            "limit source overlays",
        ),
        (
            "scripts/export_measurements.py",
            "evidence overlay is not a linear, merge-free chain",
            "evidence overlay is accepted",
            "reject nonlinear evidence overlays",
        ),
        (
            "bench/manifest.json",
            '"source_name": "quoin-benchmark-corpus"',
            '"source_name": "quoin"',
            "separate the Quoin benchmark corpus",
        ),
        (
            "Makefile",
            "../spec-artifacts-process/spec_artifacts_process",
            "../moving-module/spec_artifacts_process",
            "manifest-pinned module path",
        ),
        (
            "quality/validation-stack-lock.json",
            "e6ea5151b59a55d7ce0d43f1581cbe276f750e04",
            "a" * 40,
            "reviewed exact schema-provider stack",
        ),
        (
            "quality/spec-validation-exclusions.json",
            "spec/assurance/MP-208-authoring-tag-rate.md",
            "spec/assurance/MP-999-unreviewed.md",
            "exactly the deferred Phase-7 artifacts",
        ),
        (
            ".github/workflows/ci.yml",
            "ref: e6ea5151b59a55d7ce0d43f1581cbe276f750e04",
            "ref: " + "a" * 40,
            "does not equal the lock",
        ),
        (
            "scripts/validate_spec.py",
            'env["IX_SCHEMA_PATH"] = ""',
            'env["IX_SCHEMA_PATH"] = os.environ.get("IX_SCHEMA_PATH", "")',
            "cleared legacy module path",
        ),
    ],
)
def test_each_drift_class_fails_closed(
    tmp_path: pathlib.Path, relative: str, old: str, new: str, reason: str
) -> None:
    root = fixture(tmp_path)
    mutate(root / relative, old, new)
    result = run(root)
    assert result.returncode != 0
    assert reason in result.stderr
