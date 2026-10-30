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
this feature shall not derive a second census from Markdown.
[FR-077](./FR-077-trace-search-index.md) continues to distinguish source claims
from citations. Qualified record claims retain their own origin and provenance;
they do not impersonate Rust, Python, or TypeScript symbols.

## Inputs and public boundary

The supported boundary is a Rust library API, not a new CLI flag or a new
retained-record format. The planned public `quoin_quire::evidence::qualify` entry
point, hosted as specified below, SHALL accept the borrowed owner-issued `SelectedInventory`, exact requested
targets within that selection, actual source HEAD, and the original Quoin
`AuditInput` with its retained `RunRecord`, `Binding`, method catalog and other
owner-required audit premises, together with the original retained record/run
locators from the owner read boundary. The caller assembles those inputs from the shared
Quoin store using its existing read APIs. The Quoin bridge SHALL invoke
its same-workspace `quoin_auditor::audit::audit` over that input itself; it SHALL
NOT accept a
precomputed `AuditReport`, a caller's `healthy` flag, or an unaudited pass list as
a substitute. The bridge SHALL check that the audit obligations and HEAD are the supplied
owner selection and actual HEAD, rather than accepting a relabeled selection.
Non-criterion test-case targets use the existing declared target/reference rules
in the same corpus; no private criterion census is permitted.

`qualify` SHALL return an immutable, privately constructed Quoin bridge view containing
accepted claims and all non-qualified outcomes. It borrows original owner records
and identities; it is not a store, producer record schema, or caller-constructible
proof token. The bridge SHALL supply the view's owner-neutral backing projection to Quire's
`coverage::compute_with_evidence`, alongside its existing `coverage::compute`
corpus/registry/graph/root inputs, for reconciliation in the existing algorithm.
Quire's `evidence::trace` SHALL query that same backing projection beside the
existing source trace result: forward queries use
the exact selected target, inverse queries use the retained owner record/run
locator. A source file/symbol query keeps its existing source meaning; it SHALL
NOT manufacture a source identity for a retained record.

`evidence::strict_decision` SHALL return either success or rejection with the
complete ordered reason list specified below. The executable acceptance host
SHALL map success to process exit 0 and rejection to exit 1, print the full report
before exit, and print the cause codes with selected target/record locators on
stderr. These are planned public entry points, not APIs already delivered by
this spec PR. The existing no-evidence Rust entry points and CLI invocations stay
unchanged. A future CLI flag/binding needs its owning surface's specification;
this PR does not claim installed `quire coverage` already accepts retained input.
The producer-facing example and IR-1006 fixtures SHALL use the named Rust path,
including its strict decision, rather than a private fixture-only adapter.

### Host, dependency direction and synchronized adoption

The superseded direct `quire_rs::evidence::qualify` design would introduce a
**mutual repository dependency**: quire-rs would pin Quoin's auditor while
Quoin's `quoin-quire` already pins quire-rs. Although `quoin-auditor` does not
itself depend on `quoin-quire` and therefore does not create that crate cycle,
this reverse repository pin is not the selected design.

The proposed follow-on host is Quoin's existing `quoin-quire` crate, module
`evidence`, behind a **new optional `qualified-evidence` feature, default off**.
That feature SHALL use `quoin-auditor` from the same Quoin workspace revision.
Quire core SHALL own only the owner-neutral explicit backing input and its
existing reconciliation/trace/report logic; it SHALL NOT depend on Quoin or
re-export its auditor types. The bridge's privately constructed qualified view
and owner-approved projection are the supported qualification path; Quire's
low-level backing data alone is not a qualification certificate. This host and
feature are a proposed Quoin-owner follow-on, not an existing public API or a
capability delivered by apparatus-intake PR #689. No reverse pin, new store,
publication or release is authorized by this placement.

