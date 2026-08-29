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

validation_lock = json.loads(
    (root / "quality/validation-stack-lock.json").read_text(encoding="utf-8")
)
expected_validation_stack = {
    "schemaVersion": "quire-validation-stack-v1",
    "repositories": {
        "spec-artifacts-iso": {
            "modulePath": "spec_artifacts_iso",
            "remote": "https://github.com/agent-ix/spec-artifacts-iso",
            "remoteRef": "refs/remotes/origin/epic/264-reference-only-targets",
            "revision": "a6b1c70be8c22e9f7cb432e4410b7a3a280d0217",
        },
        "spec-artifacts-process": {
            "modulePath": "spec_artifacts_process",
            "remote": "https://github.com/agent-ix/spec-artifacts-process",
            "remoteRef": "refs/remotes/origin/epic/264-assurance-integration",
            "revision": "61a20e010d5e758f52864ad3152ccdb304a39d27",
        },
    },
}
if validation_lock != expected_validation_stack:
    errors.append("quality/validation-stack-lock.json must retain the reviewed exact schema-provider stack")

exclusions = json.loads(
    (root / "quality/spec-validation-exclusions.json").read_text(encoding="utf-8")
)
expected_exclusion_paths = {
    "spec/assurance/AP-201-detection-minting.md",
    "spec/assurance/MP-201-binding-read.md",
    "spec/assurance/MP-202-backed-trace.md",
    "spec/assurance/MP-203-dead-tags.md",
    "spec/assurance/MP-204-minting-repositories.md",
    "spec/assurance/MP-205-specific-properties.md",
    "spec/assurance/MP-206-silent-zero.md",
    "spec/assurance/MP-207-skeptic-suspicion.md",
    "spec/assurance/MP-208-authoring-tag-rate.md",
}
entries = exclusions.get("entries") if isinstance(exclusions, dict) else None
if (
    not isinstance(exclusions, dict)
    or set(exclusions) != {"schemaVersion", "entries"}
    or exclusions.get("schemaVersion") != "quire-spec-validation-exclusions-v1"
    or not isinstance(entries, list)
    or any(not isinstance(entry, dict) or set(entry) != {"path", "reason"} for entry in entries)
    or {entry.get("path") for entry in entries} != expected_exclusion_paths
    or any("Phase-7 engineering-assurance artifact" not in str(entry.get("reason")) for entry in entries)
):
    errors.append("quality/spec-validation-exclusions.json must name exactly the deferred Phase-7 artifacts")

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
for required in (
    "VALIDATION_PROCESS_ROOT ?= ../spec-artifacts-process",
    "VALIDATION_ISO_ROOT ?= ../spec-artifacts-iso",
    "python3 scripts/validate_spec.py",
    '--process-root "$(VALIDATION_PROCESS_ROOT)"',
    '--iso-root "$(VALIDATION_ISO_ROOT)"',
):
    if required not in makefile:
        errors.append(f"Makefile locked validation wiring is missing {required!r}")
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

validator = (root / "scripts/validate_spec.py").read_text(encoding="utf-8")
for required, message in (
    ('"status", "--porcelain=v1", "--untracked-files=all"', "dirty provider rejection"),
    ('"remote", "get-url", "origin"', "provider-origin verification"),
    ('"merge-base", "--is-ancestor"', "provider remote-reachability verification"),
    ('["cargo", "run", "--locked", "--quiet", "--example", "spec_validate"]', "locked validator build"),
    ('env["IX_FILAMENT_MODULES_PATH"] = str(module_root)', "isolated preferred module path"),
    ('env["IX_SCHEMA_PATH"] = ""', "cleared legacy module path"),
):
    if required not in validator:
        errors.append(f"scripts/validate_spec.py lacks {message}")

validator_example = (root / "examples/spec_validate.rs").read_text(encoding="utf-8")
for required, message in (
    ('root.join("quality/spec-validation-exclusions.json")', "governed exclusion file"),
    ("exclusions.remove(relative)", "exact-path exclusion consumption"),
    ('"AssuranceProfile" | "MeasurementPlan"', "Phase-7 type boundary"),
    ("governed validation exclusion does not name a loaded document", "stale exclusion rejection"),
):
    if required not in validator_example:
        errors.append(f"examples/spec_validate.rs lacks {message}")

ci = (root / ".github/workflows/ci.yml").read_text(encoding="utf-8")
for name, entry in expected_validation_stack["repositories"].items():
    if f"repository: agent-ix/{name}" not in ci:
        errors.append(f"CI does not checkout locked schema provider {name}")
    if f"ref: {entry['revision']}" not in ci:
        errors.append(f"CI schema-provider ref for {name} does not equal the lock")
for required in (
    "python3 scripts/validate_spec.py",
    "--process-root .ci/spec-artifacts-process",
    "--iso-root .ci/spec-artifacts-iso",
):
    if required not in ci:
        errors.append(f"CI locked validation wiring is missing {required!r}")

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
print("tool-drift audit: exact toolchain, action, runner, manifest, validation-stack, and Cargo locks verified")
PY
