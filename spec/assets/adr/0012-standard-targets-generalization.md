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
summarized on #370 shows a shared mechanism across regulated verticals —
classifications, subject-scoped obligations, tailoring, and crosswalks between
regimes — but not one universal total order. Hard-coding the NASA pair
(classification + SWE rows) would be the
one design decision that gets too narrow, and cheap-now/expensive-to-retrofit
is exactly the ADR-0009 situation: concrete vocabulary and topology belong in
module data, while the engine stays generic.

Two data spikes ground this ADR in real source material rather than
speculation:

- **RMM ingest spike (#369)**: the three LaRC NPR 7150.2D templates parse into
  101 clauses. Applicability is *not* boolean — 15 of 101 rows carry
  predicates (`Safety-Critical` ×6, `A, B` ×5, `A or B and per SWE-131
  criteria` ×3, `Per SWE-143 criteria` ×1); the class ladder is encoded as
  strict row-presence nesting (E ⊂ D ⊂ ABC, no exceptions); one row carries a
  pair of co-listed identifiers (`SWE-174/LSWE-031`); 8 rows ship with locked
  `FC`
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

### 1. Classification topology is declared per clause set, never hard-coded

A `ClauseSet` declares its own `classification_dimension`: a value set plus an
optional partial order. An absent order edge means incomparable, not lower.
NASA demonstrates why this must be more general than an ordered list: Classes
A through E form the engineering-software rigor chain, while Class F is the
separate Business/IT branch and has its own applicability and authority column.

```yaml
classification_dimension:
  name: nasa-software-class       # identity, versioned with the clause set
  values: [A, B, C, D, E, F]
  order_edges:                    # lower -> higher rigor; F is incomparable
    - [E, D]
    - [D, C]
    - [C, B]
    - [B, A]
  derivation_inputs:
    - system-usage
    - mission-criticality
    - human-dependence
    - developmental-and-operational-complexity
    - agency-investment
```

Applicability expressions reference declared values (`class in [A, B]`) and
may use order comparisons (`class >= D`) only when the dimension declares the
compared values as ordered. They additionally reference subject attributes and
cross-clause criteria — a small predicate language, not a boolean column. An
unknown input or incomparable value produces unresolved applicability, never
`false`. The RMM data makes the boolean alternative untenable: it lies about
15 of 101 rows.

DAL (DO-178C), ASIL (ISO 26262), SIL (IEC 61508), and TQL (DO-330) are total
orders representable by this model; NASA Class is the first partial-order
instance. A source that publishes coarser artifacts than its classification
(the ABC template carries A, B, C undifferentiated, then re-splits rows with
`A, B` predicates) is representable because value sets in expressions are
independent of how the source files happen to be batched.

### 2. Assurance subjects and obligated parties are distinct

Every item carries a non-empty `assurance_subject_kinds` set drawn from
`project | organization | tool | model | data | operational-process`; the
default is `[project]`. This names what the evidence is about, not the actor
grammatically obligated by the clause. Obligated actors and approval roles are
separate fields. SWE-136, for example, obligates a project manager to establish
evidence about a tool; calling the manager and tool one `subject_kind` would
conflate two independently queried facts.

The assurance subject determines what can discharge the item: tool subjects
need tool-accreditation evidence, organization subjects need management-system
evidence, model/data subjects use the AI-profile evidence classes
(engineering-assurance#8/#14), and operational-process subjects need
standing-capability evidence (Decision 3). Discharge tracking must refuse
category errors — a passing unit-test run cannot discharge an
organization-scoped item. Multiple subjects require compatible evidence for
each unless tailoring explicitly narrows the item.

### 3. Obligation styles are typed, including operational-with-clock

A non-empty set per item: `product | process | document | management-system |
operational-with-clock`. Imported provisions commonly combine styles (for
example, develop a plan and continuously execute it); the importer may split
them only when the source supplies stable sub-item identities. Otherwise the
one stable item retains multiple styles rather than choosing one and lying.

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

### 5. Crosswalk strength has explicit, asymmetric discharge semantics

Item-level edge `satisfies_also` with `strength: equivalent | partial |
informative` is directed from the source item whose evidence may be credited to
the target item:

- `equivalent` may propagate discharge source → target, pinned to the exact
  source/target clause-set versions and rendered as propagated rather than
  direct. Reverse propagation requires a reverse edge or an explicitly
  symmetric published mapping.
- `partial` contributes visible support but never fully discharges the target;
  the target remains open with residual scope until that remainder is directly
  discharged or explicitly accepted as risk.
- `informative` is navigation/context only and never affects discharge.

Every edge carries mapping provenance and version. Published mappings
(61508↔26262, 42001↔AI-RMF↔AI-Act, SSDF↔CRA) are imported as data, not authored
ad hoc. Exact-alignment customers (NASA/ESA) pin one clause set and ignore
crosswalks; everyone else gets multi-regime reuse without a silently merged
checkmark.

### 6. Identity survives renumbering; classification changes are versioned, not renamed

- `stable_id` is the identity (SWE number, ECSS PUID); the clause number is a
  presentation alias and may change freely between issues of the standard.
- Items carry a per-item `version` counter and `change_status`
  (`created | unchanged | normative-change | informative-change | deleted`),
  mirroring the EARM's RCM machinery — the only published model we found that
  gets multi-issue identity right.
- Deletion is a tombstone: the item and its stable_id persist with
  `change_status: deleted`, so an ID cannot be silently reused and a
  disappearance cannot hide.
- Stable identity is authority-scoped. Compound source rows
  (`SWE-174/LSWE-031`) become two items — one NPR item and one LaRC-overlay
  item — because their authorities may revise, tailor, or delete them
  independently. Import provenance records that one RMM row co-listed them;
  co-listing does not itself assert equivalence or propagate discharge. Aliases
  are reserved for presentation renumbering of one authority-owned identity.
- The hard case is a ladder-scheme change (IEC 62304 Ed 2: 3 classes → 2
  rigor levels). Decision: a ladder scheme is versioned with the clause set;
  a scheme change is a new `classification_dimension` version plus a distinct
  classification-value mapping (`classification_value_maps_to`) between old
  and new values, never an in-place rename. Clause-level `satisfies_also` edges
  do not map classification values. Diff tooling reports the dimension change
  as a scheme migration, not as thousands of unexplained per-item changes.

### 7. Rights model distinguishes text, structural data, basis, and jurisdiction

Rights are not a binary assertion that a skeleton is necessarily distributable:

```yaml
rights:
  text: shippable | customer-resolved | prohibited | pending-counsel | research-only
  structural_data: shippable | prohibited | pending-counsel | research-only
  basis: <license, statute, permission, or counsel-decision reference>
  jurisdictions: [US]            # empty/absent means not yet established
```

Item `text` is optional and populated only when `rights.text` permits the
distribution being built. NIST SP 800 publications may use their explicit US
no-copyright statement as a domestic basis. NPR 7150.2D remains
`pending-counsel` until engineering-assurance Q8–Q13 are answered; §105 is a
working hypothesis, not a shipping decision. ISO/IEC/SAE/RTCA content defaults
to `customer-resolved` text, while structural-data redistribution requires its
own basis rather than inheriting permission from text omission. ECSS text and
structural data remain `pending-counsel` under Q1–Q6; local research copies are
`research-only` and excluded from distributable modules.

## Consequences

- Unblocks #371–#373 (ClauseSet spec work) and, after stabilization exits,
  #374/#375 (engine) and quoin#272/#273 — all of which now target the generic
  shapes above with NASA as instance #1 and SSDF as the planned cheap
  end-to-end proof (free text, stable IDs, flat applicability).
- The engine stays generic per ADR-0009: classification topology, assurance
  subjects, obligation styles, and force vocabularies are clause-set/module
  data; the engine implements predicate evaluation, discharge-category checks,
  strength-aware crosswalk rendering, and diff semantics.
- Tailoring (per-project A/M/D/N-style verdicts with justification and
  role-gated approval, per both the RMM compliance workflow and the ECSS
  overlay contract) is confirmed as a first-class artifact class, but its
  schema is deliberately out of scope here — it composes with everything
  above and is specced with the ClauseSet artifacts.
- Rejected alternatives: hard-coded NASA classes (retrofit cost across every
  later vertical); a universal total-order ladder (cannot represent NASA Class
  F); boolean applicability (falsified by the RMM data); text-derived normative
  force (falsified by the EARM data); cross-authority compound aliases (destroy
  independent lifecycle identity); silent or strength-blind crosswalk discharge
  (audit-illegible); binary rights (cannot represent the counsel-pending state).

## References

- EPIC #368; spike #369 (findings comment); EARM findings:
  engineering-assurance `docs/earm-schema-findings.md`; counsel package:
  engineering-assurance `docs/counsel-question-package.md`; landscape findings:
  #370.
- [ADR 0009](./0009-concrete-vocabulary-is-module-data.md) — concrete
  vocabulary is module data.
