---
id: SR-20261113045746
title: "Spec review (base) - PLAT-1526 FR-090 carried fields and FR-091 field consistency (quire-rs PR 669)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@e68cc6c; spec/functional/FR-090-declared-carried-fields.md, spec/functional/FR-091-ordered-field-consistency.md, spec/functional/index.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-090", type: references}
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-091", type: references}
---

# Spec review (base) - PLAT-1526 (quire-rs PR 669)

## Summary

Ticket: PLAT-1526. Spec-only review of `git diff origin/main...e68cc6c`. Base checklist review of the two new FRs, with cross-PR consistency checks against spec-artifacts-iso#73, spec-objects-architecture#45 and quoin#855 (same ticket) and quire-rs#668 (PLAT-1417). Tools: quoin 0.30.1, quire 0.44.0 (engine 0.59.1), module spec-artifacts-process@v0.31.0. Assurance profile spec/assurance/AP-201-detection-minting.md (`review_policy: require [spec-review]`) applies, because FR-090 changes coverage-matrix and published-schema output. The reviewer ran a direct review. The enforced `ix-flow run review` needs human acknowledgement for its final transition, so it was not started here and remains for the lead. The review set is base plus integrity, dependency and ears-conformance. failure-domain, evidence, risk-complexity and scope-boundary were not run separately: their scope for these two FRs is covered by base FND-002/003/005/007.

Units examined:

- `FR-090` (examined)
- `FR-090-AC-1` (examined)
- `FR-090-AC-2` (examined)
- `FR-090-AC-3` (examined)
- `FR-090-AC-4` (examined)
- `FR-090-AC-5` (examined)
- `FR-090-AC-6` (examined)
- `FR-090-AC-7` (examined)
- `FR-090-AC-8` (examined)
- `FR-090-AC-9` (examined)
- `FR-090-CON-1` (examined)
- `FR-090-CON-2` (examined)
- `FR-091` (examined)
- `FR-091-RF` (examined)
- `FR-091-AC-1` (examined)
- `FR-091-AC-2` (examined)
- `FR-091-AC-3` (examined)
- `FR-091-AC-4` (examined)
- `FR-091-AC-5` (examined)
- `FR-091-AC-6` (examined)
- `FR-091-AC-7` (examined)
- `FR-091-AC-8` (examined)
- `FR-091-AC-9` (examined)
- `FR-091-AC-10` (examined)
- `FR-091-AC-11` (examined)
- `FR-091-AC-12` (examined)
- `FR-091-CON-1` (examined)
- `FR-091-CON-2` (examined)

## Verdict

