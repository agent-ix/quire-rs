---
id: SR-070
title: "integrity review of the semantic extraction boundary slice (issue #388)"
type: SpecReview
analysis: integrity
scope: "spec/usecase/US-019-extract-semantic-declarations.md, spec/functional/FR-069-semantic-module-contract-at-load.md, spec/functional/FR-070-typed-properties-extraction.md, spec/functional/FR-071-clause-and-operation-extraction.md, spec/functional/FR-072-semantic-extraction-surface.md, spec/non-functional/NFR-021-semantic-extraction-boundary.md, spec/tests.md TC-1599..TC-1644"
review_set: all
---

## Summary

Examined the `agent-ix/quire-rs#388` slice (US-019, FR-069..FR-072, NFR-021,
matrix rows TC-1599..TC-1644) for completeness, consistency, and atomicity
against the normative upstream contract (`agent-ix/quoin` FR-070..FR-075, the
mapping fixtures at `3e842ce`, the vendored semantic-core `0.1.0` schemas) and
the current engine (`src/filament.rs`, `src/loader/*`, `src/error.rs`).
Traceability is complete: US-019 traces to StR-001; FR-069..FR-072 each
implement US-019; NFR-021 constrains all four; every `requires` target
(FR-013, FR-045, FR-046, FR-032) and every prose dependency (FR-005, FR-011,
FR-025, FR-055, FR-067, NFR-020) exists; every AC and CON id the matrix cites
exists and every AC/CON of the five requirements is covered by at least one
TC. The verdict is **conditional pass**: no missing trace and no missing
verification, but eleven medium findings — two internal contradictions
(`not_applicable` for a module without a `semantic` block versus byte-identical
records; a fixed `endColumn` of 4 versus the nested-fence property test), a
wrong vendored-bundle path, five term/contract drifts against the quoin
mapping (missing `nullable`, no type-resolution precedence or title tie-break,
reader rules not carried, module-level load failure where quoin allocates a
document-level diagnostic, unnamed refusal codes), and an unstated bundle
index without which the golden fixtures cannot resolve `ConfigVersion`.

## Traceability

| US | FR/NFR | StR | Verification |
|----|--------|-----|--------------|
| US-019 (EX-1..4 -> TC-1610, TC-1612, TC-1613, TC-1600) | FR-069 (AC-1..9, CON-1..4) | StR-001 | TC-1599..TC-1609 |
| US-019 | FR-070 (AC-1..9, CON-1..3) | StR-001 via US-019 | TC-1610..TC-1621 |
| US-019 | FR-071 (AC-1..6, CON-1..2) | StR-001 via US-019 | TC-1622..TC-1629 |
| US-019 | FR-072 (AC-1..8, CON-1..3) | StR-001 via US-019 | TC-1630..TC-1640 |
| US-019 | NFR-021 (AC-1..4; constrains FR-069..FR-072) | StR-001 via US-019 | TC-1641..TC-1644 |

## Findings

