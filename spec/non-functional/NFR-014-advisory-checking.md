---
id: NFR-014
title: "RustSec Advisory Checking (cargo-audit)"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-004"
    type: "traces_to"
    cardinality: "1:1"
---

## Statement

`quire-rs` SHALL run `cargo-audit` against `Cargo.lock` on every PR and on a daily schedule. `cargo-audit` checks the RustSec Advisory Database for known vulnerabilities in dependencies.

This is a separate signal from `cargo deny check advisories` ([NFR-004](./NFR-004-license-hygiene.md) / deny.toml). cargo-audit pulls the advisory database fresh on each run; cargo-deny uses whatever's in the local database. Both are run; the redundancy catches RustSec advisories filed between cargo-deny releases.

### Operational policy

- CI job `audit:` in `.github/workflows/ci.yml`.
- Triggers: `pull_request`, `push` to main, `schedule: cron: "0 6 * * *"` (daily 6am UTC).
- Job uses `actions-rs/audit-check@v1` or `rustsec/audit-check@v1` action.
- New advisory match = PR fails (block merge). On scheduled runs, new advisory = issue opened.
- Ignoring an advisory requires an explicit `[advisories.ignore]` entry in `deny.toml` with a one-line rationale (same convention as ECAZ).

## Rationale

Supply-chain attacks on Rust crates have happened (e.g. `bincode` advisories, `serde_cbor` deprecation). Daily advisory checking ensures we hear about it before downstream consumers do.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-014-AC-1 | `.github/workflows/ci.yml` contains an `audit:` job running cargo-audit on PR + push + daily. | Inspection |
| NFR-014-AC-2 | An advisory ignored via `deny.toml` has a one-line rationale comment. | Inspection |
| NFR-014-AC-3 | A test PR adding a crate with a known historical advisory (e.g. an old `chrono` version with the RUSTSEC-2020-0071 fix needed) fails the `audit:` job. | Demonstration |

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Unaddressed RustSec advisories matching `Cargo.lock` | 0 | 0 | CI Gate (`cargo-audit`) |
| `audit:` job runs on PR + push + daily schedule | Pass | Pass | Inspection (CI workflow) |
| Ignored advisory carries one-line rationale in `deny.toml` | Pass | Pass | Inspection |
| New advisory match blocks merge | Pass | Pass | CI Gate |

## Verification

- CI workflow visible.
- `make cargo-audit` local target invokes the same command.