**Changes requested: five high, four medium, one low.** The direction is right: the engine holds no class name in its prose design, nothing derives, defaults or writes a class, a class stays independent of criticality, and there is no compatibility layer or standard text. The mechanics are not ready to implement. AC-4 contradicts FR-082-AC-7 on where criticality lives. The top-level carried key breaks the sealed coverage-v1 schema. Both load-time rules contradict FR-058's per-module-load rationale and would fail the sibling architecture module's own declaration. The one-hop consistency model cannot express allocation as the architecture module defines it. Clean units: FR-090-AC-1/2/3/5/6/9, CON-1/2; FR-091-AC-2/3/5/6/7/8/9, CON-1/2 (examined, no defect beyond those inherited from FND-005/FND-008).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-090-AC-4 contradicts FR-082-AC-7. It says a document stating frontmatter `criticality: high` yields obligations carrying `criticality`. FR-082-AC-7 says the opposite: frontmatter criticality goes only on the `requirements[]` entry, and obligation records keep their own FR-053 `criticality_column` value (src/obligation.rs:185 reads only the column). The description makes the same mistake when it says FR-082 already does this 'for criticality' on 'every obligation'. quoin#855 FR-160-AC-5 depends on this wrong premise. Fix: restate AC-4 for the two levels separately (entry: frontmatter criticality; obligation: column criticality, unchanged), or extend FR-082 explicitly with a CR. Do not imply that obligations already carry frontmatter criticality. | spec/functional/FR-090-declared-carried-fields.md:67 |
| FND-002 | high | Carrying the value as a top-level key named by module data (`under name`) does not fit the published contract. In schemas/output/coverage-v1.schema.json, both `Obligation` and `CoverageMatrixRequirement` are sealed (`additionalProperties: false`), and FR-053 says contract versioning lives in those schemas. Every carried value would therefore fail the shipped schema. Nothing rejects a `name` that collides with a fixed key (`criticality`, `id`, `document`, `method`, `criteria`, `status`) or with another entry's `name` after the cross-module merge. Fix: name the serialized shape, for example a nested `carried: {<name>: value}` map, state the schema change, and add an AC that refuses a reserved or duplicate `name` at load. | spec/functional/FR-090-declared-carried-fields.md:36-40 |
| FND-003 | high | AC-7 requires module load to fail when `from` names an archetype the module does not declare, 'as FR-058 fails a bad declaration'. FR-058 ('A declaration that names a kind nothing has') says this cannot be a load-time rule: `TraceabilityModel::validate` runs per module before the merge, and declarations legitimately name kinds that another module contributes. FR-058 reports the case at validation instead. A module that carries a field from another module's archetype would fail its own load. Fix: keep the load-time failure for structural defects only (blank name/key, empty `from`), and report an unknown archetype or key against the merged registry, as FR-058 does. | spec/functional/FR-090-declared-carried-fields.md:70 |
| FND-004 | high | AC-10 has the same load-time problem as FND-003, and here it breaks the sibling PR. spec-objects-architecture#45 FR-012-AC-12 has the architecture module declare a `field_consistency` entry whose `from` and `order_from` name `FR`, which is spec-artifacts-iso's archetype. Under AC-10 that entry fails the architecture module's own load ('names an undeclared archetype'), because per-module validation runs before the merge (FR-058). Fix: as for FND-003, report undeclared archetypes and schema properties against the merged registry, not at per-module load. | spec/functional/FR-091-ordered-field-consistency.md:91 |
| FND-005 | high | The model checks one hop (`from` --edges--> `to`), and that cannot express allocation as the target vocabulary defines it. In spec-objects-architecture main manifest.yaml, `allocated_to` is an allowed link of the `allocation` archetype only (`allocation: allowed_links: {allocates: ['*'], allocated_to: [systems-part]}`). The chain is FR <-allocates- allocation -allocated_to-> part, so an FR never authors `allocated_to` to a part. The worked example, AC-1 and AC-4 ('an FR ... allocated to a part'), and FR-012-AC-12 all rely on a direct edge that the module forbids. Fix: either allow a declared multi-hop path (for example `via: allocation` with an incoming `allocates` edge and an outgoing `allocated_to` edge), or rewrite the example and ACs against an edge the architecture module actually allows, and get the sibling PR to agree. | spec/functional/FR-091-ordered-field-consistency.md:33-41, 82, 85 |
| FND-006 | medium | `quire trace` JSON carries no obligation records. FR-082 itself says the FR-077 trace lookup is keyed by trace id and has no requirement rows, and quire-cli main src/commands/trace.rs never mentions obligations. So 'carries the carried key on each obligation' holds vacuously for `trace`. Both surfaces also belong to quire-cli, not this engine. Fix: state the engine payloads (`CoverageReport::obligations`, `requirements[]`) here, and leave the CLI surfaces to quire-cli FR-028. If `quire trace` is meant to carry obligations, specify that. | spec/functional/FR-090-declared-carried-fields.md:40, 71 |
| FND-007 | medium | 'each document reached over `edges` in `direction`' does not say how edges are read. Concurrent PR quire-rs#668 (FR-058 CR-416) makes relation matching accept an edge authored with the declared inverse verb from the other end, counting a pair authored both ways once. FR-086 relies on that. FR-091 is silent on it, and on dangling edges and cross-member targets (FR-085). Two implementers could read the same corpus differently: one follows only the authored verb, the other normalizes inverses. Fix: state that `field_consistency` reads edges as FR-058 relation matching does (inverse-normalized, de-duplicated, with dangling or out-of-`to` targets not compared), and add an AC. | spec/functional/FR-091-ordered-field-consistency.md:45-47 |
| FND-008 | medium | `to_field` is called a 'record key path', and `required_fields.field` (`safetyClass.class` on `part`) is not defined at all. Neither says where the value is read from. In sibling FR-012, `safetyClass` is an extracted body-record key (`## Safety Class` table, FR-008 record-key form), not frontmatter. FR-091 cites no extraction requirement (for example FR-081). AC-10's 'a property absent from its schema' does not say whether that is the frontmatter schema or the data_schema (Part.json). Fix: define the path syntax and source (frontmatter key vs extracted record path), cite the FR that supplies extracted records to corpus checks, and name which schema AC-10 checks. | spec/functional/FR-091-ordered-field-consistency.md:39, 57-66, 91 |
| FND-009 | medium | AC-12 says no engine source 'mentions ... `part`'. The engine source on main already contains `part` as a whole word 75 times in prose and identifiers (for example src/lint.rs:104 'not part of the contract'), so the AC fails before any implementation and can pass only through unrelated renames. Fix: limit it to identifiers or string literals that name an archetype, field or verb (for example 'no string literal equal to ...'), matching CON-1. | spec/functional/FR-091-ordered-field-consistency.md:93 |
| FND-010 | low | AC-11 asks for an identical order across runs but never says what the order is. FR-058 states document-then-id order (FR-058-AC-21). An order that changes when input documents are reordered but stays stable across runs would pass. Fix: state the ordering key, for example (check, from document, target document), and test it independent of document order. | spec/functional/FR-091-ordered-field-consistency.md:92 |

