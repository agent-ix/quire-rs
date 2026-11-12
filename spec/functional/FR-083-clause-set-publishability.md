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
module-relative paths. The manifest schema SHALL describe this key. Every
referenced file SHALL carry `rights` ([FR-073](./FR-073-generic-clause-sets.md));
a file without `rights` is rejected at load.

Rights remain the only licence signal. No new licence field exists. A clause set
is **restricted** when its `rights.structure` is `citation_only` and at least one
clause carries `text`. Every other loadable clause set is **publishable**:

- `text: none` (no clause carries text), whatever the structure rights;
- `text: original` or `text: explicitly_cleared` with `structure` `original` or
  `explicitly_cleared` (the `clearance_ref` requirement of FR-073-AC-6 already
  holds at load).

`citation_only` structure with clause text present is **valid but restricted**,
not invalid. The reading is deliberate. Making it invalid would make a
locally held licensed set unloadable, so the evaluate and discharge paths could
not use it. It would also leave no loadable set that a publish gate could ever
refuse, which makes the gate vacuous. Restricted means loadable and
usable locally, and never publishable.

The engine exposes one pure function:

```rust
pub fn check_publishable(module: &LoadedModule) -> Vec<RestrictedClauseSet>
```

`RestrictedClauseSet` carries `authority`, `id`, `version`, the module-relative
`path`, and `text_clauses` (the count of clauses that carry text). It never
carries clause text. The function performs no I/O, reads only the already
loaded module, and returns restricted sets ordered by `authority`, `id`,
`version`, then `path`. An empty vector means the module is publishable.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-083-AC-1 | The module-manifest schema declares `clause_sets` as a list of module-relative paths. A manifest whose `clause_sets` is not a list of strings fails schema validation. | Test |
| FR-083-AC-2 | Loading a module whose referenced clause-set file has no `rights` object fails with a load error naming the file. | Test |
| FR-083-AC-3 | `check_publishable` returns an empty vector for a synthetic module whose sets are `text: none` (any structure) or `text: original` with `structure: original`, and for a `text: explicitly_cleared` set with a non-blank `clearance_ref`. | Test |
| FR-083-AC-4 | `check_publishable` returns one `RestrictedClauseSet` for a synthetic `example.invalid` set with `structure: citation_only` and at least one clause carrying `text`, with that set's authority, id, version, path and text-clause count. The same module still loads and passes `ClauseSet::validate`. | Test |
| FR-083-AC-5 | A `citation_only` set with `text: none` and no clause text is not returned. Adding text to one clause of it without changing `rights` fails load under FR-073-AC-6, and with `text: original` it is returned. | Test |
| FR-083-AC-6 | In a module with one publishable and two restricted sets, the result lists exactly the two restricted sets, ordered by authority, id, version, path. Running it twice yields an identical vector. | Test |
| FR-083-AC-7 | No `RestrictedClauseSet` field, `Debug` output or serialized form contains any clause text from the fixture; a test searches for the fixture's text sentinel and fails if found. | Test |
| FR-083-AC-8 | `check_publishable` takes only `&LoadedModule`: it opens no file and no socket, and leaves the module directory byte-identical. A module with no `clause_sets` returns an empty vector. | Test |

## Constraints

Fixtures SHALL be original synthetic content with `example.invalid` authorities.
No real publication name, identifier or text appears in the repository.

## Dependencies

- **Upstream**: [FR-073](./FR-073-generic-clause-sets.md), [FR-013](./FR-013-archetype-loader.md)
- **Downstream**: `agent-ix/quire-cli` consumes `check_publishable` for `quire publish-check`; the engine owns the rights rule and the CLI holds no rights policy (PLAT-1419, PLAT-1420, PLAT-1421).
