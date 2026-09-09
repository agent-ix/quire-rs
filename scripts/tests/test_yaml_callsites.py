from __future__ import annotations

import json
import pathlib
import shutil
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[2]


def fixture(tmp_path: pathlib.Path) -> pathlib.Path:
    root = tmp_path / "repo"
    for relative in ["scripts/audits", "quality", "src/parser", "fuzz/fuzz_targets"]:
        (root / relative).mkdir(parents=True, exist_ok=True)
    shutil.copy(ROOT / "scripts/audits/check_yaml_callsites.py", root / "scripts/audits")
    (root / "Cargo.toml").write_text(
        '[dependencies]\nserde_yaml = { package = "yaml_serde", version = "=0.10.7" }\n'
    )
    (root / "fuzz/Cargo.toml").write_text(
        '[dependencies]\nserde_yaml = { package = "yaml_serde", version = "=0.10.7" }\n'
    )
    (root / "src/parser/frontmatter.rs").write_text(
        'fn parse(s: &str) { let _ = serde_yaml::from_str::<serde_json::Value>(s); }\n'
    )
    (root / "quality/yaml-callsite-census.json").write_text(
        json.dumps(
            {
                "schema": "quire-yaml-callsite-census/v1",
                "required_production_classes": ["frontmatter"],
                "callsites": [
                    {
                        "path": "src/parser/frontmatter.rs",
                        "scope": "production",
                        "count": 1,
                        "classes": ["frontmatter"],
                    }
                ],
            }
        )
    )
    return root


def run(root: pathlib.Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["python3", str(root / "scripts/audits/check_yaml_callsites.py")],
        text=True,
        capture_output=True,
        check=False,
    )


def test_complete_census_passes(tmp_path: pathlib.Path) -> None:
    result = run(fixture(tmp_path))
    assert result.returncode == 0, result.stderr


def test_unclassified_callsite_fails(tmp_path: pathlib.Path) -> None:
    root = fixture(tmp_path)
    (root / "src/extra.rs").write_text("fn x() { let _ = serde_yaml::from_str::<()>(\"\"); }\n")
    result = run(root)
    assert result.returncode != 0
    assert "unclassified callsites" in result.stderr


def test_count_drift_fails(tmp_path: pathlib.Path) -> None:
    root = fixture(tmp_path)
    path = root / "src/parser/frontmatter.rs"
    path.write_text(path.read_text() + "fn y() { let _ = serde_yaml::from_str::<()>(\"\"); }\n")
    result = run(root)
    assert result.returncode != 0
    assert "call counts changed" in result.stderr


def test_direct_package_name_fails(tmp_path: pathlib.Path) -> None:
    root = fixture(tmp_path)
    path = root / "src/parser/frontmatter.rs"
    path.write_text(path.read_text().replace("serde_yaml::", "yaml_serde::"))
    result = run(root)
    assert result.returncode != 0
    assert "keep serde_yaml alias" in result.stderr
