---
id: ADR-0012
title: "YAML engine maintenance and parity"
type: ADR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/non-functional/NFR-009"
    type: "relates_to"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-006"
    type: "relates_to"
---

# ADR 0012: YAML engine maintenance and parity

**Status**: Accepted and implemented
**Date**: 2026-09-07
**Decision authority**: kreneskyp (accepted 2026-09-08)

## Context

`quire-rs` used the deprecated `serde_yaml` release, whose upstream repository is
archived. YAML parsing is behavior-bearing: frontmatter contains document
identity and relationships, while module manifests, clause sets, extraction
DSL data, traceability models, and lint rules also use the same Serde surface.
A replacement therefore needs evidence of semantic parity, not merely an
API-compatible crate name.

## Decision

Keep the established Rust import name (`serde_yaml`) and select the maintained
YAML Organization package `yaml_serde` behind it through a Cargo package
rename. The version is the one `Cargo.toml` declares; this record does not
carry a second copy.

Both the old and new backends descend from the C2Rust-transpiled libyaml
implementation, so this decision is about active maintenance ownership; it does
not claim a reduction in transitive unsafe code.

The dependency change does not redesign parsing. Existing
`serde_yaml::from_str`, `from_slice`, `from_value`, `Value`, `Mapping`, and
`to_string` call sites retain their behavior and error boundaries.

Any future YAML package change reopens this decision and requires a fresh
differential and the affected parity suites.

## Alternatives considered

| Option | Disposition |
| --- | --- |
| Keep the deprecated `serde_yaml` | Rejected because it is explicitly deprecated and its upstream is archived. |
| `serde_yml` | Rejected because the relevant line is archived and affected by RUSTSEC-2025-0068. |
| `serde_yaml_ng` or `serde_norway` | Viable fallbacks, but not selected over the actively maintained YAML Organization package. |
| Pure-Rust parser or custom adapter | Deferred; it would be a parser redesign rather than this maintenance port. |
| An early `yaml_serde` release | Rejected because its selection was based only on inherited obsolete Rust metadata. |
| Current maintained `yaml_serde` release | Selected. |

## Compatibility evidence

A temporary two-engine comparator evaluated every governed leading frontmatter
block and the focused semantic cases. All old/new outcome and complete
JSON-value comparisons were equal, and an injected duplicate-key difference
produced a non-zero exit, demonstrating that the comparison could fail.

The focused cases covered duplicate keys, merge keys, aliases, implicit
scalars, timestamps, and non-string mapping keys. Existing frontmatter parity
and typed YAML consumer suites also passed unchanged. Same-runner performance
measurements remained within the existing limit.

The temporary comparator and voluminous per-input output are not
part of the product or its permanent tooling. A future version decision must
produce fresh evidence for that candidate rather than rely on this one-time
executable.

## Consequences

- The public and internal YAML API shape is unchanged.
- The root and fuzz dependency graphs use an actively maintained package line.
- The deprecated YAML package is not a dependency of either graph.
- Parser behavior changes remain out of scope; any observed difference blocks
  a future dependency move and requires a separately reviewed decision.

## Revisit triggers

Reopen this decision if the selected `yaml_serde` package or its libyaml
backend receives an advisory, the release line becomes inactive, a parser redesign is proposed, or
TypeScript/Python reference semantics intentionally change.
