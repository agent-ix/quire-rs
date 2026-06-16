---
id: StR-004
title: "Inherit Rust Safety Scaffolding from rust-lib-cookiecutter / ecaz"
type: StR
relationships:
  - target: "ix://agent-ix/rust-lib-cookiecutter"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/ecaz"
    type: "implements"
    cardinality: "1:1"
---

## Stakeholder Need

`agent-ix/ecaz` has invested heavily in Rust safety scaffolding (clippy MSRV pinning, `deny.toml` license policy, `// SAFETY:` enforcement on every `unsafe` block, CI gates for fmt/clippy/test/license/unsafe-audit). That investment was backported into `agent-ix/rust-lib-cookiecutter` as the org's first Rust scaffold.

`quire-rs` was scaffolded from that cookiecutter and SHALL keep the inherited safety configuration. Local changes to `clippy.toml`, `deny.toml`, `rustfmt.toml`, or `scripts/check_unsafe_comments.sh` are allowed only when they tighten the policy, never to loosen it.

If `ecaz` or `rust-lib-cookiecutter` advances their safety rules, `quire-rs` SHALL adopt the changes via a tracked backport (see `backport-code` skill conventions).

## Priority

Must-Have

## Acceptance

- **StR-004-AC-1**: `quire-rs/clippy.toml`, `deny.toml`, `rustfmt.toml`, `rust-toolchain.toml`, and `scripts/check_unsafe_comments.sh` match the rust-lib-cookiecutter baseline at the time of scaffold (modulo MSRV updates).
- **StR-004-AC-2**: `make ci` enforces the full suite (fmt-check, clippy `-D warnings`, test, deny, audit-unsafe) — same gates as the cookiecutter ships.
- **StR-004-AC-3**: When `agent-ix/ecaz` or `rust-lib-cookiecutter` updates a safety file, a backport issue is opened against `quire-rs` referencing the upstream commit.
