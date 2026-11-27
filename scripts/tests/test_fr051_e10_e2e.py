"""Run the pinned cross-language E-10 emitter and evidence integration."""

from __future__ import annotations

import pathlib
import sys

import pytest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[1]))

import test_fr051_e10  # noqa: E402


@pytest.mark.trace("FR-051-AC-33", "FR-051-AC-48")
def test_pinned_cross_language_e10_emitters_bind_only_passing_obligations(tmp_path, monkeypatch):
    secret_names = (
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "CARGO_REGISTRIES_CRATES_IO_TOKEN",
        "GITHUB_TOKEN",
        "IX_CONFIG_ROOT",
        "NODE_AUTH_TOKEN",
        "NPM_TOKEN",
        "PIP_INDEX_URL",
    )
    for name in secret_names:
        monkeypatch.setenv(name, "must-not-reach-child-processes")
    child_env = test_fr051_e10.task_environment(tmp_path, tmp_path / "npmrc", tmp_path / "npm-cache")
    assert not set(secret_names).intersection(child_env)
    assert test_fr051_e10.main() == 0
