---
id: SR-112
title: "Independent re-review — closed Rust toolchain drift gaps at 9e07882"
type: SpecReview
analysis: code-review
scope: "PR #422 at 9e07882 against the reviewed head 05e0e4d; SR-110 FND-001..004; scripts/audits/check_tool_drift.sh; .github/workflows/; Makefile; rustfmt.toml"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-022"
    type: reviews
---

## Summary

Re-review of the four SR-110 findings at `9e07882`. Three are closed and I
killed each with an independent mutant. The drift audit now scans every
workflow's command bodies, not just its `toolchain:` keys; the hosted static
audit job calls `make audit-static`, which globs the directory instead of
repeating a hand-written list; and the selector regex was widened from
`stable|x.y.z` to any toolchain name with a dated-nightly branch, which closes
the bare `rustup default nightly` hole and, as a bonus, the undated
`cargo +nightly` selections in the Makefile and the fuzz and sanitizer
workflows.

The fourth finding is untouched and unmentioned. Two new low items came out of
re-running the gates.

## Verdict

**CONDITIONAL** — no high findings. Three lows, all about the reach of a gate
rather than the correctness of the change.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | TC-1834 is still ✅ on a hypothetical 1.99.0 transition, and NFR-022-AC-3's seven-day clock still has no trigger | spec/tests.md:851 | correct-requirement-no-evidence |
| FND-002 | low | The new workflow scan globs `*.yml` only; a `.yaml` workflow — which GitHub accepts — escapes every selector check | scripts/audits/check_tool_drift.sh:198 | correct-requirement-no-evidence |
| FND-003 | low | `rustfmt.toml` sets two nightly-only options that the qualified stable toolchain silently ignores, emitting 120 warnings per `cargo fmt` run | rustfmt.toml:4 | correct-requirement-no-evidence |

## Finding detail

### FND-001 — carried forward from SR-110 unchanged

`spec/tests.md:851`:

```
| TC-1834 | Inspection of a simulated newer-stable event requires a recorded
compatibility result within seven days … | Inspection | P0 | NFR-022-AC-3,
NFR-022-AC-4 | ✅ Hypothetical 1.99.0 transition inspected in SR-106 |
```

`git diff 05e0e4d 9e07882 -- spec/tests.md` is empty, and SR-107's update adds
FND-4176, FND-4177 and FND-4178 for the other three findings with nothing for
this one.

Restating it once, since it was not disputed and not addressed: a P0 marked
complete on an inspection of a transition that has not happened records that
someone read the requirement, not that the requirement holds. AC-3's seven-day
clock starts on an event nothing detects. The honest mark is 🚧 with the
detection gap named, or a check that watches for a newer stable and starts the
clock.

### FND-002 — `.yaml` is a workflow extension too

The new line:

```python
build_scripts.extend(sorted((root / ".github/workflows").glob("*.yml")))
```

GitHub Actions loads both `.yml` and `.yaml` from `.github/workflows`. All eight
workflows here are `.yml`, so nothing is wrong today; the gate simply does not
cover the other half of the namespace it claims.

Measured at `9e07882` — a new `.github/workflows/zz-mutant.yaml` containing
both `run: cargo +stable test --locked` and `run: rustup default nightly`:

```
$ bash scripts/audits/check_tool_drift.sh
tool-drift audit: Rust 1.98.1 policy, action, runner, manifest,
validation-stack, and Cargo locks verified
drift_rc=0
```

The same content in a `.yml` file fails, correctly. `glob("*.yml")` →
`chain(glob("*.yml"), glob("*.yaml"))` closes it.

### FND-003 — two rustfmt settings the qualified toolchain cannot honour

```toml
# rustfmt.toml
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

Both are nightly-only. Under the exact stable 1.98.1 this PR qualifies, rustfmt
prints and discards them:

```
Warning: can't set `imports_granularity = Crate`, unstable features are only
available in nightly channel.
Warning: can't set `group_imports = StdExternalCrate`, unstable features are
only available in nightly channel.
```

120 lines of it on a single `cargo fmt --all -- --check`, two per file. Import
grouping is therefore unenforced despite being declared repository policy, and
the noise is the kind that trains a reader to skim past `cargo fmt` output.

This is in scope for a PR whose subject is exact-toolchain qualification: the
sibling qualification PR in `quire-contract-ir` found the identical pair,
recorded it as SR-035 FND-1508, and removed both settings rather than waiving
them. `git log -- rustfmt.toml` shows a single commit,
`fe14ace chore: scaffold from rust-lib-cookiecutter`, so the same two lines are
likely in every repository scaffolded from that template — worth one look at
`rust-lib-cookiecutter` rather than eight separate fixes.

## Closed since SR-110

Each verified by mutation at `9e07882`, using a different workflow from the one
the PR's own test fixture mutates.

| SR-110 | Mutation | Result |
| --- | --- | --- |
| FND-001 medium — workflow `run:` bodies unscanned | `run: cargo +stable test --locked` added to `loom.yml` | **fail**: `loom.yml:19: stable Rust build selection must be 1.98.1` |
| FND-003 low — bare `rustup default nightly` unrecognised | `run: rustup default nightly` added to `loom.yml` | **fail**: `loom.yml:19: nightly Rust build selection must use an exact date` |
| — (bonus) | `+nightly-2026-08-27 fuzz` → `+nightly fuzz` in the Makefile | **fail**: `Makefile:203: nightly Rust build selection must use an exact date` |
| FND-002 medium — CI enumerated 7 of 11 audits | a new failing `scripts/audits/zz_fail.sh` | **fail**: `make audit-static` discovered and ran it |

FND-002's fix is the better of the two available: the hosted job now calls the
same `make audit-static` target a developer runs, so the two can no longer
diverge, and the Makefile already fails hard on an empty glob.

The nightly date is consistent across every selection — Makefile, `fuzz.yml`,
`sanitize.yml` all name `nightly-2026-08-27`, and `fuzz.yml` and `sanitize.yml`
install that exact toolchain via the pinned action. There is no lane selecting
a nightly it did not install.

## Verified and found sound

- **Gates green at this head**: `cargo fmt --check`,
  `clippy --all-targets --all-features --locked -D warnings`,
  `make audit-static` (11 audits), `pytest scripts/tests` (144 passed, 3
  skipped), and `cargo test --locked --all-features` — 25 suites, 1,031
  passing, 0 failed, 0 ignored.
- **The regex widening did not weaken anything.** `(?:cargo|\$\(CARGO\))\s+\+([A-Za-z0-9_.-]+)` now
  captures every toolchain name rather than only `stable` or a version triple,
  and everything not starting with `nightly` must equal `1.98.1` exactly. A
  selector the old pattern silently ignored is now an error, not an exemption.
- **`scripts/tests/test_tool_drift.py` carries its own controls** for the two
  new cases, so the audit's behaviour is pinned in the repository and not only
  in this review.

## What this re-review does not do

It does not approve the pull request, dispatch hosted CI, publish the crate, or
promote TC-1834.
