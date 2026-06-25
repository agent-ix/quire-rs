---
id: FR-041
title: "Authorable Inverse Edge Verbs"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-015"
    type: "implements"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-040"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-025"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-026"
    type: "requires"
    cardinality: "1:1"
---

# FR-041: Authorable Inverse Edge Verbs

## Description

[FR-040](./FR-040-object-edge-vocabulary.md) lets an `edge_types` verb declare an
optional `inverse:` label (e.g. `publishes` ↔ `consumed_by`, `supersedes` ↔
`superseded_by`), but the field is **parsed and stored only** — no engine logic
reads it. [FR-040](./FR-040-object-edge-vocabulary.md)'s Tier-1 type-allowed test
is a plain key lookup: an edge `type` is allowed iff it is a key in
`resolve_allowed_links`. Consequently a relationship authored in the **inverse
direction** — `A consumed_by B`, the reverse of `B publishes A` — is reported as a
`DisallowedEdgeType` warning even though the registry already models the pairing.

Authoring the inverse direction is a legitimate need: it lets a document state "I
am the target of X's edge" **without editing X's document** (the canonical edge
lives on the other artifact). An ecosystem sweep shows ~132 such warnings across
real specs (`consumed_by`, `superseded_by`, `emitted_by`, `rendered_by`,
`derives_into`, `satisfies`, …). The fix is **not** to register each inverse as a
second, independent verb (that duplicates the vocabulary and loses the pairing);
it is to make the **declared `inverse:` label itself authorable**, resolved as the
reverse of one canonical forward edge.

`quire-rs` SHALL treat a declared `inverse:` label as an authorable verb: a
**derived view** of its forward edge, not a separately registered `edge_types`
entry. The forward edge remains the single source of truth (category, description,
target list); the inverse is recognized for authoring and normalized to the
forward direction for target enforcement. This is the decision recorded in
[ADR-0008](../assets/adr/0008-authorable-inverse-edges.md). Posture is **advisory
(warn-tier)**, consistent with [FR-040](./FR-040-object-edge-vocabulary.md);
nothing here blocks extraction or [FR-032](./FR-032-validate-document.md)
structural validation.

## Behavior

- **Inverse index.** The `Registry` ([FR-013](./FR-013-archetype-loader.md)) SHALL
  expose an inverse index built from the merged `edge_types`: for each forward
  verb `F` declaring `inverse: I`, the index maps `I → F`. The index is derived
  from the same merged registry [FR-040](./FR-040-object-edge-vocabulary.md)
  builds; it introduces no new manifest section.

- **Known-verb status.** Because an inverse label is a valid verb, it is **known**
  wherever a registered `edge_types` key is: an inverse label used as a manifest
  `allowed_links` key does **not** raise the [FR-040](./FR-040-object-edge-vocabulary.md)
  `UnknownEdgeType` load diagnostic.

- **Precedence.** If a label `I` is **both** a forward `edge_types` key and a
  declared inverse of some other verb, the **explicit forward registration wins**
  (`I` is governed by its own entry, not treated as an inverse). If two distinct
  forward verbs declare the **same** inverse label, resolution is **first-wins**
  and emits a non-fatal `DuplicateInverseEdge` diagnostic, mirroring
  `DuplicateEdgeType` ([FR-040](./FR-040-object-edge-vocabulary.md)).

- **Tier-1 — recognition (document-level).** During
  [FR-032](./FR-032-validate-document.md), an edge whose `type` is **not** a key in
  `resolve_allowed_links(doc)` is nonetheless **type-allowed** when its `type` is a
  declared inverse label in the index. Per-archetype `allowed_links` enforcement is
  **not** applied to the inverse at Tier-1: the canonical edge's `allowed_links`
  constraint lives on the *other* document (the forward source), which the
  document-level path cannot resolve. A `type` that is neither a resolved key nor a
  known inverse still yields the [FR-040](./FR-040-object-edge-vocabulary.md)
  `DisallowedEdgeType` warning.

