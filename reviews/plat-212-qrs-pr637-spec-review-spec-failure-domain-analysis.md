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
