#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

python3 - "$ROOT" <<'PY'
from __future__ import annotations

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

for workflow in sorted((root / ".github/workflows").glob("*.yml")):
    text = workflow.read_text(encoding="utf-8")
    for line_no, line in enumerate(text.splitlines(), 1):
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

requirements = (root / "requirements/ci.txt").read_text(encoding="utf-8")
for line_no, line in enumerate(requirements.splitlines(), 1):
    if line and not line.startswith("#") and not re.search(r"==[^ ]+ --hash=sha256:[0-9a-f]{64}$", line):
        errors.append(f"requirements/ci.txt:{line_no}: dependency is not exact and hash-locked")

makefile = (root / "Makefile").read_text(encoding="utf-8")
for line_no, line in enumerate(makefile.splitlines(), 1):
    if re.search(r"\$\(CARGO\)\s+(?:build|check|clippy|run|test)\b", line) and "--locked" not in line:
        errors.append(f"Makefile:{line_no}: canonical Cargo resolution is not --locked")

if errors:
    print("tool-drift audit failed:", file=sys.stderr)
    for error in errors:
        print(f"- {error}", file=sys.stderr)
    raise SystemExit(1)
print("tool-drift audit: exact toolchain, action, runner, manifest, and Cargo locks verified")
PY
