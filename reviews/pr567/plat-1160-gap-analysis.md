---
id: SR-20261102118106
title: PLAT-1160 gap-analysis frozen core review
type: SpecReview
analysis: gap-analysis
scope: agent-ix/quire-rs; PLAT-1160; PR 567; src/evidence.rs, src/coverage.rs, src/coverage/reconcile.rs,
  src/obligation/inventory.rs, src/corpus/trace_refs.rs, tests/coverage_matrix.rs,
  tests/output_contract.rs, schemas/output/coverage-v1.schema.json, README.md
review_set: subset
relationships:
- target: ix://agent-ix/quire-rs/FR-078
  type: references
---

## Summary

Ticket: PLAT-1160. Independent review of frozen 7d9b4cd497e904340562fe75a3cd4d1fb2c720c5 against 140ef204. This is an owner-neutral low-level engine stage, not a qualification certificate or completion of FR-078/PLAT-1160. Quoin FR-116 was read at 90edff33068a5398e54e4efc272e4276eec0ffe6; the actual qualifier remains absent and its owner assignment is pending.

## Verdict

**FAIL** — Resolve the method findings below; do not treat component-only inputs as completed owner qualification.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The new component tests do not exercise criterion-claim lookup refusals for a foreign selection, same display ID in another document/source, or ambiguous authored identity. The declared_target missing-document assertion tests only noncriterion UnknownIdentity; replaying owner CauseCode values does not execute inventory.lookup refusal branches. Add negative/control cases through compute_with_evidence with actual native identities. | tests/coverage_matrix.rs:685 | correct-requirement-no-evidence |
| FND-002 | medium | All new criterion claims use authored ID rows. No evidence-mode test supplies RenderedRow or CombinatorialTable identity, or reorders rendered rows and attempts an old claim. Existing inventory-only tests cannot detect a regression in this new coverage/trace integration; add native-origin and stale-identity controls without inventing qualification. | tests/coverage_matrix.rs:685 | correct-requirement-no-evidence |
| FND-003 | medium | No new test calls TraceQuery::Record. The tests exercise forward target and run lookup only, so exact record/run/entry inverse matching can regress while every test stays green. Exercise multiple entries sharing a run and prove the original complete locator returns only its matching claims. | tests/coverage_matrix.rs:685 | correct-requirement-no-evidence |

## Assurance Context

AP-201 (spec/assurance/AP-201-detection-minting.md) applies to coverage, trace binding, minting and schema interpretation. Its material-impact concern is reporting backing from a population not read. Native snapshot/identity comparisons, separate record/source binders, typed causes, strict aggregation, schema and legacy omissions were examined. AP-selected evidence and scope-boundary analyses ran as separate artifacts. No external producer qualification/reliance decision, actual performed-act observation, CI association, controlled-corpus run, mutation result, full aggregate gate or current producer-independence evidence was available. Those are not inferred from component inputs. No exception was asserted.

## Coverage

Plan completion: not assessed

Reconciliation: quire matrix (quire 0.36.2, installed engine 0.50.2) — no run evidence read. The installed reader is older than the reviewed engine; it was used for the current spec/source tag inventory only, not as execution evidence for the new API. Whole-repository matrix: 1020 rows, 638 tagged, 275 untagged, 107 method-without-symbol, 0 ignored-only. FR-078 ACs: 4 tagged (6,10,12,16), 16 untagged; constraints add three untagged and one method-without-symbol. The full matrix is retained at matrix.json. The unchanged and owner-blocked requirements remain open; this core-stage review does not demand invented positive qualification fixtures to turn them green. FAIL here is the repository assurance verdict, not a claim that this PR introduced all existing matrix gaps. Candidate-specific actionable gaps are the findings above. Full repository reverse-surface adjudication is limited to changed production paths in this PR; no global gap-free claim is made.

Optional semantic review: skipped; not separately authorized. Ordinary code/test faithfulness and the expressly requested native-boundary checks were performed. New production surface is owned by FR-078; no new store, CLI, wire producer schema, Quoin dependency, gate execution, stub qualifier, or source-symbol impersonation was found.

