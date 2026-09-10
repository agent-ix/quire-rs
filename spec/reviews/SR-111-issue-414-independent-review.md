---
id: SR-111
title: "Independent code review — maintained YAML engine migration at ef53ae7"
type: SpecReview
analysis: code-review
scope: "PR #423 at ef53ae7, diffed against its base 05e0e4d (#422); ADR-0012; NFR-009; Cargo.toml, fuzz/Cargo.toml, Cargo.lock; scripts/audits/check_dep_pins.sh; spec/evidence/yaml-migration/manifest-v1.json"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/assets/adr/0012-yaml-engine-maintenance-and-parity"
    type: reviews
---

## Summary

The migration is well chosen and the parity claim is true — I rebuilt the
deleted comparator and reproduced it independently over a **wider** population
than the PR reports: 1,008 frontmatter blocks and 27 focused semantic cases,
zero differences of any kind. Using Cargo's package rename to keep
`serde_yaml::` at every call site means the diff touches no source file at all,
which is the right shape for a dependency swap. ADR-0012 is unusually good: it
states outright that both backends descend from the same C2Rust transpile and
"does not claim a reduction in transitive unsafe code" — which my count
confirms. Two gaps: the dependency audit does not reach the fuzz graph, and the
retained evidence manifest carries a digest of a file that was never committed.

## Verdict

**CONDITIONAL** — no high findings. Two mediums and one low, all about what is
retained and gated rather than about the migration itself.

## Gates run at `ef53ae7`

Exact Rust 1.98.1, `-j 2`, `corpus/` submodule initialised.

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | pass |
| `cargo test --locked` | pass — **1,031 passed, 0 failed, 0 ignored**, identical to the base at #422 |
| `cargo deny check` | advisories, bans, licenses, sources ok |
| `cargo audit` | pass — 219 crate dependencies, zero advisories |
| `make audit-static` | pass |
| `python3 -m pytest scripts/tests` | **156 passed, 3 skipped** — matches the PR body |

The base is confirmed: `05e0e4d` (#422 head) is an ancestor of `ef53ae7`, and
everything below is diffed against it rather than against `main`.

## Independent reproduction of the differential

The PR removes the two-engine comparator, so its parity claim cannot be
re-run from the tree. I wrote my own — a crate depending on **both**
`serde_yaml =0.9.34` and `yaml_serde =0.10.7` under Cargo renames — and
compared `serde_json::Value` results.

**Corpus population.** Every leading frontmatter block in the reviewed tree
plus the sibling schema providers:

```
compared=1008  value_differences=0  both_refused=0  old_only_ok=0  new_only_ok=0
```

That is 1,008 blocks against the manifest's 775, over `quire-rs` (including the
pinned `qa-corpus` submodule), `spec-artifacts-process` and
`spec-artifacts-iso`. Not one differs, in value or in outcome.

**Focused semantic cases.** 27 hand-written inputs aimed at where two libyaml
generations actually diverge — **zero differences**:

| Case | Both engines agree on |
| --- | --- |
| `duplicate-key` | `{"a":2}` — last wins, silently, on both |
| `merge-key` | `{"child":{"<<":{"x":1},"y":2}}` — `<<` left uninterpreted on both |
| `alias` / `anchor-on-seq` | expanded identically |
| `undefined-alias` | **refused by both** |
| `implicit-bool-yes` | `yes`/`no`/`on`/`off` stay **strings** on both |
| `norway` | `NO` stays the string `"NO"` on both |
| `sexagesimal` | `12:34:56` stays a string on both |
| `timestamp` | both dates stay strings on both |
| `octal` | `0o17`→15, `017`→`"017"`, `0x1f`→31, identically |
| `big-int` | `18446744073709551616` **refused by both** |
| `float-forms` | `.inf`/`-.inf`/`.nan` → null, `1e3` → 1000.0, identically |
| `non-string-key` | `1:`/`true:` → `"1"`/`"true"`, identically |
| `seq-key`, `tab-indent`, `trailing-colon` | **refused by both** |
| `percent-directive`, `tag-str`, `deep-nest`, `unicode-escape`, block scalars, flow collections | identical |

The "Norway problem" and the `yes`/`no` question are the two that would most
plausibly break document identity in this codebase, and both engines are on the
YAML 1.2 core-schema side of them, unchanged.

This is the strongest available answer to the deleted comparator: the claim was
not taken on trust, and it holds on a larger population than was claimed.

## Independently verified

