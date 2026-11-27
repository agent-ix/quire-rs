#!/usr/bin/env python3
"""Run FR-051 E-10 emitter fixtures using only task-local pinned tools."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET
from pathlib import Path
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "tests/fixtures/fr051-e10"
RS_REV = "8a0ec153a097036e257cdc8e0ba391bf4dea93d1"
PY_REV = "508f18379a7f28e8be1429b93b97e633a9766ae7"
TS_PACKAGE = "@agent-ix/ix-trace-ts@0.1.1"
QUOIN_PACKAGE = "@agent-ix/quoin@0.37.0"
PROCESS_REV = "28c8d2ccf03a477ac07c9955de59c7c2a873238a"
PROCESS_URL = "https://github.com/agent-ix/spec-artifacts-process"
RS_URL = "https://github.com/agent-ix/ix-trace-rs"
PY_URL = "https://github.com/agent-ix/ix-trace-py"
NPM_REGISTRY = "https://registry.npmjs.org/"
LOCKED_BUILD = os.environ.get("LOCKED_BUILD") or shutil.which("locked-build")
OWNER = "peter@terra:codex-features"


def run(args: list[str], *, env: dict[str, str], cwd: Path, expect: int = 0) -> str:
    proc = subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True)
    if proc.returncode != expect:
        raise RuntimeError(
            f"command exited {proc.returncode}, expected {expect}: {args!r}\n"
            f"stdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )
    return proc.stdout + proc.stderr


def locked(args: list[str], *, env: dict[str, str], cwd: Path, expect: int = 0) -> str:
    if not LOCKED_BUILD:
        raise RuntimeError("locked-build is required; install it or set LOCKED_BUILD")
    return run(
        [
            LOCKED_BUILD,
            "run",
            "--class",
            "block",
            "--owner",
            OWNER,
            "--ticket",
            "PLAT-1982",
            "--",
            *args,
        ],
        env=env,
        cwd=cwd,
        expect=expect,
    )


def task_environment(work: Path, config: Path, cache: Path) -> dict[str, str]:
    """Build a clean child environment without forwarding host credentials."""
    home = work / "home"
    temp = work / "tmp"
    home.mkdir(exist_ok=True)
    temp.mkdir(exist_ok=True)
    env = {
        "PATH": os.environ.get("PATH", "/usr/bin:/bin"),
        "HOME": str(home),
        "TMPDIR": str(temp),
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "TZ": "UTC",
        "RUSTUP_HOME": os.environ.get("RUSTUP_HOME", str(Path.home() / ".rustup")),
        "npm_config_userconfig": str(config),
        "npm_config_registry": NPM_REGISTRY,
        "npm_config_cache": str(cache),
        "PYTHONDONTWRITEBYTECODE": "1",
    }
    if "RUSTUP_TOOLCHAIN" in os.environ:
        env["RUSTUP_TOOLCHAIN"] = os.environ["RUSTUP_TOOLCHAIN"]
    return env


def check_junit(
    path: Path,
    *,
    expected_failure: bool,
    language: str,
    expected: dict[tuple[str, str], set[str]],
    failure_symbol: tuple[str, str] | None = None,
) -> None:
    root = ET.parse(path).getroot()
    cases = root.findall(".//testcase")
    if not cases:
        raise AssertionError(f"{language}: no testcase entries in {path}")
    expected_outcome = "fail" if expected_failure else "pass"
    outcomes: dict[tuple[str, str], str] = {}
    for case in cases:
        name = case.get("name")
        classname = case.get("classname")
        file = case.get("file")
        if not name or name != classname or not file:
            raise AssertionError(
                f"{language}: bad testcase identity: name={name!r}, "
                f"classname={classname!r}, file={file!r}"
            )
        property_nodes = case.findall("./property") + case.findall("./properties/property")
        props = {p.get("name"): p.get("value") or p.text for p in property_nodes}
        ids = {item for item in (props.get("trace") or "").split(",") if item}
        key = (file, name)
        if key not in expected:
            raise AssertionError(f"{language}: unexpected testcase identity {key}")
        if ids != expected[key]:
            raise AssertionError(f"{language}: {key} has trace ids {ids}, expected {expected[key]}")
        outcome = "fail" if case.find("failure") is not None or case.find("error") is not None else "pass"
        if key in outcomes:
            raise AssertionError(f"{language}: duplicate testcase identity {key}")
        outcomes[key] = outcome
    if set(outcomes) != set(expected):
        raise AssertionError(f"{language}: testcase identities {set(outcomes)} != {set(expected)}")
    if expected_failure:
        if failure_symbol is None or outcomes.get(failure_symbol) != "fail":
            raise AssertionError(f"{language}: expected failing entry {failure_symbol}, got {outcomes}")
        if sum(outcome == "fail" for outcome in outcomes.values()) != 1:
            raise AssertionError(f"{language}: intentional run outcomes were {outcomes}")
    elif any(outcome != expected_outcome for outcome in outcomes.values()):
        raise AssertionError(f"{language}: passing run outcomes were {outcomes}")


def record(
    quoin: Path,
    process_module: Path,
    junit: Path,
    root: Path,
    *,
    tool: str,
    expected_bound_id: str,
    failure_id: str,
    designated_symbol: str,
    expected_failure: bool,
    env: dict[str, str],
) -> dict:
    repo = root / f"record-{junit.stem}"
    evidence = repo / "spec/evidence"
    evidence.mkdir(parents=True)
    shutil.copy2(ROOT / "spec/evidence/suites.md", evidence / "suites.md")
    functional = repo / "spec/functional"
    functional.mkdir(parents=True)
    shutil.copy2(ROOT / "spec/functional/FR-051-source-symbol-extraction.md", functional)
    command = [
            str(quoin),
            "evidence",
            "record",
            "--repo",
            str(repo),
            "--suite",
            "SUITE-005",
            "--kind",
            "Integration",
            "--timestamp",
            datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
            "--config-root",
            str(root / "quoin-config"),
            "--no-project-config",
            "--module",
            str(process_module),
            "--adapter",
            "junit",
            "--tool",
            tool,
            "--results",
            str(junit),
            "--json",
        ]
    proc = subprocess.run(command, cwd=repo, env=env, text=True, capture_output=True)
    if proc.returncode != 0:
        raise RuntimeError(
            f"command exited {proc.returncode}: {command!r}\n"
            f"stdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )
    if not proc.stdout.strip():
        raise AssertionError(f"Quoin returned no JSON stdout; stderr was:\n{proc.stderr}")
    result = json.loads(proc.stdout)
    run_path = Path(result["run_path"])
    if not run_path.is_absolute():
        run_path = repo / run_path
    record_data = json.loads(run_path.read_text())
    entries = record_data.get("entries", [])
    expected_entries: dict[str, tuple[str, set[str]]] = {}
    for case in ET.parse(junit).getroot().findall(".//testcase"):
        symbol = case.get("name")
        property_nodes = case.findall("./property") + case.findall("./properties/property")
        properties = {node.get("name"): node.get("value") or node.text for node in property_nodes}
        trace_ids = {item for item in (properties.get("trace") or "").split(",") if item}
        outcome = "fail" if case.find("failure") is not None or case.find("error") is not None else "pass"
        if symbol is None or symbol in expected_entries:
            raise AssertionError(f"JUnit has absent or duplicate symbol {symbol!r}")
        expected_entries[symbol] = (outcome, trace_ids)
    actual_entries = {
        entry.get("symbol"): (entry.get("outcome"), set(entry.get("traceIds", [])))
        for entry in entries
    }
    if actual_entries != expected_entries:
        raise AssertionError(f"Quoin entries differ from JUnit: expected {expected_entries}, got {actual_entries}")
    target = [entry for entry in entries if entry.get("symbol") == designated_symbol]
    if len(target) != 1:
        raise AssertionError(f"Quoin record has no unique {designated_symbol!r} entry: {record_data}")
    target = target[0]
    expected_outcome = "fail" if expected_failure else "pass"
    if target.get("outcome") != expected_outcome:
        raise AssertionError(f"Quoin recorded {designated_symbol} as {target.get('outcome')!r}, expected {expected_outcome!r}")
    expected_trace_id = failure_id if expected_failure else expected_bound_id
    if expected_trace_id not in target.get("traceIds", []):
        raise AssertionError(f"Quoin entry {designated_symbol} lost trace id {expected_trace_id}: {target}")
    raw_bound = result.get("bound", [])
    bound_ids = {
        value if isinstance(value, str) else value.get("obligation") or value.get("id")
        for value in raw_bound
    }
    if expected_failure:
        if failure_id in bound_ids:
            raise AssertionError(f"failed run bound {failure_id}: {raw_bound}")
    elif expected_bound_id not in bound_ids:
        raise AssertionError(f"passing run did not bind {expected_bound_id}: {raw_bound}; result={result}")
    binding_path = repo / "spec/evidence/bindings.json"
    binding_data = json.loads(binding_path.read_text())
    registry_ids = {binding.get("obligation") for binding in binding_data.get("bindings", [])}
    if expected_failure and failure_id in registry_ids:
        raise AssertionError(f"failed run wrote a binding for {failure_id}: {binding_data}")
    if not expected_failure and expected_bound_id not in registry_ids:
        raise AssertionError(f"passing run did not persist binding for {expected_bound_id}: {binding_data}")
    return {"result": result, "record": record_data, "repo": repo}


def main() -> int:
    work = Path(tempfile.mkdtemp(prefix="quire-fr051-e10-"))
    try:
        cache = work / "npm-cache"
        config = work / "npmrc"
        config.write_text(
            f"registry={NPM_REGISTRY}\n@agent-ix:registry={NPM_REGISTRY}\n"
            f"cache={cache}\naudit=false\nfund=false\nlegacy-peer-deps=true\n"
        )
        base_env = task_environment(work, config, cache)

        quoin_prefix = work / "quoin"
        quoin_prefix.mkdir()
        run(
            ["npm", "install", "--ignore-scripts", "--prefix", str(quoin_prefix), "--registry", NPM_REGISTRY, QUOIN_PACKAGE],
            cwd=work,
            env=base_env,
        )
        quoin = quoin_prefix / "node_modules/.bin/quoin"

        process = work / "process"
        run(["git", "clone", "--quiet", "--filter=blob:none", PROCESS_URL, str(process)], cwd=work, env=base_env)
        run(["git", "-C", str(process), "checkout", "--quiet", PROCESS_REV], cwd=work, env=base_env)
        process_module = process / "spec_artifacts_process"

        tools = work / "tools"
        cargo_target = work / "cargo-target"
        cargo_home = work / "cargo-home"
        tools.mkdir()
        cargo_home.mkdir()
        build_env = {
            **base_env,
            "CARGO_HOME": str(cargo_home),
            "CARGO_TARGET_DIR": str(cargo_target),
            "CARGO_BUILD_JOBS": "4",
        }
        locked(
            [
                "cargo",
                "install",
                "--locked",
                "--git",
                RS_URL,
                "--rev",
                RS_REV,
                "--root",
                str(tools),
                "ix-trace-run",
            ],
            env=build_env,
            cwd=work,
        )
        ix_trace_run = tools / "bin/ix-trace-run"

        py_env_dir = work / "python-venv"
        python313 = shutil.which("python3.13")
        if python313 is None:
            raise RuntimeError("Python 3.13 is required by pinned ix-trace-py revision")
        run([python313, "-m", "venv", str(py_env_dir)], cwd=work, env=base_env)
        py = py_env_dir / "bin/python"
        pip_env = {**base_env, "PIP_CACHE_DIR": str(work / "pip-cache")}
        run([str(py), "-m", "pip", "install", "-r", str(FIXTURES / "python/requirements.txt")], cwd=work, env=pip_env)
        run([str(py), "-m", "pip", "install", "--no-deps", f"git+{PY_URL}@{PY_REV}"], cwd=work, env=pip_env)

        ts = work / "typescript"
        shutil.copytree(FIXTURES / "typescript", ts)
        run(
            ["npm", "ci", "--prefix", str(ts), "--registry", NPM_REGISTRY],
            cwd=ts,
            env=base_env,
        )

        fixture_root = work / "fixtures"
        rust = fixture_root / "rust"
        shutil.copytree(FIXTURES / "rust", rust)
        if not (FIXTURES / "rust/Cargo.lock").is_file():
            raise RuntimeError("Rust fixture Cargo.lock is missing; regenerate it with the documented locked-build command")
        shutil.copy2(FIXTURES / "rust/Cargo.lock", rust / "Cargo.lock")

        outputs = work / "junit"
        outputs.mkdir()
        runs: list[tuple[str, Path, str]] = []
        rust_expected = {
            ("tests/x.rs", "top_level"): {"FR-051-AC-32"},
            ("tests/x.rs", "inner::nested"): {"FR-051-AC-32", "FR-051-AC-33"},
            ("tests/x.rs", "param"): {"FR-051-AC-32"},
            ("src/foo/bar.rs", "tests::t"): {"FR-051-AC-32"},
        }
        rust_junit = outputs / "rust-pass.xml"
        locked(
            [str(ix_trace_run), "--locked", "--manifest-path", str(rust / "Cargo.toml"), "--features", "emit", "--output", str(rust_junit), "--", "--test-threads=4"],
            env=build_env,
            cwd=rust,
        )
        check_junit(rust_junit, expected_failure=False, language="Rust", expected=rust_expected)
        runs.append(("Rust", rust_junit, f"ix-trace-run {RS_REV}"))

        rust_fail = outputs / "rust-fail.xml"
        locked(
            [str(ix_trace_run), "--locked", "--manifest-path", str(rust / "Cargo.toml"), "--features", "emit", "--output", str(rust_fail), "--", "--test-threads=4"],
            env={**build_env, "IX_TRACE_E2E_FAIL": "1"},
            cwd=rust,
            expect=1,
        )
        check_junit(rust_fail, expected_failure=True, language="Rust", expected=rust_expected, failure_symbol=("tests/x.rs", "inner::nested"))
        runs.append(("Rust-failing", rust_fail, f"ix-trace-run {RS_REV}"))

        py_fixture = FIXTURES / "python"
        python_expected = {
            ("tests/test_x.py", "test_top"): {"FR-051-AC-47"},
            ("tests/test_x.py", "TestC.test_m"): {"FR-051-AC-47"},
            ("tests/test_x.py", "TestOuter.TestInner.test_n"): {"FR-051-AC-47"},
            ("tests/test_x.py", "test_param"): {"FR-051-AC-47", "FR-051-AC-48"},
        }
        py_pass = outputs / "python-pass.xml"
        run([str(py), "-m", "pytest", str(py_fixture), "--junitxml", str(py_pass)], cwd=py_fixture, env={**pip_env, "QUIRE_TRACE_EMIT": "1"})
        check_junit(py_pass, expected_failure=False, language="Python", expected=python_expected)
        runs.append(("Python", py_pass, f"pytest 9.0.2 + ix-trace-py {PY_REV}"))

        py_fail = outputs / "python-fail.xml"
        run([str(py), "-m", "pytest", str(py_fixture), "--junitxml", str(py_fail)], cwd=py_fixture, env={**pip_env, "QUIRE_TRACE_EMIT": "1", "IX_TRACE_E2E_FAIL": "1"}, expect=1)
        check_junit(py_fail, expected_failure=True, language="Python", expected=python_expected, failure_symbol=("tests/test_x.py", "test_param"))
        runs.append(("Python-failing", py_fail, f"pytest 9.0.2 + ix-trace-py {PY_REV}"))

        for name, fail in (("typescript-pass", False), ("typescript-fail", True)):
            env = {**base_env, "QUIRE_TRACE_EMIT": "1"}
            if fail:
                env["IX_TRACE_E2E_FAIL"] = "1"
            run(["npm", "test", "--prefix", str(ts)], cwd=ts, env=env, expect=1 if fail else 0)
            junit = ts / "junit.xml"
            if not junit.exists():
                # The reporter defaults to junit.xml in the project root.
                raise AssertionError("Vitest trace reporter did not write junit.xml")
            shutil.copy2(junit, outputs / f"{name}.xml")
            typescript_expected = {
                ("tests/x.test.ts", "adds"): {"FR-051-AC-47"},
                ("tests/x.test.ts", "adds %s"): {"FR-051-AC-47", "FR-051-AC-48"},
            }
            check_junit(
                outputs / f"{name}.xml",
                expected_failure=fail,
                language="TypeScript",
                expected=typescript_expected,
                failure_symbol=("tests/x.test.ts", "adds %s") if fail else None,
            )
            runs.append(("TypeScript-failing" if fail else "TypeScript", outputs / f"{name}.xml", f"{TS_PACKAGE} + vitest 4.1.2"))

        for label, junit, tool in runs:
            language = label.removesuffix("-failing")
            expected_bound_id = "FR-051-AC-32" if language == "Rust" else "FR-051-AC-47"
            failure_id = "FR-051-AC-33" if language == "Rust" else "FR-051-AC-48"
            designated_symbol = {"Rust": "inner::nested", "Python": "test_param", "TypeScript": "adds %s"}[language]
            recorded = record(
                quoin,
                process_module,
                junit,
                work,
                tool=tool,
                expected_bound_id=expected_bound_id,
                failure_id=failure_id,
                designated_symbol=designated_symbol,
                expected_failure=label.endswith("-failing"),
                env=base_env,
            )
            print(f"{label}: recorded {recorded['result']['kind']} at {recorded['result']['run_path']}")

        print("FR-051 E-10 emitter runs passed; six task-local E-10 records were verified")
        shutil.rmtree(work)
        return 0
    except Exception as exc:
        print(f"FR-051 E-10 failed: {exc}", file=sys.stderr)
        print(f"task-local state retained at {work}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
