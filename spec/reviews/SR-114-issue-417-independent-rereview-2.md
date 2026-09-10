---
id: SR-114
title: "Independent re-review — remaining toolchain audit gaps at 8b636b7"
type: SpecReview
analysis: code-review
scope: "PR #422 at 8b636b7 against the reviewed head 9e07882; SR-112 FND-001..003; scripts/audits/check_tool_drift.sh; scripts/tests/test_tool_drift.py; rustfmt.toml"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-022"
    type: reviews
---

## Summary

Second re-review, at `8b636b7`. Two of SR-112's three findings close and both
were verified by mutation rather than read. The workflow scan now covers
`*.yaml` as well as `*.yml`, and the fix was applied to **both** loops that walk
that directory — the toolchain-action scan and the command-selector scan — not
just the one the finding named. The two nightly-only rustfmt options are gone,
`cargo fmt --all -- --check` now emits **zero** warnings where it emitted 120,
and no file's formatting changed.

TC-1834 is now open across three consecutive reviews, untouched and
unmentioned each time.

## Verdict

**CONDITIONAL** — no high findings. One low, carried forward unchanged.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | TC-1834 is still ✅ on a hypothetical 1.99.0 transition, and NFR-022-AC-3's seven-day clock still has no trigger; open since SR-110, unaddressed in both fix commits | spec/tests.md:851 | correct-requirement-no-evidence |

## Finding detail

### FND-001 — third filing, unchanged

```
| TC-1834 | Inspection of a simulated newer-stable event requires a recorded
compatibility result within seven days … | Inspection | P0 | NFR-022-AC-3,
NFR-022-AC-4 | ✅ Hypothetical 1.99.0 transition inspected in SR-106 |
```

`git diff 05e0e4d 8b636b7 -- spec/` touches one file, `SR-107`, and only to
record the closures. `spec/tests.md` is unchanged across both fix commits.

Stated once more, plainly, because it has not been disputed — only skipped:

- A P0 marked ✅ on an *inspection of a transition that has not happened*
  records that a reviewer read the requirement. It does not record that the
  requirement holds.
- AC-3 requires a compatibility result within seven days of a newer stable
  appearing. Nothing in the repository detects a newer stable, so the clock has
  no start event and the seven-day bound cannot be violated or satisfied.
- AC-4's 30-day hold expiry has the same shape.

Two honest resolutions, either acceptable: mark the row 🚧 and name the
detection gap in the status cell, or add the check that notices a newer stable
and starts the clock, and keep the ✅. What should not persist is a P0 that
cannot fail.

## Closed since SR-112

### FND-002 — `.yaml` workflows are now audited

The fix hoists the directory walk into one `workflow_paths` list built from both
extensions and reuses it in both places:

```python
workflow_paths = sorted(
    (*workflow_directory.glob("*.yml"), *workflow_directory.glob("*.yaml"))
)
```

I raised this against the selector loop only. The commit also applied it to the
action/runner loop at line 114, which I had not checked and which had the same
gap. Measured at `8b636b7`:

| Mutation | Result |
| --- | --- |
| `zz-mutant.yaml` with `run: cargo +stable test --locked` — **the SR-112 escape** | **fail**: `zz-mutant.yaml:8: stable Rust build selection must be 1.98.1` |
| `zz-mutant.yaml` with `run: rustup default nightly` | **fail**: `zz-mutant.yaml:8: nightly Rust build selection must use an exact date` |
| `zz-mutant.yaml` with a `toolchain: stable` **action key** | **fail**: `zz-mutant.yaml:8: stable Rust 1.98.1 is required` |
| `.yml` regression control — `cargo +1.94.1` in `loom.yml` | **fail**: `loom.yml:19: stable Rust build selection must be 1.98.1` |
| baseline | pass |

`test_yaml_workflow_extension_is_audited` pins the first case in the repository.

### FND-003 — the two nightly-only rustfmt options are removed

```diff
 use_small_heuristics = "Default"
-imports_granularity = "Crate"
-group_imports = "StdExternalCrate"
```

Measured at `8b636b7`: `cargo fmt --all -- --check` exits 0 with
**0** `unstable features` warnings, against 120 at `9e07882`. Removing settings
that stable rustfmt was already discarding changed no file's formatting, which
is what the clean `--check` proves. `verify_cookiecutter_inheritance: OK` still
passes, so the inheritance audit does not require the removed lines.

## Worth acting on outside this PR

The source still emits the pair. `rust-lib-cookiecutter/{{ cookiecutter.project_slug }}/rustfmt.toml`
ends with the same two lines, so every repository scaffolded from it starts with
two silently-discarded settings. A scan of `~/dev/*/rustfmt.toml` finds the pair
in 25 checkouts, including `quire-cli`, `quire-code-rs`, `quire-analyze`,
`quire-wasm`, `quire-contract-ir` (on `main`; the #62 branch removed them),
`ix-trace-rs`, `ecaz` and `tl-mltl`.

`ix-trace-rs` is the one worth naming: it is vendored into
`engineering-assurance`, which is why `cargo fmt` there emits the same warnings
despite that repository having no rustfmt.toml of its own.

Fixing the template is one two-line commit and stops the next scaffold
inheriting it. This is not a finding against #422, which fixed its own copy
correctly; it is where the same defect will keep coming from.

## Verified and found sound

- **Gates green at this head**: `cargo fmt --check` (0 warnings) ·
  `clippy --all-targets --all-features --locked -D warnings` ·
  `make audit-static` — all 11 audits OK including `check_dep_pins`,
  `check_tool_drift` and `verify_cookiecutter_inheritance` ·
  `pytest scripts/tests` (145 passed, 3 skipped — one more than at `9e07882`) ·
  `cargo test --locked --all-features` — **45 suites, 1,031 passing, 0 failed,
  0 ignored**.
- **The `.yaml` fix did not loosen the `.yml` path.** Every mutant that failed
  at `9e07882` still fails.

## What this re-review does not do

It does not approve the pull request, dispatch hosted CI, publish the crate,
promote TC-1834, or change `rust-lib-cookiecutter`.
