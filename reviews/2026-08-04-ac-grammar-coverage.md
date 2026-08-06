---
id: SR-002
title: "Base review — AC grammar + declarative traceability coverage (FR-047..FR-051, US-017, ADR-0010)"
type: SpecReview
analysis: base
scope: "spec/functional/FR-047-acceptance-criteria-grammar.md, spec/functional/FR-048-per-check-grammar-severity.md, spec/functional/FR-049-verification-reference-integrity.md, spec/functional/FR-050-declarative-coverage-computation.md, spec/functional/FR-051-source-symbol-extraction.md, spec/usecase/US-017-agent-verifies-coverage.md, spec/assets/adr/0010-smt-consistency-analysis.md, spec/tests.md"
review_set: base
relationships:
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-047", type: reviews }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-048", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-049", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-050", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/usecase/US-017", type: references }
  - { target: "ix://agent-ix/quire-rs/spec/tests", type: references }
---

## Summary

Base-checklist review of the AC-grammar/traceability-coverage slice added on
`spec/ac-grammar-coverage` (FR-047..FR-051, US-017, ADR-0010) and its
consistency with the existing grammar/lexicon/bundle machinery
(FR-042/043/044, FR-045/046, FR-036, FR-038). The slice is structurally
clean: IDs are well-formed and sequential (FR-047..051, US-017, ADR-0010,
47 new ACs, TC-707..753 with no gaps or duplicates), every AC has exactly one
TC in `spec/tests.md` (435/435 rollup verified), all internal links resolve,
and `quire validate` emits zero findings against the new artifacts (the only
grammar warnings and structural failures in the corpus are pre-existing on
main). One clear-cut cross-reference gap was fixed in place (FND-001); the
remaining four findings were initially recorded open (pre-existing artifacts /
substantive design questions), then fixed on 2026-08-04 under explicit user
authorization — see the RESOLVED annotations per finding. Post-fix state:
FR-045-CON-4 aligned with code via CR-012, FR-042 non-goals reworded,
external deliverables tracked as EXT entries in Plan-001, and FR-047's
observable-verb vocabulary made module data (AC-12/TC-757; matrix 438/438).

## Verdict

**PASS** (open notes since resolved under user authorization, 2026-08-04) —
the new slice meets the base checklist: clear
EARS-shaped normative statements, testable ACs with unique TC backing under
all six coverage rules, constraints with rationale/validation, dependencies
declared upstream/downstream, and house-style US structure (illustrative
examples + contextual dependencies, `traces_to` StR-006). ADR-0010 is
correctly `Proposed` with no decision and imposes no requirements. Open
findings below are advisory for follow-up, not blockers for this spec slice.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | low      | FIXED — FR-047 frontmatter carried a `requires` edge to FR-043 but not FR-044, while its `vague-response` check normatively consumes the merged module lexicon *and* the project glossary "exactly as the EARS check does". Added the symmetric `requires` FR-044 edge. | FR-047, FR-044 |
| FND-002 | low      | RESOLVED (2026-08-04, user-authorized) — FR-045-CON-4 reworded via CR-012: the org segment is caller-supplied (`FilamentExtractionInput.org`, no engine default, missing org is an input error), matching PR #15's shipped `normalize_ref(org, repo_name, value)` behavior (verified against `src/filament.rs`, incl. `org_qualifies_refs_from_input_not_a_literal`). | FR-045-CON-4, FR-051 |
| FND-003 | low      | RESOLVED (2026-08-04, user-authorized) — FR-042's "Non-goals (v1)" bullet minimally reworded: the stale "`GWT` (acceptance criteria)… later" anticipation now points at FR-047's realized `ac` grammar with EARS (not GWT) as the canonical AC shape; a `US` story grammar remains future work. No restructuring of FR-042. | FR-042, FR-047-AC-10 |
| FND-004 | medium   | RESOLVED (2026-08-04, user-authorized) — external deliverables now have tracked entries EXT-1..EXT-4c in Plan-001's plan.md with owning-repo attribution (`spec-artifacts-iso`, `spec-artifacts-process`, `quire-cli`, three companion marker packages) and explicit blocking relationships to the tasks they gate (Task-003/007/008/009/010; EXT-4b is a hard Task-010 prerequisite), plus a Plan-001 log entry. `spec/assets/external-blockers.md` untouched (untyped). | FR-049, FR-050, FR-051 |
| FND-005 | medium   | RESOLVED (2026-08-04, user-authorized) — FR-047's observable-verb vocabulary reworked to module data per ADR-0009: the engine ships built-in defaults, modules extend/override via an `observable_verbs` manifest registry merged first-wins (FR-043 `lexicon` pattern). Added FR-047-AC-12 + TC-757; matrix 438/438. | FR-047-AC-12, FR-043, ADR-0009 |

## Coverage

- **Coverage rule (AC→TC):** all 47 new ACs (FR-047-AC-1..10, FR-048-AC-1..9,
  FR-049-AC-1..8, FR-050-AC-1..9, FR-051-AC-1..11) map 1:1 to TC-707..753 in
  the matrix and the trace table; rollup states 435/435 (100%).
- **Option permutation:** severity levels (`off`/`warning`/`error`) ×
  source (manifest/CLI) covered by TC-716..720, TC-752; shape classes
  (`ears`/`given-when-then`/`unclassifiable`) by TC-707, TC-751.
- **Constraint boundary:** FR-047-CON-1 (no default `error` promotion) is
  Inspection-typed; FR-050-CON-2 and FR-051-CON-1/2 have Test-typed backing
  (TC-738, TC-749/750); FR-051-CON-3 sunset is Inspection-gated on user
  sign-off — consistent with the repo's Inspection-TC convention.
- **Error path:** malformed manifest sections (TC-723, TC-733), unparseable
  files (TC-749), missing model diagnostics (TC-729, TC-740), dangling
  references (TC-725) all covered.
- **State transition:** N/A — the slice is stateless analysis; determinism
  covered as Property TCs (TC-731, TC-738, TC-750).
- **Edge cases:** empty cells (TC-708), multi-annotation cells (TC-730),
  duplicate trace attachment (TC-746), positive/negative pair idiom (TC-709),
  reformat-stability (TC-742) covered.
