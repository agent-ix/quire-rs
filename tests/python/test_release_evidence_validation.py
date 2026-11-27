"""Release evidence must use the committed Markdown suite contract."""

import importlib.util
import json
from pathlib import Path
import subprocess
import sys

import pytest

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "scripts"
SPEC = importlib.util.spec_from_file_location(
    "release_evidence_validation", SCRIPTS / "release_evidence_validation.py"
)
HELPER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(HELPER)


def release_records():
    """Supply a passing symbol for each of the six declared obligations."""
    bindings = [
        {"suite": "SUITE-001", "obligation": f"FR-095-AC-{i}", "symbols": [f"test_{i}"]}
        for i in range(1, 7)
    ]
    run = {
        "schemaVersion": 1,
        "suite": "SUITE-001",
        "tool": "ix-trace-run 0.1.0+8a0ec153",
        "evidenceKind": "Unit",
        "timestamp": "2026-11-26T00:00:00.123456789Z",
        "entries": [{"symbol": f"test_{i}", "outcome": "pass"} for i in range(1, 7)],
    }
    return {"schemaVersion": 1, "bindings": bindings}, run


class TestCommittedReleaseRegistry:
    """Check actual registry parsing and fail-closed evidence admission."""

    @pytest.mark.trace("FR-095-AC-6")
    def test_committed_registry_accepts_complete_passing_bindings(self):
        """Accept the committed registry's line-wrapped declaration.

        Trace: FR-095-AC-6
        """
        store, run = release_records()
        registry = (ROOT / "spec/evidence/suites.md").read_text()
        assert HELPER.missing_required_release_bindings(registry, store, run) == []

    @pytest.mark.trace("FR-095-AC-6")
    def test_missing_or_nonpassing_obligation_is_refused(self):
        """Preserve required evidence when a binding or its pass is absent.

        Trace: FR-095-AC-6
        """
        registry = (ROOT / "spec/evidence/suites.md").read_text()
        store, run = release_records()
        store["bindings"].pop()
        assert HELPER.missing_required_release_bindings(registry, store, run) == [
            "FR-095-AC-6"
        ]
        store, run = release_records()
        run["entries"][-1]["outcome"] = "fail"
        assert HELPER.missing_required_release_bindings(registry, store, run) == [
            "FR-095-AC-6 (no passing supporting test: test_6)"
        ]

    @pytest.mark.trace("FR-095-AC-6")
    @pytest.mark.parametrize(
        "registry",
        [
            "# Suites\n",
            "SUITE-001 SHALL bind each acceptance criterion in Quire-rs FR-095 (AC-1 through AC-unknown)",
        ],
    )
    def test_registry_without_contract_is_refused(self, registry):
        """Reject a missing declaration instead of accepting an empty range.

        Trace: FR-095-AC-6
        """
        store, run = release_records()
        with pytest.raises(ValueError, match="does not declare"):
            HELPER.missing_required_release_bindings(registry, store, run)

    @pytest.mark.trace("FR-095-AC-6")
    def test_actual_registry_packages_and_verifies_offline(self, tmp_path):
        """Package and verify exact committed registry bytes with passing evidence.

        Trace: FR-095-AC-6
        """
        evidence = tmp_path / "spec/evidence"
        run_path = evidence / "runs/SUITE-001/synthetic.json"
        run_path.parent.mkdir(parents=True)
        registry = (ROOT / "spec/evidence/suites.md").read_bytes()
        committed = (ROOT / "spec/evidence/bindings.json").read_bytes()
        (evidence / "suites.md").write_bytes(registry)
        baseline = tmp_path / "committed.json"
        baseline.write_bytes(committed)
        store, run = release_records()
        (evidence / "bindings.json").write_text(json.dumps(store))
        run_path.write_text(json.dumps(run))
        junit = tmp_path / "junit.xml"
        junit.write_text('<testsuite tests="6" failures="0"/>')
        archive = tmp_path / "evidence.tar.gz"
        identity = [
            "--expected-tag",
            "v0.63.4",
            "--expected-commit",
            "a" * 40,
            "--tag-created-at",
            "1795564800",
        ]
        package = subprocess.run(
            [
                sys.executable,
                str(SCRIPTS / "package_release_evidence.py"),
                "--source-root",
                str(tmp_path),
                "--committed-bindings",
                str(baseline),
                "--run-record",
                str(run_path),
                "--junit-results",
                str(junit),
                "--archive",
                str(archive),
                *identity,
            ],
            capture_output=True,
            text=True,
        )
        assert package.returncode == 0, package.stderr
        (evidence / "bindings.json").write_bytes(committed)
        verified = tmp_path / "verified"
        verify = subprocess.run(
            [
                sys.executable,
                str(SCRIPTS / "verify_release_evidence.py"),
                "--source-root",
                str(tmp_path),
                "--archive",
                str(archive),
                "--output-dir",
                str(verified),
                *identity,
            ],
            capture_output=True,
            text=True,
        )
        assert verify.returncode == 0, verify.stderr
        assert (verified / "suites.md").read_bytes() == registry
        assert json.loads((verified / "bindings.json").read_text()) == store
