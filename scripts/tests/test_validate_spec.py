from __future__ import annotations

import importlib.util
import pathlib

import pytest

ROOT = pathlib.Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts/validate_spec.py"
SPEC = importlib.util.spec_from_file_location("validate_spec", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
validate_spec = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(validate_spec)


def test_validation_environment_replaces_both_ambient_paths(
    tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("IX_FILAMENT_MODULES_PATH", "/ambient/preferred")
    monkeypatch.setenv("IX_SCHEMA_PATH", "/ambient/legacy")
    isolated = tmp_path / "isolated"
    env = validate_spec.validation_environment(isolated)
    assert env["IX_FILAMENT_MODULES_PATH"] == str(isolated)
    assert env["IX_SCHEMA_PATH"] == ""
