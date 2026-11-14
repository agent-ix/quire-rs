---
id: SR-20261113203420
title: Base checklist - PLAT-1689
type: SpecReview
analysis: base
scope: agent-ix/quire-rs; spec/functional/FR-040-object-edge-vocabulary.md, spec/functional/FR-041-authorable-inverse-edges.md,
  spec/functional/FR-057-corpus-check-severity.md, spec/usecase/US-015-object-edge-vocabulary.md
review_set: subset
---

## Summary

Ticket: PLAT-1689. Current head 252e941aea63d0661180a238889d7a7bb497bd97. Methods: code-review with rust-review folded into it; gap-analysis; spec-review/base; spec-review/integrity; spec-review/failure-domain (edge identity/provenance and inverse normalization). EARS skipped: no new or edited SHALL requirement statement, only ACs and explanatory prose. Dependency skipped: no authored dependency-edge changes. Object, evidence, scope-boundary, risk-complexity, security, and criterion-strength analyses were not separately applicable: no domain object, verification catalog, system boundary, security obligation, or complex new algorithm was introduced. Optional gap semantic review and plan completion were not requested. Required gate execution is excluded by the user.

## Verdict

PASS for this method’s selected scope; this is not an overall approval or enforced review acceptance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Analysis

Checked ID uniqueness and sequential AC numbering, changed requirement/user-value context, input/output/error contracts, test verification cells, owning links, and the absence of added hash/pin/duplicate tracking machinery. All 11 selected FR criteria are tagged; assertion and trace defects are recorded in gap-analysis, and the stale inverse severity contract is owned by integrity rather than duplicated here.

## Assurance Context

AP-201 (`spec/assurance/AP-201-detection-minting.md`) applies: this diff changes which edges the engine examines and the diagnostics it reports. Its material impact scenario concerns plausible evidence with missing examined populations. Source, test assertions, original review reports and the current computed matrix were examined. Historical mutation/probe runs are producer claims retained from the earlier reviewer, not runs by this reviewer. No current aggregate-gate output, accepted enforced review run, independently collected controlled-corpus result, measurement comparison, or producer-reliance decision was supplied. No exception was granted. The user explicitly prohibits builds/gates and repository writes; AP-201's required accepted spec-review run and aggregate gate remain with the lead. This direct analysis does not claim enforced acceptance.

## Examined units

Initial comment scope (historical for inherited reviews):

