---
id: SR-113
title: "Independent re-review — YAML fuzz-manifest gate and narrowed evidence at fdde591"
type: SpecReview
analysis: code-review
scope: "PR #423 at fdde591 against the reviewed head ef53ae7; SR-111 FND-001..003; scripts/audits/check_dep_pins.sh; scripts/tests/test_dep_pins.py; spec/evidence/yaml-migration/manifest-v1.json"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/assets/adr/0012-yaml-engine-maintenance-and-parity"
    type: reviews
---

## Summary

Re-review of the three SR-111 findings at `fdde591`. All three are closed and
each was verified rather than accepted. The dependency audit now asks Cargo to
parse the independently resolved fuzz manifest as well as the root one, and the
exact mutation that escaped the old gate — reverting `fuzz/Cargo.toml` to
`serde_yaml = "^0.9"` — now fails it. The unverifiable `input_manifest_sha256`
is gone and the retention sentence no longer claims a digest for an artifact
that was never committed, without reintroducing the 781-path payload the
earlier review asked to keep out. The undisclosed removal of the call-site
audit is now stated in the PR body in its own paragraph, with the reason.

The rebase claim also holds: `9e07882`, the #422 head, is an ancestor of
`fdde591`.

## Verdict

**PASS** — no findings of this PR's own remain open. Three of three closed and
mutation-verified, every gate green at this head.

This PR inherits #422's three open low findings by rebase, and they are filed
there, not duplicated here. #423 cannot land ahead of #422 in any case.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings | - |

## Closed since SR-111

### FND-001 medium — the fuzz graph is changed but not gated

`check_dep_pins.sh` now runs `cargo metadata` against `fuzz/Cargo.toml` as well
as the root manifest, applies the same alias and exact-version assertions to
both under a `root`/`fuzz` label, requires the fuzz manifest to exist, and
extends the two wildcard-pin greps across both files.

Measured at `fdde591`:

| Mutation | Result |
| --- | --- |
| `fuzz/Cargo.toml` → `serde_yaml = "^0.9"` — the exact escape measured in SR-111 | **fail**: `check_dep_pins: FAIL — fuzz serde_yaml must alias the yaml_serde package` |
| `Cargo.toml` → `serde_yaml = "^0.9"` (regression control) | **fail**: `… root serde_yaml must alias …` |
| fuzz alias kept, `=0.10.7` → `^0.10` | **fail**: `… fuzz serde_yaml must use exact version =0.10.7` |
| baseline | pass |

`scripts/tests/test_dep_pins.py` carries five parametrised fuzz-manifest
mutants of its own — deprecated package, alias removed, caret range, wrong
patch, wildcard — so the behaviour is pinned in the repository and not only
here.

The floating-transitive concern raised in SR-111 does not currently bite:
`fuzz/` still has no lockfile, but `cargo info libyaml-rs` shows `0.3.0` is the
only published version, and both graphs reach it through the same exact
`yaml_serde 0.10.7`. The root `Cargo.lock` assertion therefore covers the fuzz
graph's YAML transitives as long as the direct pin stays exact — which the new
gate now enforces.

### FND-002 medium — a digest for an unretained input manifest

`input_manifest_sha256` is removed. The retention sentence changed from

> aggregate result only; the one-use comparator and per-input payload were
> removed after review

to

> aggregate counts and pinned source revisions only; no digest is claimed for
> the removed per-input manifest

That is the right direction: the claim was withdrawn rather than the 34,000-line
artifact restored to justify it. What the manifest still carries — five pinned
source revisions, the population counts, the negative control, the performance
deltas — is all independently checkable, and SR-111 already reproduced the
parity result over a wider population (1,008 frontmatter blocks and 27 focused
cases, zero differences) than the manifest claims.

### FND-003 low — the removal was not disclosed

The PR body now says so directly:

> The removal also includes the earlier exact-callsite audit script, its unit
> test and baseline, and its manually enumerated workflow step. That gate was
> deliberately retired because pinning an exact count of 95 call sites rejected
> legitimate source changes without proving YAML behavior; the root-and-fuzz
> dependency identity gate remains permanent.

Reason given, replacement named. Closed.

## Verified and found sound

- **Gates green at this head**: `cargo fmt --check`,
  `clippy --all-targets --all-features --locked -D warnings`,
  `make audit-static` (all audits OK, `check_dep_pins` and `check_tool_drift`
  included), `pytest scripts/tests` (163 passed, 3 skipped),
  `cargo deny check` (advisories, bans, licenses, sources ok), and
  `cargo metadata` on the fuzz manifest.
- **The stack is real.** `git merge-base --is-ancestor 9e07882 fdde591` exits 0,
  so the head under review genuinely contains #422's fix commit, and #422's
  gates were re-run against it.
- **The root lock still holds the intended pair**: `yaml_serde 0.10.7` and
  `libyaml-rs 0.3.0` present, `serde_yaml` and `unsafe-libyaml` absent.

## What this re-review does not do

It does not approve the pull request, take it out of draft, dispatch hosted CI,
or publish the crate. It does not revisit ADR-0012's maintenance-ownership
framing, which SR-111 already recorded as accurate: both engines descend from
the same C2Rust transpile of libyaml, and the ADR says so rather than claiming
a reduction in unsafe code.
