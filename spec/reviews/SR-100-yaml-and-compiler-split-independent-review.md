---
id: SR-100
title: "Independent review of the separated YAML decision and compiler policy"
type: SpecReview
analysis: dependency
scope: "PR #416 at 5d591c7; PR #419 at cf27f0d; ADR-0012; NFR-009; NFR-022; issues #414, #417"
review_set: subset
---

## Summary

Both SR-093 high findings are closed, each by the correction the finding asked
for rather than by softer wording. NFR-009-AC-5 and AC-10 now say what the
package swap does and does not prove, and ADR-0012 records that `libyaml-rs`
and `unsafe-libyaml` share one C2Rust-transpiled libyaml lineage. NFR-022 is out
of the YAML PR entirely and is authored under #417 as PR #419, where its three
process criteria are typed `inspection` and its three configuration fields each
carry a stated role and a named downstream consequence. One low observation
remains about how tightly the YAML migration is now bound to the compiler change.

## Verdict

**CONDITIONAL** — no high or medium findings. One low, recorded so the sequencing
is deliberate rather than discovered during implementation.

## Verification performed

| Check | Result |
| --- | --- |
| `rustup check` | 1.98.1 (48a229cea 2026-09-01) is current stable and installed here |
| `cargo info libyaml-rs` | 0.3.0, "libyaml transpiled to rust by c2rust" — the lineage claim in ADR-0012 is accurate |
| `quire validate --scope . "spec/**/*.md"` on #416 and #419 | the 11 failing documents are `spec/assets/*` frontmatter and the AP-201/MP-20x AssuranceProfile schema mismatch, identical on both branches and on `main`; no document changed by either PR fails |
| Stable-toolchain census across the repository | `.github/workflows/{ci,python,loom,overfit}.yml` pin `1.94.1` in nine places; `rust-toolchain.toml` 1.94.1; `Cargo.toml` 1.75; `clippy.toml` 1.75. No floating `stable` selection exists, so NFR-022-AC-1's "no floating selection remains" is satisfiable as written |
| `ci.yml:141-145` | the only recorded newer-Rust incident is a `cargo-audit` installer resolving `kstring 2.0.4` (needs 1.96) against the 1.94.1 job — already repaired by pinning `cargo-audit@0.22.2`, and evidence *for* advancing rather than a standing hold |

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | NFR-009-AC-8 pins the YAML migration's acceptance to exact 1.98.1 although `yaml_serde 0.10.7` needs only 1.82, so #414 cannot be implemented until #417 lands | spec/non-functional/NFR-009-dependency-pinning.md | wrong-requirement |

## Finding detail

### FND-001 — the split removed one coupling and left the other

The separation is right and it fixes the reported block: #417 no longer waits on
a YAML ADR being accepted. In the other direction the dependency is still exact.
NFR-009-AC-8 requires the migrated crate to "compile with exact Rust 1.98.1",
and ADR-0012's implementation gate 4 repeats it. The repository declares
`rust-version = "1.75"` today, and that declaration is the actual reason 0.10.2
resolved instead of 0.10.7 — ADR-0012 says so, correctly.

But the constraint `yaml_serde 0.10.7` imposes is **1.82**, not 1.98.1. The YAML
migration needs `Cargo.toml` `rust-version` raised past 1.82; it does not need
the compiler-lifecycle policy accepted. As written, #414's implementation cannot
start until #419 merges and #417 lands the declaration changes.

That may well be the intended order — landing both together is simpler than
staging an interim 1.82 — but the ADR states the opposite relationship
("consumes 1.98.1 as the implementation-run compiler and does not own the
compiler lifecycle"), and NFR-022's own Dependencies section says dependency
decisions "may cite this compiler baseline but do not own or gate it." Here the
baseline gates the dependency decision. Either say in NFR-009-AC-8 that the
migration deliberately waits on #417, or state the requirement as the engine's
real floor (at least 1.82, and the repository's accepted stable at the
implementation revision).

## Closed since SR-093

Verified closed:

- **FND-001 (high) — the ban was a rename.** AC-5 now reads "neither
  `serde_yaml 0.9.34+deprecated` nor its former `unsafe-libyaml` backend; this
  verifies the selected maintenance line and package identity, not a reduction
  in transitive unsafe code." AC-10 is now scoped to "the existing first-party
  unsafe-surface gate" and states explicitly that it makes no transitive claim.
  TC-1820 and TC-1829 carry the same correction, the measurement row says
  "first-party unsafe-surface", and ADR-0012's Consequences name the shared
  C2Rust lineage. The engine choice was always right; the claim now matches it.
- **FND-002 (high) — two decisions, one acceptance gate.** NFR-022 is deleted
  from #416 (76 lines), removed from the non-functional index, dropped from
  ADR-0012's relationships, decision drivers, and reopen triggers, and its five
  TCs and EC-205 are removed from `spec/tests.md`. It is authored under #417 as
  #419. #417 is unblocked.
- **FND-003 — process obligations marked as tests.** NFR-022-AC-3 and AC-5 are
  now `inspection`, matching AC-4, and the measurement row for "Days from a
  newer stable release to a recorded compatibility result" is `inspection`
  rather than `integration-testing`. TC-1834 and TC-1835 are typed `Inspection`.
  No test is asked to observe a calendar or adjudicate a judgement.
- **FND-004 — three fields, one number.** NFR-022's Scope now separates
  `Cargo.toml` `rust-version` (the consumer minimum), `rust-toolchain.toml`
  (the qualification compiler) and `clippy.toml` `msrv` (the lint assumption
  bound), states that ending support below 1.98.1 is intentional, and names the
  consumers that must move: `quire-cli`, the in-repository Python and WASM
  lanes, `quire-wasm`, and external Rust consumers.
- **FND-005 — #417 re-authoring NFR-022.** #419 *is* the NFR-022 authoring, so
  the duplicate requirement never came into existence.
- **FND-006 — eleven reviews of a withdrawn option.** SR-076..SR-086 each carry
  "Superseded by SR-087. This review evaluates the withdrawn 0.10.2 candidate
  and remains only as decision history", and SR-087 itself carries "Superseded
  by SR-095". The history is kept and is no longer ambiguous.

## What is correct

- The engine selection and its stated reason are unchanged and remain the right
  argument: an inherited configuration value is not an architectural reason to
  freeze new work.
- NFR-022's Rationale is careful in the same way — "Neither number is justified
  merely because it exists" — and the requirement forbids exactly the three
  cheap excuses (formatting drift, repairable lint findings, existing metadata)
  that would otherwise let a stale pin become policy.
- Neither PR touches a manifest, lockfile, production source, or test. For a
  decision PR and a policy PR that is the correct footprint.
- NFR-009's acceptance set for the migration itself (AC-6 differential parity,
  AC-7 cross-language parity, AC-12 reproducibility identities, AC-14 call-site
  census) is unchanged and remains a strong bar.
