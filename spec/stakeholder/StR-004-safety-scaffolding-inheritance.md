---
id: StR-004
title: "Inherit Rust Safety Scaffolding from rust-lib-cookiecutter / ecaz"
type: StR
relationships:
  - target: "ix://agent-ix/rust-lib-cookiecutter"
    type: "satisfied_by"
    cardinality: "1:1"
  - target: "ix://agent-ix/ecaz"
    type: "satisfied_by"
    cardinality: "1:1"
---

## Stakeholder Need

`agent-ix/ecaz` has invested heavily in Rust safety scaffolding (clippy MSRV pinning, `deny.toml` license policy, `// SAFETY:` enforcement on every `unsafe` block, CI gates for fmt/clippy/test/license/unsafe-audit). That investment was backported into `agent-ix/rust-lib-cookiecutter` as the org's first Rust scaffold.

`quire-rs` was scaffolded from that cookiecutter and SHALL keep the inherited safety configuration. Local changes to `clippy.toml`, `deny.toml`, `rustfmt.toml`, or `scripts/check_unsafe_comments.sh` are allowed only when they tighten the policy, never to loosen it.

If `ecaz` or `rust-lib-cookiecutter` advances their safety rules, `quire-rs` SHALL adopt the changes via a tracked backport (see `backport-code` skill conventions).

## Rationale

This need exists because `agent-ix/ecaz` invested heavily in Rust safety scaffolding — clippy MSRV pinning, a `deny.toml` license policy, `// SAFETY:` enforcement on every `unsafe` block, and CI gates for fmt/clippy/test/license/unsafe-audit — and that investment was backported into `rust-lib-cookiecutter` as the org's first Rust scaffold. `quire-rs` was scaffolded from that cookiecutter, so letting its inherited safety configuration drift or loosen would forfeit the org-wide guarantees the scaffold exists to provide and would fork `quire-rs` away from the shared baseline. Keeping local changes constrained to tightening-only, and adopting upstream advances via tracked backports, preserves the single source of safety truth.

## Validation Criteria

| ID | Criteria | Validation |
|----|----------|------------|
| StR-004-VC-1 | `quire-rs/clippy.toml`, `deny.toml`, `rustfmt.toml`, `rust-toolchain.toml`, and `scripts/check_unsafe_comments.sh` match the `rust-lib-cookiecutter` baseline at scaffold time, modulo MSRV updates. | Inspection |
| StR-004-VC-2 | `make ci` enforces the full suite — fmt-check, clippy `-D warnings`, test, deny, audit-unsafe — the same gates the cookiecutter ships. | Demonstration |
| StR-004-VC-3 | Every upstream safety-file update in `ecaz` or `rust-lib-cookiecutter` produces a backport issue against `quire-rs` that references the upstream commit. | Inspection |

## Priority

Must-Have

## Acceptance

- **StR-004-AC-1**: `quire-rs/clippy.toml`, `deny.toml`, `rustfmt.toml`, `rust-toolchain.toml`, and `scripts/check_unsafe_comments.sh` match the rust-lib-cookiecutter baseline at the time of scaffold (modulo MSRV updates).
- **StR-004-AC-2**: `make ci` enforces the full suite (fmt-check, clippy `-D warnings`, test, deny, audit-unsafe) — same gates as the cookiecutter ships.
- **StR-004-AC-3**: When `agent-ix/ecaz` or `rust-lib-cookiecutter` updates a safety file, a backport issue is opened against `quire-rs` referencing the upstream commit.