Adoption SHALL proceed in one direction: first merge the reviewed Quire engine
owner-neutral backing input, then have `quire-cli` and Quoin's `quoin-quire`
separately adopt **the same immutable reviewed published engine revision and
its exact package version** in their manifests and owner lockfiles. Before any
qualification claim, verify both resolved dependency graphs against those
manifest pins and each other; equal package versions alone are insufficient.
There SHALL be no reciprocal exact-Git-SHA cycle, copied version ledger or
assumed parity. A later engine change repeats the paired adoption/verification;
publication, tagging or release still requires Peter's first-hand instruction.

Measured baseline: Quoin main `9d0d6a3d` pins quire-rs `=0.50.2` at
`0649f7c0a88ff81f1a75b6754d55bd8cddc937f2`, while CLI main `374d5085` pins the
same version at `ba384acadb9d859dc3f46b247d884fad0fdc6db5`. SHA parity is **not**
established today. The Quoin PLAT-1162 local candidate `5a03b780` adopts the
published EA `6e8c0d978846726cb8d52ab1709a886d12b69011` (`=0.7.1`, default
features off); that unmerged candidate supplies neither this bridge nor closed
qualification dependencies. EA apparatus adoption and engine-pin synchronization
are distinct owner changes.

### Owner inventory to audit obligations

The bridge SHALL construct `AuditInput.obligations` from the supplied
`SelectedInventory` entries, using each borrowed `entry.obligation()` and the
existing Quoin projection semantics. It SHALL preserve exactly `id`, `statement`,
`method`, `criticality`, `parameters` and `target_ids`; empty
`parameters` and `target_ids` become `None` on the Quoin reader, as the existing
projection specifies. It SHALL NOT reparse documents, render IDs, or substitute legacy `derive` output for the selected entries.

The current `quoin-core::ops::quire::obligation` converter is **private**, and
core already depends on `quoin-quire`. The bridge SHALL NOT call or depend on
core. One owner-approved shared/extracted bridge adapter SHALL reuse those exact
projection semantics, with core delegating to that adapter rather than keeping
a copied converter. This specifies the required ownership/refactor, not an
invented existing public converter name.

Because `quoin_quire_types::Obligation` omits source/document/origin/selection,
the bridge SHALL retain a borrowed side index from each projected audit entry to
its original owner `CriterionEntry`/`CriterionIdentity`. It SHALL preserve
`AuthoredRow`, `RenderedRow` and `CombinatorialTable` and the captured selection
equality basis when mapping audit decisions back. It SHALL reject duplicate or
ambiguous flat IDs and any supplied-inventory/audit-obligation mismatch rather
than collapse distinct owner identities, pick the first match, or claim a fresh
Complete selection. A non-Complete inventory keeps its original causes and the
already specified strict rejection. Neither the reader projection nor the side
index is a parallel inventory census or retained evidence store.

### Owner qualification premises

The term **producer definition** means the Quoin-owner-qualified method contract
that declares the verification act, its required invocations, actor, source
revision, tool/configuration premises, and linkage between those invocations and
a retained CI run. Its method vocabulary comes from the Quoin `MethodCatalog`;
this is not permission for Quire to add fields to that catalog. Current
`RunRecord` fields alone do not define required Clippy invocations or CI
association. The owner has not yet supplied a contract/carrier for all those
premises; the dependency table below blocks their positive acceptance until it
does. A Quire-local `RunEntry.config` map, suite name, arbitrary key, or assertion
that the audit is healthy SHALL NOT implement the missing owner contract.

When the approved owner qualification contract becomes available, `qualify`
SHALL consume that contract's typed premises and decisions alongside the
original audit inputs; it SHALL NOT reproduce its checker or infer missing
premises. Until then it SHALL report the affected premises as unevaluated and
SHALL NOT create positive bindings for the blocked cases. The spec deliberately
does not assign fictitious fields or a wire format to this pending owner input.

## Outputs and reconciliation

The report SHALL retain qualified record-to-declaration claims with original
record/run locators, exact owner target/selection identity, method, actor and
qualification provenance. It SHALL expose every rejected or unanswered requested
verification act in `evidence_results`, with its target, available record locator,
originating owner cause and the stable cause category below. An absent requested
record has a target and `absent` cause without an invented record locator.
Multiple causes SHALL remain visible; a summarized strict failure SHALL NOT
collapse failed, stale, waiver and absent into one unnamed unbacked outcome.

