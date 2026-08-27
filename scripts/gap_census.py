#!/usr/bin/env python3
"""Partition ecosystem-authored obligation rows under FR-066.

The controlled corpus is the per-change gate. This slower census explains the
remaining rows and routes each cause; it is intentionally schedule/dispatch
only (agent-ix/quire-rs#277).
"""

from __future__ import annotations

import argparse
import fnmatch
import json
import pathlib
import re
import subprocess
import sys
from dataclasses import dataclass
from datetime import date
from typing import Iterable

import yaml

from check_engine import Drift, assert_capabilities, build_engine, reported_engine
from corpus import repos
from sweep_coverage import coverage

REQUIRED_CAPABILITIES = (
    "binding_census",
    "binding_census.tagged",
    "metrics_envelope",
    "minted_targets",
    "unmatched_tags",
)
BINDING_FLOOR = 0.10
DISPOSITIONS = (
    "instrument-unread",
    "declaration-unreached",
    "marker-form-mismatch",
    "id-class-unminted",
    "method-exempt",
    "authoring-absent",
)
OWNERS = {
    "backed": "nobody",
    "instrument-unread": "engine",
    "declaration-unreached": "declaration-or-repository",
    "marker-form-mismatch": "module-declaration",
    "id-class-unminted": "module-declaration",
    "method-exempt": "nobody",
    "authoring-absent": "repository",
}
NEXT_ACTIONS = {
    "backed": "none",
    "instrument-unread": "repair the named language binding surface, then rerun",
    "declaration-unreached": "repair the target archetype, section, or id-column declaration",
    "marker-form-mismatch": "add or repair the declared tag form without widening unrelated forms",
    "id-class-unminted": "decide and declare whether this id class is a trace target",
    "method-exempt": "none; the declared method does not mint a source symbol",
    "authoring-absent": "add an applicable trace tag and its controlled corpus case",
}

ID_TOKEN = re.compile(r"\b[A-Za-z][A-Za-z0-9]*(?:[-_][A-Za-z0-9]+){1,6}\b")
HEADING = re.compile(r"^(#{1,6})\s+(.+?)\s*$")
SEPARATOR = re.compile(r"^:?-{3,}:?$")


class CensusError(RuntimeError):
    """The census cannot honestly publish a complete partition."""


@dataclass(frozen=True, order=True)
class Row:
    repo: str
    document: str
    row_id: str
    line: int
    target: str | None = None
    method: str | None = None
    status: str | None = None

    @property
    def key(self) -> tuple[str, str, str, int]:
        return self.repo, self.document, self.row_id, self.line


@dataclass(frozen=True)
class Target:
    name: str
    archetype: str
    sections: tuple[str, ...]
    id_column: str
    exclude: tuple[str, ...]


@dataclass(frozen=True)
class Classified:
    row: Row
    outcome: str
    owner: str
    reason: str
    next_action: str
    status_lie: bool = False
    source_locus: str | None = None


def split_cells(line: str) -> list[str]:
    stripped = line.strip()
    if not stripped.startswith("|"):
        return []
    body = stripped[1:-1] if stripped.endswith("|") else stripped[1:]
    cells = []
    current = []
    escaped = False
    for char in body:
        if escaped:
            current.append(char)
            escaped = False
        elif char == "\\":
            current.append(char)
            escaped = True
        elif char == "|":
            cells.append("".join(current).strip())
            current = []
        else:
            current.append(char)
    cells.append("".join(current).strip())
    return cells


def tables(text: str) -> Iterable[tuple[str, list[str], list[str], int]]:
    """Yield heading, header, row cells and 1-based row line."""
    lines = text.splitlines()
    heading = ""
    index = 0
    while index < len(lines):
        match = HEADING.match(lines[index])
        if match:
            heading = match.group(2).strip().removesuffix(" #").strip()
        header = split_cells(lines[index])
        if header and index + 1 < len(lines):
            separator = split_cells(lines[index + 1])
            if len(separator) == len(header) and all(
                SEPARATOR.match(cell) for cell in separator
            ):
                cursor = index + 2
                while cursor < len(lines):
                    cells = split_cells(lines[cursor])
                    if not cells:
                        break
                    cells = (cells + [""] * len(header))[: len(header)]
                    yield heading, header, cells, cursor + 1
                    cursor += 1
                index = cursor
                continue
        index += 1


