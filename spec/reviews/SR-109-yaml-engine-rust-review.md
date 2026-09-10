---
id: SR-109
title: "Rust review of the maintained YAML engine migration"
type: SpecReview
analysis: code-review
scope: "issue #414 dependency and audit diff against the exact-Rust baseline"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-009"
    type: reviews
---

## Summary

This review applies `agent-skills/rust-review/SKILL.md` and the repository's
AGENTS.md and CLAUDE.md conventions. The final production change is a dependency
alias and lock update; it adds no parser branches, public Rust API, unsafe code,
async work, locks, allocation path, or error conversion.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-4145 | high | Closed: the first implementation promoted a disposable dual-parser comparator into a permanent Rust workspace, increasing dependency, parsing, panic, resource, and maintenance surface unrelated to runtime behavior. The tool and its private lock were removed after the aggregate result was retained. | ADR-0012; SR-108 | wrong-requirement |
| FND-4148 | medium | Closed: the dependency audit covered the root manifest but not the independently resolved fuzz manifest. Cargo now parses both surfaces, and five fuzz-manifest mutants prove that the deprecated package, alias removal, version drift, and wildcard pin all fail closed. | `scripts/audits/check_dep_pins.sh`; `scripts/tests/test_dep_pins.py`; NFR-009-AC-1/3/5 | correct-requirement-no-evidence |
| FND-4149 | medium | Closed: the aggregate evidence claimed a digest for an input manifest that was never retained. The unverifiable digest was removed rather than reintroducing a 781-path artifact; retained claims are now explicitly limited to aggregate counts and pinned source revisions. | `spec/evidence/yaml-migration/manifest-v1.json` | correct-requirement-no-evidence |
| FND-4150 | medium | Closed after SR-115: the fuzz audit proved the alias but not the resolved graph, so a direct `unsafe-libyaml` declaration passed. Full offline Cargo metadata now checks the fuzz graph for both required and forbidden package identities, and direct `serde_yaml` and `unsafe-libyaml` mutants fail. | `scripts/audits/check_dep_pins.sh:35`; `scripts/audits/check_dep_pins.sh:63`; `scripts/tests/test_dep_pins.py:112` | correct-requirement-no-evidence |
| FND-4146 | medium | Closed: an exact source call-count census would fail on benign test refactors and duplicated compiler coverage without proving semantic compatibility. It and its bespoke Python mutation suite were removed; existing behavioral suites remain the gate. | NFR-009-AC-7; TC-1822; TC-1826 | wrong-requirement |
| FND-4147 | medium | Closed: the first durable dependency gate matched one exact TOML line, making harmless field ordering part of policy and widening into unrelated validator/range checks. The final gate asks Cargo to parse the YAML alias, inspects only the selected/forbidden YAML lock entries, and includes a passing field-order control. | `scripts/audits/check_dep_pins.sh`; `scripts/tests/test_dep_pins.py` | implementation-bug-despite-evidence |

## Review checklist

- Patch fidelity: the existing `serde_yaml` import name and all call semantics
  remain unchanged; no parser code was modified.
- Errors and panic surface: no product error path, unwrap, expect, index, or
  arithmetic operation was added.
- Ownership and API: no new public item, trait, wire field, state, or lifecycle
  boundary exists.
- Safety and concurrency: no first-party unsafe, blocking/async interaction,
  lock, channel, task, or cancellation path was introduced.
- Resource bounds: removing the comparator eliminates the only new recursive
  walk and whole-corpus accumulator.
- Tests: 21 dependency-audit tests exercise the durable policy, including two
  direct deprecated-package mutants in the independently resolved fuzz graph. No Rust
  test was added, so no new ix-trace-rs marker is required; existing Rust tests
  retain their repository-standard markers.
- Dependency boundary: the root lock contains only the selected YAML package
  pair, and the audit independently verifies the exact alias in both root and
  fuzz manifests while refusing the deprecated locked pair.

## Gate results

| Gate | Result |
| --- | --- |
| exact compiler | Rust 1.98.1 |
| `cargo fmt -- --check` | pass; stable rustfmt emitted only the repository's known nightly-option warnings |
| all-target/all-feature Clippy with warnings denied | pass |
| full locked all-feature Rust suite | pass; 609 library tests plus every integration and doc suite |
| parser parity | pass; 89 tests |
| focused dependency-audit suite | pass; 21 tests, including five alias/pin mutants and two forbidden-package fuzz-graph mutants |
| complete audit-script suite | pass; 166 passed, 3 skipped |
| fuzz workspace bins | pass |
| cargo-deny | pass; only pre-existing unused allowance/exception warnings |

## Review disposition

Pass after SR-115 remediation with no open Rust finding.
