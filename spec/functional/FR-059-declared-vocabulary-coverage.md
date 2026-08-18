---
id: FR-059
title: "Declared-Vocabulary Coverage"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-057"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "implements"
    cardinality: "1:1"
---
# FR-059: Declared-Vocabulary Coverage

## Description

A spec bundle can be **100% acceptance-criterion covered and still carry no requirement anywhere**
for reliability, security, or maintainability. Every document is individually well-formed; what is
wrong is the *set*. No per-document check can see it, and neither can downward coverage, which
measures whether what was written is verified rather than whether anything was written.

The check that answers it is **not 25010-specific**. It is a generic primitive nothing in the
engine provides:

> Given a declared vocabulary and a declared projection from documents onto it, which vocabulary
> values does no document claim?

ISO 25010 quality characteristics are one instance. Test-type coverage over a Test Matrix and
STRIDE-category coverage over declared threats are others, and all three are the same walk with
different declarations.

### The vocabulary is read, never authored

`quire-rs` SHALL read the vocabulary from the **frontmatter schema of the archetype being
projected**, at the `enum` declared for the named field. A module names an archetype and a field;
it does not restate the values.

This is the load-bearing decision. `agent-ix/quire-rs#162` was filed against a scope that proposed
walking a hardcoded nine-item list; the vocabulary already exists as module data, and it has
**twelve** values, not nine
(`spec-artifacts-iso/spec_artifacts_iso/schemas/nfr-frontmatter.schema.json`). A second list in the
manifest would be free to drift from the first — exactly the defect CR-015 closed.

### Justified absence is an answer, not a gap

A value a bundle deliberately does not address SHALL be recorded, and a recorded value counts as
**covered**. "This is a CLI that controls no physical process, so it has no safety characteristic"
is a real answer, and a check that cannot accept one forces the author into a false finding they
cannot clear or a fabricated requirement written to silence it.

The record lives in a module-declared frontmatter field on **any document in the bundle**, not only
on the archetype being counted. "This product has no safety characteristic" is a statement about
the product; its natural home is the spec or master-requirements document. Requiring it on an NFR
would mean authoring an NFR to say an NFR is unnecessary.

### An empty projection is one fact

When **no document of the projected archetype exists at all**, the finding SHALL be a single
statement that nothing projects onto the vocabulary — not one finding per declared value.

This came out of the fit-check rather than out of design. Over 243 `~/dev` bundles, **90 carry no
NFR document at all**, and reporting each of the twelve characteristics as unowned turned that one
fact into **1080 of the sweep's 2792 findings** — every one of them saying "no document claims
security", "no document claims safety", and so on, when what is true and actionable is that nothing
in the bundle projects onto the vocabulary at all.

**This is not a widening to lower a count.** No bundle that was reported stops being reported; one
specific statement replaces twelve vaguer ones, and the number of unowned values is still named in
the message. A check should make the most specific true statement it can.

### What the fit-check measured

**[RAN]** over 243 `~/dev` bundles with a purpose-built module whose only declaration is this one,
so every finding is attributable to this check (the ticket's *one check per sweep* requirement):

| Population | Repos | What the finding means there |
|---|---|---|
| No NFR document at all | 90 | The projection is empty — one finding each after the collapse above |
| NFRs present, none labelled | 104 | **Labelling debt**: the requirements exist and do not carry the field |
| NFRs present, ≥1 labelled | 49 | The check doing what it was designed to do |

Across the corpus, **285 of 689 NFR documents (41.4%) carry `quality_attribute`, and 0 of 2474 FR
documents do.** Eleven of the twelve declared values are claimed somewhere; **`safety` is claimed by
no document in the ecosystem**. One document claims `interoperability`, which is not in the
vocabulary — already caught today by frontmatter schema validation, so this FR adds nothing there.

**On the corpus as it stands this check therefore reports labelling debt more often than quality
gaps, and the FR says so rather than leaving a reader to infer it.** That is a real finding and it
lands: the specifications *should* say which characteristic each constraint is about. It is also
why findings ship advisory and severity is settable per repository.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-059-AC-1 | A declared value no document claims is reported, naming the value and the declaration; a claimed value is not. | Test (TC-911) |
| FR-059-AC-2 | The vocabulary is read from the projected archetype's frontmatter schema `enum`. A manifest that restates the values is not how the engine learns them. | Test (TC-912) |
| FR-059-AC-3 | A value named in the declared justified-absence field counts as covered. | Test (TC-913) |
| FR-059-AC-4 | The justification may live on **any** document in the bundle, not only on the projected archetype. | Test (TC-914) |
| FR-059-AC-5 | Findings carry their own `trace:<check>` severity key: advisory by default, droppable with `off`, promotable with `error`, independent of every other corpus check ([FR-057](./FR-057-corpus-check-severity.md)). | Test (TC-915) |
| FR-059-AC-6 | A module declaring no `vocabulary_coverage` produces byte-identical output to one that never heard of this FR. | Test (TC-917) |
| FR-059-AC-7 | A declaration whose archetype or field yields no `enum` reports **itself** under `trace:undeclared-coverage-vocabulary`, rather than silently reporting no unowned value. | Test (TC-916) |
| FR-059-AC-8 | When no document of the projected archetype exists, the finding is a single statement that nothing projects, naming how many values are unowned — not one finding per value. | Test (TC-918) |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-059-CON-1 | The engine SHALL contain no vocabulary value, no field name, and no archetype name belonging to any particular vocabulary. A second vocabulary is a manifest entry, never a second check. | Design | Inspection |
| FR-059-CON-2 | The **verdict** on a justified-absence record — whether the stated reason is acceptable, and any hard-100% policy — stays in `agent-ix/quoin#81`. This FR computes coverage and records what the bundle says; it does not judge it. | Design | Inspection |
| FR-059-CON-3 | Judging requirement *quality* per characteristic is [FR-056](./FR-056-requirement-quality-lints.md)'s lane, not this one. | Design | Inspection |
| FR-059-CON-4 | Promotion of `trace:unowned-*` from `warning` to `error` requires a corpus sweep and explicit user sign-off, as every check before it. The fit-check above is that sweep's baseline, not its authorisation. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-057](./FR-057-corpus-check-severity.md) (per-check severity), [FR-050](./FR-050-declarative-coverage-computation.md) (the declarative-model precedent)
- **Downstream**: `agent-ix/quoin#81` consumes the coverage and owns the verdict policy
