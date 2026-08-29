#!/usr/bin/env python3
"""Freeze and adjudicate the `tag-on-non-binding-symbol` population (#355).

The diagnostic's ecosystem count is a census, not a precision estimate.  This
harness preserves the candidate rows emitted by the real engine, samples the
function and module-scope strata deterministically, and reports ambiguity
instead of silently dropping it.

Collection builds the engine from the consuming workspace, reads identity from
its payload, and aborts on an unreadable repository::

    python3 scripts/tag_precision_sample.py collect \
      --module ../spec-artifacts-process/spec_artifacts_process \
      --frame reports/2026-08-27-tag-non-binding-frame.json \
      --rulings reports/2026-08-27-tag-non-binding-rulings.yaml

After every sampled row has a ruling, render the decision record::

    python3 scripts/tag_precision_sample.py report \
      --frame reports/2026-08-27-tag-non-binding-frame.json \
      --rulings reports/2026-08-27-tag-non-binding-rulings.yaml \
      --output reports/2026-08-27-tag-non-binding-precision.md
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import pathlib
import re
import subprocess
import sys
from datetime import date

import yaml

from check_engine import Drift, build_engine, reported_engine
from corpus import markdown_files, repos, source_files
from sweep_coverage import coverage

REASON = "tag-on-non-binding-symbol"
SEED = "agent-ix/quire-rs#355-v1"
RULINGS = ("authored-tag", "prose-citation", "other", "ambiguous", "unresolved")
SAMPLE_QUOTAS = {"production-symbol": 80, "module-scope": 30}
MESSAGE = re.compile(
    r"^trace id `(?P<trace_id>[^`]+)` is written on `(?P<symbol>[^`]+)` "
    r"at .+?:\d+, a (?P<kind>[^ ]+) — .* The form `(?P<form>[^`]+)` matched,"
)


class CalibrationError(RuntimeError):
    """The calibration cannot publish a complete or comparable result."""


def digest(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def measurement_input_digest(repo: pathlib.Path) -> str:
    measured = hashlib.sha256()
    paths = sorted({*source_files(repo), *markdown_files(repo)})
    for path in paths:
        try:
            content = path.read_bytes()
        except OSError as error:
            raise CalibrationError(
                f"{repo.name}/{path.relative_to(repo)}: {error}"
            ) from error
        measured.update(str(path.relative_to(repo)).encode("utf-8"))
        measured.update(b"\0")
        measured.update(content)
        measured.update(b"\0")
    return measured.hexdigest()


def repository_state(repo: pathlib.Path) -> tuple[str, bool | None, str, str | None]:
    revision = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "HEAD"],
        capture_output=True,
        text=True,
        check=False,
    )
    if revision.returncode:
        input_digest = measurement_input_digest(repo)
        return f"input:{input_digest}", None, "input-digest", input_digest
    status = subprocess.run(
        ["git", "-C", str(repo), "status", "--porcelain"],
        capture_output=True,
        text=True,
        check=False,
    )
    if status.returncode:
        raise CalibrationError(f"{repo.name}: cannot read git status")
    dirty = bool(status.stdout.strip())
    input_digest = measurement_input_digest(repo) if dirty else None
    provenance = "git-plus-input-digest" if dirty else "git"
    return revision.stdout.strip(), dirty, provenance, input_digest


def module_revision(module: pathlib.Path) -> str:
    done = subprocess.run(
        ["git", "-C", str(module), "rev-parse", "HEAD"],
        capture_output=True,
        text=True,
        check=False,
    )
    if done.returncode:
        raise CalibrationError(f"cannot resolve module revision for {module}")
    return done.stdout.strip()


def source_occurrences(
    repo: pathlib.Path, rel_path: str, diagnostic_line: int, trace_id: str
) -> tuple[list[dict], str]:
    path = repo / rel_path
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeDecodeError) as error:
        raise CalibrationError(
            f"{repo.name}/{rel_path}: cannot read context: {error}"
        ) from error
    if diagnostic_line < 1 or diagnostic_line > len(lines):
        raise CalibrationError(
            f"{repo.name}/{rel_path}:{diagnostic_line}: diagnostic locus is outside the file"
        )
    token = re.compile(rf"(?<![A-Za-z0-9_-]){re.escape(trace_id)}(?![A-Za-z0-9_-])")
    occurrences = []
    for index, text in enumerate(lines):
        if not token.search(text):
            continue
        start = max(0, index - 2)
        end = min(len(lines), index + 3)
        occurrences.append(
            {
                "line": index + 1,
                "text": text,
                "context": [
                    f"{cursor + 1}: {lines[cursor]}" for cursor in range(start, end)
                ],
            }
        )
    if not occurrences:
        raise CalibrationError(
            f"{repo.name}/{rel_path}:{diagnostic_line}: `{trace_id}` is absent from its source file"
        )
    measured = json.dumps(occurrences, sort_keys=True, separators=(",", ":"))
    return occurrences, digest(measured)


def candidate(
    repo: pathlib.Path,
    revision: str,
    dirty: bool | None,
    provenance_state: str,
    input_digest: str | None,
    finding: dict,
) -> dict:
    message = finding.get("message")
    match = MESSAGE.match(message or "")
    path = finding.get("path")
    line = finding.get("line")
    if not match or not isinstance(path, str) or not isinstance(line, int):
        raise CalibrationError(
            f"{repo.name}: {REASON} changed shape; refusing an unreviewable row: {finding!r}"
        )
    parsed = match.groupdict()
    if finding.get("value") != parsed["trace_id"]:
        raise CalibrationError(
            f"{repo.name}/{path}:{line}: value and message ids disagree"
        )
    stratum = "module-scope" if parsed["kind"] == "container" else "production-symbol"
    occurrences, context_digest = source_occurrences(
        repo, path, line, parsed["trace_id"]
    )
    identity = "\0".join(
        [repo.name, revision, path, str(line), parsed["trace_id"], parsed["symbol"]]
    )
    return {
        "id": digest(identity)[:20],
        "repo": repo.name,
        "repo_revision": revision,
        "repo_dirty": dirty,
        "repo_provenance": provenance_state,
        "repo_input_sha256": input_digest,
        "path": path,
        "line": line,
        "trace_id": parsed["trace_id"],
        "symbol": parsed["symbol"],
        "kind": parsed["kind"],
        "form": parsed["form"],
        "stratum": stratum,
        "occurrences": occurrences,
        "context_sha256": context_digest,
    }


def select_sample(
    rows: list[dict], quotas: dict[str, int], seed: str = SEED
) -> list[str]:
    selected: list[str] = []
    for stratum, quota in sorted(quotas.items()):
        available = [row for row in rows if row["stratum"] == stratum]
        if len(available) < quota:
            raise CalibrationError(
                f"{stratum}: sample asks for {quota} of only {len(available)} candidates"
            )
        ranked = sorted(
            available, key=lambda row: (digest(f"{seed}\0{row['id']}"), row["id"])
        )
        selected.extend(row["id"] for row in ranked[:quota])
    return sorted(selected)


def collect(args: argparse.Namespace) -> tuple[dict, dict]:
    root = pathlib.Path(args.root).expanduser().resolve()
    module = pathlib.Path(args.module).expanduser().resolve()
    consumer = pathlib.Path(args.consumer).expanduser().resolve()
    try:
        quire = build_engine(consumer, release=True)
    except Drift as error:
        raise CalibrationError(str(error)) from error

    enumerated = repos(root)
    excluded = set(args.exclude)
    selected_repos = [repo for repo in enumerated if repo.name not in excluded]
    rows: list[dict] = []
    engine_identity: tuple[str, str, tuple[str, ...]] | None = None
    repo_states = []
    for index, repo in enumerate(selected_repos, start=1):
        report = coverage(quire, repo, str(module))
        if not report or "error" in report:
            raise CalibrationError(
                f"{repo.name}: coverage failed: {(report or {}).get('error', 'no payload')}"
            )
        try:
            engine, capabilities = reported_engine(report)
        except Drift as error:
            raise CalibrationError(f"{repo.name}: {error}") from error
        current = (report["engine"]["cli"], engine, tuple(capabilities))
        if engine_identity is None:
            engine_identity = current
        elif current != engine_identity:
            raise CalibrationError(
                f"{repo.name}: engine identity changed during collection"
            )
        revision, dirty, provenance_state, input_digest = repository_state(repo)
        findings = [
            diagnostic
            for diagnostic in report.get("diagnostics", [])
            if diagnostic.get("reason") == REASON
        ]
        repo_states.append(
            {
                "repo": repo.name,
                "revision": revision,
                "dirty": dirty,
                "provenance": provenance_state,
                "input_sha256": input_digest,
                "candidates": len(findings),
            }
        )
        rows.extend(
            candidate(
                repo,
                revision,
                dirty,
                provenance_state,
                input_digest,
                finding,
            )
            for finding in findings
        )
        print(
            f"tag precision: {index}/{len(selected_repos)} {repo.name} ({len(findings)})",
            file=sys.stderr,
        )
    if engine_identity is None:
        raise CalibrationError("no repository produced a payload")
    rows.sort(key=lambda row: row["id"])
    if len({row["id"] for row in rows}) != len(rows):
        raise CalibrationError("candidate ids are not unique")
    quotas = {
        "production-symbol": args.sample_production,
        "module-scope": args.sample_module,
    }
    sampled = select_sample(rows, quotas, args.seed)
    population = {
        stratum: sum(row["stratum"] == stratum for row in rows) for stratum in quotas
    }
    frame = {
        "schema": "tag-non-binding-precision-frame-v1",
        "date": args.date,
        "reason": REASON,
        "seed": args.seed,
        "provenance": {
            "cli": engine_identity[0],
            "engine": engine_identity[1],
            "capabilities": list(engine_identity[2]),
            "module_revision": module_revision(module),
            "repositories_enumerated": len(enumerated),
            "repositories_scanned": len(selected_repos),
            "excluded": sorted(excluded),
            "repository_states": repo_states,
        },
        "population": population,
        "sample_quotas": quotas,
        "sample_ids": sampled,
        "candidates": rows,
    }
    frame_digest = digest(json.dumps(frame, sort_keys=True, separators=(",", ":")))
    rulings = {
        "schema": "tag-non-binding-precision-rulings-v1",
        "frame_sha256": frame_digest,
        "decision": "unresolved",
        "recall_effect": "unresolved",
        "locality_effect": "unresolved",
        "rulings": [
            {"id": row_id, "ruling": "unresolved", "rationale": ""}
            for row_id in sampled
        ],
    }
    return frame, rulings


def wilson(successes: int, total: int, z: float = 1.96) -> tuple[float, float]:
    if total == 0:
        return 0.0, 0.0
    p = successes / total
    denominator = 1 + z * z / total
    centre = (p + z * z / (2 * total)) / denominator
    margin = z * math.sqrt((p * (1 - p) + z * z / (4 * total)) / total) / denominator
    return centre - margin, centre + margin


def assess(frame: dict, rulings: dict) -> dict:
    expected_digest = digest(json.dumps(frame, sort_keys=True, separators=(",", ":")))
    if rulings.get("frame_sha256") != expected_digest:
        raise CalibrationError("rulings do not name this frame digest")
    sample_ids = set(frame.get("sample_ids", []))
    rows = rulings.get("rulings")
    if not isinstance(rows, list):
        raise CalibrationError("rulings must be a list")
    by_id = {}
    for row in rows:
        if not isinstance(row, dict) or row.get("id") in by_id:
            raise CalibrationError("every ruling must have one unique id")
        if row.get("ruling") not in RULINGS:
            raise CalibrationError(
                f"{row.get('id')}: unknown ruling {row.get('ruling')!r}"
            )
        if not str(row.get("rationale", "")).strip():
            raise CalibrationError(f"{row.get('id')}: ruling has no rationale")
        by_id[row["id"]] = row
    if set(by_id) != sample_ids:
        missing = sorted(sample_ids - set(by_id))
        extra = sorted(set(by_id) - sample_ids)
        raise CalibrationError(
            f"ruling/sample mismatch; missing={missing}, extra={extra}"
        )
    candidates = {row["id"]: row for row in frame["candidates"]}
    strata = {}
    unresolved = []
    for stratum, population in frame["population"].items():
        sample = [
            row_id
            for row_id in frame["sample_ids"]
            if candidates[row_id]["stratum"] == stratum
        ]
        counts = {ruling: 0 for ruling in RULINGS}
        for row_id in sample:
            ruling = by_id[row_id]["ruling"]
            counts[ruling] += 1
            if ruling == "unresolved":
                unresolved.append(row_id)
        lower = counts["authored-tag"] / len(sample)
        upper = (counts["authored-tag"] + counts["ambiguous"]) / len(sample)
        strata[stratum] = {
            "population": population,
            "sample": len(sample),
            "counts": counts,
            "precision_lower": lower,
            "precision_upper": upper,
            "wilson_95": wilson(counts["authored-tag"], len(sample)),
        }
    total_population = sum(frame["population"].values())
    weighted_lower = (
        sum(row["population"] * row["precision_lower"] for row in strata.values())
        / total_population
    )
    weighted_upper = (
        sum(row["population"] * row["precision_upper"] for row in strata.values())
        / total_population
    )
    sampling_lower = (
        sum(row["population"] * row["wilson_95"][0] for row in strata.values())
        / total_population
    )
    sampling_upper = (
        sum(
            row["population"]
            * wilson(
                row["counts"]["authored-tag"] + row["counts"]["ambiguous"],
                row["sample"],
            )[1]
            for row in strata.values()
        )
        / total_population
    )
    exact_loci = sum(
        any(
            occurrence["line"] == candidates[row_id]["line"]
            for occurrence in candidates[row_id]["occurrences"]
        )
        for row_id in frame["sample_ids"]
    )
    return {
        "strata": strata,
        "population": total_population,
        "sample": len(sample_ids),
        "precision_lower": weighted_lower,
        "precision_upper": weighted_upper,
        "sampling_95_lower": sampling_lower,
        "sampling_95_upper": sampling_upper,
        "exact_loci": exact_loci,
        "unresolved": unresolved,
        "decision": rulings.get("decision", "unresolved"),
        "recall_effect": rulings.get("recall_effect", "unresolved"),
        "locality_effect": rulings.get("locality_effect", "unresolved"),
    }


def percentage(value: float) -> str:
    return f"{value * 100:.1f}%"


def render(frame: dict, rulings: dict, result: dict) -> str:
    lines = [
        "# `tag-on-non-binding-symbol` precision calibration",
        "",
        f"- Frame date: `{frame['date']}`",
        f"- Candidate population: **{result['population']}**",
        f"- Deterministic sample: **{result['sample']}** (`{frame['seed']}`)",
        f"- Engine: `{frame['provenance']['cli']}` / `{frame['provenance']['engine']}`",
        f"- Module revision: `{frame['provenance']['module_revision']}`",
        f"- Decision: **{result['decision']}**",
        "",
        "## Adjudication",
        "",
        "| Stratum | Population | Sample | Authored tag | Prose citation | Other | Ambiguous | Unresolved | Precision interval |",
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for name, row in sorted(result["strata"].items()):
        counts = row["counts"]
        lines.append(
            f"| `{name}` | {row['population']} | {row['sample']} | "
            f"{counts['authored-tag']} | {counts['prose-citation']} | {counts['other']} | "
            f"{counts['ambiguous']} | {counts['unresolved']} | "
            f"{percentage(row['precision_lower'])}–{percentage(row['precision_upper'])} |"
        )
    lines += [
        "",
        "Ambiguity is included in the upper precision bound and never excluded from the denominator. "
        f"Population-weighted precision is **{percentage(result['precision_lower'])}–"
        f"{percentage(result['precision_upper'])}**; the conservative stratified 95% sampling "
        f"interval is **{percentage(result['sampling_95_lower'])}–"
        f"{percentage(result['sampling_95_upper'])}**.",
        "",
        "## Recall and locality",
        "",
        f"- Recall effect: {result['recall_effect']}",
        f"- Locality effect: {result['locality_effect']}",
        f"- Exact emitted-line locality in the sample: {result['exact_loci']}/{result['sample']}",
        "",
        "## Sample rulings",
        "",
        "| Candidate | Ruling | Rationale |",
        "|---|---|---|",
    ]
    candidates = {row["id"]: row for row in frame["candidates"]}
    by_id = {row["id"]: row for row in rulings["rulings"]}
    for row_id in frame["sample_ids"]:
        candidate_row = candidates[row_id]
        ruling = by_id[row_id]
        locus = (
            f"{candidate_row['repo']}/{candidate_row['path']}:{candidate_row['line']}"
        )
        rationale = str(ruling["rationale"]).replace("|", "\\|").replace("\n", " ")
        lines.append(f"| `{row_id}` `{locus}` | `{ruling['ruling']}` | {rationale} |")
    if result["unresolved"]:
        lines += [
            "",
            "## Explicit unresolved rows",
            "",
            *[f"- `{row_id}`" for row_id in result["unresolved"]],
        ]
    return "\n".join(lines) + "\n"


def write_json(path: pathlib.Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


def write_yaml(path: pathlib.Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(yaml.safe_dump(payload, sort_keys=False), encoding="utf-8")


def load_json(path: pathlib.Path) -> dict:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise CalibrationError(f"{path}: expected an object")
    return value


def load_yaml(path: pathlib.Path) -> dict:
    value = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise CalibrationError(f"{path}: expected a mapping")
    return value


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    collect_parser = subparsers.add_parser("collect")
    collect_parser.add_argument("--root", default="~/dev")
    collect_parser.add_argument("--consumer", default="../quire-cli")
    collect_parser.add_argument("--module", required=True)
    collect_parser.add_argument("--date", default=date.today().isoformat())
    collect_parser.add_argument("--seed", default=SEED)
    collect_parser.add_argument("--sample-production", type=int, default=80)
    collect_parser.add_argument("--sample-module", type=int, default=30)
    collect_parser.add_argument("--exclude", action="append", default=[])
    collect_parser.add_argument("--frame", required=True)
    collect_parser.add_argument("--rulings", required=True)
    report_parser = subparsers.add_parser("report")
    report_parser.add_argument("--frame", required=True)
    report_parser.add_argument("--rulings", required=True)
    report_parser.add_argument("--output", required=True)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        if args.command == "collect":
            frame_path = pathlib.Path(args.frame)
            rulings_path = pathlib.Path(args.rulings)
            if frame_path.exists() or rulings_path.exists():
                raise CalibrationError(
                    "collection refuses to overwrite a frozen frame or rulings"
                )
            frame, rulings = collect(args)
            write_json(frame_path, frame)
            write_yaml(rulings_path, rulings)
            print(frame_path)
            print(rulings_path)
        else:
            frame = load_json(pathlib.Path(args.frame))
            rulings = load_yaml(pathlib.Path(args.rulings))
            result = assess(frame, rulings)
            output = pathlib.Path(args.output)
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_text(render(frame, rulings, result), encoding="utf-8")
            print(output)
    except (CalibrationError, OSError, json.JSONDecodeError, yaml.YAMLError) as error:
        print(f"tag_precision_sample: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
