---
id: ADR-0008
title: "Inverse edges are authorable as derived views, not duplicate verbs"
type: ADR
---

# ADR 0008: Inverse edges are authorable as derived views, not duplicate verbs

**Status**: Accepted
**Date**: 2026-06-21
**Decision authority**: kreneskyp

## Context

[FR-040](../../functional/FR-040-object-edge-vocabulary.md) gives each `edge_types`
verb an optional `inverse:` label (`publishes` ↔ `consumed_by`, `supersedes` ↔
`superseded_by`, …), intended for reverse-edge query labelling. The field is
currently parsed and stored but read by no engine logic, and the Tier-1
type-allowed test is a plain key lookup over `resolve_allowed_links`.

A full-ecosystem smoketest of the shipped FR-040 vocabulary (234 repos, 5,656
spec docs) found the warn-tier invariant holds, but the corpus uses ~50 verbs the
registry never registered. The single largest mechanical bucket (~132 warnings)
is authors writing the **inverse direction** of an existing edge — `A consumed_by
B` instead of `B publishes A`. This is deliberate and useful: it lets a document
state that it is the *target* of another artifact's edge without editing that
other document (the canonical edge lives on the far side).

The question: how should an inverse verb become valid to author?

- **Option A — register each inverse as its own `edge_types` entry.** Manifest-only,
  no engine change. But the engine then treats `publishes` and `consumed_by` as two
  **unrelated** verbs: the pairing is lost, two definitions drift, and graph
  queries do not connect them unless the inverse is *also* hand-wired back —
  duplicating both sides.
- **Option B — make the declared `inverse:` label authorable as a derived view of
  its forward edge.** One canonical definition (category, description, target list)
  stays the source of truth; the inverse is recognized for authoring and
  normalized to the forward direction for enforcement. Requires an engine change,
  and activates the otherwise-dead `inverse:` field.

A semantic wrinkle constrains the design: `A consumed_by B` ≡ `B publishes A`, so
the per-archetype `allowed_links` constraint for the edge lives on **B** (the
forward source). The document-level Tier-1 path validating A cannot resolve B, so
it cannot enforce B's `allowed_links` there.

## Decision

Adopt **Option B**, specified in
[FR-041](../../functional/FR-041-authorable-inverse-edges.md): a declared
`inverse:` label is an authorable verb, a derived view of one canonical forward
edge — never a second registered `edge_types` entry
([FR-041-CON-1](../../functional/FR-041-authorable-inverse-edges.md)). Enforcement
splits across the existing FR-040 tiers to respect the wrinkle above:

- **Tier-1 (document-level)** *recognizes* inverse labels as vocabulary (clears the
  warning) but does not apply per-archetype `allowed_links` to them — that
  constraint belongs to the forward source, which Tier-1 cannot resolve.
- **Tier-2 (corpus-level)** *enforces* by normalizing the inverse edge to its
  forward form (swap source/target, verb → forward) before FR-040's
  `target_satisfies`, so the canonical-direction target rule still holds
  (bundle-local, as in FR-040). Diagnostics are reported with the authored
  (inverse) source/target/edge_type so they point at the document the author wrote.

This sits inside FR-040's warn-tier posture; nothing blocks extraction or
structural validation.

### Companion vocabulary policy (the rest of the sweep)

The same review settled how to treat the other sweep buckets (recorded with the
findings, applied via a `spec-artifacts-iso` vocabulary PR and per-repo doc
normalizations, not in this engine ADR):

- **Register** verbs that are semantically distinct **and** recurring (e.g. a
  `derives_from`/`derives_into` lineage pair, `migrated_from`, `generated_from`,
  `imports`, `delegates_to`).
- **Alias** generic or loosely-used verbs to the closest existing verb (e.g.
  `mirrors` → `represents`, vague 1–2-use laterals → `references`).
- **Normalize** spelling/format variants in the source (e.g. `derived_from` →
  `derives_from`, hyphenated `consumed-by` → `consumed_by`).

## Consequences

- The `inverse:` field becomes load-bearing; one canonical edge stays the single
  source of truth and the inverse is a derived view — no duplicate verbs to keep in
  sync.
- Bidirectional authoring is permitted, so the forward and inverse of one fact can
  both appear; corpus/graph consumers SHOULD treat an inverse edge as identical to
  its normalized forward (dedupe on traversal).
- Per-archetype `allowed_links` is enforced for inverse edges only at Tier-2
  (bundle-local), matching where the constraint is resolvable; Tier-1 is
  recognition-only for inverses.
- A small precedence rule is needed: an explicit forward registration outranks an
  inverse label of the same name, and colliding inverse declarations are first-wins
  with a `DuplicateInverseEdge` diagnostic.