Verification: inspected the durable focused tests/clippy logs under /home/peter/dev/qualification-logs/plat1160-core-7d9b4cd (53 tests reported passing); these are coder-owned logs, not an independently rerun gate. No Cargo command, Kani, credentials, pin/digest work or aggregate gate ran in this review. The worktree target directory is absent. Hosted PR checks currently expose CLA checks only, including cla / cla-check FAILURE; no hosted Rust result was present at inspection. Hosted/aggregate merge readiness is therefore unproven.
## Examined Units

```yaml
scope:
- id: FR-078-AC-1
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: context_only
  excerpt: A current, attributable, passing performed-Inspection record from the shared
    evidence contract backs exactly its selected obligation without a source symbol;
    the result retains its record/run locator and actor provenance.
- id: FR-078-AC-2
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: context_only
  excerpt: A qualified Analysis record retains the declared Analysis method. For the
    lint acceptance fixture, both warnings-denied Clippy lanes and their CI association
    are required by the producer definition; a single lane, declaration-only record,
    or mismatched method grants no credit.
- id: FR-078-AC-3
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: context_only
  excerpt: A retained passing shell or Makefile gate execution binds its named selected
    target only when its producer definition, source revision, tool/configuration
    identity, and outcome match the qualification context; script existence alone
    grants no credit.
- id: FR-078-AC-4
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: context_only
  excerpt: An actually executed Rust `compile_fail` doctest can bind through the qualified
    channel when its producer reports the expected compiler refusal as a passing verification
    act; an unrun doctest, wrong compiler failure, or failed verification remains
    non-passing.
- id: FR-078-AC-5
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: context_only
  excerpt: Every declaration-only or citation-only control in the Verification Plan
    remains unbacked when no qualified performed record exists for its target.
- id: FR-078-AC-6
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: examined
  excerpt: Each non-passing requested act retains its originating cause category from
    the strict-decision table in both the report and strict rejection reasons; failed,
    errored, skipped, absent and unevaluated are not collapsed.
- id: FR-078-AC-7
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: context_only
  excerpt: A retained record that fails a required owner qualification premise yields
    that premise's distinct non-qualified cause and strict rejection reason and adds
    no record backing; no local configuration comparison substitutes for the pending
    owner contract.
- id: FR-078-AC-8
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: context_only
  excerpt: An owner-recognized waiver is distinguishable from absent, failed and skipped
    evidence in both report and strict disposition according to Peter's pending policy
    ruling; it is never mislabeled as a passed verification act. The final waiver
    disposition/exit oracle remains blocked on that ruling.
- id: FR-078-AC-9
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: examined
  excerpt: Exact owner target lookup refuses each missing, foreign, unknown or ambiguous
    identity with its corresponding cause; it never selects a sibling or the first
    duplicate.
- id: FR-078-AC-10
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: examined
  excerpt: Repeated identical owner record identities contribute once; conflicting
    content or qualification premises for one identity produce a deterministic conflict
    without choosing a winner by traversal order.
- id: FR-078-AC-11
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: context_only
  excerpt: A newer failed or errored result revokes an older passing binding according
    to the existing owner audit/freshness rules; reordering the supplied records cannot
    restore passing credit.
- id: FR-078-AC-12
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: examined
  excerpt: Without opting into evidence mode, existing source-backed results, report
    omission/serialization and strict behavior are unchanged; an explicit empty evidence
    input is evaluated as specified above and never removes a genuine source-symbol
    claim or promotes a citation.
- id: FR-078-AC-13
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: examined
  excerpt: '`no_source_symbol` remains a classification: an unqualified exempt row
    stays in `unbacked_rows` and `no_symbol_rows`, while an otherwise unbacked complete
    non-exempt row stays in `status_lies`; qualified backing changes only the exact
    target''s membership and matrix representation specified above.'
- id: FR-078-AC-14
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: examined
  excerpt: Forward and inverse trace lookup expose the same qualified record claims
    and provenance separately from source-symbol claims and citations; a plain occurrence
    of the same ID remains a citation.
- id: FR-078-AC-15
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: examined
  excerpt: Every criterion binding uses the borrowed owner inventory identity without
    a private census; `AuthoredRow`, `RenderedRow` (rendered ID and ordinal) and `CombinatorialTable`
    (rendered whole-table ID) retain their distinct origins and captured selection
    equality basis, including for unmapped criteria.
- id: FR-078-AC-16
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: examined
  excerpt: On the opted-in evidence path, a partial, invalid or non-Complete owner
    inventory yields `inventory-incomplete` in the report and strict rejection reasons
    while retaining typed causes and partial rows, even when every observed row is
    backed; legacy no-evidence behavior is unchanged.
- id: FR-078-AC-17
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: context_only
  excerpt: Producer-facing documentation and an executable acceptance example show
    the existing retained-record/audit-to-coverage path with actual HEAD and exact
    qualification premises, and show a genuine failing control; no decorative source
    tags or delivery-status downgrade are required.
- id: FR-078-AC-18
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: examined
  excerpt: Before/after acceptance fixtures exercise an Inspection-only RT criterion,
    the TC-198/TC-199 executed-gate boundary, and the FR-003-AC-2 doctest boundary
    or equivalent source-free fixtures. Their strict outcomes and row-level changes
    are recorded without claiming all RT or IR gaps resolved.
- id: FR-078-AC-19
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: context_only
  excerpt: Each unknown or incompatible method/kind, missing catalog/provenance or
    unsupported owner qualification version yields its explicit non-qualified owner
    cause and strict rejection reason, rather than an inferred pass.
- id: FR-078-AC-20
  path: spec/functional/FR-078-qualified-evidence-binding.md
  role: examined
  excerpt: Reading coverage and trace over retained evidence causes no gate/code execution,
    network/service access, evidence-store mutation, or publication; malformed retained
    inputs cannot execute commands or create passing evidence.
```

