---
id: SR-094
title: "Base review of exact Rust 1.98.1 qualification"
type: SpecReview
analysis: base
scope: "NFR-022; TC-1832..TC-1835; issue #417"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-022"
    type: reviews
---

## Summary

This QUOIN base review checks the independently authored compiler-lifecycle
policy requested by issue #417 and split from YAML issue #414. The requirement
now states the different meanings and consequences of Cargo MSRV, repository
toolchain, and Clippy MSRV; accepts the deliberate downstream floor increase;
and assigns calendar and hold judgements to inspection evidence.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-201 | high | Closed by the split: NFR-022 is independently reviewable and implementable under #417; ADR-0012 may consume the accepted compiler baseline without owning or blocking it. | NFR-022 Dependencies; #414; #417 | wrong-requirement |
| FND-202 | medium | Closed: the scope and AC-1 distinguish consumer MSRV, selected build compiler, and Clippy suggestion floor, and explicitly state the downstream break below 1.98.1. | NFR-022 Scope; NFR-022-AC-1; TC-1832 | missing-requirement |
| FND-203 | medium | Closed: newer-release timeliness and hold adjudication use governed inspection evidence, while executable integration evidence remains limited to the real build and qualification gates. | NFR-022-AC-2..AC-5; TC-1833..TC-1835 | wrong-requirement |

## Review disposition

The base review passes this specification subset. Implementation remains
pending and must apply Rust review, bare ix-trace-rs markers on changed Rust
acceptance tests, and the applicable real qualification gates before release.
