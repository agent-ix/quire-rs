---
id: SR-115
title: "Independent re-review — maintained YAML engine migration at e0a19b0"
type: SpecReview
analysis: code-review
scope: "PR #423 at e0a19b0 against its merged base 85dfe9d; SR-111 FND-4148/FND-4149; scripts/audits/check_dep_pins.sh; scripts/tests/test_dep_pins.py; spec/evidence/yaml-migration/manifest-v1.json"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-009"
    type: reviews
---

## Summary

Re-review at `e0a19b0`, the head produced by rebasing onto the merged `#422`
squash `85dfe9d`. The PR-only delta since the reviewed head `ef53ae7` is exactly
the two SR-111 remediations and nothing else: the dependency audit gained a
second Cargo-parsed manifest surface, and the unverifiable
`input_manifest_sha256` was deleted rather than reconstructed. Both close. All
gates were run at this head, not assumed.

One new finding: the fuzz-manifest coverage added for FND-4148 verifies the
*declared* alias only. The forbidden-package check still reads the root
`Cargo.lock` alone, and `fuzz/Cargo.lock` is untracked, so the fuzz graph has no
verified resolution. A fixture that declares `unsafe-libyaml` — the exact
package ADR-0012 forbids — in `fuzz/Cargo.toml` passes the audit with exit 0.

## Verdict

**CONDITIONAL** — no high findings. One new medium; both SR-111 findings close.

## Gates run at `e0a19b0`

Exact Rust 1.98.1, isolated `CARGO_TARGET_DIR`, `corpus/` submodule initialised
at `7442f27`.

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | pass |
| `cargo test --locked` | pass — **1,031 passed, 0 failed, 0 ignored**, identical to the SR-111 baseline |
| `cargo deny --locked check` | advisories, bans, licenses, sources ok |
| `make audit-static` | pass, including `check_dep_pins` and `check_tool_drift` |
| `python3 -m pytest scripts/tests` | **164 passed, 3 skipped** |
| `python3 -m pytest scripts/tests/test_dep_pins.py` | **19 passed** — matches the PR body |

The pytest total moved 156 → 164 because this delta adds the eight new
dependency-pin cases; no previously passing case was removed.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The fuzz graph's resolved packages are never verified; a `unsafe-libyaml` declaration in `fuzz/Cargo.toml` passes `check_dep_pins.sh` | scripts/audits/check_dep_pins.sh:62; scripts/tests/test_dep_pins.py:73 | correct-requirement-no-evidence |

## Finding detail

### FND-001 — the fuzz surface is checked for identity, not for the forbidden pair

`check_dep_pins.sh` now runs `cargo metadata` over both manifests, and the
Python block loops over `("root", …)` and `("fuzz", …)`. That loop only ever
inspects dependencies whose `rename` is `serde_yaml`, so it answers one
question: *is the alias present and exactly `=0.10.7`?*

The second question — *is the deprecated pair absent, and is `libyaml-rs` the
selected implementation?* — is answered at lines 62–77 against `LOCKFILE`,
which is `$ROOT/Cargo.lock` only. There is no fuzz equivalent, and there cannot
easily be one: `.gitignore:32` untracks `fuzz/Cargo.lock`, which is also why the
fuzz `cargo metadata` at line 35 correctly omits `--locked`. The consequence is
that the fuzz graph is resolved fresh on every run and its resolution is never
compared to policy.

`--no-deps` compounds it: the fuzz metadata contains declared dependencies only,
so a transitive reappearance is invisible, and a *direct* declaration is
filtered out by the `rename == "serde_yaml"` predicate before it can be seen.

Demonstrated, not inferred. Using this branch's own audit script against a
fixture built like `scripts/tests/test_dep_pins.py::fixture`, with one line added
to the fuzz manifest:

```toml
[dependencies]
serde_yaml = { package = "yaml_serde", version = "=0.10.7" }
unsafe-libyaml = "0.2"
```

```
$ bash scripts/audits/check_dep_pins.sh
check_dep_pins: OK
AUDIT_EXIT=0
```

The PR body states that "root and fuzz graphs select `libyaml-rs 0.3.0` and
remove the deprecated `serde_yaml`/`unsafe-libyaml` pair". At this head that
holds in fact — the fuzz graph resolves to `libyaml-rs 0.3.0` and
`yaml_serde 0.10.7`, confirmed by `cargo metadata` on `fuzz/` — but the gate
proves it only for `root`. The claim and the gate are not the same width.

The five new mutants inherit the same boundary: every one of them edits the
`serde_yaml` line, so the suite cannot distinguish "the alias is right" from
"the forbidden pair is absent" on the fuzz surface.

Two honest resolutions, either acceptable:

- drop `--no-deps` for the fuzz manifest and apply the existing
  required/forbidden package assertions to its resolved graph, and add a mutant
  that declares `unsafe-libyaml`; or
- narrow the PR body and `NFR-009-AC-5` to the claim the gate actually makes —
  the fuzz manifest's declared YAML alias is exact — and record the fuzz graph's
  resolution as unpinned by design.

## Closed since SR-111

| SR-111 finding | Status at `e0a19b0` |
| --- | --- |
| FND-4148 — dependency audit did not cover the fuzz manifest | Closed for the declared alias. Cargo parses both manifests; five fuzz mutants fail closed. Residual scope is FND-001 above. |
| FND-4149 — evidence claimed a digest for an input manifest that was never retained | Closed. `input_manifest_sha256` is deleted and `retention` now states that no digest is claimed. No 781-path artifact was reintroduced. |

## Review checklist

- The rebase is honest: the PR-only patch at `85dfe9d..e0a19b0` differs from
  `05e0e4d..ef53ae7` in exactly the two remediations and the fuzz manifest pin.
  No unrelated change entered under cover of the rebase.
- `85dfe9d` is the squash-merge of `#422` and is tree-identical to that PR's
  head `aac18a3`; the base is what it claims to be.
- No parser source or public API changed; the 1,031-test result is unchanged
  from the base, so the dependency swap remains behaviour-preserving under the
  suite that exists.
- The `mktemp` pair is released by a single `trap … EXIT` covering both files.
- The wildcard scan iterates both manifests and reports the offending path
  relative to `$ROOT`, so a fuzz-side wildcard is attributable.