## Disposition Review Round 1

Frozen head: `137e0019d64f8549bbd8e031a330da23a2d22383`; fix commits `e2582c8`, `a2161fc`, `137e001`. Original findings and original review verdict above are preserved verbatim. All candidate-specific findings are fixed; no new defect was found in the fix diff. The bounded core stage has no remaining review finding. The historical whole-repository matrix result is not promoted to a global PASS and is not expanded into unrelated remediation. Actual Quoin FR-116 qualification, real producer acceptance, waiver policy and synchronized consumer uptake remain outside this core-stage acceptance.

Inspected coder-owned review-selected-red.log (actual failed diagnostic assertion, not setup failure), review-tests.log (56 passing: 17 coverage_matrix, 2 baseline, 11 output_contract, 26 obligations) and review-clippy.log (success), under /home/peter/dev/qualification-logs/plat1160-core-137e001-review/. No Cargo or aggregate gate was run by this reviewer. Hosted checks at inspection still show only CLA checks, including cla / cla-check FAILURE; no hosted Rust results are available. Merge remains conditional on the required final aggregate/hosted gates and Luna authority.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1: The new native-lookup test executes UnknownIdentity through an equal-selection stopped prefix, foreign-root SelectionMismatch, duplicate-authored AmbiguousTarget, and same-ID document/source negative controls with two accepted claims as the healthy control. |
| FND-002 | fixed | Round 1: The new origin test supplies RenderedRow and CombinatorialTable identities through compute_with_evidence; preserves three unmapped population rows; checks rendered ordinal and combinatorial origin; refuses a missing-method combinatorial claim rather than inventing a method; and rejects old rendered identity after reorder before accepting the fresh identity control. |
| FND-003 | fixed | Round 1: Exact inverse queries now distinguish two entries sharing record/run, compare each with its forward target result, and refuse independently changed record, run and entry locators. A run query still returns both entries. |

### Round 1 Fixed Excerpts

```yaml
dispositions:
- fnd: FND-001
  outcome: fixed
  after_excerpt: "let foreign_claims = [RecordClaim {\n        target: foreign.entries().next().unwrap().identity().into(),\n\
    \        ..claims[0]\n    }];"
- fnd: FND-002
  outcome: fixed
  after_excerpt: "assert!(matches!(\n        entries[2].identity().origin(),\n   \
    \     CriterionOrigin::CombinatorialTable { .. }\n    ));"
- fnd: FND-003
  outcome: fixed
  after_excerpt: "for claim in claims {\n        let inverse = trace(&backing, TraceQuery::Record(claim.locator));\n\
    \        assert_eq!(inverse.len(), 1);\n        assert_eq!(inverse, trace(&backing,\
    \ TraceQuery::Target(claim.target)));\n        assert_eq!(inverse[0].locator.entry,\
    \ claim.locator.entry);"
```
