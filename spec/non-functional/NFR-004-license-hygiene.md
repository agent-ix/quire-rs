---
id: NFR-004
title: "License Hygiene Across Transitive Dependencies"
type: NFR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-004"
    type: "implements"
    cardinality: "1:1"
---

## Statement

Every direct and transitive dependency of `quire-rs` SHALL satisfy the license policy in `deny.toml` (inherited from `rust-lib-cookiecutter`):

Allowed licenses: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, CDLA-Permissive-2.0, ISC, Unicode-3.0, Zlib.

`cargo deny check licenses` SHALL exit 0 in CI. Any new dependency whose license is not in the allowlist requires either:

1. A license exception in `deny.toml` with a comment explaining why this specific dependency is acceptable, or
2. A different dependency choice.

Unknown registries and unknown git sources SHALL be denied; explicit `[sources]` entries are required for any exception.

## Rationale

Org-wide license policy prevents copyleft surprises in a redistributable crate. The `rust-lib-cookiecutter` baseline reflects org-level legal posture; deviations require explicit acknowledgment.

## Acceptance Criteria

- **NFR-004-AC-1**: `make deny` (alias for `cargo deny check licenses`) exits 0.
- **NFR-004-AC-2**: A test PR adding a GPL-licensed dependency fails the `licenses` job in CI.
- **NFR-004-AC-3**: `deny.toml` source bans (`unknown-registry = "deny"`, `unknown-git = "deny"`) are preserved.

## Verification

- CI workflow `.github/workflows/ci.yml` runs the `licenses` job on every PR.