def frontmatter_type(text: str) -> str | None:
    if not text.startswith("---\n"):
        return None
    end = text.find("\n---\n", 4)
    if end < 0:
        return None
    loaded = yaml.safe_load(text[4:end])
    return loaded.get("type") if isinstance(loaded, dict) else None


def id_signature(value: str) -> str:
    parts = re.split(r"[-_]", value.upper())
    return ":".join(
        "#" if part.isdigit() else re.sub(r"\d+", "#", part) for part in parts
    )


def section_matches(patterns: tuple[str, ...], heading: str) -> bool:
    return any(fnmatch.fnmatchcase(heading, pattern) for pattern in patterns)


def excluded(rel: str, patterns: tuple[str, ...]) -> bool:
    return any(fnmatch.fnmatchcase(rel, pattern) for pattern in patterns)


def read_targets(module: pathlib.Path) -> list[Target]:
    manifest = yaml.safe_load((module / "manifest.yaml").read_text(encoding="utf-8"))
    traceability = manifest.get("traceability", {})
    out = []
    for raw in traceability.get("trace_targets", []):
        section = raw.get("section")
        sections = tuple(section if isinstance(section, list) else [section])
        if not raw.get("archetype") or not raw.get("id_column") or not all(sections):
            continue
        out.append(
            Target(
                name=raw["name"],
                archetype=raw["archetype"],
                sections=sections,
                id_column=raw["id_column"],
                exclude=tuple(raw.get("exclude", [])),
            )
        )
    return out


def scan_rows(repo: pathlib.Path, targets: list[Target]) -> tuple[dict, dict]:
    authored: dict[tuple[str, str, str, int], Row] = {}
    minted: dict[tuple[str, str, str, int], Row] = {}
    for path in sorted((repo / "spec").rglob("*.md")):
        rel = path.relative_to(repo).as_posix()
        text = path.read_text(encoding="utf-8", errors="replace")
        archetype = frontmatter_type(text)
        parsed = list(tables(text))
        for _heading, header, cells, line in parsed:
            if not header or header[0] not in {"ID", "Test ID", "Integration ID"}:
                continue
            row_id = cells[0]
            if not ID_TOKEN.fullmatch(row_id):
                continue
            values = dict(zip(header, cells))
            row = Row(
                repo.name,
                rel,
                row_id,
                line,
                method=values.get("Type") or values.get("Verification Method"),
                status=values.get("Status") or values.get("Coverage Status"),
            )
            authored.setdefault(row.key, row)
        for target in targets:
            if archetype != target.archetype or excluded(rel, target.exclude):
                continue
            for heading, header, cells, line in parsed:
                if (
                    not section_matches(target.sections, heading)
                    or target.id_column not in header
                ):
                    continue
                values = dict(zip(header, cells))
                row_id = values[target.id_column]
                if not row_id or not ID_TOKEN.fullmatch(row_id):
                    continue
                row = Row(
                    repo.name,
                    rel,
                    row_id,
                    line,
                    target.name,
                    values.get("Type") or values.get("Verification Method"),
                    values.get("Status") or values.get("Coverage Status"),
                )
                minted.setdefault(row.key, row)
    return authored, minted


def unread_languages(report: dict) -> set[str]:
    unread = {
        str(row.get("value"))
        for row in report.get("diagnostics", [])
        if row.get("reason") in {"no-symbol-bound", "low-symbol-binding"}
        and row.get("value")
    }
    for row in report.get("binding_census", []):
        candidates = row.get("candidates", 0)
        if candidates and row.get("bound", 0) / candidates < BINDING_FLOOR:
            unread.add(row["language"])
    return unread