## New findings (disposition pass 1)

Found at head `5f7b2fd` while regression-checking the fix round.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-011 | high | Regression: `carried_vocabulary` is placed as 'an object on the `coverage_matrix`', but in the published schemas/output/coverage-v1.schema.json, `coverage_matrix` is a JSON array of `CoverageMatrixRequirement`. An array has no key to add, so AC-10 cannot be met and AC-12's 'rejects any other unknown key on ... the matrix' names an object that does not exist. quoin#855 FR-160 reads `carried_vocabulary` from the run. Fix: put `carried_vocabulary` on the top-level coverage report object (also `additionalProperties: false`, so the amendment names it there), or state a breaking change to `coverage_matrix`'s shape. Keep quoin FR-160's read path in step. | spec/functional/FR-090-declared-carried-fields.md:51-59, 100, 102 |
| FND-012 | medium | Regression: FR-090 says coverage-v1.schema.json is `additionalProperties: false` on 'the row document object', and amends it there. The FR-084 relation report is not part of the coverage payload: coverage-v1 has no relation-report or row-document definition, and the type is `RowDocument` in src/corpus/relation_report.rs. The AC-12 clause about the row document therefore tests a definition that does not exist. FR-084-AC-12 also fixes the exact key order of the row serialization, and FR-090 neither says where `carried` sits in that order nor adds a CR note to FR-084. Fix: name the relation report's own contract (or say it has no published schema), state the key position of `carried` in the FR-084 document object, and add a CR note on FR-084. | spec/functional/FR-090-declared-carried-fields.md:45-47, 57-60, 99, 102 |
| FND-013 | medium | After the cross-module merge, the `from` list is a union, and its order (so 'the first `from` archetype') depends on module load order. The architecture entry `from: [part]` merges with the ISO entry. `part` states the key through the open ObjectFrontmatter with no `enum` (sibling FR-012). If `part` comes first, 'first archetype's enum' gives nothing while another `from` schema has one, which contradicts 'no enum in any `from` schema'. Two implementers would emit different vocabularies for the same corpus. Fix: define the vocabulary independent of order, for example the enum of the first `from` archetype that has one in a stated order, or require all `from` enums that exist to be equal and report a mismatch. | spec/functional/FR-090-declared-carried-fields.md:51-53, 68-70 |
| FND-014 | medium | Two allocations reaching the same part yield one pair, but each finding must name 'the `via` document', and findings are ordered by `via` id. With two routes there are two `via` documents, and the spec does not say which one the single finding names (lowest id, all of them as a list, or the first read). One implementer names one, another lists both. Fix: state it, for example 'names every `via` document of the pair, in lexical id order, and orders by the first', and assert it in AC-4. | spec/functional/FR-091-ordered-field-consistency.md:62-63, 76-77, 85 |