```yaml
scope:
- id: FR-040-AC-1
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: context_only
  excerpt: A manifest declaring `edge_types` (verb → category, description, optional
    inverse and composition marker) and `roles` (name → description) loads; omitted
    composition markers default to false; the merged `Registry` exposes both, and
    re-declaring a name with an identical body across two modules is silently idempotent
    (no diagnostic).
- id: FR-040-AC-2
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: context_only
  excerpt: Re-declaring an `edge_types`/`roles` name with a differing body across
    modules is **first-wins** and emits a non-fatal `DuplicateEdgeType`/`DuplicateRole`
    diagnostic (mirroring `DuplicateArchetype`, [FR-014](./FR-014-module-activation.md));
    the default load still succeeds.
- id: FR-040-AC-3
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: context_only
  excerpt: An `allowed_links` key absent from `edge_types`, or a `roles:` entry /
    target token absent from `roles`, emits a non-fatal `UnknownEdgeType`/`UnknownRole`
    diagnostic (advisory, default load succeeds); the opt-in `load_strict` path ([FR-014-AC-3](./FR-014-module-activation.md))
    escalates AC-2 and AC-3 diagnostics to errors.
- id: FR-040-AC-4
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: context_only
  excerpt: '`allowed_links` in array form `[calls, publishes]` normalizes to `{calls:
    ["*"], publishes: ["*"]}`, and map form `{contains: [value_object]}` round-trips
    through `Registry::load_module` as an `AllowedLinks` map (superseding the FR-031
    array-only parse per CR-001).'
- id: FR-040-AC-5
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: context_only
  excerpt: 'An object type''s `roles: [..]` list is parsed onto its `CompiledArchetype`
    and readable via a `roles()` accessor; an archetype declaring no roles reads as
    empty.'
- id: FR-040-AC-6
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: context_only
  excerpt: '`resolve_allowed_links(T, Some(O))` returns the union of both axes'' verbs;
    for a verb present on both, the target lists are unioned and `"*"` absorbs concrete/role
    tokens. With `object = None` it returns the artifact vocabulary alone.'
- id: FR-040-AC-7
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: '`target_satisfies` is true when the token equals the target archetype''s
    name, when the token is a role the target carries, or when the token is `"*"`;
    false otherwise.'
- id: FR-040-AC-8
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: During [FR-032](./FR-032-validate-document.md), a document whose frontmatter-harvested
    edge `type` is not in `resolve_allowed_links` yields exactly one warning `DisallowedEdgeType`
    naming source and verb; in-vocabulary-only yields none; when `object:` is unknown,
    the vocabulary falls back to the artifact axis alone and Tier-1 still runs.
- id: FR-040-AC-9
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: During corpus validation, an authored frontmatter relationship to a resolved
    target passes when its registered document archetype, registered `object:` type,
    or object role satisfies the verb's target list, including across modules; a declared
    but unresolved `object:` contributes no type or roles, while the registered document
    archetype is still checked. Resolver-synthesized reference edges are not target-kind
    checked.
- id: FR-040-AC-10
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: Tier-1 findings, and Tier-2 findings under `Okf`, are warnings only — they
    do not block extraction or [FR-032](./FR-032-validate-document.md) structural
    validation, and a corpus with disallowed edges still loads; diagnostics are sorted
    by `(source, target, edge_type)` and identical across runs and thread counts ([NFR-006](../non-functional/NFR-006-determinism.md)).
- id: FR-040-AC-11
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: '`input_skeleton` for an archetype with an optional `object` argument renders
    a Relationships block listing each resolved verb with its category, description,
    and target list; without `object`, only the artifact vocabulary is listed.'
- id: FR-040-AC-12
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: 'A target list is matched against the target document''s archetype name
    as well as its object type and roles: an `adr` source whose `realizes` list is
    `[FR, NFR]` passes to `FR`/`NFR` documents and is flagged to an `adr` document,
    none of them declaring `object:`.'
- id: FR-040-AC-13
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: Under `Strict` a Tier-2 `disallowed-edge-target` is an error (bundle invalid);
    under `Okf` it is a warning. The finding's path is the source document's file
    path and its message names the source and target file paths, not only their ids.
- id: FR-040-AC-14
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: '`edges:disallowed-edge-target` set to `error`, `warning` or `off` in `grammar_severity`
    overrides the posture-routed tier in both postures.'
- id: FR-041-AC-1
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: context_only
  excerpt: The merged `Registry` exposes an inverse index mapping each declared `inverse:`
    label to its forward verb; a registry with no declared inverses exposes an empty
    index
- id: FR-041-AC-2
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: context_only
  excerpt: A frontmatter-harvested edge whose `type` is a declared inverse label is
    type-allowed during [FR-032](./FR-032-validate-document.md) (no `DisallowedEdgeType`),
    even when the label is absent from `resolve_allowed_links`; a `type` that is neither
    a resolved key nor a known inverse still yields exactly one `DisallowedEdgeType`
    warning
- id: FR-041-AC-3
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: context_only
  excerpt: When a label is both a forward `edge_types` key and an inverse of another
    verb, the forward registration governs; when two forward verbs declare the same
    inverse label, resolution is first-wins and emits a non-fatal `DuplicateInverseEdge`
    diagnostic (default load succeeds)
- id: FR-041-AC-4
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: examined
  excerpt: During corpus validation, an inverse-verb edge `(source, I, target)` is
    normalized to `(target, F, source)` before `target_satisfies`; a target valid
    in the forward direction passes, and a forward-direction mismatch yields one `DisallowedEdgeTarget`
    reported with the authored inverse source/target/edge_type
- id: FR-041-AC-5
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: examined
  excerpt: Inverse recognition and normalization are warnings only — they never block
    extraction or [FR-032](./FR-032-validate-document.md) structural validation; the
    inverse index and diagnostics are deterministic across runs and thread counts
    ([NFR-006](../non-functional/NFR-006-determinism.md))
- id: FR-057-AC-1
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: Validating a bundle on disk under `Okf` with a registry mapping `refs:dangling-reference`
    to `error` puts the dangling-reference finding in `errors` and makes `is_valid()`
    false, where the same bundle with no mapping reports it as a warning.
- id: FR-057-AC-2
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: The same key mapped `warning` under `Strict` demotes a finding that is
    a hard error by default, and `is_valid()` becomes true — the promotion lever works
    in both directions.
- id: FR-057-AC-3
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: A check mapped `off` records no finding in `errors` and none in `warnings`,
    in either posture.
- id: FR-057-AC-4
  path: spec/functional/FR-057-corpus-check-severity.md
  role: examined
  excerpt: 'With no entry for its key, every check in the pack table keeps the exact
    tier it had before this FR — for the checks that existed before this FR: posture-routed
    for the four posture-routed checks, warning-in-both-postures for the fixed ones
    other than `edges:disallowed-edge-target`, which FR-040-AC-13 later made posture-routed
    — verified per check rather than in aggregate; checks added after this FR are
    covered by their own criteria (FR-057-AC-11).'
- id: FR-057-AC-5
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: Overrides layered through `Registry::with_grammar_severity` (the path `quire
    validate --severity` takes, FR-048-AC-5) apply to corpus checks, and a CLI-shaped
    entry wins over a conflicting module-declared entry for the same key.
- id: FR-057-AC-6
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: 'Mapping one check `off` leaves its siblings reporting: with `refs:dangling-reference=off`,
    a bundle that also has a dangling **trace** reference still reports `trace:dangling-trace-reference`.'
- id: FR-057-AC-7
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: Every `BundleFinding` carries the severity that was applied, so a surface
    renders the configured level rather than inferring it from which vector the finding
    landed in.
- id: FR-057-AC-8
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: The `reason` token of every check listed in the pack table is byte-identical
    to its pre-FR value, so a consumer matching on the machine surface is unaffected.
- id: FR-057-AC-9
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: Every corpus finding routed through the pack surface carries a well-formed
    `<pack>:<check>` key accepted by `is_severity_key`, so a pack cannot ship unregistrable
    — asserted over the packs the engine emits, not a hardcoded list.
- id: FR-057-AC-10
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: Findings appear in the same order with and without a severity map, and
    repeated runs over one bundle are identical (NFR-006).
- id: FR-057-AC-11
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: '`trace:reference-kind-mismatch` follows the same registry rules as `trace:dangling-trace-reference`:
    unconfigured it is an error under `Strict` and a warning under `Okf`; `error`,
    `warning` and `off` override it in both postures without affecting the sibling
    `trace:dangling-trace-reference` count.'
- id: US-015-AC-3
  path: spec/usecase/US-015-object-edge-vocabulary.md
  role: examined
  excerpt: "Declaring a frontmatter relationship to a resolved target\n  whose registered\
    \ document archetype, object type, or object role does not\n  satisfy the verb's\
    \ target list surfaces a `DisallowedEdgeTarget` finding; an\n  edge whose target\
    \ carries the required **role** passes even when source and\n  target live in\
    \ different modules. A declared but unresolved `object:` adds\n  no candidate\
    \ type or roles, while the registered document archetype remains\n  checked. Resolver-synthesized\
    \ reference links are not target-kind checked\n  (exercises [FR-040](../functional/FR-040-object-edge-vocabulary.md))."
- id: FR-040::CR-002
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: '

    ### CR-002 — PLAT-1689 target-kind enforcement


    The original Tier-2 rule checked only a target''s `object:` archetype and roles,

    and treated every mismatch as a warning. FR-040-AC-9 asserted that rule. The

    target-kind rule now also checks the target document''s registered `type:`

    archetype and routes an unconfigured mismatch by bundle posture: error under

    `Strict`, warning under `Okf`. Tier-2 applies only to an edge declared in the'
- id: FR-040::resolution
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: "- It is **target-allowed** iff some token `t` in `verbs(doc)[type]` satisfies\
    \ the\n  resolved target archetype (`target_satisfies`): `t == \"*\"`, or `t`\
    \ equals the\n  target's object type, or `t` is a role the target carries, or\
    \ `t` equals the\n  target document's own archetype name (its `type:` as the manifest\
    \ declares it).\n  Target checking is skipped when the edge is dangling ([FR-026](./FR-026-intra-spec-reference-resolution.md)\n\
    \  already reports that) or when the target list contains `\"*\"`."
