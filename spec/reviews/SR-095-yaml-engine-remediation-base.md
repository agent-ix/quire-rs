---
id: SR-095
title: "Base review of YAML decision remediation"
type: SpecReview
analysis: base
scope: "ADR-0012; NFR-009; TC-330..TC-332; TC-1820..TC-1831; issue #414"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/assets/adr/0012-yaml-engine-maintenance-and-parity"
    type: reviews
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-009"
    type: reviews
---

## Summary

This QUOIN base review re-examines the YAML decision after SR-093. The compiler
lifecycle is now independently specified by #417, and the YAML decision states
the evidence it actually supports: active maintenance ownership with retained
C2Rust parser lineage, not a transitive unsafe-code reduction.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-204 | high | Closed: ADR-0012 and NFR-009 explicitly state that libyaml-rs and the former backend share C2Rust-transpiled libyaml lineage; AC-5 is a package-resolution/maintenance identity check, and AC-10 is limited to the unchanged first-party unsafe gate without a transitive-safety claim. | ADR-0012 Decision and Consequences; NFR-009-AC-5; NFR-009-AC-10; TC-1820; TC-1829 | wrong-requirement |
| FND-205 | high | Closed: NFR-022 and TC-1832..TC-1835 are removed from #414 and independently authored under #417 / PR #419. This ADR consumes Rust 1.98.1 only as the implementation-run compiler and does not own stable-release lifecycle policy. | ADR-0012 Decision; #417; PR #419 | wrong-requirement |
| FND-206 | low | Closed: SR-076..SR-086 visibly identify the withdrawn 0.10.2 candidate and point to SR-087; SR-087 identifies its intermediate bundled-policy scope and points to this review. Historical reviews remain retained without appearing current. | SR-076..SR-087; SR-095 | correct-requirement-no-evidence |

## Review disposition

The corrected YAML-only specification subset passes the owner-selected base
review. ADR acceptance and implementation evidence remain separate gates: the
0.10.7 differential, typed-consumer suites, dependency graph, performance,
license/advisory, first-party unsafe, call-site census, and bare ix-trace-rs
reconciliation must still be produced at the implementation revision.
