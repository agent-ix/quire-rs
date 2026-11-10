---
id: SR-20262
title: spec review of FR-069 import_types and import diagnostics
type: SpecReview
analysis: base
scope: agent-ix/quire-rs; spec/functional/FR-069-semantic-module-contract-at-load.md; spec/tests.md
review_set: subset
date: '2026-11-10'
---

# spec review of FR-069 import_types and import diagnostics

## Summary

Ticket: PLAT-212 (agent-ix/quire-rs PR #637, head e2b6bc42). Scope is `git diff origin/main...HEAD`: the FR-069 Inputs, Outputs, Behavior and AC-16..20 edits and the matching spec/tests.md rows. Base review set: requirement-statement quality, consistency with the existing FR-069 text, and agreement with the sibling Quoin contract on the same ticket (agent-ix/quoin branch `plat-212-quoin-spec`, FR-070/FR-075). The fixed decisions hold: `import_types` is an optional sibling and `imports` is unchanged, the code names `semantic.unknown-import-type` (error) and `semantic.unused-import` (warning) match Quoin, the declared-import `$ref` is admitted and unchecked when the provider is not loaded, and the cycle is type-level when lists exist. Three defects.

## Scope

- `FR-069#inputs-block-shape` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069#outputs-semantic-module` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069#declared-import-ref` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069#import-types` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069#unused-import` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069#import-cycle` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-16` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-17` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-18` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-19` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-20` (examined) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-10` (context_only) — spec/functional/FR-069-semantic-module-contract-at-load.md
- `FR-069-AC-5` (context_only) — spec/functional/FR-069-semantic-module-contract-at-load.md

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-069 refuses only a non-array list or a non-string entry in `import_types`, but the sibling Quoin FR-070 (same ticket) also rejects a key absent from `imports`, an empty list, a duplicate and an empty string; quire-rs would load manifests Quoin refuses, and FR-069 leaves an `import_types` key with no matching `imports` entry undefined. | spec/functional/FR-069-semantic-module-contract-at-load.md:253 |
| FND-002 | low | The type-level cycle names nodes `<org>/<repo>#<Type>`, while Quoin FR-075 names them `<org>/<repo>:<Type>`; the same cycle gets two spellings across the two tools. | spec/functional/FR-069-semantic-module-contract-at-load.md:199 |
| FND-003 | low | The declared-import `$ref` bullet states its main rule descriptively ("is a declared-import reference: the resolver admits it") with no SHALL, unlike the surrounding refusal bullets. | spec/functional/FR-069-semantic-module-contract-at-load.md:127 |

## Details

- **FND-001** (FR-069-AC-16): Quoin FR-070 lines 38-39 and AC-9 on `plat-212-quoin-spec`. Fix: state the same shape rule in FR-069 (non-empty array of unique non-empty strings; every key also in `imports`; violation refused as a manifest-schema violation naming the package) and add those cases to AC-16 and TC-2214.
- **FND-002** (FR-069#import-cycle): Pick one separator in both specs; the reason text is a shared contract.
- **FND-003** (FR-069#declared-import-ref): Rewrite as "the resolver SHALL admit a `$ref` ... without following it into the module bundle or fetching it".

## Verdict

Mergeable after FND-001 is fixed (medium, cross-tool divergence). Clean: the `imports` shape is unchanged, `semantic.import-unresolved` and `semantic.schema-ref-unshipped` are kept and agree with the new rules, AC-5 still holds for undeclared packages, the code names and severities match Quoin, and AC-16..20 each map to one new TC.