| ID | Severity | Summary | Refs |
| ------- | -------- | -------------------------------- | ------ |
| FND-320 | medium | FR-069 Inputs pin the semantic-core bundle at `agent-ix/filament-core-data` `packages/semantic-core/schemas/` revision `d48b8da`; that revision ships the bundle at `packages/semantic-core/generated/json-schema/` with `toolchain.json` at `packages/semantic-core/generated/toolchain.json` (the path quoin's provenance records). FR-069-CON-2 and FR-069-AC-8 would record a path that does not exist at the pinned revision. | FR-069, FR-069-CON-2, FR-069-AC-8, TC-1606 |
| FND-321 | medium | FR-069 refuses a target "outside the vendored target registry" (AC-3 `targets: [go]`), but Inputs list only the module-manifest schema and the semantic-core bundle; the target registry (`filament-core-data` `schema/semantic/v1/common.schema.json`, which quoin vendors at `d48b8da`) is an unstated vendored input with no provenance or revision under CON-2. | FR-069, FR-069-AC-3, FR-069-CON-2 |
| FND-322 | medium | FR-069 Outputs promise a `semantic.*` reason code "for every refusal below", but no code is named for an unknown block key, an undeclared export, a bad `package`, a bad target, or the ambiguous `{ schema, digest, type }` form; TC-1601/TC-1602 can only assert prose. `ArchetypeLoadFailure` today is `{module, archetype, path, reason}` (src/error.rs), so whether the code is a new field or a `reason` prefix, and how a module-level refusal with no archetype is represented, are unstated. | FR-069, FR-069-AC-3, FR-069-AC-4, TC-1601, TC-1602 |
| FND-323 | medium | Allocation drift versus quoin FR-070: quoin states "Quire SHALL apply the same block schema at artifact-validation time and report an invalid block as a document-level diagnostic"; FR-069 instead fails the whole module at load, so no artifact of that module is validated at all. One of the two contracts must change or FR-069 must record the deviation. | FR-069, quoin FR-070 |
| FND-324 | medium | FR-070's closed `Constraints` keyword set admits the flag `identity` only; quoin FR-071 also admits the flag `nullable` mapping to `FieldDecl.nullable` (present in the vendored `FieldDecl.json`). Under FR-070 a `nullable` cell fails with `semantic.unknown-constraint-keyword`, contradicting the mapping contract. | FR-070, FR-070-AC-6, quoin FR-071 |
| FND-325 | medium | FR-070 `Type` resolution has no precedence order (quoin FR-071: kernel scalar, then bundle `id`, then exact `title`, then import), restricts enumerations to `id` only where quoin admits `id` or `title` for both objects and enumerations, and states no policy when two declarations share a title (quoin: fail naming both). A lookup over four sources with no tie-break has more than one valid interpretation. | FR-070, FR-070-AC-4, quoin FR-071 |
| FND-326 | medium | quoin FR-071 requires the semantic-core reader rules (bounds, flags only on collections, `decimal` iff `Decimal`, `unit` only on unit-allowed scalars, uniqueness by name, `identity` only on `1..1` non-`JsonObject`) reported at the row or fence-line locus. FR-070 carries only bare `Decimal` and the multiplicity flags; the remaining rules are schema `description` text, not schema constraints, so FR-070's "validates against `FieldDecl.json`" gate cannot catch a duplicate field name, `identity` on a collection, or `unit` on `String`, and no code is named for them. | FR-070, FR-070-AC-4, FR-070-AC-6, TC-1621 |
| FND-327 | medium | FR-070-AC-1/AC-2 and FR-071-AC-1 expect `parent | ConfigVersion` and `Returns: ConfigVersion[1]` to resolve to `ix://agent-ix/config-service/type/ConfigVersion`, but the golden artifact's `id` is `FR-006` and its `title` is `ConfigVersion Entity`, so under FR-070's own rules the token is unresolved. The module (`package: agent-ix/config-service`) and bundle index the golden cases run under are named nowhere in FR-070/FR-071 Inputs (quoin's only fixture module is `module-ok`, package `agent-ix/spec-objects-fixture`). | FR-070-AC-1, FR-070-AC-2, FR-071-AC-1, TC-1610, TC-1611, TC-1622 |
| FND-328 | medium | Internal contradiction on `not_applicable`: FR-072 defines it as "no `semantic` block, or no section", but FR-070-AC-9, FR-072-AC-3, and FR-069-CON-3 require an artifact of a module without a block to yield a record byte-identical to the pre-change extraction (no `semantic` key at all). The no-block condition therefore has two defined outcomes, and FR-072-AC-2's "fixtures cover `not_applicable` for each kind" can only be met by the no-section reading. | FR-072, FR-072-AC-2, FR-072-AC-3, FR-070-AC-9, FR-069-CON-3 |
| FND-329 | medium | FR-071 fixes `sourceSpan.endColumn` at 4 (closing line of exactly three backticks at column 1), while TC-1629 generates nested fences, whose outer fence must close with four or more backticks (end column 5+), and CRLF bodies. The Outputs rule and the matrix row contradict; the span must derive from the closing fence's actual length, which FR-071-CON-2 already implies. | FR-071, FR-071-CON-2, TC-1629 |
| FND-330 | medium | FR-071 states each `### <clauseId>` heading "SHALL own exactly one fenced block", yet also admits the external form `Clause: ./<file>.md#<id>` (only its duplicate-authority failure is specified). An external-only clause (heading plus `Clause:` line, no fence) has no defined `ClauseRef`, `sourceSpan`, or `clauseText` outcome, violates the exactly-one rule, and whether the referenced file is read (offline/corpus boundary, FR-070-CON-2, NFR-021-AC-2) is unstated. | FR-071, FR-071-AC-3, quoin FR-072 |
| FND-331 | low | FR-072's `missing` state hinges on "a section the module's `body_extraction` marks `required`"; `required` is a per-locator flag (FR-011), and the mapping from body_extraction locators to the three declaration kinds (`fields`, `clauses`, `operations`) is unstated, so `missing` versus `not_applicable` is not decidable from the spec. | FR-072, FR-072-AC-2, FR-011 |
| FND-332 | low | FR-071 `pre`/`post` entries are "`ClauseRef` entries without a span", but `ClauseRef.json` requires `language`; that it is copied from the referenced clause (as the fixture shows) is unstated. `Pre: <clauseId>` is singular where quoin FR-072 says "lines listing clause ids", and `OperationDecl.json` requires `params` while FR-071 makes the table optional without stating the empty-array outcome. | FR-071, FR-071-AC-1, FR-071-AC-5 |
| FND-333 | low | FR-070's multiplicity grammar enumerates `1`, empty, `0..1`, `a..b`, `1..*` but not the general `n..*` that quoin FR-071 admits (`0..*`), and omits quoin's rule that backticks around a cell value are stripped. | FR-070, FR-070-AC-5, quoin FR-071 |
| FND-334 | low | FR-069's `SemanticModule` record lists eight keys and silently drops `mappings` and `sweep_report`, two keys the vendored block schema admits; whether they are retained, ignored, or rejected is unstated. | FR-069, quoin FR-070 |
| FND-335 | low | FR-070 Inputs scope the vendored fixtures to `tests/fixtures/semantic-module/mapping/`, but AC-8 and `legacy.expected.json` depend on the pinned FR-006 copy under `../corpus/config-service/` (pinned by its own `PROVENANCE.json`), which is not an FR-070 input. | FR-070, FR-070-AC-8, TC-1617 |
| FND-336 | low | Frontmatter `relationships` diverge from the prose Dependencies: FR-069 omits FR-067; FR-070 omits FR-025 and FR-011; FR-071 omits FR-005; FR-072 omits FR-069, FR-070, FR-071, and FR-055. Graph consumers see a different upstream set than readers. | FR-069, FR-070, FR-071, FR-072 |
| FND-337 | low | FR-072 emits `advisory` into the existing `CoreExtractionResult.diagnostics.severity` field beside `info`/`warning`/`error`; FR-072-CON-1 states "every addition is a new optional key", but this widens an existing field's value set. Separately, FR-072 says the schema's `format_version` is `1` while the record field list omits `format_version`, so whether the record carries it is ambiguous. | FR-072, FR-072-CON-1, FR-072-AC-8 |
| FND-338 | low | FR-071 fixes `sourceIdentity` as `ix://<org>/<repo>/spec` and `path` as corpus-relative without stating where `<org>/<repo>` comes from (`semantic.package`?) or what `path` is for the single-document `extract_semantic(document, context)` call with no corpus. FR-071-AC-6 is verified by `Inspection` yet bundles a runtime round-trip assertion (TC-1627 Static, TC-1629 Property). | FR-071, FR-071-AC-6, FR-072 |

## Failure Domain Check

- Extension failures: covered for unsupported contract/semantic-core versions
  (FR-069-AC-2) and unknown block keys (AC-3); refusal codes are incomplete
  (FND-322).
- Identity keys: the placeholder `unresolved/<Token>` identity is specified;
  the resolution precedence and shared-title collision are not (FND-325,
  FND-327).
- Evaluation purity: NFR-021 and FR-070/071 constraints exclude parsing,
  network, and rendering; the external `Clause:` reference is the one path
  whose read boundary is unstated (FND-330).
- Topological robustness: `$ref` cycles and self-`$id` fragments are covered
  (FR-069-AC-5); nested-fence spans are not (FND-329).

## Dispositions (applied 2026-09-03, same branch, before Plan-003)

| ID | Disposition |
| --- | --- |
| FND-320 | Fixed — path `packages/semantic-core/generated/json-schema/` and `generated/toolchain.json`, digest constant recorded. |
| FND-321 | Fixed — `common.schema.json` vendored with provenance under `schemas/vendored/`. |
| FND-322 | Fixed — every refusal has a code; shape is one `ArchetypeLoadFailure` per object type with `reason` prefixed by the code. |
| FND-323 | Accepted, recorded — allocation note in FR-069 Behavior; load-time refusal is the stricter reading; Quoin wording to reconcile. |
| FND-324 | Fixed — `nullable` flag admitted. |
| FND-325 | Fixed — precedence, enumeration by id or names, `semantic.ambiguous-type`. |
| FND-326 | Fixed — reader rules carried (see SR-069 FND-308). |
| FND-327 | Fixed — golden cases run under a quire-rs-authored `config-version.bundle.json`; corpus-mode names are id, title, frontmatter `name` (flagged to module authoring, spec-objects-business#4). |
| FND-328 | Fixed — no block ⇒ no `semantic` key; `not_applicable` only with a block present. |
| FND-329 | Fixed — `endColumn` = one past the closing fence line length. |
| FND-330 | Fixed — external-only `Clause:` is `semantic.clause-external-unsupported`, file never read. |
| FND-331 | Fixed — `missing` keyed to the section's `body_extraction` `required` locator. |
| FND-332 | Fixed — pre/post copy the referenced clause's language; `params: []` when no table; `Pre:` lists. |
| FND-333 | Fixed — `n..*`, `*`, backtick stripping. |
| FND-334 | Fixed — `mappings`/`sweep_report` accepted and ignored, stated. |
| FND-335 | Fixed — corpus copy listed as an input. |
| FND-336 | Fixed — frontmatter `requires` edges now match prose dependencies. |
| FND-337 | Fixed — severity set unchanged on existing contracts; `formatVersion: 1` in the record. |
| FND-338 | Fixed — see SR-068 FND-280; FR-071-AC-6 is now a Test. |
