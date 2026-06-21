---
id: FR-040
title: "Object-Axis Typed Edge Vocabulary and Cross-Domain Targets"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-015"
    type: "implements"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-031"
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
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-029"
    type: "requires"
    cardinality: "1:1"
---

## Description

Typed edges in Filament are declared as `allowed_links`, but only on the
**artifact** axis ([FR-031](./FR-031-unified-archetype-shape.md) carries
`allowed_links` as a bare `Vec<String>` and nothing consumes it). The **object**
axis — the archetypes selected by a document's `object:` frontmatter — contributes
no edge vocabulary, so the part of the model that describes entities, schemas,
endpoints, and state machines cannot declare its own relationships. Three further
gaps compound this: the edge `type` is a free string nobody validates, edges have
no category or description, and the canonical module-manifest schema already
models object `allowed_links` as a **map** while `quire-rs` parses only the array
form and silently drops the map.

`quire-rs` SHALL close these gaps. It SHALL load two new **mergeable** manifest
registries — `edge_types` (the controlled verb vocabulary, each verb carrying a
category and description) and `roles` (capability tags object types opt into) —
accept `allowed_links` on **both** axes in array or map form, **resolve** the
union of artifact-axis and object-axis vocabulary for a document, and **validate**
harvested edges ([FR-026](./FR-026-intra-spec-reference-resolution.md)) against
it. Validation is **advisory** in this revision (warnings, never blocking
extraction or [FR-032](./FR-032-validate-document.md) structural validation); a
later CR flips it to errors once the corpus is migrated.

### API

```rust
/// A verb in the merged edge-type registry (manifest `edge_types`).
pub struct EdgeType {
    pub name: String,
    pub category: EdgeCategory,
    pub description: String,
    pub inverse: Option<String>,   // optional; need not itself be declared
}

pub enum EdgeCategory {
    Structural, Behavioral, Dataflow, Dependency, Realization, Governance, Traceability,
}

/// A capability tag object types opt into (manifest `roles`).
pub struct Role { pub name: String, pub description: String }

/// Per-archetype `allowed_links`, normalized to a map. The array authoring
/// form `[a, b]` normalizes to `{a: ["*"], b: ["*"]}`. A target token is a
/// concrete object-type name, a role name, or "*".
pub type AllowedLinks = BTreeMap<String, Vec<String>>;
```

The `Registry` ([FR-013](./FR-013-archetype-loader.md)) SHALL expose the merged
`edge_types` and `roles` registries and a resolver:

```rust
impl Registry {
    /// Union of the artifact archetype's allowed_links with the object
    /// archetype's (when `object` is Some); target lists for a shared verb
    /// are unioned, and "*" absorbs concrete/role tokens.
    pub fn resolve_allowed_links(
        &self,
        artifact: &CompiledArchetype,
        object: Option<&CompiledArchetype>,
    ) -> AllowedLinks;

    /// True when `candidate` (the resolved target archetype) satisfies a
    /// target token: token == candidate.name, OR token is a role the
    /// candidate carries, OR token == "*".
    pub fn target_satisfies(&self, token: &str, candidate: &CompiledArchetype) -> bool;
}
```

### Mergeable registries

`edge_types` and `roles` are top-level manifest sections merged across all active
modules into one registry, in the **same manner as archetypes**
([FR-014](./FR-014-module-activation.md)): cross-module name collisions are
**first-wins** and emit a non-fatal `Diagnostic` (`DuplicateEdgeType` /
`DuplicateRole`), mirroring `DuplicateArchetype`. Redeclaring a name with an
identical body is silently idempotent (no diagnostic). A verb used as an
`allowed_links` key, or a role used in an object type's `roles:` list or as a
target token, that is **absent** from the merged registry emits a non-fatal
`Diagnostic` (`UnknownEdgeType` / `UnknownRole`) — advisory, consistent with this
feature's warn-tier posture, **not** a fatal load error. The opt-in
`load_strict` path ([FR-014-AC-3](./FR-014-module-activation.md)) escalates all of
these diagnostics to errors. This keeps the vocabulary controlled (every
violation is surfaced) without the engine inventing a stricter default-load
policy than any other manifest section uses.

### allowed_links normalization

`allowed_links` SHALL deserialize from either shape:

- **array** — `[calls, publishes]` → `{calls: ["*"], publishes: ["*"]}` (the
  artifact-axis authoring form; verbs allowed against any target).
