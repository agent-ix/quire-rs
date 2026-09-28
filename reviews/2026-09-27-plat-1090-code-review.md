---
id: SR-119
title: "Code review — drop hard-coded Cargo.toml version from tool-drift audit (#496)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@8884a77849ee135d06bfe015d3a022f39dd169ed; scripts/audits/check_tool_drift.sh, scripts/tests/test_tool_drift.py"
review_set: subset
---

## Summary

Ticket: PLAT-1090. PR: quire-rs#496, reviewed at `8884a77`.

The diff removes the clause in `scripts/audits/check_tool_drift.sh` that
required `Cargo.toml`'s `version` to equal the literal `0.46.0`, and removes
the one parametrized case in `scripts/tests/test_tool_drift.py` that
exercised it. Main is at `0.47.1`, so the clause failed `make audit-static`
and therefore `make ci` on main.

History: `git log -G` shows the clause was added in `506e936` (#366, "build:
lock canonical toolchain inputs"). That same commit bumped `Cargo.toml` from
`0.33.0` to `0.46.0`, and its commit body gives no reason for the pin.
`85dfe9d` (#422) and `616a7e9` (#410) did not touch it. The package version
is a release identity, not a toolchain input, so it is not drift. Any correct
release breaks the clause, and it guards nothing content-dependent. Under the
repo owner's pins/gates rule it is ceremony, and removing it is the right fix.

Examined, clean: the remaining `rust-toolchain.toml` channel check, the
`Cargo.toml` `rust-version` check and the `clippy.toml` `msrv` check are
untouched and still keyed on `accepted_stable_rust = "1.98.1"`. Their test
cases (toolchain `stable` and `1.94.1`, `rust-version 1.75`, `msrv 1.75`)
remain in the parametrize list. The removed case was the only coverage of
the removed clause, so no orphaned coverage and no lost coverage of retained
behavior.

## Gate results (run on the worktree at 8884a77, 2026-09-27)

| Gate | Result |
| --- | --- |
| `bash scripts/audits/check_tool_drift.sh` | exit 0 — "Rust 1.98.1 policy, action, runner, manifest, validation-stack, and Cargo locks verified" |
| `python3 -m pytest scripts/tests/test_tool_drift.py -q` | 26 passed |
| `make audit-static` | exit 0 |
| `gh pr checks 496` | only CLA checks run on the PR; both pass |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Test fixture still writes `version = "0.46.0"` into the synthetic Cargo.toml. The audit no longer reads it, so the stale literal suggests to a later reader that the version is still guarded. This is optional cleanup and does not block. | scripts/tests/test_tool_drift.py:30 |

## Verdict

**PASS**: no high or medium findings. The removal is correct, the retained
toolchain, MSRV and clippy-MSRV checks and their tests are intact, and all
gates pass. FND-001 is optional and non-blocking.

## Dispositions

Round 1, reviewed at `a524caa121f36fbcc06c9d0ed0753026f04d65fc`. The fix
commit `a524caa` changes `scripts/tests/test_tool_drift.py` (fixture line) and
adds this file under `reviews/`. No regressions: the drift audit exits 0,
`pytest scripts/tests/test_tool_drift.py` gives 26 passed, and
`make audit-static` exits 0.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | a524caa |