- **Tier-2 — enforcement (corpus-level).** During corpus validation
  ([FR-025](./FR-025-spec-corpus-model.md)), an edge `(source, I, target)` whose
  verb `I` is an inverse label SHALL be **normalized to its forward form**
  `(target, F, source)` before [FR-040](./FR-040-object-edge-vocabulary.md)'s
  `target_satisfies` check runs — i.e. source and target are swapped and `I` is
  replaced by `F`, so the existing target-typing rule applies in the canonical
  direction against the forward source's resolved `allowed_links`. A mismatch
  yields [FR-040](./FR-040-object-edge-vocabulary.md)'s `DisallowedEdgeTarget`
  warning, reported with the **authored** (inverse) `source`/`target`/`edge_type`
  so the diagnostic points at the document the author wrote. As in
  [FR-040](./FR-040-object-edge-vocabulary.md), target checking is bundle-local and
  skipped for `"*"` lists, targets with no `object:`, and dangling/cross-repo
  edges.

- **Warn-tier and determinism.** Inverse handling never errors and never blocks
  ([FR-040](./FR-040-object-edge-vocabulary.md) posture). The inverse index uses a
  `BTreeMap` and diagnostics keep the `(source, target, edge_type)` ordering
  ([NFR-006](../non-functional/NFR-006-determinism.md)), identical across runs and
  thread counts.

- **Scope — validation only.** This FR governs edge *validation* (Tier-1
  recognition, Tier-2 enforcement). The corpus edge set
  ([FR-026](./FR-026-intra-spec-reference-resolution.md)) still records edges
  exactly as authored — inverse normalization is applied *for the target check*,
  not written back into the harvested edges. Graph consumers treat an authored
  inverse edge as identical to its normalized forward
  ([ADR-0008](../assets/adr/0008-authorable-inverse-edges.md)); that dedup is a
  consumer concern, not changed here.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-041-CON-1 | An inverse label SHALL NOT be registered as an independent `edge_types` entry to make it authorable; authorability derives solely from a forward verb's `inverse:` field | Architecture | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-041-AC-1 | The merged `Registry` exposes an inverse index mapping each declared `inverse:` label to its forward verb; a registry with no declared inverses exposes an empty index | Test |
| FR-041-AC-2 | A frontmatter-harvested edge whose `type` is a declared inverse label is type-allowed during [FR-032](./FR-032-validate-document.md) (no `DisallowedEdgeType`), even when the label is absent from `resolve_allowed_links`; a `type` that is neither a resolved key nor a known inverse still yields exactly one `DisallowedEdgeType` warning | Test |
| FR-041-AC-3 | When a label is both a forward `edge_types` key and an inverse of another verb, the forward registration governs; when two forward verbs declare the same inverse label, resolution is first-wins and emits a non-fatal `DuplicateInverseEdge` diagnostic (default load succeeds) | Test |
| FR-041-AC-4 | During corpus validation, an inverse-verb edge `(source, I, target)` is normalized to `(target, F, source)` before `target_satisfies`; a target valid in the forward direction passes, and a forward-direction mismatch yields one `DisallowedEdgeTarget` reported with the authored inverse source/target/edge_type | Test |
| FR-041-AC-5 | Inverse recognition and normalization are warnings only — they never block extraction or [FR-032](./FR-032-validate-document.md) structural validation; the inverse index and diagnostics are deterministic across runs and thread counts ([NFR-006](../non-functional/NFR-006-determinism.md)) | Test |

## Dependencies

- **Upstream**: [FR-040](./FR-040-object-edge-vocabulary.md) (edge_types + the
  `inverse:` field, resolve_allowed_links, target_satisfies, the Tier-1/Tier-2
  hosts), [FR-032](./FR-032-validate-document.md) (Tier-1 host),
  [FR-025](./FR-025-spec-corpus-model.md) (Tier-2 host),
  [FR-026](./FR-026-intra-spec-reference-resolution.md) (edge harvest carries the
  verb), [FR-013](./FR-013-archetype-loader.md) (registry)
- **Downstream**: `spec-artifacts-iso` vocabulary PR (adds forward `edge_types` +
  `inverse:` declarations so the inverse verbs the corpus already uses become
  authorable); quire-cli surfacing of the advisory diagnostics
- **Decision**: [ADR-0008](../assets/adr/0008-authorable-inverse-edges.md)