- **map** — `{contains: [value_object], exposes: [domain-object]}` (the
  object-axis form; each verb constrained to a target list of concrete type
  names, role names, or `"*"`).

This supersedes the array-only parse in
[FR-031](./FR-031-unified-archetype-shape.md); the carry-over field becomes an
`AllowedLinks` map (see **CR-001** below).

### Object roles on the compiled archetype

An object type's `roles: [..]` list SHALL be parsed onto its
`CompiledArchetype` (alongside `allowed_links` in the carry-over fields) and
exposed via a `roles()` accessor. `target_satisfies` reads this list. This is
**engine work owned by `quire-rs`** — the module-manifest JSON Schema for the
`roles:` field is `filament-core-service`'s, but parsing and carrying the value
is the loader's, not deferred downstream.

### CR-001 — FR-031 carry-over `allowed_links` type change

[FR-031-AC-3](./FR-031-unified-archetype-shape.md) specifies `allowed_links` as
retained and readable via an accessor returning `&[String]`. FR-040 changes the
carry-over representation to an `AllowedLinks` map (`BTreeMap<String,
Vec<String>>`) and the accessor's return type accordingly. This is a deliberate
contract change recorded here per the CR-note convention (FR-024 CR-002 pattern);
FR-031-AC-3's array assertion is superseded by FR-040-AC-5. No silent edit to
FR-031 is made from the implementation branch.

### Resolution and target matching

For a document with artifact archetype `T` and optional object archetype `O`:

- `verbs(doc) = resolve_allowed_links(T, O)` — the union described above.
- An edge `(type, target)` harvested by
  [FR-026](./FR-026-intra-spec-reference-resolution.md) is **type-allowed** iff
  `type ∈ verbs(doc).keys()`.
- It is **target-allowed** iff some token `t` in `verbs(doc)[type]` satisfies the
  resolved target archetype (`target_satisfies`): `t == "*"`, or `t` equals the
  target's object type, or `t` is a role the target carries. Target checking is
  skipped when the edge is dangling ([FR-026](./FR-026-intra-spec-reference-resolution.md)
  already reports that) or when the target list contains `"*"`.

### Validation tiers (advisory)

- **Tier 1 — edge type (document-level).** During
  [FR-032](./FR-032-validate-document.md), after the object archetype is composed,
  the document's frontmatter `relationships` are harvested **within the validate
  path** (the same `(target, type)` extraction
  [FR-026](./FR-026-intra-spec-reference-resolution.md) performs in the corpus —
  reused here, not re-derived). Each harvested edge whose `type` is not
  type-allowed yields a warning diagnostic `DisallowedEdgeType { source,
  edge_type, allowed }`. When `object:` names an **unknown** archetype (the
  existing [FR-032](./FR-032-validate-document.md) unknown-object warning path),
  the resolved vocabulary falls back to the **artifact axis alone** (no object
  archetype is available to contribute verbs); Tier-1 still runs against that
  reduced set.
- **Tier 2 — edge target (corpus-level).** During corpus validation
  ([FR-025](./FR-025-spec-corpus-model.md)), for each **resolved** edge the corpus
  looks up the target document, reads its `object:` frontmatter, resolves that to
  a `CompiledArchetype`, and evaluates `target_satisfies` against the verb's
  target list. A mismatch yields a warning `DisallowedEdgeTarget { source, target,
  edge_type, target_object_type, allowed_targets }`. Target checking is **skipped**
  when the verb's target list contains `"*"`, when the target document declares no
  `object:` (no object type to constrain), or when the edge is **dangling**
  ([FR-026](./FR-026-intra-spec-reference-resolution.md) already reports those).
  Because cross-repo `ix://` targets resolve as dangling, Tier-2 type-checks
  targets **only within a single loaded bundle** — cross-repo target typing is out
  of scope this revision.

Both tiers are **warnings** ([FR-032](./FR-032-validate-document.md) structural
errors are unaffected); neither blocks extraction, validation, or sync.
Determinism ([NFR-006](../non-functional/NFR-006-determinism.md)) holds:
diagnostics are sorted by `(source, target, edge_type)` — matching the corpus
edge ordering ([FR-026](./FR-026-intra-spec-reference-resolution.md)) — and are
identical across runs and thread counts. The carry-over map and registries use
`BTreeMap` in the validate paths.

### Skeleton presentation