## New findings (disposition pass 2)

Found at head `3483c6f` (branch merged with origin/main) while regression-checking the round-2 fixes.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-015 | medium | Regression from the round-2 fix: the position of `carried` in a resolved `linked` entry is stated two ways. FR-084 defines a resolved `linked` entry as 'a document plus `verb` and `resolution`', and CR-419 puts `carried` last in the Document key order, so it falls before `verb` and `resolution`. That is also what the flattened `RowDocument` in src/corpus/relation_report.rs:179-190 would emit. FR-090-AC-9 says resolved `linked` entries carry `carried` 'as the last key', which is after `resolution`. A test written to AC-9 fails an implementation that follows FR-084, and the reverse. Fix: say in AC-9 that `carried` is the last key of the Document part (before `verb`, `resolution`), or state the full linked-entry key order in FR-084. | spec/functional/FR-090-declared-carried-fields.md:107; spec/functional/FR-084-two-way-relation-coverage-report.md:121-123,129 |
| FND-016 | low | FR-091 now adds three declarations (`field_consistency`, `required_fields`, `vocabulary`), but AC-6 still says 'neither entry'. Nothing states that a module declaring none of the three, or no `vocabulary` entry, is unchanged. Fix: 'declares none of the three entries'. | spec/functional/FR-091-ordered-field-consistency.md:145 |

## New findings (disposition pass 3)

Found at head `3572ba8` while checking the renumbering and cross-references.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-017 | low | Same stale count as FND-016: FR-091 now has three declarations, but CON-1 still says 'for either declaration', so on a literal reading the new `vocabulary` declaration is outside the no-names constraint. Fix-forward, not blocking: 'for any of the three declarations'. | spec/functional/FR-091-ordered-field-consistency.md:163 |

## Dispositions

