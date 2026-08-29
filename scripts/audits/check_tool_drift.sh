#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

python3 - "$ROOT" <<'PY'
from __future__ import annotations

import json
import pathlib
import re
import sys

root = pathlib.Path(sys.argv[1])
errors: list[str] = []

toolchain = (root / "rust-toolchain.toml").read_text(encoding="utf-8")
match = re.search(r'^channel\s*=\s*"([^"]+)"', toolchain, re.MULTILINE)
if not match or not re.fullmatch(r"\d+\.\d+\.\d+", match.group(1)):
    errors.append("rust-toolchain.toml must pin an exact x.y.z toolchain")

manifest = (root / "Cargo.toml").read_text(encoding="utf-8")
version = re.search(r'^version\s*=\s*"([^"]+)"', manifest, re.MULTILINE)
if not version or version.group(1) != "0.46.0":
    errors.append("Cargo.toml must declare the guarded post-v0.45 version 0.46.0")

benchmark = json.loads((root / "bench/manifest.json").read_text(encoding="utf-8"))
benchmark_inputs = [*benchmark.get("corpora", []), benchmark.get("module_source")]
for entry in benchmark_inputs:
    if not isinstance(entry, dict):
        errors.append("bench/manifest.json must declare module_source")
        continue
    path = (root / str(entry.get("path", ""))).resolve()
    try:
        path.relative_to(root)
        external = False
    except ValueError:
        external = True
    if (external or entry is benchmark.get("module_source")) and (
        entry.get("identity") != "sha"
        or not re.fullmatch(r"[0-9a-f]{40}", str(entry.get("pinned_sha", "")))
    ):
        errors.append(
            f"bench/manifest.json input {entry.get('name', '<unnamed>')} must use a full immutable SHA"
        )
quoin_input = next(
    (entry for entry in benchmark.get("corpora", []) if entry.get("name") == "quoin"),
    None,
)
if quoin_input is None or quoin_input.get("source_name") != "quoin-benchmark-corpus":
    errors.append(
        "bench/manifest.json must separate the Quoin benchmark corpus from the Quoin producer source"
    )

for workflow in sorted((root / ".github/workflows").glob("*.yml")):
    text = workflow.read_text(encoding="utf-8")
    for line_no, line in enumerate(text.splitlines(), 1):
        if line.lstrip().startswith("#") or re.match(r"\s*-\s+name:", line):
            continue
        use = re.search(r"\buses:\s*[^\s@]+@([^\s#]+)", line)
        if use and not re.fullmatch(r"[0-9a-f]{40}", use.group(1)):
            errors.append(f"{workflow.relative_to(root)}:{line_no}: action is not pinned by full SHA")
        if re.search(r"runs-on:\s*[^#\n]*latest", line):
            errors.append(f"{workflow.relative_to(root)}:{line_no}: mutable runner label `latest`")
        if re.search(r"dtolnay/rust-toolchain@", line):
            # The action implementation and the installed compiler are separate
            # drift surfaces. The following `with.toolchain` must name the latter.
            window = "\n".join(text.splitlines()[line_no:line_no + 5])
            if not re.search(r"toolchain:\s*(?:\d+\.\d+\.\d+|nightly-\d{4}-\d{2}-\d{2})", window):
                errors.append(f"{workflow.relative_to(root)}:{line_no}: Rust action lacks exact toolchain")
        if "tool:" in line and re.search(r"cargo-(?:deny|mutants|fuzz)\s*$", line):
            errors.append(f"{workflow.relative_to(root)}:{line_no}: installed Cargo utility lacks exact version")
        if "pip install" in line and not line.lstrip().startswith("#") and "--require-hashes" not in line and "--no-index" not in line:
            noncanonical = 'GOVERNED_EVIDENCE: "false"' in text
            exact_direct = re.search(r"pip install\s+[A-Za-z0-9_.-]+==[A-Za-z0-9_.+-]+\s*$", line)
            if not noncanonical or not exact_direct:
                errors.append(f"{workflow.relative_to(root)}:{line_no}: Python helper install is not hash-locked")
        if re.search(r"cargo(?:\s+\+\S+)?\s+(?:bench|build|check|clippy|test)\b", line) and "--locked" not in line:
            errors.append(f"{workflow.relative_to(root)}:{line_no}: Cargo resolution is not --locked")
        if re.search(r"cargo\s+deny\b", line) and "--locked" not in line:
            errors.append(f"{workflow.relative_to(root)}:{line_no}: cargo-deny resolution is not --locked")
        if re.search(r"cargo\s+mutants\b", line) and "--cargo-arg=--locked" not in line:
            errors.append(f"{workflow.relative_to(root)}:{line_no}: cargo-mutants does not pass --locked to Cargo")

requirements = (root / "requirements/ci.txt").read_text(encoding="utf-8")
for line_no, line in enumerate(requirements.splitlines(), 1):
    if line and not line.startswith("#") and not re.search(r"==[^ ]+ --hash=sha256:[0-9a-f]{64}$", line):
        errors.append(f"requirements/ci.txt:{line_no}: dependency is not exact and hash-locked")

makefile = (root / "Makefile").read_text(encoding="utf-8")
if "BENCH_MODULE ?= ../spec-artifacts-process/spec_artifacts_process" not in makefile:
    errors.append("Makefile BENCH_MODULE must select the manifest-pinned module path")
for line_no, line in enumerate(makefile.splitlines(), 1):
    if re.search(r"\$\(CARGO\)\s+(?:build|check|clippy|run|test)\b", line) and "--locked" not in line:
        errors.append(f"Makefile:{line_no}: canonical Cargo resolution is not --locked")
    if re.search(r"\$\(CARGO\)\s+deny\b", line) and "--locked" not in line:
        errors.append(f"Makefile:{line_no}: cargo-deny resolution is not --locked")
    if re.search(r"(?:^|\s)cargo(?:\s+\+\S+)?\s+(?:bench|build|check|clippy|run|test)\b", line) and "--locked" not in line and not line.lstrip().startswith(("#", "@echo")):
        errors.append(f"Makefile:{line_no}: direct Cargo resolution is not --locked")

check_engine = (root / "scripts/check_engine.py").read_text(encoding="utf-8")
if not re.search(r'"cargo",\s*"build",\s*"--locked"', check_engine):
    errors.append("scripts/check_engine.py programmatic Cargo build is not --locked")

exporter = (root / "scripts/export_measurements.py").read_text(encoding="utf-8")
if 'build_engine(consumer, release=True)' not in exporter:
    errors.append("measurement exporter must build the canonical release profile")
if 'value.get("buildProfile") != "release"' not in exporter:
    errors.append("measurement exporter must require the attested release profile")
if 'allowed_overlay_paths=("spec/evidence/measurements",)' not in exporter:
    errors.append("measurement exporter must limit source overlays to governed evidence")
if 'evidence overlay is not a linear, merge-free chain' not in exporter:
    errors.append("measurement exporter must reject nonlinear evidence overlays")

if errors:
    print("tool-drift audit failed:", file=sys.stderr)
    for error in errors:
        print(f"- {error}", file=sys.stderr)
    raise SystemExit(1)
print("tool-drift audit: exact toolchain, action, runner, manifest, and Cargo locks verified")
PY
