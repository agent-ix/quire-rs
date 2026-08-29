#!/usr/bin/env bash
# `spec/tests.md` states each test's state TWICE, and nothing compared them
# (agent-ix/quire-rs#347, SR-055 FND-007).
#
#   * the Test Case Summary LEDGER — one row per TC, column `Status`
#   * the Coverage tables — one row per requirement, column `Coverage Status`,
#     citing the TCs that back it
#
# Measured when the ticket was filed: 22 Coverage rows were marked done while
# citing a TC the same document marked 🚧.
#
# WHY A COMPARISON AND NOT A HAND PASS. The error runs BOTH ways. `TC-636`
# carries a live `#[trace("TC-636")]` in `src/registry.rs`, so there the ledger
# is stale and the summary is right; `TC-001` appears only in a fixture, so
# there the summary is wrong. Adjudicating 22 rows by hand produces a THIRD
# projection of the same facts, which is how the file came to have two. The
# check is the deliverable; which side to correct is a per-row decision this
# gate deliberately does not make.
#
# Engine-independent by necessity: `quire validate` reads one document against
# its archetype and has no reason to compare two tables inside it.
set -euo pipefail

python3 - "$@" <<'PY'
import re, sys, pathlib

doc = pathlib.Path("spec/tests.md")
if not doc.is_file():
    print("check_status_agreement: no spec/tests.md — nothing to compare")
    sys.exit(0)

def cells(line):
    """A markdown row's cells. Split rather than regexed: a greedy `.*\|`
    grabbed the wrong cell on long rows and reported a DESCRIPTION as a
    status — the first draft read `TC-030=byte-equal markdown`."""
    if not line.startswith("|"):
        return []
    return [c.strip() for c in line.strip().strip("|").split("|")]

DONE = ("✅", "Complete", "Implemented")
# A RETIRED test is a legitimate citation, not a pending one. The first draft
# counted `⛔ RETIRED — render removal (CR-042)` as a disagreement, which would
# have made every requirement citing a deliberately-retired test a finding.
RETIRED = ("⛔", "RETIRED")

lines = doc.read_text().splitlines()
ledger = {}
for line in lines:
    c = cells(line)
    if len(c) >= 2 and re.fullmatch(r"TC-\d+[a-z]?", c[0]):
        ledger[c[0]] = c[-1]

problems = []
for n, line in enumerate(lines, start=1):
    c = cells(line)
    if len(c) < 4 or not re.match(r"^(?:FR|NFR|StR|US)-\d+", c[0]):
        continue
    status = c[-1]
    if not any(tok in status for tok in DONE):
        continue
    pending = []
    for tc in re.findall(r"\bTC-\d+[a-z]?\b", " ".join(c[1:-1])):
        mark = ledger.get(tc)
        # An id ABSENT from the ledger is a different defect (#340's
        # territory); only a DISAGREEMENT is reported here.
        if mark is None:
            continue
        if any(tok in mark for tok in RETIRED):
            continue
        if not any(tok in mark for tok in DONE):
            pending.append(tc)
    if pending:
        shown = ", ".join(pending[:6]) + (f" (+{len(pending)-6} more)" if len(pending) > 6 else "")
        problems.append(f"  spec/tests.md:{n}  {c[0][:48]:<48} '{status[:28]}' cites {shown}")

if problems:
    # ADVISORY, NOT A GATE, and deliberately so. This repository's own rule:
    # ship a new check at warning so findings land and stay visible; promotion
    # to a failure is a separate, measured, user-gated decision. This check has
    # not been calibrated — the ticket measured 22 rows and this reads more, and
    # until that difference is adjudicated a red build would teach people to
    # disable it rather than to read it (#347).
    print(f"check_status_agreement: {len(problems)} ADVISORY finding(s) — a requirement is "
          f"marked done while citing a test the ledger does not:", file=sys.stderr)
    print("\n".join(problems), file=sys.stderr)
    print("  The error runs BOTH ways (#347): TC-636 carries a live trace tag so the LEDGER is "
          "stale there, while TC-001 exists only in a fixture so the SUMMARY is wrong. Correct "
          "the side that is wrong, per row — do not make the marks agree.", file=sys.stderr)
    print(f"  Advisory until calibrated: #347 measured 22 rows and this reads {len(problems)}. "
          f"Promotion to a gate is a separate decision.", file=sys.stderr)
    sys.exit(0)

print(f"check_status_agreement: OK ({len(ledger)} ledger entries compared)")
PY