- id: FR-040::Tier-2
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: "  **kind**: its registered document archetype, plus — when it declares\
    \ a\n  registered `object:` — that object type and its roles. If `object:` is\n\
    \  declared but unresolved, only a registered document archetype is considered;\n\
    \  the unknown object contributes no kind or roles. Resolver-synthesized\n  `references`\
    \ edges from frontmatter URIs, body `ix://` links or relative\n  Markdown links\
    \ are not checked by Tier-2. The list is satisfied when any\n  token matches any\
    \ candidate kind (`target_satisfies`); no archetype or object"
- id: FR-041::Tier-2
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: examined
  excerpt: "  direction against the forward source's resolved `allowed_links`. A mismatch\n\
    \  yields [FR-040](./FR-040-object-edge-vocabulary.md)'s `DisallowedEdgeTarget`\n\
    \  routed by bundle posture and reported with the **authored** (inverse)\n  `source`/`target`/`edge_type`\
    \ so the diagnostic points at the document the\n  author wrote. Its target-kind\
    \ description names the authored target document's\n  registered `type:` and,\
    \ when resolvable, its `object:` type. An unresolved\n  `object:` contributes\
    \ no type or roles; the registered document archetype"
- id: FR-057::edges-tier
  path: spec/functional/FR-057-corpus-check-severity.md
  role: examined
  excerpt: '| `edges:disallowed-edge-target` | posture-routed (FR-040-AC-13) |'
- id: FR-041-AC-5
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: examined
  excerpt: Inverse recognition and normalization are warnings only — they never block
    extraction or [FR-032](./FR-032-validate-document.md) structural validation; the
    inverse index and diagnostics are deterministic across runs and thread counts
    ([NFR-006](../non-functional/NFR-006-determinism.md))
- id: FR-040-AC-9
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: During corpus validation, an authored frontmatter relationship to a resolved
    target passes when its registered document archetype, registered `object:` type,
    or object role satisfies the verb's target list, including across modules; a declared
    but unresolved `object:` contributes no type or roles, while the registered document
    archetype is still checked. Resolver-synthesized reference edges are not target-kind
    checked.
- id: FR-040-AC-9
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: During corpus validation, an authored frontmatter relationship to a resolved
    target passes when its registered document archetype, registered `object:` type,
    or object role satisfies the verb's target list, including across modules; a declared
    but unresolved `object:` contributes no type or roles, while the registered document
    archetype is still checked. Resolver-synthesized reference edges are not target-kind
    checked.
```

Current-head disposition scope:

```yaml
scope:
- id: FR-040-AC-1
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: context_only
  excerpt: A manifest declaring `edge_types` (verb → category, description, optional
    inverse and composition marker) and `roles` (name → description) loads; omitted
    composition markers default to false; the merged `Registry` exposes both, and
    re-declaring a name with an identical body across two modules is silently idempotent
    (no diagnostic).
