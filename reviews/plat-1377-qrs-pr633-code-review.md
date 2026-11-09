---
id: SR-20261103118148
title: "Code review — PLAT-1377 relock semantic-core 0.4.0 and repoint fixture refs (quire-rs PR 633)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; CHANGELOG.md, Cargo.lock, crates/quire-fixtures/fixtures/module-ok/schemas/Entity.json, crates/quire-fixtures/fixtures/record-sections/schemas/*.json, schemas/output/semantic-v1.schema.json, tests/fixtures/semantic/cases.json"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-069", type: references}
---

## Summary

Ticket: PLAT-1377. PR head: `bccc1ef` (branch `plat-1377-semantic-core-040`, merge base `32b3ed7`). Code review with the Rust lane folded in: the PR changes no `.rs` file and no `Cargo.toml`; its only Rust-side change is one `Cargo.lock` source line, so the Rust lane covers the lock only.

What was measured, independently of the PR text:

- `Cargo.lock`: the diff is exactly one line, the `agent-ix-semantic-schema` source moving from filament-core-data `6f3da0f` to `7a416a9a24e5113da6b6ec47683de509a036890a` (FCD #345, "release semantic-core 0.4.0"). No other package changes.
- Stale refs: `git grep 'semantic-core/0\.3\|0\.3\.3'` over the PR head finds no remaining `$ref`. The only hits are historical review files under `reviews/` (not to be edited), the new CHANGELOG line itself, and `tests/it/semantic_contract.rs:123`, a TC-1599 tuple that sets a module's declared `semantic_core: 0.3.3` value to prove FR-069's "SHALL NOT refuse a module over a `semantic_core` value". That is test input, not a stale ref. `origin/main` has the same 40 refs as the merge base, so main added no new 0.3.3 refs.
- Bundle content: `packages/semantic-core` at `7a416a9` is byte-identical to `6f3da0f` once `0.3.3`/`0.4.0` are normalised. 0.4.0 is a version renumber: `TypeRefTarget` was already the `{kind, value}` tagged union in 0.3.3.
- Validation against the 0.4.0 bundle (scratch Python `jsonschema` 4.26 run, registry built from the 0.4.0 generated JSON Schemas plus the record-sections fixture schemas): all 40 `semantic-core` `$ref`s in the changed files resolve to a 0.4.0 `$id`. 16 record-sections cases whose schema references semantic-core validate their expected `record` with the result their `recordValid` states (0 mismatches). 7 `expect.values` entries in `tests/fixtures/semantic/cases.json` validate against their `dataSchema` property (0 invalid). A `{kind, value}` `TypeRef` target validates and a bare-string target is refused. The fixtures need nothing beyond the version repoint.
- The relock pulls about 30 FCD commits, not only the semantic-core release. Among the schemas quire-rs embeds: the module-manifest `semantic` block no longer has or requires `semantic_core`, `PACKAGE_LOCK` is dropped from `SCHEMAS`, the manifest gains the FR-149 keys, and `semantic-ir` changes. quire-rs reads only `properties.semantic` of the manifest schema (`src/semantic/contract.rs:125`) and never `PACKAGE_LOCK`. Its `semantic_core` instance strip (contract.rs:175) is mandated by FR-069 ("SHALL NOT refuse a module over a `semantic_core` value"), so loading behaviour is unchanged.
- Mergeability: GitHub reports `CONFLICTING`. `main` released 0.55.0 (#632) after the branch point, and the CHANGELOG hunk conflicts (FND-001).

No aggregate gate was run; the brief states the coder ran `make -k ci` green on the head.

## Verdict

**CHANGES REQUIRED.** The diff does what it claims. The lock moves one dependency. Every 0.3.3 `$ref` is repointed. The fixtures validate against the real 0.4.0 bundle, and no ceremony is added. The PR is not mergeable as it stands, because it conflicts with `main` in `CHANGELOG.md` (FND-001). FND-002 and FND-003 are low and can be fixed in the same rebase.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | PR conflicts with `main` (GitHub `CONFLICTING`). `main` cut 0.55.0 (#632) after the branch point, so the PLAT-1377 entry sits above the PLAT-847 entry under what is now `## [0.55.0]`. It needs a rebase, with the entry placed under the new empty `## [Unreleased]` and its own `### Changed` heading. | CHANGELOG.md:12 |
| FND-002 | low | The CHANGELOG entry says the bundle moved to "`{kind, value}` ids, IR-994". The embedded semantic-core 0.4.0 schemas are byte-identical to 0.3.3 apart from the version segment, and `TypeRefTarget` was already `{kind, value}`, so the text describes a wire change that does not happen. It also omits the contract changes the relock does bring: the `semantic` block drops `semantic_core`, `PACKAGE_LOCK` leaves `SCHEMAS`, and the manifest gains the FR-149 keys. It should say plainly that this is a version renumber plus an FCD relock. | CHANGELOG.md:12 |
| FND-003 | low | After the relock the embedded module-manifest `semantic` block's `required` is `["contract_version", "package"]`, so `block_validator`'s `required.retain(\|key\| key != "semantic_core")` workaround and its comment are dead code. Delete them. The instance strip at contract.rs:175 stays, because FR-069 requires it. | src/semantic/contract.rs:128-132 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| `Cargo.lock` `agent-ix-semantic-schema` source | examined | Only this dependency changes; target commit is FCD #345 |
| `crates/quire-fixtures/fixtures/module-ok/schemas/Entity.json` | examined | Repointed; refs resolve in 0.4.0 |
| `crates/quire-fixtures/fixtures/record-sections/schemas/*.json` (11 changed) | examined | Repointed; 16 cases validate as `recordValid` states |
| `schemas/output/semantic-v1.schema.json` `typeRef`, `constraint` | examined | Repointed; resolve in 0.4.0 |
| `tests/fixtures/semantic/cases.json` (12 refs) | examined | Repointed; 7 expected values validate |
| `CHANGELOG.md` PLAT-1377 entry | examined | FND-001, FND-002 |
| Repo-wide `0.3.3` grep (PR head and `origin/main`) | examined | No stale `$ref` left |
| `src/semantic/contract.rs` `block_validator`, `read_semantic_block` | context_only | FND-003 (dead code made dead by the relock) |
| FR-069 version-agnostic ref resolution and `semantic_core` tolerance | context_only | No spec change needed, as the PR body claims |

## Dispositions

Round 1, head `fe997ce` (fix commit `fe997ce` on top of `18085f8`, which is rebased onto `7e61e14`, the 0.55.0 release). GitHub reports `MERGEABLE`. The fixture, schema and test `$ref` hunks are identical to the reviewed head `bccc1ef`. No `semantic-core/0.3` ref remains outside `reviews/` and `CHANGELOG.md`. No gate run (per brief).

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | Rebased onto 0.55.0; the entry sits under `## [Unreleased]` with its own `### Changed` (CHANGELOG.md:10-12); GitHub `mergeable: MERGEABLE` |
| FND-002 | fixed | Entry now says "There is no `{kind, value}` wire change in quire-rs" and lists the relocked FCD contract changes (semantic block drops `semantic_core`, `PACKAGE_LOCK` leaves `SCHEMAS`, manifest gains FR-149 keys) |
| FND-003 | fixed | `block_validator` now clones the `semantic` block with no `required.retain`; the FR-069 instance strip (`map.remove("semantic_core")`, contract.rs:170) is kept |
