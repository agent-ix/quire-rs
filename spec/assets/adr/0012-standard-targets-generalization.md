---
id: ADR-0012
title: "Standard targets, generalized: ladders, subjects, obligation styles, crosswalks, identity, rights"
type: ADR
---

# ADR 0012: Standard targets, generalized — ladders, subjects, obligation styles, crosswalks, identity, rights

**Status**: Proposed
**Date**: 2026-08-28
**Decision authority**: kreneskyp

## Context

EPIC #368 introduces external standards as first-class verification targets: a
`ClauseSet` artifact kind holding the requirement inventory of a standard
(NPR 7150.2D first), per-project applicability derivation, and discharge
tracking against the evidence store. The cross-industry landscape research
(`~/dev_bak/filament-research/standards-landscape-part2.md` §5) shows that
every regulated vertical is the same machine with different labels — graded
criticality ladders, subject-scoped obligations, tailoring, crosswalks between
regimes. Hard-coding the NASA pair (Class ladder + SWE rows) would be the one
design decision that gets too narrow, and cheap-now/expensive-to-retrofit is
exactly the ADR-0009 situation: concrete vocabulary belongs in module data,
the engine stays generic.

Two data spikes ground this ADR in real source material rather than
speculation:

- **RMM ingest spike (#369)**: the three LaRC NPR 7150.2D templates parse into
  101 clauses. Applicability is *not* boolean — 15 of 101 rows carry
  predicates (`Safety-Critical` ×6, `A, B` ×5, `A or B and per SWE-131
  criteria` ×3, `Per SWE-143 criteria` ×1); the class ladder is encoded as
  strict row-presence nesting (E ⊂ D ⊂ ABC, no exceptions); one row carries a
  compound identity (`SWE-174/LSWE-031`); 8 rows ship with locked `FC`
  compliance prefills; the Technical Authority column is a tailoring-approval
  role vocabulary (`EngTA`, `EngTA and SMA TA`, `EngTA and CIO TA`,
  `LMS Waiver`), and the templates mix a center-local overlay (LSWE rows from
  LPR 7150.2B) into the NPR base.
- **EARM schema findings (engineering-assurance#10)**: the ECSS DOORS export
  makes normative force a row-level enum (`Requirement` / `Recommendation` /
  `Permission`) that is authoritative over the text — 661 Requirements contain
  no modal verb a regex could classify; identity is a revision-independent
  PUID with a per-requirement version counter (`RCM Version`), a change-status
  enum, and `<<deleted>>` tombstones (515 rows) so an ID can never be silently
  reused; DRD provisions are ordinary rows whose invocation edge exists only
  in prose; per-project tailoring is an empty-column overlay contract
  (`A/M/D/N` + replacement text + justification, keyed by PUID).

The decisions below fix the generic mechanism before any spec (#371–#373) or
engine (#374/#375) work.

## Decisions

### 1. Criticality ladders are declared per clause set, never hard-coded

A `ClauseSet` declares its own `criticality_dimension`:

```yaml
criticality_dimension:
  name: nasa-software-class      # dimension identity, versioned with the set
  ordered_levels: [E, D, C, B, A]   # ascending rigor
  derivation_inputs:             # what a project answers to derive its level
    - human-rating
    - safety-critical
    - mission-criticality
```

Applicability expressions reference declared levels (`level >= D`,
`level in [A, B]`) plus subject attributes and cross-clause criteria
references — a small predicate language, not a boolean column. The RMM data
makes the boolean alternative untenable: it lies about 15 of 101 rows.

DAL (DO-178C), ASIL (ISO 26262), SIL (IEC 61508), TQL (DO-330), and NASA
Class are all instances of this one machine; NASA Class is instance #1. A
source that publishes coarser artifacts than its ladder (the ABC template
carries A, B, C undifferentiated, then re-splits rows with `A, B` predicates)
is representable because level sets in expressions are independent of how the
source files happen to be batched.

### 2. Every clause-set item carries a `subject_kind`

Enum: `project | organization | tool | model | data | operational-process`.
Default `project`. The subject kind determines what can discharge the item:
tool items are discharged by tool-accreditation evidence (the SWE-136 shape),
organization items by management-system evidence, model/data items by the
AI-profile evidence classes (engineering-assurance#8/#14), operational-process
items by standing-capability evidence (Decision 3). Discharge tracking must
refuse category errors — a passing unit-test run cannot discharge an
organization-scoped item.

### 3. Obligation styles are typed, including operational-with-clock

Enum per item: `product | process | document | management-system |
operational-with-clock`.

- `document` reifies what ECSS leaves in prose: the item carries a typed
  deliverable edge (`mandates_document` → a document archetype), so "the VP
  shall conform to the DRD in Annex A" becomes machine-checkable against
  `body_extraction`-style archetypes instead of a sentence. This exceeds the
  source — the EARM itself has no typed requirement→DRD link — and that is
  deliberate.
- `operational-with-clock` carries clock parameters (e.g. the CRA 24 h/72 h
  reporting shape) and is dischargeable only by standing-capability evidence
  plus exercise records, never by a one-time test. The evidence records for
  this style are EPIC quoin#267's scope; this ADR reserves the schema slot
  only.

### 4. Normative force is authored metadata, authoritative over text

Per item: `normative_force: requirement | recommendation | permission`. The
EARM's 661 no-modal Requirements prove force cannot be derived from text by
regex — force is data, text is evidence. Consequence for existing lints: the
shall-language lints keep applying to natively authored FR/NFR/StR artifacts;
imported clause-set items are governed by their authored force field, and a
faithful import of a Recommendation must not be flagged as a defect.

### 5. Crosswalk edges propagate discharge, visibly

Item-level edge `satisfies_also` with `strength: equivalent | partial |
informative` from an item in one clause set to an item in another.
Crosswalk-propagated discharge always renders distinct from direct discharge —
an auditor must be able to see that CRA item X is claimed via SSDF item Y plus
a published mapping, never a silently merged checkmark. Published mappings
(61508↔26262, 42001↔AI-RMF↔AI-Act, SSDF↔CRA) are imported as data with their
own provenance, not authored ad hoc. Exact-alignment customers (NASA/ESA) pin
one clause set and ignore crosswalks; everyone else gets multi-regime reuse.

### 6. Identity survives renumbering; the ladder-scheme change is versioned, not renamed

- `stable_id` is the identity (SWE number, ECSS PUID); the clause number is a
  presentation alias and may change freely between issues of the standard.
- Items carry a per-item `version` counter and `change_status`
  (`created | unchanged | normative-change | informative-change | deleted`),
  mirroring the EARM's RCM machinery — the only published model we found that
  gets multi-issue identity right.
- Deletion is a tombstone: the item and its stable_id persist with
  `change_status: deleted`, so an ID cannot be silently reused and a
  disappearance cannot hide.
- Compound source rows (`SWE-174/LSWE-031`) become one item whose
  `component_ids` list the aliases; two clause sets may also each carry an
  item joined by a crosswalk edge, but one source row never silently becomes
  two obligations.
- The hard case is a ladder-scheme change (IEC 62304 Ed 2: 3 classes → 2
  rigor levels). Decision: a ladder scheme is versioned with the clause set;
  a scheme change is a new `criticality_dimension` version plus crosswalk
  edges between old-level and new-level applicability, never an in-place
  rename. Diff tooling reports it as a scheme migration, not as thousands of
  per-item applicability changes.

### 7. Rights model: skeleton-not-text is the default

Module-level `rights: text-shippable | skeleton-only`; item `text` is
optional and populated only where rights allow. US-government works ship with
text (NPR 7150.2D and the NIST SSDF family, per 17 USC §105 — confirmation
questions Q8–Q13 in engineering-assurance's counsel package); ISO/IEC/SAE/
RTCA/ECSS sets ship skeleton-only (identifiers, force, applicability,
edges), with requirement text resolved at the customer's site from the
customer's own licensed copy, joined by `stable_id`. ECSS content of any kind
additionally waits on counsel (ECSS-P-00C Rev.1 clause 5.8; the EARM
click-through and EU database-right questions Q1–Q6). No ECSS-derived file or
text extract enters any repo before that answer.

## Consequences

- Unblocks #371–#373 (ClauseSet spec work) and, after stabilization exits,
  #374/#375 (engine) and quoin#272/#273 — all of which now target the generic
  shapes above with NASA as instance #1 and SSDF as the planned cheap
  end-to-end proof (free text, stable IDs, flat applicability).
- The engine stays generic per ADR-0009: ladders, subject kinds, obligation
  styles, and force vocabularies are clause-set/module data; the engine
  implements the predicate evaluation, discharge-category checks, crosswalk
  rendering, and diff semantics.
- Tailoring (per-project A/M/D/N-style verdicts with justification and
  role-gated approval, per both the RMM compliance workflow and the ECSS
  overlay contract) is confirmed as a first-class artifact class, but its
  schema is deliberately out of scope here — it composes with everything
  above and is specced with the ClauseSet artifacts.
- Rejected alternatives: hard-coded NASA classes (retrofit cost across every
  later vertical); boolean applicability (falsified by the RMM data);
  text-derived normative force (falsified by the EARM data); silent crosswalk
  discharge (audit-illegible).

## References

- EPIC #368; spike #369 (findings comment and
  `~/dev_bak/filament-research/standards-data/derived/`); EARM findings:
  engineering-assurance `docs/earm-schema-findings.md`; counsel package:
  engineering-assurance `docs/counsel-question-package.md`; landscape:
  `~/dev_bak/filament-research/standards-landscape-part2.md` §5.
- [ADR 0009](./0009-concrete-vocabulary-is-module-data.md) — concrete
  vocabulary is module data.
