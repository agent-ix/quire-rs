---
id: SR-093
title: "Independent review of the corrected YAML engine and compiler decision"
type: SpecReview
analysis: dependency
scope: "PR #416; ADR-0012; NFR-009; NFR-022; issues #414, #417"
review_set: subset
---

## Summary

The engine choice is right and the corrected reasoning is right: `yaml_serde`
0.10.7 is published by the YAML organisation, is MIT OR Apache-2.0, requires
Rust 1.82, and resolves `libyaml-rs 0.3.0` — all verified against crates.io
here. Two problems block acceptance as written. NFR-009-AC-5 promises a safety
improvement the swap does not deliver, and NFR-022 — a repository-wide compiler
lifecycle policy that #417 needs — is bundled into a YAML dependency decision,
which is what currently blocks #417.

## Verdict

**FAIL** — one AC states an outcome the change does not produce, and an
unrelated requirement is gated behind this ADR's acceptance.

## Verification performed

| Check | Result |
| --- | --- |
| `cargo info yaml_serde@0.10.7` | exists; "serde_yaml maintained by The YAML Organization"; repo `github.com/yaml/yaml-serde`; MIT OR Apache-2.0; rust-version 1.82 |
| `cargo info libyaml-rs` | 0.3.0; "libyaml transpiled to rust by c2rust"; repo `github.com/yaml/libyaml-rs`; MIT; rust-version 1.60 |
| `cargo tree` on `yaml_serde 0.10.7` | `indexmap`, `itoa`, `libyaml-rs 0.3.0`, `ryu`, `serde` |
| `rustup check` | 1.98.1 (48a229cea 2026-09-01) is current stable; the toolchain is installed here |
| Repository declarations at `f45ce24` | `Cargo.toml:5` `rust-version = "1.75"`; `clippy.toml:1` `msrv = "1.75"`; `rust-toolchain.toml:2` `1.94.1` |
| `Cargo.lock` | `unsafe-libyaml 0.2.11` present today, via `serde_yaml 0.9.34+deprecated` |

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Banning `unsafe-libyaml` while adopting `libyaml-rs` renames the same c2rust transpile; AC-10's unsafe gate cannot see the difference | spec/non-functional/NFR-009-dependency-pinning.md | wrong-requirement |
| FND-002 | high | NFR-022 is a compiler-lifecycle policy bundled into a YAML dependency PR, so #417 is blocked on accepting an unrelated ADR | spec/non-functional/NFR-022-current-stable-rust.md | wrong-requirement |
| FND-003 | medium | NFR-022-AC-3 and AC-5 declare verification methods that cannot discharge a calendar SLA or a governance judgement | spec/non-functional/NFR-022-current-stable-rust.md | wrong-requirement |
| FND-004 | medium | NFR-022-AC-1 collapses MSRV, clippy's lint gate and the build toolchain onto one number without stating the consumer impact | spec/non-functional/NFR-022-current-stable-rust.md | missing-requirement |
| FND-005 | medium | #417's first deliverable re-authors the requirement NFR-022 already is | spec/non-functional/NFR-022-current-stable-rust.md | wrong-requirement |
| FND-006 | low | Eleven SpecReviews of the rejected 0.10.2 candidate stay indexed beside one review of the accepted decision | spec/reviews/SR-076-yaml-engine-base.md | correct-requirement-no-evidence |

## Finding detail

### FND-001 — the ban is a rename

NFR-009-AC-5 requires that the lock and tree "contain neither
`serde_yaml 0.9.34+deprecated` nor `unsafe-libyaml`", and AC-10 requires the
"unsafe-surface and static dependency gates" to accept the selected graph with
zero findings. Read together those promise a reduction in unsafe surface.

`unsafe-libyaml` and `libyaml-rs` have the same stated provenance — libyaml
transpiled to Rust by c2rust — and `libyaml-rs` is the only non-serde
dependency `yaml_serde 0.10.7` pulls. The transpiled parser is not smaller or
safer for having moved to the YAML organisation's repository; it is
better-maintained, which is the actual and sufficient argument.

AC-10 will pass either way and prove nothing: the repository's unsafe gate is
`scripts/check_unsafe_comments.sh`, and NFR-003-AC-3/AC-4 scope it to
first-party `src/` (`rg 'unsafe\s*\{' src/`). It has never looked at the
dependency graph and does not now.

Two ways to close this, both cheap. State the claim the change supports —
"the parser core is unchanged; what changes is that it has a maintainer" — and
drop `unsafe-libyaml` from AC-5 to a lock-hygiene check rather than a safety
one. Or, if the transitive unsafe surface is genuinely in scope, add an AC that
measures it (`cargo geiger`, or a counted baseline) so a future engine swap has
something to compare against.

