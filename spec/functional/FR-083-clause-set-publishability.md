---
id: FR-083
title: "Clause-set manifest declaration and publishability check"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-073"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-013"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "traces_to"
---
# FR-083: Clause-set manifest declaration and publishability check

## Description

A module declares its clause sets in the manifest key `clause_sets`, a list of
module-relative paths. Every referenced file SHALL carry `rights`
([FR-073](./FR-073-generic-clause-sets.md)); a file without `rights` is rejected
at load.

Rights remain the only licence signal, and the licence posture is one value of
the existing `rights.text` field: `licensed`. A clause set whose `rights.text`
is `licensed` holds clause text that its holder may use locally but may not
publish. It is **restricted**. Every other loadable clause set (`text` of
`none`, `original` or `explicitly_cleared`) is **publishable**. Structure rights
(`structure`) do not affect publishability. `licensed` permits clause text at
load and requires no `clearance_ref` (FR-073). The `TextRights::Licensed` value
and its validation are implemented under PLAT-1420.

The engine exposes one pure function:

```rust
pub fn check_publishable(module: &LoadedModule) -> Vec<RestrictedClauseSet>
```

`RestrictedClauseSet` carries `authority`, `id`, `version` and `text_clauses`
(the count of clauses that carry text, zero when none). The
`authority/id@version` triple names the set. It never carries clause text. The
function reads only the already loaded module and returns restricted sets
ordered by `authority`, `id`, then `version`. An empty vector means the module
is publishable.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-083-AC-1 | A `clause_sets` value that is not a list of strings fails module load with an error naming the `clause_sets` key. | Test |
| FR-083-AC-2 | Loading a module whose referenced clause-set file has no `rights` object fails with a load error naming the file. | Test |
| FR-083-AC-3 | `check_publishable` returns an empty vector for a synthetic `example.invalid` module whose sets are `text: none`, `text: original`, or `text: explicitly_cleared` with a non-blank `clearance_ref`. | Test |
| FR-083-AC-4 | `check_publishable` returns one `RestrictedClauseSet` for a synthetic set with `text: licensed` and clause text present, carrying that set's authority, id, version and text-clause count. The module still loads and the set passes `ClauseSet::validate`. | Test |
| FR-083-AC-5 | Structure rights do not change the verdict: each of `original`, `citation_only` and `explicitly_cleared` structure over a publishable `text` value is not returned. | Test |
| FR-083-AC-6 | In a module with one publishable and two `licensed` sets, the result lists exactly the two licensed sets, ordered by authority, id, version. Two runs yield an identical vector. | Test |
| FR-083-AC-7 | No `RestrictedClauseSet` field or `Debug` output contains any clause text from the fixture; a test searches for the fixture's text sentinel and fails if found. | Test |
| FR-083-AC-8 | Deleting the module directory after load leaves the `check_publishable` result unchanged. A module with no `clause_sets` returns an empty vector. | Test |
| FR-083-AC-9 | A `text: licensed` set with no clause text loads and is returned with `text_clauses` of zero; a `text: licensed` set with `clearance_ref` absent loads. | Test |

## Constraints

Fixtures SHALL be original synthetic content with `example.invalid` authorities.
No real publication name, identifier or text appears in the repository.

## Dependencies

- **Upstream**: [FR-073](./FR-073-generic-clause-sets.md), [FR-013](./FR-013-archetype-loader.md)
- **Downstream**: `agent-ix/quire-cli` consumes `check_publishable` for `quire publish-check`. The engine owns the rights rule and the CLI holds no rights policy (PLAT-1419, PLAT-1420, PLAT-1421).