- id: FR-040-AC-2
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: context_only
  excerpt: Re-declaring an `edge_types`/`roles` name with a differing body across
    modules is **first-wins** and emits a non-fatal `DuplicateEdgeType`/`DuplicateRole`
    diagnostic (mirroring `DuplicateArchetype`, [FR-014](./FR-014-module-activation.md));
    the default load still succeeds.
- id: FR-040-AC-3
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: context_only
  excerpt: An `allowed_links` key absent from `edge_types`, or a `roles:` entry /
    target token absent from `roles`, emits a non-fatal `UnknownEdgeType`/`UnknownRole`
    diagnostic (advisory, default load succeeds); the opt-in `load_strict` path ([FR-014-AC-3](./FR-014-module-activation.md))
    escalates AC-2 and AC-3 diagnostics to errors.
- id: FR-040-AC-4
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: context_only
  excerpt: '`allowed_links` in array form `[calls, publishes]` normalizes to `{calls:
    ["*"], publishes: ["*"]}`, and map form `{contains: [value_object]}` round-trips
    through `Registry::load_module` as an `AllowedLinks` map (superseding the FR-031
    array-only parse per CR-001).'
- id: FR-040-AC-5
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: context_only
  excerpt: 'An object type''s `roles: [..]` list is parsed onto its `CompiledArchetype`
    and readable via a `roles()` accessor; an archetype declaring no roles reads as
    empty.'
- id: FR-040-AC-6
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: context_only
  excerpt: '`resolve_allowed_links(T, Some(O))` returns the union of both axes'' verbs;
    for a verb present on both, the target lists are unioned and `"*"` absorbs concrete/role
    tokens. With `object = None` it returns the artifact vocabulary alone.'
- id: FR-040-AC-7
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: '`target_satisfies` is true when the token equals the target archetype''s
    name, when the token is a role the target carries, or when the token is `"*"`;
    false otherwise.'
- id: FR-040-AC-8
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: During [FR-032](./FR-032-validate-document.md), a document whose frontmatter-harvested
    edge `type` is not in `resolve_allowed_links` yields exactly one warning `DisallowedEdgeType`
    naming source and verb; in-vocabulary-only yields none; when `object:` is unknown,
    the vocabulary falls back to the artifact axis alone and Tier-1 still runs.
- id: FR-040-AC-9
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: During corpus validation, an authored frontmatter relationship to a resolved
    target passes when its registered document archetype, registered `object:` type,
    or object role satisfies the verb's target list, including across modules; a declared
    but unresolved `object:` contributes no type or roles, while the registered document
    archetype is still checked. Resolver-synthesized reference edges are not target-kind
    checked.
- id: FR-040-AC-10
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: Tier-1 findings, and Tier-2 findings under `Okf`, are warnings only — they
    do not block extraction or [FR-032](./FR-032-validate-document.md) structural
    validation, and a corpus with disallowed edges still loads; diagnostics are sorted
    by `(source, target, edge_type)` and identical across runs and thread counts ([NFR-006](../non-functional/NFR-006-determinism.md)).
- id: FR-040-AC-11
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: '`input_skeleton` for an archetype with an optional `object` argument renders
    a Relationships block listing each resolved verb with its category, description,
    and target list; without `object`, only the artifact vocabulary is listed.'
- id: FR-040-AC-12
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: 'A target list is matched against the target document''s archetype name
    as well as its object type and roles: an `adr` source whose `realizes` list is
    `[FR, NFR]` passes to `FR`/`NFR` documents and is flagged to an `adr` document,
    none of them declaring `object:`.'
- id: FR-040-AC-13
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: Under `Strict` a Tier-2 `disallowed-edge-target` is an error (bundle invalid);
    under `Okf` it is a warning. The finding's path is the source document's file
    path and its message names the source and target file paths, not only their ids.
- id: FR-040-AC-14
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: '`edges:disallowed-edge-target` set to `error`, `warning` or `off` in `grammar_severity`
    overrides the posture-routed tier in both postures.'
- id: FR-041-AC-1
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: context_only
  excerpt: The merged `Registry` exposes an inverse index mapping each declared `inverse:`
    label to its forward verb; a registry with no declared inverses exposes an empty
    index
- id: FR-041-AC-2
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: context_only
  excerpt: A frontmatter-harvested edge whose `type` is a declared inverse label is
    type-allowed during [FR-032](./FR-032-validate-document.md) (no `DisallowedEdgeType`),
    even when the label is absent from `resolve_allowed_links`; a `type` that is neither
    a resolved key nor a known inverse still yields exactly one `DisallowedEdgeType`
    warning
- id: FR-041-AC-3
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: context_only
  excerpt: When a label is both a forward `edge_types` key and an inverse of another
    verb, the forward registration governs; when two forward verbs declare the same
    inverse label, resolution is first-wins and emits a non-fatal `DuplicateInverseEdge`
    diagnostic (default load succeeds)
- id: FR-041-AC-4
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: examined
  excerpt: During corpus validation, an inverse-verb edge `(source, I, target)` is
    normalized to `(target, F, source)` before `target_satisfies`; a target valid
    in the forward direction passes, and a forward-direction mismatch yields one `DisallowedEdgeTarget`
    reported with the authored inverse source/target/edge_type
- id: FR-041-AC-5
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: examined
  excerpt: Inverse recognition and normalization are warnings only — they never block
    extraction or [FR-032](./FR-032-validate-document.md) structural validation; the
    inverse index and diagnostics are deterministic across runs and thread counts
    ([NFR-006](../non-functional/NFR-006-determinism.md))
- id: FR-057-AC-1
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: Validating a bundle on disk under `Okf` with a registry mapping `refs:dangling-reference`
    to `error` puts the dangling-reference finding in `errors` and makes `is_valid()`
    false, where the same bundle with no mapping reports it as a warning.
- id: FR-057-AC-2
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: The same key mapped `warning` under `Strict` demotes a finding that is
    a hard error by default, and `is_valid()` becomes true — the promotion lever works
    in both directions.
- id: FR-057-AC-3
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: A check mapped `off` records no finding in `errors` and none in `warnings`,
    in either posture.
- id: FR-057-AC-4
  path: spec/functional/FR-057-corpus-check-severity.md
  role: examined
  excerpt: 'With no entry for its key, every check in the pack table keeps the exact
    tier it had before this FR — for the checks that existed before this FR: posture-routed
    for the four posture-routed checks, warning-in-both-postures for the fixed ones
    other than `edges:disallowed-edge-target`, which FR-040-AC-13 later made posture-routed
    — verified per check rather than in aggregate; checks added after this FR are
    covered by their own criteria (FR-057-AC-11).'
- id: FR-057-AC-5
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: Overrides layered through `Registry::with_grammar_severity` (the path `quire
    validate --severity` takes, FR-048-AC-5) apply to corpus checks, and a CLI-shaped
    entry wins over a conflicting module-declared entry for the same key.
- id: FR-057-AC-6
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: 'Mapping one check `off` leaves its siblings reporting: with `refs:dangling-reference=off`,
    a bundle that also has a dangling **trace** reference still reports `trace:dangling-trace-reference`.'
- id: FR-057-AC-7
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: Every `BundleFinding` carries the severity that was applied, so a surface
    renders the configured level rather than inferring it from which vector the finding
    landed in.
- id: FR-057-AC-8
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: The `reason` token of every check listed in the pack table is byte-identical
    to its pre-FR value, so a consumer matching on the machine surface is unaffected.
- id: FR-057-AC-9
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: Every corpus finding routed through the pack surface carries a well-formed
    `<pack>:<check>` key accepted by `is_severity_key`, so a pack cannot ship unregistrable
    — asserted over the packs the engine emits, not a hardcoded list.
- id: FR-057-AC-10
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: Findings appear in the same order with and without a severity map, and
    repeated runs over one bundle are identical (NFR-006).
- id: FR-057-AC-11
  path: spec/functional/FR-057-corpus-check-severity.md
  role: context_only
  excerpt: '`trace:reference-kind-mismatch` follows the same registry rules as `trace:dangling-trace-reference`:
    unconfigured it is an error under `Strict` and a warning under `Okf`; `error`,
    `warning` and `off` override it in both postures without affecting the sibling
    `trace:dangling-trace-reference` count.'
- id: US-015-AC-3
  path: spec/usecase/US-015-object-edge-vocabulary.md
  role: examined
  excerpt: "Declaring a frontmatter relationship to a resolved target\n  whose registered\
    \ document archetype, object type, or object role does not\n  satisfy the verb's\
    \ target list surfaces a `DisallowedEdgeTarget` finding; an\n  edge whose target\
    \ carries the required **role** passes even when source and\n  target live in\
    \ different modules. A declared but unresolved `object:` adds\n  no candidate\
    \ type or roles, while the registered document archetype remains\n  checked. Resolver-synthesized\
    \ reference links are not target-kind checked\n  (exercises [FR-040](../functional/FR-040-object-edge-vocabulary.md))."
- id: FR-040::CR-002
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: '

    ### CR-002 — PLAT-1689 target-kind enforcement


    The original Tier-2 rule checked only a target''s `object:` archetype and roles,

    and treated every mismatch as a warning. FR-040-AC-9 asserted that rule. The

    target-kind rule now also checks the target document''s registered `type:`

    archetype and routes an unconfigured mismatch by bundle posture: error under

    `Strict`, warning under `Okf`. Tier-2 applies only to an edge declared in the'
- id: FR-040::resolution
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: "- It is **target-allowed** iff some token `t` in `verbs(doc)[type]` satisfies\
    \ the\n  resolved target archetype (`target_satisfies`): `t == \"*\"`, or `t`\
    \ equals the\n  target's object type, or `t` is a role the target carries, or\
    \ `t` equals the\n  target document's own archetype name (its `type:` as the manifest\
    \ declares it).\n  Target checking is skipped when the edge is dangling ([FR-026](./FR-026-intra-spec-reference-resolution.md)\n\
    \  already reports that) or when the target list contains `\"*\"`."
