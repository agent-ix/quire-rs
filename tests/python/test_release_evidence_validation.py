"""Trace the shared release evidence regression checks in the Python harness."""

from pathlib import Path
import subprocess
import sys

import pytest


@pytest.mark.trace("FR-095-AC-6")
def test_committed_release_registry_preflight():
    """Run the same actual-registry checks used before release evidence builds.

    Trace: FR-095-AC-6
    """
    checks = Path(__file__).with_name("release_evidence_checks.py")
    result = subprocess.run(
        [sys.executable, str(checks)], capture_output=True, text=True
    )
    assert result.returncode == 0, result.stdout + result.stderr
