---
id: SR-106
title: "Base review of Rust 1.98.1 implementation and qualification"
type: SpecReview
analysis: base
scope: "NFR-022; TC-1832..TC-1835; issue #417 implementation"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-022"
    type: reviews
---

## Summary

This QUOIN base review checks the implementation against the accepted NFR-022
policy and its four test cases. All active stable declarations now agree on
Rust 1.98.1, the real qualification matrix passes without a compiler hold, and
the inspection-only lifecycle cases have concrete pass and refusal witnesses.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-4171 | high | Closed: the advisory gate exposed locked `anyhow` 1.0.102 as affected by RUSTSEC-2026-0190; the lock now selects patched 1.0.103 and a fresh audit reports no advisory. | `Cargo.lock`; RUSTSEC-2026-0190; NFR-022-AC-2 | implementation-bug-despite-evidence |
| FND-4172 | medium | Closed: the inherited drift audit accepted any numeric Rust pin and did not compare Cargo, Clippy, workflows, or build-script selectors to one policy value; six adversarial mutations now prove those roles fail closed. | `scripts/audits/check_tool_drift.sh:16-38`; `scripts/audits/check_tool_drift.sh:127-142`; `scripts/audits/check_tool_drift.sh:182-207`; TC-1832 | correct-requirement-no-evidence |
| FND-4173 | medium | Closed: Rust 1.98.1 surfaced nine repairable Clippy findings and 21 strict-rustdoc findings; all were repaired and neither class was misrepresented as tool incompatibility. | `src/corpus/query.rs:93`; `src/grammar/property.rs:771`; `tests/corpus_case/grading.rs:22`; TC-1833 | correct-requirement-no-evidence |

## Qualification evidence

| Gate class | Exact command or observation | Result |
| --- | --- | --- |
| declarations | `bash scripts/audits/check_tool_drift.sh`; 24 mutation/fixture tests | pass; Cargo, Clippy, repository toolchain, six stable CI installs, and stable build-script selectors agree on 1.98.1 |
| formatting | `cargo +1.98.1 fmt --all -- --check` | pass; rustfmt reports the inherited nightly-only import-grouping settings but no drift |
| all targets and features | `cargo +1.98.1 clippy --locked --all-targets --all-features -- -D warnings` | pass |
| default tests | `cargo +1.98.1 test --locked` with the pinned qa-corpus submodule | pass; 609 library tests and every integration/doc suite pass |
| Python | `cargo +1.98.1 check --locked --features python`; ABI3 release wheel installed into an isolated venv | pass; 40 binding tests pass; the optional `filament_parser` speedup comparison skips because that external package is absent |
| WASM | locked `wasm32-unknown-unknown` check plus the four semantic contract/property/clause/surface suites | pass; 37 host-executed WASM-feature tests pass |
| documentation | `RUSTDOCFLAGS='-D warnings' cargo +1.98.1 doc --locked --no-deps --all-features` | pass |
| advisory | `cargo +1.98.1 audit` after updating `anyhow` to 1.0.103 | pass; zero advisories reported |
| license and unsafe | locked `cargo deny check licenses`; unsafe and property-purity audits | pass; cargo-deny retains non-failing unused-allowance warnings |
| static audits | `make audit-static` | pass; the pre-existing status-agreement audit reports 52 advisory ledger findings and remains accurately advisory |
| concurrency | `RUSTFLAGS='--cfg loom' cargo +1.98.1 test --locked --test concurrency` | pass; two loom tests |
| benchmark build | `cargo +1.98.1 bench --locked --no-run` | pass; library and six benchmark executables compile |
| release | `cargo +1.98.1 build --locked --release --all-features` | pass |

The scripts test sweep additionally reports 142 passed and three intentional
consumer-workspace skips because this linked worktree has no sibling
`quire-cli`; those tests are not substituted for any NFR-022 gate above.

## Lifecycle inspection

For TC-1834, the reviewer simulated a hypothetical Rust 1.99.0 release on
2026-10-01. NFR-022 requires a recorded compatibility result no later than
2026-10-08. An advance must move every stable declaration together and rerun
the matrix above. A hold recorded on 2026-10-08 is admissible only when it names
the required failing tool and exact command, retains the failing 1.99.0 run and
successful 1.98.1 control, links the upstream issue, assigns an owner and rerun
trigger, and expires no later than 2026-11-07.

For TC-1835, the reviewer separately proposed holds whose sole basis was
formatting drift, a repairable lint, the inherited pin, speculation, or an old
downstream consumer. NFR-022-AC-5 rejects every proposal; none supplies the
required failing-tool reproduction in AC-4.

## Review disposition

Pass. TC-1832 through TC-1835 are satisfied for the Rust 1.98.1 transition,
and no compiler hold is recorded or required. Publication and merge remain
subject to independent review of the implementation branch.