- id: FR-040::Tier-2
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: "  **kind**: its registered document archetype, plus — when it declares\
    \ a\n  registered `object:` — that object type and its roles. If `object:` is\n\
    \  declared but unresolved, only a registered document archetype is considered;\n\
    \  the unknown object contributes no kind or roles. Resolver-synthesized\n  `references`\
    \ edges from frontmatter URIs, body `ix://` links or relative\n  Markdown links\
    \ are not checked by Tier-2. The list is satisfied when any\n  token matches any\
    \ candidate kind (`target_satisfies`); no archetype or object"
- id: FR-041::Tier-2
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: examined
  excerpt: "  direction against the forward source's resolved `allowed_links`. A mismatch\n\
    \  yields [FR-040](./FR-040-object-edge-vocabulary.md)'s `DisallowedEdgeTarget`\n\
    \  routed by bundle posture and reported with the **authored** (inverse)\n  `source`/`target`/`edge_type`\
    \ so the diagnostic points at the document the\n  author wrote. Its target-kind\
    \ description names the authored target document's\n  registered `type:` and,\
    \ when resolvable, its `object:` type. An unresolved\n  `object:` contributes\
    \ no type or roles; the registered document archetype"
- id: FR-057::edges-tier
  path: spec/functional/FR-057-corpus-check-severity.md
  role: examined
  excerpt: '| `edges:disallowed-edge-target` | posture-routed (FR-040-AC-13) |'