FR-050's `coverage_matrix` is amended as follows, including for methods in
`no_source_symbol`:

- `binders` remains the source-declaration-site list, unchanged in meaning.
  Accepted records appear in a separate `record_binders` list carrying the
  qualified claim and its owner locators/identity/provenance; rejected records
  never enter this list. Neither source binder nor fake `SymbolGraph.verifies`
  entries are generated for records. In evidence mode, a criterion with no
  accepted records has an empty `record_binders` list, not an absent one.
- A criterion with one or more accepted record binders has computed status
  `qualified-record`, taking precedence over `method-without-symbol` and the
  three source-tag statuses. Without an accepted record binder, FR-050's
  existing four-way static status rule is unchanged. Existing source binders
  remain visible even when `qualified-record` wins; a rejected run does not
  delete them or relabel the declaration as having no source symbol.
- Exact accepted backing participates in the existing own-obligation-ID
  reconciliation. The qualified target is counted backed, its otherwise
  unbacked reference row leaves `unbacked_rows`, and an otherwise applicable
  `status_lies` or `no_symbol_rows` entry leaves that list. It cannot discharge
  a sibling through a shared test-case reference. An unqualified no-symbol row
  stays in `unbacked_rows` and `no_symbol_rows`, not `status_lies`, as before.
- Evidence-mode matrix rows use the owner-selected inventory population and
  identity, including unmapped rows; partial enumeration is labeled incomplete.
  The additive record fields and global evidence results are omitted on the
  legacy no-evidence path. Explicit evidence mode with zero records is different
  from not opting in: requested records are absent, and inventory completeness
  must still be evaluated. An explicitly empty request list does not silently
  exempt existing unbacked rows or bypass the owner's selection validation.

These are explicit amendments to FR-050-AC-16 and AC-47..49, not post-processing
of its report. Matrix status reports backing origin, not a replacement for the
per-act outcome or strict verdict. A source-tagged criterion can therefore retain
its static status while an explicitly requested failed run rejects strict mode.

## Behavior and strict decisions

When a retained entry passes the shared owner's audit and every required owner
qualification premise is positively evaluated, Quire SHALL bind it only to the
exact declaration named within the selected corpus. Analysis SHALL remain
Analysis, including a gate obligation requiring both warnings-denied Clippy
lanes and their CI association. Quire SHALL NOT invent a method, infer a missing
kind, or weaken the act to a convenient subcheck.

If an evidence request has a failed, errored, skipped, stale, missing or
unevaluated result, then Quire SHALL retain its distinct cause and SHALL NOT
accept it as passing backing. Newer failed/error runs SHALL revoke older record
bindings under owner audit/freshness precedence without deleting genuine source
claims. If multiple causes apply, Quire SHALL retain all causes in stable order
rather than choose whichever record was traversed first.

The following table defines strict behavior for the opted-in evidence path.
All rejecting categories yield exit 1 through the named acceptance host, with
different machine-readable reason codes and their original owner details. They
do not require a separate process exit number per category. A successful result
requires no rejection reason and all inherited FR-050 strict premises to pass.

| Situation | Report / strict reason | Strict result |
|-----------|------------------------|---------------|
| Qualified passing act | Accepted claim and original passing outcome | No evidence rejection; inherited strict checks still apply |
| Producer verification failed | `failed`, with original outcome | Reject |
| Producer errored | `errored`, with original outcome | Reject |
| Producer skipped | `skipped`, with original outcome; never a waiver alias | Reject |
| Stale or behind-HEAD run | `stale`, with owner freshness detail | Reject |
| Wrong source revision or selected corpus | `wrong-revision` or `selection-mismatch` respectively | Reject |
| Required requested record is absent | `absent`, without invented producer outcome | Reject |
| Required check/premise not evaluated | `unevaluated`, naming the unanswered premise | Reject |
| Unknown/mismatched method, actor, tool/configuration, or unsupported qualification version | `qualification-rejected`, retaining each owner subtype; an unanswered owner contract is `unevaluated` | Reject |
| Missing, unknown, ambiguous target or conflicting record | `invalid-target`, `ambiguous-target`, or `record-conflict` respectively | Reject |
| Invalid, partial or non-Complete selected inventory | `inventory-incomplete`, with original typed causes and partial rows | Reject, even if every emitted row has backing |
| Owner-recognized waiver | Distinct `waiver` outcome with original policy reference, once defined by the policy owner | **Blocked on Peter's ruling**; SHALL NOT be silently treated as failure, absence, Skip or passing verification |
| Waiver-shaped input without the approved owner representation/policy | `unevaluated`, naming `waiver-policy-pending`; preserve input diagnostic without claiming an owner-recognized waiver | Reject pending policy; this is not the eventual waiver exit policy |

