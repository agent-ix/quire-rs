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

`quire-rs` used `serde_yaml 0.9.34+deprecated`, whose upstream repository is
archived. YAML parsing is behavior-bearing: frontmatter contains document
identity and relationships, while module manifests, clause sets, extraction
DSL data, traceability models, and lint rules also use the same Serde surface.
A replacement therefore needs evidence of semantic parity, not merely an
API-compatible crate name.

## Decision

Keep the established Rust import name and select the maintained YAML
Organization package exactly:

```toml
serde_yaml = { package = "yaml_serde", version = "=0.10.7" }
```

The selected package resolves to `libyaml-rs 0.3.0`. Both old and new backends
descend from the C2Rust-transpiled libyaml implementation, so this decision is
about active maintenance ownership; it does not claim a reduction in
transitive unsafe code.

The dependency change does not redesign parsing. Existing
`serde_yaml::from_str`, `from_slice`, `from_value`, `Value`, `Mapping`, and
`to_string` call sites retain their behavior and error boundaries.

`scripts/audits/check_dep_pins.sh` enforces the exact alias and locked package
pair and rejects reintroduction of `serde_yaml` or `unsafe-libyaml`. Any future
YAML package or version change reopens this decision and requires a fresh
differential and the affected parity suites.

## Alternatives considered

| Option | Disposition |
| --- | --- |
| Keep `serde_yaml 0.9.34` | Rejected because it is explicitly deprecated and its upstream is archived. |
| `serde_yml` | Rejected because the relevant line is archived and affected by RUSTSEC-2025-0068. |
| `serde_yaml_ng` or `serde_norway` | Viable fallbacks, but not selected over the actively maintained YAML Organization package. |
| Pure-Rust parser or custom adapter | Deferred; it would be a parser redesign rather than this maintenance port. |
| `yaml_serde 0.10.2` | Rejected because its selection was based only on inherited obsolete Rust metadata. |
| `yaml_serde 0.10.7` | Selected; Rust 1.98.1 satisfies its Rust 1.82 minimum. |

## Compatibility evidence

At implementation revision `397ed175bf5518a5d3f16befea7636b447cc4d97`, a
temporary two-engine comparator evaluated 775 complete leading frontmatter
blocks and six focused semantic cases. All 781 old/new outcome and complete
JSON-value comparisons were equal. An injected duplicate-key difference
produced exactly one difference and a non-zero exit, demonstrating that the
comparison could fail.

The focused cases covered duplicate keys, merge keys, aliases, implicit
scalars, timestamps, and non-string mapping keys. Existing frontmatter parity
and typed YAML consumer suites also passed unchanged. Same-runner performance
measurements remained within the existing 10% limit.

The aggregate result, input revisions, population, negative-control result, and
gate summary are retained in `spec/evidence/yaml-migration/manifest-v1.json`
and SR-108. The temporary comparator and voluminous per-input output are not
part of the product or its permanent tooling. A future version decision must
produce fresh evidence for that candidate rather than rely on this one-time
executable.

## Consequences

- The public and internal YAML API shape is unchanged.
- The root and fuzz dependency graphs use an actively maintained package line.
- Deprecated YAML packages are rejected by the existing dependency audit.
- Parser behavior changes remain out of scope; any observed difference blocks
  a future dependency move and requires a separately reviewed decision.

## Revisit triggers

Reopen this decision if `yaml_serde 0.10.7` or `libyaml-rs 0.3.0` receives an
advisory, the release line becomes inactive, a parser redesign is proposed, or
TypeScript/Python reference semantics intentionally change.