### FND-002 — two decisions, one acceptance gate

#414 asks for a YAML engine decision. NFR-022 is a repository-wide compiler
lifecycle policy: which stable Rust the repository qualifies on, how fast it
follows a new release, and what may justify a hold. Those are independent, and
NFR-022 is what #417 needs.

Bundling them means #417 cannot start until an ADR about a YAML parser is
accepted, and it means an owner who wants to accept the compiler policy has to
accept the dependency migration's evidence requirements at the same time. That
coupling is the reported block, not a review backlog.

ADR-0012 only needs 1.98.1 as an *input* — "0.10.7 requires Rust 1.82; current
stable satisfies it". It does not need to own the policy. Splitting NFR-022 into
its own spec change under #417, and leaving ADR-0012 to cite it, unblocks both.

### FND-003 — process obligations marked as tests

- NFR-022-AC-3: "A stable release newer than 1.98.1 triggers the same
  compatibility matrix within seven calendar days …" — Verification:
  `integration-testing`.
- NFR-022-AC-5: "A proposed hold based only on formatting drift, a repairable
  lint finding, existing repository metadata, or an untested compatibility
  concern is rejected." — Verification: `property-testing`.
- Measurement table: "Days from a newer stable release to compatibility result |
  at most 7 | 7 | integration-testing".

No integration test observes the calendar, and no property test adjudicates
whether a proposed hold is well-founded. These are `inspection` obligations, as
AC-4 already is. Left as they are, both will eventually be marked satisfied by a
test that verifies something adjacent — which is exactly the failure mode the
rest of this NFR set is careful about.

### FND-004 — three fields, three meanings, one number

AC-1 requires `Cargo.toml`, `rust-toolchain.toml` and `clippy.toml` to "name
exact Rust 1.98.1". Those fields do not mean the same thing:

- `Cargo.toml` `rust-version` is the MSRV **consumers** must satisfy. Raising it
  from 1.75 to 1.98.1 makes quire-rs unbuildable for anything below 1.98.1 —
  quire-cli, the maturin wheel lane, the WASM lane, and any external consumer.
- `clippy.toml` `msrv` gates which lints clippy is allowed to suggest.
- `rust-toolchain.toml` `channel` selects the compiler this repository builds
  with, and is the only one of the three that is genuinely a qualification
  statement.

The Scope section's "does not promise support for an older compiler merely
because an existing file names one" reads like a deliberate decision, and it may
well be the right one — but the consumer consequence is not stated anywhere in
NFR-022 or ADR-0012, and NFR-022 lists no downstream repository. Say which
consumers are expected to move, or narrow AC-1 to the toolchain and clippy
files and treat `rust-version` as a separate, argued decision.

### FND-005 — #417 should implement NFR-022, not re-author it

#417's first deliverable is "Use QUOIN `/specify` to update or add the narrow
normative toolchain requirement, then run the owner-selected `/spec-review` set
before implementation". That requirement is NFR-022. Once NFR-022 is accepted,
#417 is an implementation ticket against NFR-022-AC-1..AC-5 plus the
`Cargo.toml:5` / `clippy.toml:1` / `rust-toolchain.toml:2` / CI edits it already
enumerates. Filing a second requirement for the same policy is how two
declarations drift apart, which is the defect #417 opens by describing.

### FND-006 — eleven reviews of a withdrawn option

SR-076..SR-086 review the rejected 0.10.2 candidate; SR-087 reviews the
corrected one. Keeping the history is right, and the PR body is explicit that
their approval does not carry. But the reviews index does not distinguish them,
so the accepted decision is backed by one `base` review sitting among eleven
that look equally current. A `superseded_by` relationship, or a one-line status
in each, would make that readable without deleting the record.

## Not findings

- The engine selection is sound and the rejection of 0.10.2 ("inherited Rust
  1.75 metadata prevented Cargo from choosing the current line — existing
  configuration is not an architectural reason") is exactly the right argument.
- NFR-009's expanded AC set (AC-6 differential parity, AC-7 TS/Python parity,
  AC-12 reproducibility identities, AC-14 call-site census) is a genuinely
  strong acceptance bar for the migration, and AC-14 in particular closes the
  "a loader disappeared from the population" hole before it opens.
- This PR touches no manifest, lockfile, production source or test. That
  restraint is correct for a decision PR and should survive the split proposed
  in FND-002.