A waiver is not proof that a verification act passed. Whether an approved waiver
permits a non-green policy disposition, and how that disposition maps to strict
exit, remains Peter's policy decision; the final waiver outcome/exit test stays
blocked until that ruling. This spec SHALL NOT claim AC-8 closed by a fabricated
waiver fixture or rejection of an unknown record. It SHALL NOT pre-decide that a
future approved waiver must use the same exit policy as a failed run.

Non-Complete inventory is an explicit new strict rejection premise **only for the
opted-in evidence path**. It augments the inherited FR-050 zero-target,
unbacked-row, contradicted-status and declared-input evaluation premises. Legacy
calls with no evidence mode retain their existing report and strict behavior;
this feature SHALL NOT introduce an inventory scan/trigger on that path. Qualified
records SHALL NOT erase inventory errors, unknown criteria, or unmapped selected
criteria. Identity changes SHALL be checked through the owner's captured selection
equality basis and freshness rules again.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-078-CON-1 | Quire shall reuse the shared Quoin record, audit, provenance, and method contracts without a parallel evidence store or copied contract schema. | Architecture | Inspection |
| FR-078-CON-2 | Coverage and trace shall inspect retained inputs without executing a gate, invoking inspected code, contacting a service, or publishing an artifact. | Boundary | Test |
| FR-078-CON-3 | The qualified channel shall preserve source-symbol backing, own-obligation-ID reference rules, and the classification-only meaning of `no_source_symbol`. | Compatibility | Test |
| FR-078-CON-4 | Identical evidence, selection, source and qualification inputs SHALL produce byte-identical claims, cause lists and strict reasons, ordered by selected declaration/document/origin and then owner record/run locator and cause code, independent of supplied record order. | Determinism | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-078-AC-1 | A current, attributable, passing performed-Inspection record from the shared evidence contract backs exactly its selected obligation without a source symbol; the result retains its record/run locator and actor provenance. | Test |
| FR-078-AC-2 | A qualified Analysis record retains the declared Analysis method. For the lint acceptance fixture, both warnings-denied Clippy lanes and their CI association are required by the producer definition; a single lane, declaration-only record, or mismatched method grants no credit. | Test |
| FR-078-AC-3 | A retained passing shell or Makefile gate execution binds its named selected target only when its producer definition, source revision, tool/configuration identity, and outcome match the qualification context; script existence alone grants no credit. | Test |
| FR-078-AC-4 | An actually executed Rust `compile_fail` doctest can bind through the qualified channel when its producer reports the expected compiler refusal as a passing verification act; an unrun doctest, wrong compiler failure, or failed verification remains non-passing. | Test |
| FR-078-AC-5 | Every declaration-only or citation-only control in the Verification Plan remains unbacked when no qualified performed record exists for its target. | Test |
| FR-078-AC-6 | Each non-passing requested act retains its originating cause category from the strict-decision table in both the report and strict rejection reasons; failed, errored, skipped, absent and unevaluated are not collapsed. | Test |
| FR-078-AC-7 | A retained record that fails a required owner qualification premise yields that premise's distinct non-qualified cause and strict rejection reason and adds no record backing; no local configuration comparison substitutes for the pending owner contract. | Test |
| FR-078-AC-8 | An owner-recognized waiver is distinguishable from absent, failed and skipped evidence in both report and strict disposition according to Peter's pending policy ruling; it is never mislabeled as a passed verification act. The final waiver disposition/exit oracle remains blocked on that ruling. | Test |
| FR-078-AC-9 | Exact owner target lookup refuses each missing, foreign, unknown or ambiguous identity with its corresponding cause; it never selects a sibling or the first duplicate. | Test |
| FR-078-AC-10 | Repeated identical owner record identities contribute once; conflicting content or qualification premises for one identity produce a deterministic conflict without choosing a winner by traversal order. | Test |
| FR-078-AC-11 | A newer failed or errored result revokes an older passing binding according to the existing owner audit/freshness rules; reordering the supplied records cannot restore passing credit. | Test |
| FR-078-AC-12 | Without opting into evidence mode, existing source-backed results, report omission/serialization and strict behavior are unchanged; an explicit empty evidence input is evaluated as specified above and never removes a genuine source-symbol claim or promotes a citation. | Test |
| FR-078-AC-13 | `no_source_symbol` remains a classification: an unqualified exempt row stays in `unbacked_rows` and `no_symbol_rows`, while an otherwise unbacked complete non-exempt row stays in `status_lies`; qualified backing changes only the exact target's membership and matrix representation specified above. | Test |
| FR-078-AC-14 | Forward and inverse trace lookup expose the same qualified record claims and provenance separately from source-symbol claims and citations; a plain occurrence of the same ID remains a citation. | Test |
| FR-078-AC-15 | Every criterion binding uses the borrowed owner inventory identity without a private census; `AuthoredRow`, `RenderedRow` (rendered ID and ordinal) and `CombinatorialTable` (rendered whole-table ID) retain their distinct origins and captured selection equality basis, including for unmapped criteria. | Test |
| FR-078-AC-16 | On the opted-in evidence path, a partial, invalid or non-Complete owner inventory yields `inventory-incomplete` in the report and strict rejection reasons while retaining typed causes and partial rows, even when every observed row is backed; legacy no-evidence behavior is unchanged. | Test |
| FR-078-AC-17 | Producer-facing documentation and an executable acceptance example show the existing retained-record/audit-to-coverage path with actual HEAD and exact qualification premises, and show a genuine failing control; no decorative source tags or delivery-status downgrade are required. | Test |
| FR-078-AC-18 | Before/after acceptance fixtures exercise an Inspection-only RT criterion, the TC-198/TC-199 executed-gate boundary, and the FR-003-AC-2 doctest boundary or equivalent source-free fixtures. Their strict outcomes and row-level changes are recorded without claiming all RT or IR gaps resolved. | Test |
| FR-078-AC-19 | Each unknown or incompatible method/kind, missing catalog/provenance or unsupported owner qualification version yields its explicit non-qualified owner cause and strict rejection reason, rather than an inferred pass. | Test |
| FR-078-AC-20 | Reading coverage and trace over retained evidence causes no gate/code execution, network/service access, evidence-store mutation, or publication; malformed retained inputs cannot execute commands or create passing evidence. | Test |

