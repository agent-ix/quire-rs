---
id: FR-078
title: "Qualified retained evidence in coverage and trace"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-053"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-077"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-007"
    type: "traces_to"
---

# FR-078: Qualified retained evidence in coverage and trace

## Description

When a caller supplies retained verification evidence, Quire shall qualify its
binding to the selected declaration before granting coverage credit. A performed
Inspection, Analysis, shell or Makefile gate, or Rust doctest can carry evidence
without being a source-symbol declaration. A citation, method name, suite
registration, or schema-valid Inspections table is not a passing execution or a
performed inspection.

This is an additional evidence channel, not a change to the source-marker
grammar. [FR-050](./FR-050-declarative-coverage-computation.md) remains the owner of
coverage reconciliation and the `no_source_symbol` status classification.
[FR-053](./FR-053-obligation-record.md) owns the selected criterion population;
this feature must not derive a second census from Markdown.
[FR-077](./FR-077-trace-search-index.md) continues to distinguish source claims
from citations. Qualified record claims retain their own origin and provenance;
they do not impersonate Rust, Python, or TypeScript symbols.

## Inputs

- The selected corpus and Quire-owned criterion inventory, including its typed
  selection and criterion identities, partial rows, and terminal condition.
- Existing declared targets and reference rules for test cases and other
  non-criterion obligations in that same selection.
- Retained run entries, bindings, provenance, and audit results from the shared
  Quoin evidence contract, plus the expected source, producer, tool/configuration,
  and method premises for the selected verification act.

The existing Quoin `RecordRequest`/`record_run`, `RunRecord`, `RunEntry`, `Binding`,
`EvidenceLineage`, method catalog, and auditor assembly remain their owners'
contracts. Quire does not introduce another store or copy those schemas. The
owner audit must use the actual source HEAD and the selected obligations; a
caller-supplied label saying that an audit passed is insufficient.

## Outputs

- Qualified record-to-declaration claims with their retained record/run locator,
  selected target identity, method, outcome, and qualification premises.
- Coverage and trace results that identify whether backing came from a source
  symbol or a qualified retained record.
- Distinct typed non-passing causes for failed execution, stale or wrong-revision
  evidence, a waiver, absent or unevaluated evidence, invalid references,
  incompatible qualification premises, and conflicting records.

## Behavior

When a retained entry passes the shared owner's audit and all required
qualification premises match, Quire shall bind it only to the declaration that
entry actually names within the selected corpus. The selected obligation's
method and producer definition determine the verification act; Quire must not
invent a method, infer a missing kind from the presence of entries, or weaken the
scope to a convenient subcheck. Analysis remains Analysis, including a selected
gate obligation that requires multiple invocations and its CI association.

When a selected result is failed, errored, skipped, waived, stale, missing, or
unevaluated, Quire shall retain the cause without treating it as passing backing.
An absence of audit findings is not positive qualification when the owner reports
an unanswered method, absent catalog, missing provenance, or an unevaluated act.
A newer failed run cannot be hidden by selecting an older passing run; owner
freshness, statement-suspect, and audit precedence rules remain authoritative.

When a qualified claim is accepted, Quire shall feed its explicit backing identity
into the existing reconciliation path. Post-processing a completed report to
reclassify it, changing statuses by hand, or constructing a fictitious
`SymbolGraph.verifies` source symbol does not implement this channel. Existing
own-obligation-ID rules remain intact: a sibling's evidence or a shared test-case
reference cannot silently discharge a different criterion.