| Claim | Check | Result |
| --- | --- | --- |
| `yaml_serde` is the maintained successor | `cargo info yaml_serde` | "serde_yaml maintained by The YAML Organization", `repository: https://github.com/yaml/yaml-serde`, MIT OR Apache-2.0, `rust-version: 1.82` — satisfied by 1.98.1 |
| `libyaml-rs` provenance | `cargo info libyaml-rs` | "libyaml transpiled to rust by c2rust", `https://github.com/yaml/libyaml-rs`, MIT |
| ADR-0012's unsafe claim | counted `unsafe` in the vendored sources of all four crates | `libyaml-rs 0.3.0`: **242** over 12,449 lines; `unsafe-libyaml 0.2.11`: **248** over 12,446. Serde layer: 62 vs 66. ADR-0012's "does not claim a reduction in transitive unsafe code" is exactly right |
| The source really is untouched | `git diff 05e0e4d..HEAD --stat` | **no `src/` file changes at all**; the Cargo package rename carries every `serde_yaml::` call site |
| `serde_yaml`/`unsafe-libyaml` leave the root graph | lock diff | both `[[package]]` blocks removed; `yaml_serde 0.10.7` + `libyaml-rs 0.3.0` added |

## Mutation evidence — `check_dep_pins.sh`

| # | Mutant | Result |
| --- | --- | --- |
| M0 | none (control) | passes, as it must |
| M1 | root `serde_yaml = "^0.9"` | **killed** — "serde_yaml must alias the yaml_serde package" |
| M2 | root version `=0.10.6` | **killed** — "must use exact version =0.10.7" |
| M3 | root `package = "serde_yaml_ng"` | **killed** |
| M4 | root `version = "^0.10"` (caret, not exact) | **killed** |
| M5 | **`fuzz/Cargo.toml`** reverted to `serde_yaml = "^0.9"` | **SURVIVED** |

M5 is FND-001.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The dependency audit never reads `fuzz/Cargo.toml`, so the fuzz graph can revert to the deprecated pair and pass | scripts/audits/check_dep_pins.sh:1, fuzz/Cargo.toml:13 | correct-requirement-no-evidence |
| FND-002 | medium | The retained manifest records `input_manifest_sha256` for a file that was never committed in any revision, and the differential payload was committed and then deleted | spec/evidence/yaml-migration/manifest-v1.json:22 | correct-requirement-no-evidence |
| FND-003 | low | The evidence-narrowing commit also removed a CI-wired standing audit, its implementation, its unit test and its baseline, which the PR body's summary does not say | .github/workflows/ci.yml:262, scripts/audits/check_yaml_callsites.sh | missing-requirement |

## Finding detail

### FND-001 — the fuzz graph is changed but not gated

The PR body says "root **and fuzz** graphs select libyaml-rs 0.3.0 and remove
the deprecated serde_yaml/unsafe-libyaml pair", and ADR-0012 says
"`scripts/audits/check_dep_pins.sh` enforces the exact alias and locked package
pair and rejects reintroduction of `serde_yaml` or `unsafe-libyaml`."

The audit reads the root manifest and the root `Cargo.lock`. `grep -n fuzz`
over both `scripts/audits/check_dep_pins.sh` and `scripts/tests/test_dep_pins.py`
returns **nothing**.

Measured: restoring `serde_yaml = "^0.9"` in `fuzz/Cargo.toml` — the exact line
this PR changed — leaves `bash scripts/audits/check_dep_pins.sh` at exit **0**.

Two things make this worse than a symmetric omission:

1. `fuzz/` has **no `Cargo.lock`** — only `fuzz/Cargo.toml` exists — so its
   graph resolves fresh on every run. A caret range there is not merely
   ungated, it floats.
2. The fuzz targets are where YAML parser behaviour is stress-tested. A fuzz
   workspace resolving the *old* engine would be exercising a parser the
   product no longer uses, which is the one place a silent divergence would be
   least likely to be noticed and most costly.

The fix is the same shape as the existing check: run the manifest assertions
against `fuzz/Cargo.toml` too, and add the case to
`scripts/tests/test_dep_pins.py`, which already parametrises the root ones.

### FND-002 — a digest whose subject was never retained

`manifest-v1.json` carries:

```json
"input_manifest_sha256": "6201f5aebf5d34f7ac85b6216e99d803837542ea9fea725ac450ef7c6a65a381",
"retention": "aggregate result only; the one-use comparator and per-input payload were removed after review"
```

Searching the whole repository history for a file matching `*input-manifest*`
or `*input_manifest*` returns nothing: the digest's subject was never committed
in **any** revision, so it cannot be checked by anyone, at any time, including
the author later.

