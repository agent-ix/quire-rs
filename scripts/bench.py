#!/usr/bin/env python3
"""The engine benchmark: score the corpus, ratchet the baselines.

`agent-ix/quire-rs#231`, implementing the engine half of `agent-ix/quoin`
FR-043. Extends the existing sweep tooling (`sweep_coverage.py`,
`ac_corpus_sweep.py`) rather than replacing it — this adds the pinned manifest,
the score report and the ratchet.

Why a ratchet and not a threshold: a hand-picked threshold invites the number
to be tuned to it, which is how `ac:unclassifiable` came to pass 99.2% of
corpus cells (CR-019). A baseline can only be moved deliberately, with a diff.

    make bench          # score and compare; fails on a regression
    make bench-update   # deliberate regeneration, diff goes in the PR
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

from check_engine import Drift, build_engine

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "bench" / "manifest.json"
BASELINES = ROOT / "bench" / "baselines.json"


class BenchError(RuntimeError):
    """A benchmark run that must not be scored."""


# ── the ratchet ──────────────────────────────────────────────────────────────
#
# Pure, so it is unit-testable without running a sweep. The sweeps are slow and
# need a corpus on disk; the semantics are what regress.


def compare(metric: str, direction: str, observed: float, baseline: float | None):
    """One metric against its baseline.

    Returns `(verdict, new_baseline)`, where verdict is `improved`, `held`,
    `regressed` or `new`.

    `gate-zero` is not a ratchet: it is a gate. There is no baseline to beat and
    no tolerance to spend — the silent-zero sentinel is the class of defect that
    made three published SpecReviews wrong, and a benchmark that let it score
    0.98 and pass would be measuring the wrong thing.
    """
    if direction == "gate-zero":
        return ("held" if observed == 0 else "regressed"), 0

    if baseline is None:
        return "new", observed

    better = observed > baseline if direction == "higher-is-better" else observed < baseline
    if better:
        return "improved", observed
    if observed == baseline:
        return "held", baseline
    return "regressed", baseline


def score(manifest: dict, observed: dict, baselines: dict) -> dict:
    """Build the score report. Deterministic: sorted keys, no timestamps.

    A timestamp would make two runs over identical inputs differ, which is the
    property FR-043-AC-9 asks for and the reason `coverage_baseline` is
    byte-diffed rather than eyeballed.
    """
    metrics = manifest["metrics"]
    rows = []
    for corpus in sorted(observed):
        for name in sorted(observed[corpus]):
            if name not in metrics:
                raise BenchError(
                    f"{corpus}: metric {name!r} is not in the dictionary — "
                    "a number this benchmark does not declare is a number it "
                    "does not emit"
                )
            spec = metrics[name]
            value = observed[corpus][name]
            base = baselines.get(corpus, {}).get(name)
            verdict, new_base = compare(name, spec["direction"], value, base)
            rows.append(
                {
                    "corpus": corpus,
                    "metric": name,
                    "unit": spec["unit"],
                    "population": spec["population"],
                    "method": spec["method"],
                    "direction": spec["direction"],
                    "value": value,
                    "baseline": base,
                    "verdict": verdict,
                    "next_baseline": new_base,
                }
            )
    return {"rows": rows}


def render(report: dict) -> tuple[str, bool]:
    """Human report plus whether the run failed."""
    lines = []
    failed = False
    for row in report["rows"]:
        mark = {"improved": "++", "held": "ok", "new": "**", "regressed": "!!"}[row["verdict"]]
        base = "—" if row["baseline"] is None else row["baseline"]
        lines.append(
            f"{mark} {row['corpus']}/{row['metric']}: {row['value']} "
            f"(baseline {base}) [{row['unit']}]"
        )
        if row["verdict"] == "regressed":
            failed = True
            lines.append(
                f"     regressed against {base}; population: {row['population']}"
            )
    return "\n".join(lines), failed


# ── corpus identity ──────────────────────────────────────────────────────────


def resolve_identity(entry: dict) -> str:
    """The identity a corpus entry was scored at, refusing an unpinned tier 2.

    A tier-2 score is only a measurement because the corpus cannot move under
    it. Scoring a different SHA against the same key silently answers a
    different question (agent-ix/quoin FR-043-AC-8).
    """
    path = (ROOT / entry["path"]).resolve()
    if entry["identity"] == "working-tree":
        return "working-tree"
    if not path.exists():
        raise BenchError(f"{entry['name']}: corpus absent at {path}")
    head = subprocess.run(
        ["git", "-C", str(path), "rev-parse", "--short", "HEAD"],
        capture_output=True, text=True, check=False,
    ).stdout.strip()
    pinned = entry["pinned_sha"]
    if not head.startswith(pinned) and not pinned.startswith(head):
        raise BenchError(
            f"{entry['name']}: pinned at {pinned} but the tree is at {head}. "
            "Refusing to score — the answer key was adjudicated against the "
            "pinned commit, and a score over a different tree is not comparable "
            "to it. Check the corpus out at the pin, or move the pin "
            "deliberately."
        )
    return head


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--update", action="store_true",
                    help="rewrite baselines from this run (deliberate regen)")
    ap.add_argument("--observed", type=Path,
                    help="score a pre-collected observation file instead of sweeping")
    # #265: no PATH lookup. `--quire` defaulted to whatever was installed, so
    # the checked-in ratchet baseline was produced by a binary nobody chose —
    # measured on this machine, the installed `quire 0.30.2` emits no `engine`
    # block at all. The binary is built from the consuming workspace at its
    # pinned rev instead, exactly as `sweep_coverage.py` now does.
    ap.add_argument("--consumer", default="../quire-cli",
                    help="workspace to build the engine from (default: ../quire-cli)")
    ap.add_argument("--module", default=None,
                    help="module directory supplying the traceability model")
    args = ap.parse_args()

    manifest = json.loads(MANIFEST.read_text())
    baselines = json.loads(BASELINES.read_text()) if BASELINES.exists() else {}

    try:
        if args.observed:
            observed = json.loads(args.observed.read_text())
        else:
            consumer = Path(args.consumer).expanduser()
            if not consumer.is_absolute():
                consumer = (ROOT / consumer).resolve()
            try:
                quire = build_engine(consumer, release=True)
            except Drift as error:
                raise BenchError(str(error)) from error
            observed = collect(manifest, quire, args.module)
    except BenchError as exc:
        print(f"bench: {exc}", file=sys.stderr)
        return 2

    report = score(manifest, observed, baselines)
    text, failed = render(report)
    print(text or "no metrics collected")

    if args.update:
        nxt = {}
        for row in report["rows"]:
            nxt.setdefault(row["corpus"], {})[row["metric"]] = row["next_baseline"]
        BASELINES.write_text(json.dumps(nxt, indent=2, sort_keys=True) + "\n")
        print(f"\nbaselines rewritten: {BASELINES.relative_to(ROOT)}")
        return 0

    improved = [r for r in report["rows"] if r["verdict"] == "improved"]
    if improved:
        print(
            f"\n{len(improved)} metric(s) improved. `make bench-update` to "
            "tighten the baselines; the diff belongs in the PR."
        )
    return 1 if failed else 0


def coverage_payload(quire: str, scope: Path, module: str | None) -> dict:
    """`quire coverage --scope <scope> --json`, or a raised BenchError.

    Deliberately raises rather than returning an error dict: a corpus entry the
    engine could not read must not be scored at all, and the sweep scripts'
    convention of recording `{"error": ...}` per repo is right for a census and
    wrong for a gate.
    """
    cmd = [quire, "coverage", "--scope", str(scope), "--json"]
    if module:
        cmd += ["--module", module]
    done = subprocess.run(cmd, capture_output=True, text=True, timeout=900, check=False)
    if done.returncode != 0 or not done.stdout.strip():
        tail = (done.stderr or "no output").strip().splitlines()[-1:] or [""]
        raise BenchError(f"quire coverage failed: {tail[0][:200]}")
    return json.loads(done.stdout)


def metrics_from(payload: dict) -> dict[str, Any]:
    """The declared metrics, read off one coverage payload.

    Every number here comes from the FR-063 envelope or from a list the payload
    already carries. Nothing is re-derived by parsing prose, which is the
    property that lets the benchmark move when the engine does.
    """
    totals = payload.get("totals", {})
    backed, total = totals.get("backed", 0), totals.get("total", 0)
    census = payload.get("binding_census", [])
    candidates = sum(c["candidates"] for c in census)
    bound = sum(c["bound"] for c in census)
    criteria = totals.get("criteria")
    specific = totals.get("specific_shaped")
    specific_metric = next(
        (
            metric
            for metric in payload.get("metrics", [])
            if metric.get("name") == "coverage.specific_shaped"
        ),
        None,
    )

    out: dict[str, Any] = {
        "coverage.backed_pct": pct(backed, total),
        "coverage.dead_tags": len(payload.get("untracked_symbols", [])),
    }
    minting = next(
        (
            metric
            for metric in payload.get("metrics", [])
            if metric.get("name") == "minting.section_hit_rate"
        ),
        None,
    )
    if minting and minting.get("state") == "measured":
        out["coverage.minting_repos"] = 1 if minting.get("value", 0) > 0 else 0
    else:
        skip(
            "coverage.minting_repos",
            "payload carries no measured minting.section_hit_rate",
        )
    # A metric is omitted when it CANNOT be read, and the reason is printed.
    # Reporting 0% for either of these would be the silent zero this benchmark
    # exists to catch, produced by the benchmark itself.
    #
    # Two different absences, deliberately distinguished:
    #   - the key is missing entirely  → the engine predates the field, and the
    #     score would be measuring the toolchain's version, not its quality;
    #   - the key is present and empty → the corpus genuinely has none.
    # #265 AC-4 says a sweep lacking a required capability must abort rather
    # than "omit the metric and continue". This skip STAYS, because it is not
    # that failure mode: `skip()` records the metric as not-computed WITH its
    # reason (the FR-063 posture — a measurement that did not run carries
    # `because` and no numbers, so it can never be read as a zero), which is
    # already the honest answer. What AC-4 forbids is printing a figure over a
    # missing premise, and this prints no figure. The behaviour is quoin#197's
    # dictionary — a sibling programme, explicitly not this EPIC's to redefine
    # — and `test_metrics_are_omitted_rather_than_zeroed_when_unreadable` pins
    # it. Building from the pinned workspace, above, is what #265 actually owes
    # this script.
    if "binding_census" not in payload:
        skip("coverage.binding_read_pct", "payload carries no binding_census "
             "(engine predates FR-050-AC-27 — needs quire-rs >= v0.43.0)")
    elif candidates:
        out["coverage.binding_read_pct"] = pct(bound, candidates)
    else:
        skip("coverage.binding_read_pct", "no evidence symbols examined")

    if census and any("tagged" not in row for row in census):
        skip("authoring.tag_rate", "binding_census carries no tagged count "
             "(engine predates agent-ix/quire-rs#271)")
    elif candidates:
        out["authoring.tag_rate"] = pct(sum(c["tagged"] for c in census), candidates)
    else:
        skip("authoring.tag_rate", "no evidence symbols examined")

    if specific_metric and specific_metric.get("state") == "measured":
        population = specific_metric.get("population", 0)
        if population:
            out["properties.specific_shaped_pct"] = pct(
                specific_metric.get("value", 0), population
            )
        else:
            skip("properties.specific_shaped_pct", "no binding criteria")
    elif specific is None:
        skip(
            "properties.specific_shaped_pct",
            "payload carries neither a measured coverage.specific_shaped metric "
            "nor the legacy totals.specific_shaped field",
        )
    elif criteria:
        out["properties.specific_shaped_pct"] = pct(specific, criteria)
    else:
        skip("properties.specific_shaped_pct", "no binding criteria")
    out["sentinel.silent_zero"] = silent_zeros(payload)

    # The language-coverage guard (CR-103). A check measured on one language and
    # shipped to all of them is what put 549 suspicions on 551 TypeScript
    # candidates in v0.44.0; this is the number that would have said so.
    if candidates:
        out["skeptic.suspicion_rate"] = pct(len(payload.get("suspicions", [])), candidates)
    else:
        skip("skeptic.suspicion_rate", "no evidence symbols examined")
    return out


def selected(entry: dict, measured: dict[str, Any]) -> dict[str, Any]:
    """The metrics a corpus entry declares, or all of them.

    A `metrics` allowlist exists so a corpus can be carried for **language
    coverage** without its content churn thrashing a ratchet. `quoin` and
    `spec-artifacts-process` are working trees this repository does not
    control: their `backed_pct` moves whenever somebody writes a spec row, and
    ratcheting that would train everyone to run `bench-update` reflexively —
    which is how a ratchet stops being one. What they are here to hold still is
    the pair of gates that failed in v0.44.0, and those do not move with
    content.
    """
    declared = entry.get("metrics")
    if declared is None:
        return measured
    missing = [m for m in declared if m not in measured]
    for name in missing:
        skip(name, f"declared by corpus {entry['name']!r} but not measurable in its payload")
    return {k: v for k, v in measured.items() if k in declared}


def skip(metric: str, why: str) -> None:
    print(f"     omitted {metric}: {why}", file=sys.stderr)


def pct(num: int, den: int) -> float:
    return round((num * 100.0) / den, 2) if den else 0.0


def silent_zeros(payload: dict) -> int:
    """Metrics that published a ratio over input they could not read, with no
    diagnostic saying so.

    The engine already reports this as `hollow-denominator` (FR-063-AC-5); the
    sentinel counts the ones that slipped past it, so the gate does not depend
    on the same code path it is checking.

    **Count-shaped metrics are exempt (CR-102).** For a tally `matched` and the
    value are the same fact, so `matched: 0` reports that none was found rather
    than that none was read. Before the shape existed this sentinel read `0`
    only because the engine's own false-positive `hollow-denominator` was
    satisfying the `covered` clause above — two defects cancelling. Fixing the
    engine alone would have turned this gate red on `coverage.implements`
    reading an honest 0 of 42.
    """
    reasons = {d.get("reason") for d in payload.get("diagnostics", [])}
    covered = "hollow-denominator" in reasons or "no-symbol-bound" in reasons
    silent = 0
    for metric in payload.get("metrics", []):
        if metric.get("state") != "measured":
            continue
        # Absent `shape` means an engine predating CR-102; treat it as a ratio,
        # which is the reading that can still fail the gate rather than the one
        # that silently passes it.
        if metric.get("shape", "ratio") != "ratio":
            continue
        if metric.get("population", 0) > 0 and metric.get("examined", 0) > 0 \
                and metric.get("matched", 0) == 0 and not covered:
            silent += 1
    return silent


def collect(
    manifest: dict,
    quire: str,
    module: str | None,
    raw_evidence: dict[str, Any] | None = None,
) -> dict:
    """Run the corpus. Skips an absent entry loudly rather than scoring 0.

    A missing corpus scored as zero is the silent-zero defect this benchmark
    exists to catch, reproduced inside the thing catching it.
    """
    observed: dict[str, dict[str, Any]] = {}
    for entry in manifest["corpora"]:
        try:
            identity = resolve_identity(entry)
            entry_module = entry.get("module", module)
            if entry_module:
                entry_module_path = Path(entry_module).expanduser()
                if not entry_module_path.is_absolute():
                    entry_module = str((ROOT / entry_module_path).resolve())
            payload = coverage_payload(
                quire, (ROOT / entry["path"]).resolve(), entry_module
            )
        except (BenchError, subprocess.TimeoutExpired, json.JSONDecodeError) as exc:
            print(f"skip {entry['name']}: {exc}", file=sys.stderr)
            continue
        print(f"…{entry['name']} @ {identity}", file=sys.stderr)
        observed[entry["name"]] = selected(entry, metrics_from(payload))
        if raw_evidence is not None:
            raw_evidence[entry["name"]] = {
                "identity": identity,
                "module": entry_module,
                "payload": payload,
            }
    if not observed:
        raise BenchError(
            "no corpus entry could be scored — refusing to report a pass over "
            "nothing. Check that `quire` is on PATH and the pinned corpora are "
            "checked out."
        )
    return observed


if __name__ == "__main__":
    sys.exit(main())