When selection enumeration is non-Complete, Quire shall retain its original
causes and partial observations without claiming complete coverage. Qualified
records must not erase inventory failures, unknown criteria, or unmapped selected
criteria. A declaration, source symbol, or evidence record changing identity
requires the owner-defined freshness and reference checks again.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-078-CON-1 | Quire shall reuse the shared Quoin record, audit, provenance, and method contracts without a parallel evidence store or copied contract schema. | Architecture | Inspection |
| FR-078-CON-2 | Coverage and trace shall inspect retained inputs without executing a gate, invoking inspected code, contacting a service, or publishing an artifact. | Boundary | Test |
| FR-078-CON-3 | The qualified channel shall preserve source-symbol backing, own-obligation-ID reference rules, and the classification-only meaning of `no_source_symbol`. | Compatibility | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-078-AC-1 | A current, attributable, passing performed-Inspection record from the shared evidence contract backs exactly its selected obligation without a source symbol; the result retains its record/run locator and actor provenance. | Test |
| FR-078-AC-2 | A qualified Analysis record retains the declared Analysis method. For the lint acceptance fixture, both warnings-denied Clippy lanes and their CI association are required by the producer definition; a single lane, declaration-only record, or mismatched method grants no credit. | Test |
| FR-078-AC-3 | A retained passing shell or Makefile gate execution binds its named selected target only when its producer definition, source revision, tool/configuration identity, and outcome match the qualification context; script existence alone grants no credit. | Test |
| FR-078-AC-4 | An actually executed Rust `compile_fail` doctest can bind through the qualified channel when its producer reports the expected compiler refusal as a passing verification act; an unrun doctest, wrong compiler failure, or failed verification remains non-passing. | Test |
| FR-078-AC-5 | A citation, suite declaration, unexecuted script, method name, and merely validated Inspections table each remain unbacked in a paired control with the same target and no qualified performed record. | Test |
| FR-078-AC-6 | Failed, errored, skipped, absent, and unevaluated records remain distinguishable from qualified passing records, retain their originating cause, and cannot make a remaining unbacked selection pass strict coverage. | Test |
| FR-078-AC-7 | Wrong source revision, wrong selected corpus, changed statement, and mismatched producer/tool/configuration premises are reported with their respective owner-qualified causes and grant no passing credit. | Test |
| FR-078-AC-8 | A waiver remains explicitly distinguishable from absent or failed evidence and never becomes a passing verification act or a green strict result for the waived obligation. | Test |
| FR-078-AC-9 | Missing, wrong, duplicate, and ambiguous obligation/test-case identities are rejected against the selected owner inventory and declared target rules; a valid identity from a different document, declaration source, or selection cannot discharge the requested target, and ambiguous lookup never selects the first row. | Test |
| FR-078-AC-10 | Repeated identical owner record identities contribute once; conflicting content or qualification premises for one identity produce a deterministic conflict without choosing a winner by traversal order. | Test |
| FR-078-AC-11 | A newer failed or errored result revokes an older passing binding according to the existing owner audit/freshness rules; reordering the supplied records cannot restore passing credit. | Test |
| FR-078-AC-12 | With no qualified records supplied, existing source-backed results and report omission/serialization behavior are unchanged; qualified input cannot remove a genuine source-symbol claim or promote a citation. | Test |
| FR-078-AC-13 | `no_source_symbol` alone remains classification rather than evidence. An unsupported completed claim and a method-exempt unbacked row retain their existing distinction; only a separately qualified passing record can add backing. | Test |
| FR-078-AC-14 | Forward and inverse trace lookup expose the same qualified record claims and provenance separately from source-symbol claims and citations; a plain occurrence of the same ID remains a citation. | Test |
| FR-078-AC-15 | The owner-issued selected inventory retains all selected criteria, including criteria with neither mappings nor records; each accepted binding uses the owner's typed criterion/selection identity and equality basis, retaining authored-versus-rendered ID origin rather than assuming an authored ID, deriving a private Markdown census, or inferring an ID hierarchy. | Test |
| FR-078-AC-16 | A partial, invalid, or non-Complete inventory retains its typed causes and partial rows and cannot yield a complete green strict report, even when every observed row has a passing record. | Test |
| FR-078-AC-17 | Producer-facing documentation and an executable acceptance example show the existing retained-record/audit-to-coverage path with actual HEAD and exact qualification premises, and show a genuine failing control; no decorative source tags or delivery-status downgrade are required. | Test |
| FR-078-AC-18 | Before/after acceptance fixtures exercise an Inspection-only RT criterion, the TC-198/TC-199 executed-gate boundary, and the FR-003-AC-2 doctest boundary or equivalent source-free fixtures. Their strict outcomes and row-level changes are recorded without claiming all RT or IR gaps resolved. | Test |
| FR-078-AC-19 | Unknown method/kind, missing catalog/provenance, and unsupported owner qualification versions are explicit non-qualified outcomes rather than inferred passes; identical inputs emit byte-identical, stably ordered claims and diagnostics. | Test |
| FR-078-AC-20 | Reading coverage and trace over retained evidence causes no gate/code execution, network/service access, evidence-store mutation, or publication; malformed retained inputs cannot execute commands or create passing evidence. | Test |

## Verification Plan

TC-1963 through TC-1982 are reserved for AC-1 through AC-20 respectively. The
test matrix contains planned acceptance cases, not execution receipts. Before
implementation, bank each defect's real failure/control pair in the declarative
corpus with an attributed origin; assert both the intended finding and the
absence of spurious findings in its healthy control. A schema/setup failure is
not the intended behavioral RED. The source-free positive cases must use actual
shared-owner record/audit types and rules, not a new fixture-only record format.

The Inspection and Analysis cases vary one qualification premise at a time. The
gate and doctest cases distinguish producer success from process exit alone:
the expected compile refusal is success only for the declared compile-fail
verification act. The duplicate/conflict and latest-failure cases run both
record orders. The source-preservation cases include existing source claims,
plain citations, and the declared no-symbol classification. Selection cases
include an unmapped criterion and a retained partial inventory. Documentation
examples are executed against the same supported public boundary as consumers.

## Dependencies

- [FR-050](./FR-050-declarative-coverage-computation.md), including the separate
  own-obligation-ID repair, owns row reconciliation. The bounded backing-input
  hook is an implementation integration point, not permission to weaken that
  repair or maintain another reconciliation algorithm.
- [FR-053](./FR-053-obligation-record.md) and PLAT-1165 own selected criterion
  identities and terminal completeness. Their reviewed contract precedes
  binding completeness; this feature does not replace `derive` or its census.
- [FR-077](./FR-077-trace-search-index.md) owns the source claims/citations
  distinction. Qualified claims add a separate origin, not fake symbols.
- [Quoin evidence store](ix://agent-ix/quoin/spec/functional/FR-030),
  [auditor](ix://agent-ix/quoin/spec/functional/FR-032), and
  [evidence-backed matrix](ix://agent-ix/quoin/spec/functional/FR-115) own the
  retained run/binding and actual-HEAD audit assembly. Inspection and Analysis
  use the owner's explicit method catalog and evidence kind. A missing kind or
  catalog is unanswerable, not proof of conformance.
- The existing Quoin run outcome is Pass/Fail/Skip/Error. Its representation of
  a retained waiver and the supported read-only audited-input handoff must be
  confirmed with that owner before admission implementation; Skip must not be
  locally relabeled as a waiver, and no replacement store/schema is authorized.
  The current owner has no approved waiver-origin representation; producer
  trust decisions and unknown-field preservation do not supply one. Also, a
  healthy audit alone does not establish a positively declared compatible
  method kind or the required provenance/configuration qualification. Those
  producer guarantees require an owner-reviewed contract before this channel
  can claim the affected acceptance cases; rejecting or omitting those cases
  does not close them.
- IR-1006 owns the IR/RT method-consumer acceptance fixtures and remeasurement
  against this single channel. IR-499, IR-1003, and the broader architecture
  gate remain separate consumer acceptance scopes.