## Verification Plan

The existing TC-1963..1982 one-to-one AC mapping remains. Each listed partition
is a required separate assertion/control within its mapped test, not a choice
of one representative case. TC-1981 additionally verifies CON-4 determinism;
TC-1982 verifies CON-2 inertness; TC-1974 and TC-1975 verify CON-3 preservation.
CON-1 remains independently inspected against owner dependency use.

| Mapped criterion | Required independent partitions |
|------------------|---------------------------------|
| AC-5 / TC-1967 | Citation, suite declaration, unexecuted script, bare method name, merely validated Inspections table; each paired with the same target and absent qualified performed evidence |
| AC-7 / TC-1969 | Wrong source revision, foreign selected corpus, wrong actor, incompatible producer/tool/configuration; each varies one premise with its own cause and strict reason |
| AC-9 / TC-1971 | Missing target, unknown ID, same display ID in another document/source, foreign selection, duplicate authored identity; lookup must expose the owner's distinct refusal |
| AC-15 / TC-1977 | AuthoredRow, RenderedRow and CombinatorialTable; unmapped criterion; changed captured selection; rendered-row renumbering never reuses an old binding by display ID alone |
| AC-19 / TC-1981 | Unknown method, absent kind, absent catalog, absent provenance, unsupported owner version; evaluate refusals independently and separately assert CON-4 ordering/byte identity on reordered inputs |