Round 1, at head `5f7b2fd`, checked against current main (quire-rs#668 merged).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | \| FR-090-AC-4 \| A document stating `criticality: high` and no class yields an entry with `criticality` and no `carried`; a document stating a class and no criticality yields `carried` and no `criticality`; one stating both carries both as written; obligation records of all three are byte-identical to their output without `carried_fields`. \| Test \| |
| FND-002 | fixed | the engine SHALL carry the value as written under `name` inside **one fixed key**, `carried`, an object, on the document's `requirements[]` entry ... The engine adds no other key to any entry, row or obligation record ... This requirement SHALL amend it to add the optional `carried` object ... Two entries with the same `name` and different `key` fail the merge naming both modules. |
| FND-003 | fixed | \| FR-090-AC-8 \| An entry whose `from` names an archetype that no loaded module declares, or whose `key` is a property of none of the `from` frontmatter schemas and carried by no document, reports `trace:undeclared-relation-vocabulary` naming the entry when the corpus is merged; the same entry in a module whose `from` archetype another loaded module declares reports nothing and does not fail that module's load. \| Test \| |
| FND-004 | fixed | \| FR-091-AC-11 \| An entry in the architecture module naming the ISO module's `FR` archetype and field loads without error; with both modules loaded it works, and with the ISO module not loaded the merged corpus reports `trace:undeclared-relation-vocabulary` naming the entry. \| Test \| |
| FND-005 | fixed | \| FR-091-AC-1 \| With the `field_consistency` entry above, an `FR` stating `class-1`, an `allocation` that `allocates` it and is `allocated_to` a `part` stating `class-3` yields one `trace:class-below-part` finding naming the `FR`, the `allocation`, the `part` and both values. \| Test \| |
| FND-006 | fixed | \| FR-090-AC-9 \| The relation report row `document` objects of FR-084 carry `carried` for a document that states a value, and no `carried` key otherwise. \| Test \| (the `quire trace` claim is removed) |
| FND-007 | fixed | Every hop is read as FR-058 relation matching reads an edge: an edge authored with the declared counterpart label from the opposite end is read as the listed verb (CR-416, quire-rs#668), a pair authored both ways counts once ... A far end held in another corpus member resolves as FR-085 resolves it; one in a member that is not loaded is dangling and not compared. |
| FND-008 | fixed | `field` and `to_field` are both **top-level frontmatter keys**, read as FR-082 and FR-090 read them; no extracted body-record key is read. |
| FND-009 | fixed | \| FR-091-AC-15 \| A fixture corpus whose manifest names neutral archetypes, fields and verbs (for example `alpha`, `beta`, `rank_field`, `links_to`) and no name from this specification yields the same findings, so the engine holds no name. \| Test \| |
| FND-010 | fixed | \| FR-091-AC-14 \| Findings are in `check`, `from` path, `via` id, target id order, identical for two runs and for the corpus documents supplied in reverse order. \| Test \| |
| FND-011 | still-open | New in round 1; not yet fixed. |
| FND-012 | still-open | New in round 1; not yet fixed. |
| FND-013 | still-open | New in round 1; not yet fixed. |
| FND-014 | still-open | New in round 1; not yet fixed. |
| FND-011 | fixed | `carried_vocabulary`, on the **top-level coverage report object** (the object that holds the `coverage_matrix` array; the matrix itself is an array and gains no key) ... add ... the optional `carried_vocabulary` object (string-keyed arrays of strings) on the top-level report object, and no other key. |
| FND-012 | fixed | The FR-084 relation report has no published schema; its `document` object gains `carried` as its last key by the CR-419 note on FR-084 and keeps the key order that FR-084-AC-12 fixes otherwise. (FR-084: Document `{"id", "path", "archetype", "repo", "verification", "criteria", "carried"}` + CR-419 note.) |
| FND-013 | fixed | The vocabulary does not depend on module load order. The `enum`s that exist among the `from` archetypes' frontmatter schemas for `key` must all be identical in content and order; the vocabulary is that enum. A `from` archetype with no `enum` for `key` ... contributes nothing and does not conflict. When two existing `enum`s differ, the engine reports `trace:carried-vocabulary-mismatch` ... (FR-090-AC-14) |
| FND-014 | fixed | \| FR-091-AC-4 \| ... two allocations reaching the same part yield exactly one finding for that pair, which names both `via` documents in lexical id order. \| Test \| |
| FND-015 | still-open | New in round 2; not yet fixed. |
| FND-016 | still-open | New in round 2; not yet fixed. |
| FND-015 | rejected | Moot at 3572ba8: relation-report `carried` was removed from FR-090 (text, old AC-9, FR-084 edge and dependency) and the FR-084 CR-419 note was deleted, so FR-084 is byte-identical to origin/main and no linked-entry key order is stated anywhere. The defect no longer exists in the artifact. |
| FND-016 | fixed | \| FR-091-AC-6 \| A corpus whose modules declare none of the three (`field_consistency`, `required_fields`, `vocabulary`) yields no finding and output identical to the output before this requirement. \| Test \| |
| FND-017 | deferred | Fix-forward, not blocking: one-word wording fix ('either' to 'any of the three'); rides with the PLAT-1578 implementation PR or a trailing commit. |