The per-input payload is a different case, and its history is worth stating
precisely. Commit `75c724e` ("docs: retain YAML migration qualification
evidence") committed 34,812 lines of it — `differential-v1.json` at 25,609
lines, `negative-control-v1.json` at 8,816, `callsite-census-v1.json`, and
twelve per-benchmark JSON files. Commit `ef53ae7` deleted all of them. Retention
was not merely possible, it was done and then reversed.

ADR-0012 gives a real justification for not keeping the comparator — "A future
version decision must produce fresh evidence for that candidate rather than
rely on this one-time executable" — and I agree with it. That reasoning does
not extend to the digest: an attestation to a vanished file is weaker than no
attestation, because it reads as verifiable and is not.

The cheap close is to retain the input manifest alone. It is a list of 781
paths with digests — kilobytes, not 34,000 lines — and it turns
`input_manifest_sha256` into something a later reviewer can check, and the 781
population into something they can re-derive without the comparator. That is
exactly the distinction this repository drew for VP04 in `quire-verification`,
where the retained fixture bytes are what made a recorded digest mean anything.

### FND-003 — the removal was broader than "evidence volume"

The PR body describes the deletion as: "The one-use dual-parser workspace,
roughly 34,000 lines of per-input output, exact 95-call registry, and redundant
exploratory review artifacts were removed. **They are not production
capabilities or useful permanent gates.**"

Hand-reading `ef53ae7`, it also removed:

```
scripts/audits/check_yaml_callsites.sh      (6 lines)
scripts/audits/check_yaml_callsites.py      (122 lines)
scripts/tests/test_yaml_callsites.py        (81 lines)
quality/yaml-callsite-census.json           (28 lines)
.github/workflows/ci.yml                    (the check_yaml_callsites step)
```

A script in `scripts/audits/` is a permanent gate by construction — that
directory is what `make audit-static` globs, deliberately, so that "a new
`scripts/audits/*.sh` runs because it exists, not because somebody remembered".
`check_yaml_callsites.sh` was in it, and had a named step in the hosted
`audit-static` job, two commits before being called "not a useful permanent
gate".

I think the decision is defensible: a census pinning an exact count of 95
`serde_yaml::` call sites fails on any legitimate new call, which is an overfit
gate, and dependency-level enforcement survives in `check_dep_pins.sh` (M1–M4
above). This is low because the judgement is the owner's and the outcome is
reasonable. It is recorded because the summary sentence describes the removal
as evidence volume plus a registry, and a reader would not learn from it that a
CI-wired audit script, its Python implementation, its unit test and its
baseline went in the same commit.

## What is correct

- **The package-rename approach is the right one.** `serde_yaml = { package =
  "yaml_serde", version = "=0.10.7" }` keeps every `from_str`, `from_slice`,
  `from_value`, `Value`, `Mapping` and `to_string` call site untouched, so the
  diff has no source churn to review and no opportunity for a hand-port error.
  The comment above it says why.
- **ADR-0012 pre-empts the reviewer's first question.** Stating that both
  backends are C2Rust transpiles of libyaml and that the decision "does not
  claim a reduction in transitive unsafe code" is precisely the sentence that
  stops a reader inferring a safety win from the name `unsafe-libyaml`
  disappearing. My count — 242 against 248 — says it is accurate.
- **The alternatives table is real.** `serde_yml` is rejected with a named
  advisory (RUSTSEC-2025-0068), `serde_yaml_ng`/`serde_norway` are recorded as
  viable but not selected, and `yaml_serde 0.10.2` is rejected for having been
  chosen on obsolete metadata — an earlier version of this same decision being
  overturned rather than quietly replaced.
- **The exact pin is exact.** `=0.10.7`, not a caret, in both manifests, and M4
  proves the audit rejects the caret form.
- **The test count is unchanged.** 1,031 at the base and 1,031 here, including
  the 89 parser-parity tests, so the migration moved no test's outcome.
- **The stacking is honest.** The PR is draft, based on `#422`'s branch rather
  than `main`, and the body says it "remains draft until #422 lands and
  independent review is refreshed at this head". That is the correct handling
  and it is why this review is diffed against `05e0e4d`.

## One observation, not a finding

Both engines silently accept a duplicate mapping key and keep the last value —
`a: 1\na: 2` → `{"a":2}` on both. That is unchanged by this PR and therefore
out of its scope. It is worth noting because the migration's negative control
was an "injected duplicate-key outcome difference", which reads as though
duplicate keys are discriminated somewhere; in the frontmatter path they are
not, on either engine. If document identity should refuse a repeated key, that
is a separate decision against FR-006, not a YAML-package one.