def classify_repo(repo: pathlib.Path, report: dict, targets: list[Target]) -> dict:
    authored, _independent_minted = scan_rows(repo, targets)
    minted: dict[tuple[str, str, str, int], Row] = {}
    for record in report.get("minted_targets", []):
        key = (repo.name, record["document"], record["id"], record["line"])
        authored_row = authored.get(key)
        minted[key] = Row(
            repo.name,
            record["document"],
            record["id"],
            record["line"],
            record["target"],
            authored_row.method if authored_row else None,
            authored_row.status if authored_row else None,
        )
    expected_total = report.get("totals", {}).get("total")
    if len(minted) != expected_total:
        raise CensusError(
            f"{repo.name}: payload carries {len(minted)} minted target records but totals.total is "
            f"{expected_total}; refusing a partition over disagreeing engine facts"
        )
    backed_keys = {
        (repo.name, record["document"], record["id"], record["line"])
        for record in report.get("minted_targets", [])
        if record["backed"]
    }
    if len(backed_keys) != report.get("totals", {}).get("backed"):
        raise CensusError(
            f"{repo.name}: backed minted target records disagree with totals.backed"
        )
    absent_from_p3 = sorted(key for key in minted if key not in authored)
    if absent_from_p3:
        raise CensusError(
            f"{repo.name}: P4 contains {len(absent_from_p3)} rows absent from the independent P3 scan; "
            f"first is {absent_from_p3[0][1]}:{absent_from_p3[0][3]} {absent_from_p3[0][2]}"
        )

    no_symbol_ids = {
        target_id
        for row in report.get("no_symbol_rows", [])
        for target_id in row.get("target_ids", [])
    }
    status_lie_ids = {
        target_id
        for row in report.get("status_lies", [])
        for target_id in row.get("target_ids", [])
    }
    tags: dict[str, list[str]] = {}
    for record in report.get("unmatched_tags", []):
        missing = [
            key
            for key in ("trace_id", "language", "path", "line", "symbol")
            if key not in record
        ]
        if missing:
            raise CensusError(
                f"{repo.name}: unmatched tag record lacks {', '.join(missing)}"
            )
        locus = f"{record['language']}:{record['path']}:{record['line']}"
        tags.setdefault(str(record["trace_id"]), []).append(locus)
    for loci in tags.values():
        loci.sort()
    unread = unread_languages(report)
    signature_targets: dict[str, set[str]] = {}
    for row in minted.values():
        signature_targets.setdefault(id_signature(row.row_id), set()).add(
            row.target or ""
        )

    classified: list[Classified] = []
    for row in sorted(authored.values()):
        if row.key in minted and row.key in backed_keys:
            outcome, reason = "backed", "engine reports the minted row backed"
        elif row.key not in minted:
            if id_signature(row.row_id) in signature_targets:
                outcome = "declaration-unreached"
                reason = "the id class has an active target but this authored row was not minted"
            else:
                outcome = "id-class-unminted"
                reason = "no active trace target mints this authored id class"
        else:
            loci = tags.get(row.row_id, [])
            tagged_unread = [
                locus for locus in loci if locus.split(":", 1)[0] in unread
            ]
            if tagged_unread:
                outcome = "instrument-unread"
                reason = (
                    "the row has a tag on a language surface below the binding floor"
                )
            elif loci:
                outcome = "marker-form-mismatch"
                reason = "the minted unbacked row has an authored id token no declared form bound"
            elif row.row_id in no_symbol_ids:
                outcome = "method-exempt"
                reason = "the declared verification method mints no source symbol"
            else:
                outcome = "authoring-absent"
                reason = (
                    "the row is minted and unbacked, with no authored source tag found"
                )
        loci = tags.get(row.row_id, [])
        classified.append(
            Classified(
                row=row,
                outcome=outcome,
                owner=OWNERS[outcome],
                reason=reason,
                next_action=NEXT_ACTIONS[outcome],
                status_lie=row.row_id in status_lie_ids,
                source_locus=loci[0] if loci else None,
            )
        )

    counts = {name: 0 for name in ("backed", *DISPOSITIONS)}
    for item in classified:
        counts[item.outcome] += 1
    if sum(counts.values()) != len(authored):
        raise CensusError(f"{repo.name}: partition sum does not equal authored rows")
    if set(counts) != {"backed", *DISPOSITIONS}:
        raise CensusError(f"{repo.name}: disposition vocabulary drifted")

    census = report.get("binding_census", [])
    populations = {
        "P1_evidence_symbols": sum(row.get("candidates", 0) for row in census),
        "P2_tagged_symbols": sum(row.get("tagged", 0) for row in census),
        "P3_authored_rows": len(authored),
        "P4_minted_rows": len(minted),
    }
    zero_tag_readable = (
        populations["P1_evidence_symbols"] > 0 and populations["P2_tagged_symbols"] == 0
    )
    if populations["P2_tagged_symbols"] == 0 and tags:
        raise CensusError(
            f"{repo.name}: unmatched tag records disagree with a zero tagged-symbol population"
        )
    if zero_tag_readable and counts["instrument-unread"]:
        raise CensusError(
            f"{repo.name}: a zero-tag repository entered instrument-unread; "
            "authoring absence is not an instrument defect"
        )

    return {
        "repo": repo.name,
        "populations": populations,
        "counts": counts,
        "status_lie_overlay": sum(item.status_lie for item in classified),
        "zero_tag_readable": zero_tag_readable,
        "examples": [classified_json(item) for item in first_examples(classified)],
    }


