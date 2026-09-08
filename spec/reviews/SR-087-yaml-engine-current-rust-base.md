---
id: SR-087
title: "Base review of the YAML engine and current Rust correction (#414)"
type: SpecReview
analysis: base
scope: "ADR-0012, NFR-009, NFR-022, TC-330..TC-332, TC-1820..TC-1835"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/assets/adr/0012-yaml-engine-maintenance-and-parity"
    type: reviews
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-009"
    type: reviews
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-022"
    type: reviews
---
# SR-087: Base review of the YAML engine and current Rust correction (#414)

## Summary

The owner selected the `base` review subset. The corrected candidate separates
the repository-wide compiler policy from the YAML dependency decision, selects
current `yaml_serde 0.10.7` on exact Rust 1.98.1, and rejects the prior 0.10.2
selection because it was derived only from inherited Rust 1.75 metadata. The six
changed/index artifacts validate 6/6 grammar-clean with zero findings.

The specification is internally ready for an owner decision, but implementation
remains blocked by two honest evidence boundaries: the selected 0.10.7 engine has
not yet passed the retained differential, and repository-wide Quire validation
still fails the separately owned Engineering Assurance AP/MP contract mismatch.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-449 | high | **Closed:** the prior candidate selected `yaml_serde 0.10.2` solely because `Cargo.toml` and Clippy inherited Rust 1.75, even though `rust-toolchain.toml` already selected 1.94.1 and current stable is 1.98.1. Existing configuration is not compatibility evidence. ADR-0012 now selects current 0.10.7 with its current backend and makes the real parity/tool gates dispositive. | ADR-0012; NFR-009; TC-1820..TC-1824 |
| FND-450 | medium | **Closed:** the first correction placed repository-wide compiler lifecycle policy inside YAML dependency pinning. NFR-022 now independently governs exact current stable, future stable transitions, and bounded required-tool incompatibility holds; ADR-0012 depends on it. | NFR-022; ADR-0012 decision driver 3; TC-1832..TC-1835 |
| FND-451 | high | **Open external gate:** full `quire validate --scope . "spec/**/*.md" --summary` still fails 11 pre-existing documents because the installed Engineering Assurance contract rejects AP-201 and MP-201..MP-208, and two untyped assets lack archetypes. #414 does not rewrite those artifacts around their contract owner. | engineering-assurance #59; AP-201; MP-201..MP-208; FND-443 |
| FND-452 | high | **Open implementation evidence:** the 575-input exploratory result compared the deprecated engine with 0.10.2, not selected 0.10.7, and omitted reproducibility identities. It is explicitly non-dispositive. TC-1821 and TC-1827 require a new zero-difference result with complete provenance before the dependency change may land. | ADR-0012 differential evidence and required evidence 1; NFR-009-AC-6/12 |

## Base checklist and coverage

- ADR-0012, NFR-009, NFR-022, their criteria, and TC-330..TC-332 plus
  TC-1820..TC-1835 use valid, unique identifiers. NFR-022 follows the existing
  NFR-021 sequence; new tests follow TC-1831.
- NFR-009 AC-1..AC-15 and NFR-022 AC-1..AC-5 each map to at least one test.
  Every test row carries type, priority, trace, and an honest pending status.
- The YAML option table covers hold, archived/advised fork, alternate maintained
  forks, selected current engine, and non-drop-in redesign. The test permutations
  cover exact/current, deprecated, old, ranged, wrong, and absent selections.
- The semantic-risk population covers duplicate keys, merge keys, implicit
  scalars/timestamps, aliases, and non-string keys; a single difference blocks.
- The compiler transition matrix covers current, newer-compatible,
  newer-required-tool-incompatible, expired, unattributed, formatting-only,
  lint-only, existing-pin-only, and speculative holds.
- Package resolution, backend identity, source revisions/digests, language
  parity, typed consumers, compiler targets, license/advisory, static/unsafe,
  performance, census, and canonical ix-trace-rs reconciliation are distinct
  obligations rather than one aggregate assertion.
- No state transition permits implementation from a Proposed ADR, acceptance
  from a non-reproducible differential, or an older compiler from an existing
  pin. FND-451 and FND-452 remain visible instead of being converted into pass.

## Verdict

**PASS WITH EXTERNAL IMPLEMENTATION GATES** — the corrected #414 specification
passes the owner-selected base review. Human acceptance of ADR-0012 remains
pending. Implementation and package replacement remain prohibited until that
acceptance, a clean repository-wide validation result, and the new 0.10.7
differential evidence are present.
