"""Trace checks for the abi3 wheel verification workflow."""

from pathlib import Path

import pytest
import yaml


@pytest.mark.trace("FR-023-AC-6", "NFR-016-AC-3")
def test_same_abi3_wheel_is_smoke_imported_on_two_python_minors():
    """Check CI reuses one built wheel for imports on two Python minors.

    Trace: FR-023-AC-6, NFR-016-AC-3
    """
    workflow_path = (
        Path(__file__).resolve().parents[2] / ".github/workflows/wheels.yml"
    )
    workflow = yaml.safe_load(workflow_path.read_text(encoding="utf-8"))
    jobs = workflow["jobs"]

    smoke_job = jobs.get("abi3-smoke")
    assert smoke_job is not None, "workflow must provide the cross-minor smoke job"
    assert smoke_job["needs"] == "build"
    assert smoke_job["strategy"]["matrix"]["python-version"] == ["3.12", "3.13"]
    assert "abi3-smoke" in jobs["publish"]["needs"]

    linux_target = "x86_64-unknown-linux-gnu"
    targets = [
        entry["target"] for entry in jobs["build"]["strategy"]["matrix"]["include"]
    ]
    assert linux_target in targets

    upload = next(
        step
        for step in jobs["build"]["steps"]
        if step.get("uses") == "actions/upload-artifact@v4"
    )
    assert upload["with"]["name"] == "wheels-${{ matrix.target }}"
    assert upload["with"]["path"] == "dist/*.whl"

    steps = smoke_job["steps"]
    download = next(
        step for step in steps if step.get("uses") == "actions/download-artifact@v4"
    )
    assert download["with"]["name"] == "wheels-x86_64-unknown-linux-gnu"
    assert download["with"]["path"] == "dist"

    setup_python = next(
        step for step in steps if step.get("uses") == "actions/setup-python@v5"
    )
    assert setup_python["with"]["python-version"] == "${{ matrix.python-version }}"

    install = next(
        step for step in steps if step.get("name") == "Install the built wheel"
    )
    assert install["run"] == "python -m pip install --no-index --find-links dist quire"

    smoke_import = next(
        step for step in steps if step.get("name") == "Smoke-import quire"
    )
    assert "import quire" in smoke_import["run"]