def first_examples(rows: list[Classified]) -> list[Classified]:
    seen = set()
    out = []
    for row in rows:
        if row.outcome == "backed" or row.outcome in seen:
            continue
        seen.add(row.outcome)
        out.append(row)
    return out


def classified_json(item: Classified) -> dict:
    return {
        "outcome": item.outcome,
        "owner": item.owner,
        "repo": item.row.repo,
        "document": item.row.document,
        "row_id": item.row.row_id,
        "line": item.row.line,
        "reason": item.reason,
        "next_action": item.next_action,
        "status_lie": item.status_lie,
        **({"source_locus": item.source_locus} if item.source_locus else {}),
    }


def module_sha(module: pathlib.Path) -> str:
    done = subprocess.run(
        ["git", "-C", str(module), "rev-parse", "HEAD"], capture_output=True, text=True
    )
    if done.returncode:
        raise CensusError(
            f"cannot resolve module commit for {module}: {done.stderr.strip()}"
        )
    return done.stdout.strip()


def engine_identity(
    report: dict, prior: tuple | None = None
) -> tuple[str, str, tuple[str, ...]]:
    try:
        engine, capabilities = reported_engine(report)
        assert_capabilities(capabilities, list(REQUIRED_CAPABILITIES))
    except Drift as error:
        raise CensusError(str(error)) from error
    identity = (report["engine"]["cli"], engine, tuple(capabilities))
    if prior is not None and identity != prior:
        raise CensusError("engine identity changed during the census")
    return identity


def aggregate(repo_rows: list[dict]) -> dict:
    populations = {
        name: 0
        for name in (
            "P1_evidence_symbols",
            "P2_tagged_symbols",
            "P3_authored_rows",
            "P4_minted_rows",
        )
    }
    counts = {name: 0 for name in ("backed", *DISPOSITIONS)}
    examples = []
    for row in repo_rows:
        for name, value in row["populations"].items():
            populations[name] += value
        for name, value in row["counts"].items():
            counts[name] += value
        examples.extend(row["examples"])
    if sum(counts.values()) != populations["P3_authored_rows"]:
        raise CensusError("aggregate partition sum does not equal authored rows")
    chosen = {}
    for example in examples:
        chosen.setdefault(example["outcome"], example)
    return {
        "populations": populations,
        "counts": counts,
        "status_lie_overlay": sum(row["status_lie_overlay"] for row in repo_rows),
        "zero_tag_repositories": sum(row["zero_tag_readable"] for row in repo_rows),
        "examples": [chosen[name] for name in DISPOSITIONS if name in chosen],
    }


