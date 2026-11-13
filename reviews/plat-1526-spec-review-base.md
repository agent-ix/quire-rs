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