- id: FR-041-AC-5
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: examined
  excerpt: Inverse recognition and normalization are warnings only — they never block
    extraction or [FR-032](./FR-032-validate-document.md) structural validation; the
    inverse index and diagnostics are deterministic across runs and thread counts
    ([NFR-006](../non-functional/NFR-006-determinism.md))
- id: FR-040-AC-9
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: During corpus validation, an authored frontmatter relationship to a resolved
    target passes when its registered document archetype, registered `object:` type,
    or object role satisfies the verb's target list, including across modules; a declared
    but unresolved `object:` contributes no type or roles, while the registered document
    archetype is still checked. Resolver-synthesized reference edges are not target-kind
    checked.
- id: FR-040-AC-9
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: During corpus validation, an authored frontmatter relationship to a resolved
    target passes when its registered document archetype, registered `object:` type,
    or object role satisfies the verb's target list, including across modules; a declared
    but unresolved `object:` contributes no type or roles, while the registered document
    archetype is still checked. Resolver-synthesized reference edges are not target-kind
    checked.
```

## Round 2 review

Current pushed head 6716fd11f05fc64039acabbc1b3cdc72a69a01b6. Independently inspected the final diff and the round-2 delta. User reports focused rustfmt, edge-target tests 10/10, inverse-normalization 1/1 and clippy --test it -D warnings passed using locked-build.sh; these are producer-reported execution results, not reruns by this reviewer. This pass ran source inspection, computed matrix, scoped grammar checking and artifact validation only. The untracked .report.md and corpus directories were not changed. No code, git state or GitHub comments were changed.

No prior clean review record was overwritten.

```yaml
scope:
- id: src/corpus/validate.rs::registered-kinds
  path: src/corpus/validate.rs
  role: examined
  excerpt: "            let target_obj = document_object(target_doc);\n          \
    \  let target_type_arch = registry.archetype(target_type);\n            let target_obj_arch\
    \ = target_obj.and_then(|o| registry.archetype(o));\n            let satisfied\
    \ = targets.iter().any(|t| {\n                target_type_arch.is_some_and(|a|\
    \ registry.target_satisfies(t, a))\n                    || target_obj_arch.is_some_and(|a|\
    \ registry.target_satisfies(t, a))\n            });"
- id: src/corpus/validate.rs::diagnostic
  path: src/corpus/validate.rs
  role: examined
  excerpt: "                        format!(\n                            \"authored\
    \ edge '{}' from '{}' ({}) targets '{}' ({}); normalized forward edge '{}' from\
    \ '{}' ({}) to '{}' ({}), whose target {} does not satisfy the forward target\
    \ list {:?}\","