Rendered identities SHALL be rebound through a newly issued owner selection
after row reordering/renumbering or captured selected-byte changes; neither an
unchanged rendered string nor the same ordinal alone permits reuse. A
CombinatorialTable identity names the owner's whole table, not an invented row.
TC-1977 SHALL preserve owner equality for unselected/excluded changes rather than
inventing a broader corpus digest. TC-1978 SHALL distinguish healthy empty
Complete inventory from empty selectors, malformed unknown-membership documents,
zero limits and exact-N limit stops. TC-1975 SHALL assert the new matrix status,
separate source/record binder lists and exact unbacked/no-symbol/status-lie delta;
TC-1974 SHALL assert legacy no-input byte identity independently of opted-in empty
input. No invented positive owner record is permitted to make these tests green.


TC-1963 through TC-1982 are reserved for AC-1 through AC-20 respectively. The
test matrix contains planned acceptance cases, not execution receipts. Before
implementation, bank each defect's real failure/control pair in the declarative
corpus with an attributed origin; assert both the intended finding and the
absence of spurious findings in its healthy control. A schema/setup failure is
not the intended behavioral RED. The source-free positive cases SHALL use actual
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

The following owner contracts and decisions remain prerequisites; publication of
a draft API or apparatus-intake repair does not satisfy them.

### Explicit blocked acceptance work

| Owner gap | Blocked acceptance cases | Required owner decision/delivery |
|-----------|--------------------------|----------------------------------|
| Quoin positive kind/actor/revision/tool/configuration qualification and required invocation/CI association carrier | AC-1/TC-1963, AC-2/TC-1964, AC-3/TC-1965, AC-4/TC-1966, AC-7/TC-1969, AC-19/TC-1981 positive qualification boundary | Quoin owner follow-on (may ride PLAT-1162) must define and review the authoritative contract and typed handoff; `AuditReport.healthy`, a local config map or inferred Test method cannot close them |
| Owner-recognized waiver representation and report/strict policy | AC-8/TC-1970 | Peter policy ruling through luna:IR:plan, followed by the shared Quoin owner representation; no Skip mapping or fixture-only waiver |
| Named public retained-input implementation | AC-14/TC-1976, AC-17/TC-1979 and AC-18/TC-1980 end-to-end acceptance | Implement the named Rust boundary after spec acceptance and integrate the approved owner contracts; planned signatures are not published APIs |

Positive end-to-end fixtures in any other TC also remain blocked wherever they
need one of these missing premises. Negative/rejection test design is independent
work, not a substitute for positive acceptance. None of these dependencies is
claimed closed by this spec PR, the apparatus-intake Quoin #689 changes, or a
schema-valid Inspections artifact. No blocked AC is dropped or downgraded.


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
  confirmed with that owner before admission implementation; Skip shall not be
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

- The `spec-artifacts-process` module owns the `Inspections` declaration/schema
  and obligation-reference validation. It supplies the performed-Inspection
  producing artifact and its mapping to the shared Quoin run/binding with actor,
  source and retained locator; Quire consumes that owner-produced evidence.
  Merely validating the declaration supplies neither performance nor a passing
  outcome. This producer handoff is coordinated with that owner; it is not a
  new Quire Inspections parser or evidence store.

## Change record

PLAT-1160 / PR #550 review repair explicitly extends FR-050-AC-16 and AC-47..49
for opt-in retained backing, adds the conditional strict inventory premise,
names the Rust input/audit path, and lists blocked owner acceptance. It leaves
legacy source-only behavior and #515's own-obligation-ID repair intact. Planned
tests remain unexecuted; no coverage count, banking grade or owner qualification
is claimed by these normative changes.
