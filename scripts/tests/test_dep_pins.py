from __future__ import annotations

import pathlib
import shutil
import subprocess

import pytest

ROOT = pathlib.Path(__file__).resolve().parents[2]


def fixture(tmp_path: pathlib.Path) -> pathlib.Path:
    root = tmp_path / "repo"
    (root / "scripts/audits").mkdir(parents=True)
    shutil.copy(ROOT / "scripts/audits/check_dep_pins.sh", root / "scripts/audits")
    (root / "Cargo.toml").write_text(
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\n"
        "[dependencies]\n"
        "serde_yaml = { package = \"yaml_serde\", version = \"=0.10.7\" }\n"
        "serde_json = \"^1\"\n"
        "indexmap = { version = \"^2\", features = [\"serde\"] }\n"
        "jsonschema = { version = \"~0.18\", default-features = false }\n"
    )
    return root


def run(root: pathlib.Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["bash", str(root / "scripts/audits/check_dep_pins.sh")],
        text=True,
        capture_output=True,
        check=False,
    )


def mutate(path: pathlib.Path, old: str, new: str) -> None:
    text = path.read_text()
    assert old in text
    path.write_text(text.replace(old, new, 1))


def test_exact_yaml_package_and_policy_pins_pass(tmp_path: pathlib.Path) -> None:
    result = run(fixture(tmp_path))
    assert result.returncode == 0, result.stderr


@pytest.mark.parametrize(
    ("old", "new", "reason"),
    [
        ('package = "yaml_serde"', 'package = "serde_yaml"', "yaml_serde package"),
        ('package = "yaml_serde", ', "", "yaml_serde package"),
        ('version = "=0.10.7"', 'version = "^0.10.7"', "exact version =0.10.7"),
        ('version = "=0.10.7"', 'version = "=0.10.6"', "exact version =0.10.7"),
        ('version = "=0.10.7"', 'version = "*"', "exact version =0.10.7"),
        ('version = "~0.18"', 'version = "^0.18"', "jsonschema ~0.18"),
        ('serde_json = "^1"', 'serde_json = "*"', "wildcard"),
    ],
)
def test_each_load_bearing_pin_mutation_fails_closed(
    tmp_path: pathlib.Path, old: str, new: str, reason: str
) -> None:
    root = fixture(tmp_path)
    mutate(root / "Cargo.toml", old, new)
    result = run(root)
    assert result.returncode != 0
    assert reason in result.stderr