def render_markdown(payload: dict) -> str:
    provenance = payload["provenance"]
    aggregate_row = payload["aggregate"]
    lines = [
        "# Gap disposition census",
        "",
        f"- Date: `{payload['date']}`",
        f"- CLI: `{provenance['cli']}`",
        f"- Engine: `{provenance['engine']}`",
        f"- Capabilities: `{', '.join(provenance['capabilities'])}`",
        f"- Module commit: `{provenance['module_sha']}`",
        f"- Repositories: {provenance['repos_scanned']} scanned / {provenance['repos_enumerated']} enumerated",
        f"- Exclusions: {', '.join(provenance['exclusions']) or 'none'}",
        "",
        "## Populations",
        "",
        "| Population | Count |",
        "|---|---:|",
    ]
    for name, value in aggregate_row["populations"].items():
        lines.append(f"| `{name}` | {value} |")
    lines += ["", "## Partition", "", "| Outcome | Rows | Owner |", "|---|---:|---|"]
    for name, value in aggregate_row["counts"].items():
        lines.append(f"| `{name}` | {value} | {OWNERS[name]} |")
    lines += [
        "",
        f"Invariant: **PASS** — {sum(aggregate_row['counts'].values())} classified = "
        f"{aggregate_row['populations']['P3_authored_rows']} authored rows.",
        f"Status-lie overlay: {aggregate_row['status_lie_overlay']} (not part of the sum).",
        f"Readable zero-tag repositories: {aggregate_row['zero_tag_repositories']}.",
        "",
        "## Actionable examples",
        "",
        "| Outcome | Where | Why | Owner / next action |",
        "|---|---|---|---|",
    ]
    for row in aggregate_row["examples"]:
        where = f"`{row['repo']}/{row['document']}:{row['line']}` `{row['row_id']}`"
        action = f"{row['owner']} — {row['next_action']}"
        lines.append(f"| `{row['outcome']}` | {where} | {row['reason']} | {action} |")
    return "\n".join(lines) + "\n"


def write_reports(
    payload: dict, output_dir: pathlib.Path
) -> tuple[pathlib.Path, pathlib.Path]:
    output_dir.mkdir(parents=True, exist_ok=True)
    stem = f"{payload['date']}-gap-census"
    json_path = output_dir / f"{stem}.json"
    md_path = output_dir / f"{stem}.md"
    json_bytes = json.dumps(payload, indent=2, sort_keys=True) + "\n"
    markdown = render_markdown(payload)
    json_path.write_text(json_bytes, encoding="utf-8")
    md_path.write_text(markdown, encoding="utf-8")
    return json_path, md_path


def run(args: argparse.Namespace) -> dict:
    root = pathlib.Path(args.root).expanduser().resolve()
    module = pathlib.Path(args.module).expanduser().resolve()
    targets = read_targets(module)
    if not targets:
        raise CensusError("module declares no usable trace targets")
    try:
        quire = build_engine(
            pathlib.Path(args.consumer).expanduser().resolve(), release=True
        )
    except Drift as error:
        raise CensusError(str(error)) from error

    enumerated = repos(root)
    exclusions = sorted(set(args.exclude))
    selected = [repo for repo in enumerated if repo.name not in exclusions]
    repo_rows = []
    identity = None
    for repo in selected:
        report = coverage(quire, repo, str(module))
        if not report or "error" in report:
            raise CensusError(
                f"{repo.name}: coverage failed: {(report or {}).get('error', 'no payload')}"
            )
        current = engine_identity(report, identity)
        if identity is None:
            identity = current
        repo_rows.append(classify_repo(repo, report, targets))
        print(f"census: {repo.name}", file=sys.stderr)
    if identity is None:
        raise CensusError("no repository produced a payload; nothing was measured")
    return {
        "date": args.date,
        "provenance": {
            "cli": identity[0],
            "engine": identity[1],
            "capabilities": list(identity[2]),
            "module_sha": module_sha(module),
            "repos_enumerated": len(enumerated),
            "repos_scanned": len(selected),
            "exclusions": exclusions,
        },
        "aggregate": aggregate(repo_rows),
        "repositories": repo_rows,
    }


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default="~/dev")
    parser.add_argument("--consumer", default="../quire-cli")
    parser.add_argument("--module", required=True)
    parser.add_argument("--output-dir", default="reports")
    parser.add_argument("--date", default=date.today().isoformat())
    parser.add_argument("--exclude", action="append", default=[])
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    try:
        payload = run(args)
        paths = write_reports(payload, pathlib.Path(args.output_dir))
    except (CensusError, OSError, yaml.YAMLError) as error:
        print(f"gap_census: {error}", file=sys.stderr)
        return 1
    print("\n".join(str(path) for path in paths))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
