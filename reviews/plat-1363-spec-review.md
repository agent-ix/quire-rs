---
id: SR-20261103118135
title: "Spec review — PLAT-1363 vendored/pinned prose cleanup in FR-069..072 (quire-rs PR 617)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-069..FR-072, spec/tests.md, spec/non-functional/NFR-021, spec/integration/IT-001, spec/functional/FR-065"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-072", type: references}
---

## Summary

Ticket: PLAT-1363. PR head `6abef71`. The reviewer checked the rewritten prose against the code
at the head:

- FR-070 Inputs names `crates/quire-fixtures/fixtures/mapping/`. That directory exists and holds
  every file listed.
- FR-070-AC-8, TC-1617, FR-072-AC-5 and TC-1634 name `legacy-free-table.md`. It is the first case in
  `legacy.expected.json` (`free-column-table`, line 17) and the file `validate_document_surface`
  reads (`tests/it/semantic_surface.rs:501`, line 17 asserted).
- FR-071 Inputs: `operations*` live in the quire-fixtures crate.
- FR-069's "module-manifest schema" and "target registry" are the embedded
  `agent-ix-semantic-schema` bundle (`src/semantic/embedded.rs:16-36`).
- TC-1646 dropped "same digest". `Registry::from_inline_parts` has no digest rule.
- TC-1626's "semantic-core schemas" matches the embedded bundle.
- FR-072's FCD#36 clause now reads "the assurance export of FR-067 is unchanged". This is true:
  `src/assurance.rs` carries no semantic projection.
- FR-072-AC-8 and TC-1638 now name `cases.expected.json`, which matches the test.

The listed FRs and TCs no longer say "vendored", "pinned copy" or "same digest". `make validate`
result: see Verdict.

## Verdict

**FAIL (medium)**. The rewrites are true. But the cleanup misses two TCs of the FRs in scope,
and the same false "vendored" prose remains in NFR-021 and IT-001.

FR-065's "vendored" is a different case, and the prose is accurate. The `corpus` submodule
(agent-ix/qa-corpus) really does carry a 39-file copy of `spec-artifacts-process` and
`spec-artifacts-iso` under `corpus/modules/ecosystem/`. Rewording FR-065 would make it false.
The copy itself is a vendoring antipattern to remove in agent-ix/qa-corpus, after which FR-065
gets rewritten. It is outside this PR.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Two TCs of the in-scope FRs still say "vendored". TC-1608 (FR-069-CON-1) says "an `https://` `$ref` outside the vendored bundle", and TC-1621 (FR-070-AC-4/5/6/10) says "the vendored `FieldDecl.json`". The bundle is embedded from the `agent-ix-semantic-schema` git dependency (`src/semantic/embedded.rs:16-20`), not vendored. Reword both to "embedded semantic-core bundle", as TC-1626 was. | spec/tests.md:804, spec/tests.md:816 |
| FND-002 | medium | NFR-021 and IT-001 carry the same stale "vendored" prose. NFR-021 has "the vendored schema bundles" at :27 and "with the vendored bundle embedded" at :60 (AC-2), where the bundle is the embedded git dependency. IT-001 :25 and :45 say "vendored assurance-v1 schema", and :34 states the precondition "Quoin vendors the assurance schema". That is false: quoin's `rust/crates/quoin-quire/src/schema.rs:77-80` uses `quire_rs::assurance::ASSURANCE_V1_SCHEMA` and holds no copy. All five occurrences belong in this cleanup. | spec/non-functional/NFR-021-semantic-extraction-boundary.md:27, spec/non-functional/NFR-021-semantic-extraction-boundary.md:60, spec/integration/IT-001-quire-quoin-assurance-export.md:25, spec/integration/IT-001-quire-quoin-assurance-export.md:34, spec/integration/IT-001-quire-quoin-assurance-export.md:45 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-069 Requirements (manifest schema, target registry) and allocation note | examined | True against embedded.rs; clean |
| FR-070 Inputs, FR-070-AC-1, AC-2, AC-8 | examined | True; clean |
| FR-071 Inputs, FR-071-AC-1 | examined | True; clean |
| FR-072 disposition paragraph, FR-072-AC-5, AC-8 | examined | True; clean |
| TC-1610, 1611, 1617, 1622, 1626, 1634, 1638, 1646 | examined | True; clean |
| TC-1608, TC-1621 | examined | Still "vendored" (FND-001) |
| NFR-021, IT-001 | examined | Stale "vendored" (FND-002) |
| FR-065 "vendored ecosystem module" | examined | Accurate; the copy is a qa-corpus antipattern outside this PR |
| TC-1606 "sourced from the published @agent-ix/semantic-core package" | context_only | Bytes match the npm package, but the source is now the git dependency; not this ticket's kind |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The properties change to FR-072-AC-9 is accurate about ownership, since quire-cli builds the envelope, but it weakens the requirement. "Emitted criterion records validate against the properties-v1 `Criterion` definition" cannot fail when a value or the order changes. FR-072-CON-1 (:148) still says no properties-v1 key may change "meaning, order, value set, or bytes". So CON-1's properties clause now has no AC that could detect a violation. FR-072 Inputs (:64) still lists "the existing ... properties-v1 fixture outputs", and none exist in quire-rs. quire-rs owns the record values (`classify_document_criteria` → `AcClassification`). The honest fix is a checked-in golden of those records, for example the tc857 document's records through `record_json`, compared byte for byte, with AC-9 and TC-2055 saying "emitted criterion records equal their checked-in fixture". Narrowing CON-1 and Inputs to schema conformance instead would be an explicit weakening and needs the owner's decision. | spec/functional/FR-072-semantic-extraction-surface.md:164, spec/functional/FR-072-semantic-extraction-surface.md:148, spec/functional/FR-072-semantic-extraction-surface.md:64, spec/tests.md:1131 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | b31e4ea TC-1608 reads "outside the embedded bundle"; TC-1621 reads "the semantic-core `FieldDecl.json`" |
| FND-002 | fixed | b31e4ea NFR-021 :27 "embedded schema bundles", :60 "with the semantic-core bundle embedded"; IT-001 :25/:45 drop "vendored"; :34 now "Quoin takes the assurance schema from the `quire_rs` dependency and holds no copy", matching quoin-quire/src/schema.rs:77-80 |