The input contract ([FR-029](./FR-029-archetype-input-contract.md)) SHALL render
a **Relationships** block in the authoring skeleton, listing each verb in
`verbs(doc)` with its registry category, description, and resolved target list.
`input_contract_for` / `input_skeleton` SHALL accept an **optional object**
archetype name so the rendered block is the composed (artifact ∪ object)
vocabulary; with no object, it lists the artifact vocabulary alone.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-040-AC-1 | A manifest declaring `edge_types` (verb → category, description, optional inverse) and `roles` (name → description) loads; the merged `Registry` exposes both, and re-declaring a name with an identical body across two modules is silently idempotent (no diagnostic). | Test |
| FR-040-AC-2 | Re-declaring an `edge_types`/`roles` name with a differing body across modules is **first-wins** and emits a non-fatal `DuplicateEdgeType`/`DuplicateRole` diagnostic (mirroring `DuplicateArchetype`, [FR-014](./FR-014-module-activation.md)); the default load still succeeds. | Test |
| FR-040-AC-3 | An `allowed_links` key absent from `edge_types`, or a `roles:` entry / target token absent from `roles`, emits a non-fatal `UnknownEdgeType`/`UnknownRole` diagnostic (advisory, default load succeeds); the opt-in `load_strict` path ([FR-014-AC-3](./FR-014-module-activation.md)) escalates AC-2 and AC-3 diagnostics to errors. | Test |
| FR-040-AC-4 | `allowed_links` in array form `[calls, publishes]` normalizes to `{calls: ["*"], publishes: ["*"]}`, and map form `{contains: [value_object]}` round-trips through `Registry::load_module` as an `AllowedLinks` map (superseding the FR-031 array-only parse per CR-001). | Test |
| FR-040-AC-5 | An object type's `roles: [..]` list is parsed onto its `CompiledArchetype` and readable via a `roles()` accessor; an archetype declaring no roles reads as empty. | Test |
| FR-040-AC-6 | `resolve_allowed_links(T, Some(O))` returns the union of both axes' verbs; for a verb present on both, the target lists are unioned and `"*"` absorbs concrete/role tokens. With `object = None` it returns the artifact vocabulary alone. | Test |
| FR-040-AC-7 | `target_satisfies` is true when the token equals the target archetype's name, when the token is a role the target carries, or when the token is `"*"`; false otherwise. | Test |
| FR-040-AC-8 | During [FR-032](./FR-032-validate-document.md), a document whose frontmatter-harvested edge `type` is not in `resolve_allowed_links` yields exactly one warning `DisallowedEdgeType` naming source and verb; in-vocabulary-only yields none; when `object:` is unknown, the vocabulary falls back to the artifact axis alone and Tier-1 still runs. | Test |
| FR-040-AC-9 | During corpus validation, a resolved edge whose target document's `object:` archetype (and its roles) fails the verb's target list yields a warning `DisallowedEdgeTarget`; the same verb to a target carrying the required role passes, including cross-module source/target. Target checking is skipped for `"*"` lists, targets with no `object:`, and dangling (cross-repo) targets. | Test |
| FR-040-AC-10 | Tier-1 and Tier-2 findings are warnings only — they do not block extraction or [FR-032](./FR-032-validate-document.md) structural validation, and a corpus with disallowed edges still loads; diagnostics are sorted by `(source, target, edge_type)` and identical across runs and thread counts ([NFR-006](../non-functional/NFR-006-determinism.md)). | Test |
| FR-040-AC-11 | `input_skeleton` for an archetype with an optional `object` argument renders a Relationships block listing each resolved verb with its category, description, and target list; without `object`, only the artifact vocabulary is listed. | Test |

## Dependencies

- **Upstream**: [FR-031](./FR-031-unified-archetype-shape.md) (carry-over
  `allowed_links`, superseded to a map), [FR-032](./FR-032-validate-document.md)
  (compose type + object; Tier-1 host), [FR-025](./FR-025-spec-corpus-model.md)
  (corpus; Tier-2 host), [FR-026](./FR-026-intra-spec-reference-resolution.md)
  (edge harvest carries the type), [FR-029](./FR-029-archetype-input-contract.md)
  (input contract/skeleton), [FR-013](./FR-013-archetype-loader.md) /
  [FR-014](./FR-014-module-activation.md) (loader + activation merge)
- **Downstream**: `filament-core-service` module-manifest schema (`edge_types`,
  `roles`, `ObjectTypeEntry.roles`); `spec-artifacts-iso` + `spec-objects-*`
  vocabulary population; quire-cli surfacing of the advisory diagnostics
