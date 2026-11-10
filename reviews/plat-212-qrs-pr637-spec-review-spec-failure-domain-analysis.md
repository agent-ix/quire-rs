---
id: SR-20265
title: failure-domain review of FR-069 import_types
type: SpecReview
analysis: failure-domain
scope: agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md
review_set: subset
date: '2026-11-10'
---

# failure-domain review of FR-069 import_types

## Summary

Ticket: PLAT-212 (agent-ix/quire-rs PR #637, head e2b6bc42). Looked for unstated failure modes, identity confusion and topological edge cases in the new resolver, `import_types`, `unused-import` and type-level cycle rules. Four defects.

## Scope

- `FR-069#declared-import-ref` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069#import-types` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069#import-cycle` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069#unused-import` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-1` (context_only) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-17` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-19` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-20` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A declared-import `$ref` is "admitted without following it", but FR-069 never says what that subschema means once the compiled validator runs (AC-1): it does not say whether it accepts any value or resolves to the loaded provider's schema, or whether the validator compiles at all. | spec/functional/FR-069-semantic-module-contract-at-load.md:127 |
| FND-002 | medium | "The provider is loaded" is undefined: `semantic.import-unresolved` uses "provides it at that exact version", but the new rules do not say whether a provider at another version, or one refused at load, counts as loaded for `semantic.unknown-import-type`. | spec/functional/FR-069-semantic-module-contract-at-load.md:184 |
| FND-003 | medium | The type-level import graph defines its nodes (imported types) but not its edges: `import_types` says which types a module uses, not which of its own types uses them. So AC-20's "the types used do not depend on each other" cannot be decided, and the mixed case (one module on the cycle with lists and one without) is unstated. | spec/functional/FR-069-semantic-module-contract-at-load.md:199 |
| FND-004 | low | The `$ref` type name is "the final path segment without `.schema.json`", which leaves two cases undefined: a `$ref` under a declared base whose file does not end in `.schema.json`, and a reference into `$defs` by fragment. | spec/functional/FR-069-semantic-module-contract-at-load.md:131 |

## Details

- **FND-001** (FR-069#declared-import-ref): Fix: state the validation meaning, for example "an admitted declared-import `$ref` validates as the empty schema (accepts any value)". Or, if the provider is loaded and its schema is in memory, it resolves to it. Add the matching assertion to AC-19.
- **FND-002** (FR-069#import-types): Fix: define "loaded provider" once as a module that is not refused, whose `package` equals the key and whose version equals the `imports` version (or say any version), and use it in both `unknown-import-type` rules and AC-17.
- **FND-003** (FR-069#import-cycle): Fix: define an edge, for example from type `A#X` to `B#Y` when `X`'s data schema has a `$ref` to `B`'s schema base naming `Y`, or else from every exported type of the importing module to each listed type. State how a package-level node joins a type-level graph, and add the mixed case to AC-20.
- **FND-004** (FR-069#declared-import-ref): Fix: say how such a `$ref` is handled, either refused or left unchecked.

## Verdict

Mergeable after FND-001..FND-003 are fixed; each leaves the implementer to invent behaviour that an AC tests.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | The cycle rule contradicts itself and Quoin. Behavior says the loader SHALL refuse every module on any import-graph cycle and only changes how the cycle is named, and AC-10 agrees. AC-20 instead says a package back edge with no type-level cycle "is not refused when type-level edges exist". Quoin PR #796 FR-075/FR-124 reports an error for every import cycle and builds its type-level view from `import_types` lists, while FR-069 now builds it from `$ref` edges whatever the lists say, so the two tools refuse and name the same cycle differently. | spec/functional/FR-069-semantic-module-contract-at-load.md:278 |

- **FND-005** (FR-069-AC-20): Behavior lines 214-223 vs AC-20 line 278. Quoin #796 FR-075: "Where the modules on an import cycle carry `semantic.import_types` lists, the failure SHALL name the cycle over the types"; FR-124: "one error per import cycle (named over types where type lists exist)". Fix: choose one rule and state it in both specs. Either every package cycle is refused and the type-level graph only changes how it is named, which means dropping the non-refusal clause from AC-20. Or type-level cycles alone are refused, which needs Behavior and AC-10 to say so and Quoin FR-075/FR-124 to agree. Also say whether `import_types` lists or `$ref` edges define the type-level graph, the same in both specs.

## Dispositions

Round 1, against head 35ef824e (fix commit 35ef824).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