- id: tests/it/edge_target_kinds.rs::synthetic-trace
  path: tests/it/edge_target_kinds.rs
  role: examined
  excerpt: '/// Only declared `relationships` entries receive Tier-2 target-kind checks;

    /// frontmatter URIs, body URIs and relative navigation links stay references.

    #[trace("FR-040-AC-9", "FR-040-AC-12")]

    #[test]

    fn synthesized_reference_edges_are_not_target_kind_checked() {'
- id: tests/it/edge_target_kinds.rs::unknown-object-trace
  path: tests/it/edge_target_kinds.rs
  role: examined
  excerpt: '/// An unresolved `object:` contributes no object kind or roles, while
    the

    /// target''s registered document archetype remains checked.

    #[trace("FR-040-AC-9", "FR-040-AC-12")]

    #[test]

    fn unresolved_object_uses_registered_document_archetype() {'
- id: tests/it/edge_target_kinds.rs::dedup-case
  path: tests/it/edge_target_kinds.rs
  role: examined
  excerpt: "\n/// A deduplicated edge keeps authored relationship provenance when\
    \ the same\n/// target also appears as a synthesized body reference. A body-only\
    \ reference\n/// remains navigation and receives no target-kind finding.\n#[trace(\"\
    FR-040-AC-9\", \"FR-040-AC-12\")]\n#[test]\nfn authored_and_synthetic_reference_edges_keep_authored_provenance()\
    \ {\n    let authored_and_body = \"---\\nid: ADR-1\\ntype: adr\\nrelationships:\\\
    n- target: ix://o/r/FR-1\\n  type: references\\n---\\n# ADR-1\\nSee ix://o/r/FR-1.\\\
    n\";"
- id: tests/it/edge_target_kinds.rs::unregistered-test
  path: tests/it/edge_target_kinds.rs
  role: examined
  excerpt: "/// An unregistered document `type:` string cannot satisfy an allow-list\
    \ role.\n#[trace(\"FR-040-AC-9\")]\n#[test]\nfn unregistered_target_type_does_not_satisfy_role_token()\
    \ {\n    use quire_rs::grammar::{GrammarSeverityLevel, GrammarSeverityMap};\n\n\
    \    let root = tmpdir(\"unregistered-type\");\n    write(&root, \"ADR-1.md\"\
    , &doc(\"ADR-1\", \"adr\", Some(\"TARGET-1\")));\n    write(\n        &root,\n\
    \        \"TARGET-1.md\",\n        \"---\\nid: TARGET-1\\ntype: domain-object\\\
    n---\\n# TARGET-1\\n\",\n    );"
- id: tests/it/edge_target_kinds.rs::architecture-failure
  path: tests/it/edge_target_kinds.rs
  role: examined
  excerpt: "    write(\n        &root,\n        \"design/order_schema.md\",\n    \
    \    include_str!(\"../fixtures/corpus_cases/plat-1689/failure/spec/design/order_schema.md\"\
    ),\n    );\n    write(&root, \"functional/FR-004.md\", &doc(\"FR-004\", \"FR\"\
    , None));\n    let report = validate_bundle_at(&root, &registry, BundlePosture::Strict);"
- id: tests/it/edge_target_kinds.rs::architecture-control
  path: tests/it/edge_target_kinds.rs
  role: examined
  excerpt: "    let control_root = tmpdir(\"architecture-order-schema-control\");\n\
    \    write(\n        &control_root,\n        \"design/order_schema.md\",\n   \
    \     include_str!(\"../fixtures/corpus_cases/plat-1689/control/spec/design/order_schema.md\"\
    ),\n    );\n    write(\n        &control_root,\n        \"functional/FR-004.md\"\
    ,\n        include_str!(\"../fixtures/corpus_cases/plat-1689/control/spec/functional/FR-004.md\"\
    ),"
- id: tests/fixtures/corpus_cases/plat-1689/README.md::language
  path: tests/fixtures/corpus_cases/plat-1689/README.md
  role: examined
  excerpt: '

    Language coverage: `n/a` applies because this is Markdown bundle edge resolution,

    not a source-language parser. Rust, Python, and TypeScript parser paths are out

    of scope because they cannot produce this Markdown relationship-resolution

    behavior.'
- id: spec/functional/FR-040-object-edge-vocabulary.md::target-rule
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: "- It is **target-allowed** iff some token `t` in `verbs(doc)[type]` satisfies\
    \ the\n  resolved target archetype (`target_satisfies`): `t == \"*\"`, or `t`\
    \ matches a\n  registered target archetype's name or role, or `t` matches the\
    \ name or role of\n  the target's registered `object:` archetype. A document's\
    \ frontmatter `type:`\n  names its document archetype; an unregistered value is\
    \ not a candidate kind and\n  cannot satisfy an allow-list token.\n  Target checking\
    \ is skipped when the edge is dangling ([FR-026](./FR-026-intra-spec-reference-resolution.md)"
- id: spec/functional/FR-040-object-edge-vocabulary.md::skip-rule
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: "  ([FR-057](./FR-057-corpus-check-severity.md)). Target checking is **skipped**\n\
    \  when the verb's target list contains `\"*\"`, when the target has no frontmatter\n\
    \  `type:` value, when it has neither a registered document archetype nor a\n\
    \  registered `object:` archetype, or when the edge is **dangling**\n  ([FR-026](./FR-026-intra-spec-reference-resolution.md)\
    \ already reports those)."
- id: spec/functional/FR-041-authorable-inverse-edges.md::normalization
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: examined
  excerpt: "- **Tier-2 — enforcement (corpus-level).** During corpus validation\n\
    \  ([FR-025](./FR-025-spec-corpus-model.md)), the corpus validator SHALL\n  **normalize\
    \ an edge `(source, I, target)` whose verb `I` is an inverse label\n  to its forward\
    \ form**\n  `(target, F, source)` before [FR-040](./FR-040-object-edge-vocabulary.md)'s\n\
    \  `target_satisfies` check runs — i.e. source and target are swapped and `I`\
    \ is\n  replaced by `F`, so the existing target-typing rule applies in the canonical"
- id: FR-041-AC-5
  path: spec/functional/FR-041-authorable-inverse-edges.md
  role: examined
  excerpt: Inverse recognition and Tier-1 findings remain warnings only. A Tier-2
    inverse target mismatch follows FR-040's posture-routed severity and override;
    even when it is an error under `Strict`, it does not block edge extraction or
    document loading. Neither Tier-1 nor Tier-2 findings block [FR-032](./FR-032-validate-document.md)
    structural validation. The inverse index and diagnostics are deterministic across
    runs and thread counts ([NFR-006](../non-functional/NFR-006-determinism.md)).
- id: CHANGELOG.md
  path: CHANGELOG.md
  role: examined
  excerpt: "- **Breaking, PLAT-1689:** `edges:disallowed-edge-target` checks resolved,\n\
    \  frontmatter-authored relationships against the target document's registered\n\
    \  archetype and, when resolvable, its `object:` type and roles. Synthesized\n\
    \  `references` links without authored relationship provenance remain navigation\n\
    \  edges and are not kind-checked. A same-key deduplicated edge with authored\n\
    \  relationship provenance is checked once. A\n  mismatch is an error under `Strict`\
    \ and a warning under `Okf` (previously a"
- id: src/corpus/resolve.rs::provenance
  path: src/corpus/resolve.rs
  role: context_only
  excerpt: "                            Some(a)\n                        }\n     \
    \                   _ => None,\n                    };\n                    existing.references.extend(info.references);\n\
    \                    existing.authored_relationship |= info.authored_relationship;\n\
    \                }"
- id: FR-040
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: "  ([FR-057](./FR-057-corpus-check-severity.md)). Target checking is **skipped**\n\
    \  when the verb's target list contains `\"*\"`, when the target has no frontmatter\n\
    \  `type:` value, when it has neither a registered document archetype nor a\n\
    \  registered `object:` archetype, or when the edge is **dangling**\n  ([FR-026](./FR-026-intra-spec-reference-resolution.md)\
    \ already reports those)."
- id: tests/it/edge_target_kinds.rs::unregistered_target_type_does_not_satisfy_role_token
  path: tests/it/edge_target_kinds.rs
  role: examined
  excerpt: "/// An unregistered document `type:` string cannot satisfy an allow-list\
    \ role.\n#[trace(\"FR-040-AC-9\")]\n#[test]\nfn unregistered_target_type_does_not_satisfy_role_token()\
    \ {\n    use quire_rs::grammar::{GrammarSeverityLevel, GrammarSeverityMap};\n\n\
    \    let root = tmpdir(\"unregistered-type\");\n    write(&root, \"ADR-1.md\"\
    , &doc(\"ADR-1\", \"adr\", Some(\"TARGET-1\")));\n    write(\n        &root,\n\
    \        \"TARGET-1.md\",\n        \"---\\nid: TARGET-1\\ntype: domain-object\\\
    n---\\n# TARGET-1\\n\",\n    );"
```

## Current verdict (round 2)

PASS — see this method’s latest dispositions and the separate enforcement status.

## Assurance enforcement (round 2)

AP-201 applies and requires an accepted enforced spec-review operation. Owned run plat-1689-review-round-2 uses the installed spec-review workflow, repo_root /home/peter/dev/worktrees/plat-1689-engine, config_root /home/peter/.ix, and state_dir /tmp/codex-safety/reviews/quire-678/flows. The subset is base plus integrity, failure-domain and ears-conformance. Four review_doc items reference the actual scratch artifacts. Intake and analyses transitions passed. The run is at reviews_rendered; advance to validated returned state=invariant_failed, code WORKFLOW_PATH_INVALID, message "document escapes repository". CLI exit status was 0 but JSON ok=false; this is a refusal, not success. Machine terra, /home/peter/.npm-global/bin/ix-flow 0.3.3, /home/peter/.npm-global/bin/quoin 0.30.1. Minimal reproduction: ix-flow advance plat-1689-review-round-2 validated --config-root /home/peter/.ix --state-dir /tmp/codex-safety/reviews/quire-678/flows --json. The user permits only scratch artifacts and prohibits repo edits, so repository-contained paths cannot be supplied without new authorization. No path-containment workaround or false repository root was used. No human gate was acknowledged. The run is not validated or accepted; direct analysis and explicit-path SR validation do not discharge AP-201. Once findings are corrected, authorization is needed for this reviewer to place the reviewed SRs under the repository reviews/ path, register those paths, revalidate, and then seek explicit human acceptance. No coder/lead SR editing is requested.

## Additional examined criterion scope (round 2)

```yaml
scope:
- id: FR-040-AC-9
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: During corpus validation, each resolved edge with authored frontmatter
    `relationships` provenance is checked once against the target's registered document
    archetype and, when resolvable, registered `object:` archetype and its roles,
    including across modules. A declared but unresolved `object:` contributes no kind
    or roles; the registered document archetype remains checked. A target with an
    unregistered `type:` contributes no document kind.
- id: FR-040-AC-10
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: Tier-1 findings and Tier-2 findings under `Okf` are warnings only; they
    do not block extraction or [FR-032](./FR-032-validate-document.md) structural
    validation, and a corpus with disallowed edges still loads. Diagnostics are sorted
    by `(source, target, edge_type)` and identical across runs and thread counts ([NFR-006](../non-functional/NFR-006-determinism.md)).
- id: FR-040-AC-12
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: 'A target list is matched against the target document''s registered archetype
    name as well as its registered object type and roles: an `adr` source whose `realizes`
    list is `[FR, NFR]` passes to registered `FR`/`NFR` documents and is flagged to
    a registered `adr` document, none of them declaring `object:`.'
- id: FR-040-AC-13
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: Under `Strict` a Tier-2 `disallowed-edge-target` is an error (bundle invalid);
    under `Okf` it is a warning. The finding's path is the source document's file
    path and its message names the source and target file paths, not only their ids.
- id: FR-040-AC-14
  path: spec/functional/FR-040-object-edge-vocabulary.md
  role: examined
  excerpt: '`edges:disallowed-edge-target` set to `error`, `warning` or `off` in `grammar_severity`
    overrides the posture-routed tier in both postures.'
```
