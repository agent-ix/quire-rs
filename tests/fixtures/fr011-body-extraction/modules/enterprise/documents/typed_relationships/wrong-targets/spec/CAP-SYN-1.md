---
id: CAP-SYN-1
title: "SyntheticCapability"
type: capability
object: capability
relationships:
  - target: KPI-SYN-1
    type: realizes
  - target: KPI-SYN-1
    type: references
---
<!-- capability authoring skeleton (spec-objects-enterprise). Fill every section
     with substantive content. Contract (manifest body_extraction asserts):
     - Frontmatter MUST carry id, title, type: capability, object: capability.
     - "Properties" (H2): one typed row per attribute, header exactly
       `Field | Type | Multiplicity | Constraints`. At least one row carries the
       `identity` constraint — a capability is a node in the architecture graph.
     - "Sub-capabilities" (H2, required): the child capabilities, each with a
       substantive description (no TODO/TBD/placeholder text).
     - "Invariants" (H2): one `### <clauseId>` per clause, each owning
       exactly one ```ocl``` fence.
     A capability is an ability the organization holds, not an interface it
     exposes: Capability.json forbids `operations`. -->
# [CAP-SYN-1] SyntheticCapability

The Order Fulfillment capability is the organization's ability to take a placed
customer order and deliver the right goods to the right address within the
promised window. It spans orchestration of payment, inventory, warehouse and
carrier systems and is owned by the Fulfillment Platform group.

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| capability_id | UUID | 1..1 | identity |
| name | String | 1..1 | minLength: 1, maxLength: 120 |
| scope_statement | String | 1..1 | minLength: 1 |
| realizing_function | SupplyChainManagement | 0..* unique | |
| governing_principle | PromiseFromLiveCapacity | 0..* unique | |

## Sub-capabilities

- **Order Orchestration** — sequences payment capture, inventory reservation
  and shipment creation across regional fulfillment centers, with compensating
  actions on partial failure.
- **Warehouse Operations** — pick, pack and ship execution inside each
  fulfillment center, including wave planning and packing-station guidance.
- **Carrier Management** — carrier rate shopping, label generation, customs
  documentation and delivery-promise calculation across contracted carriers.
- **Returns Processing** — RMA intake, return-label issuance, inbound
  inspection and restock-or-refurbish disposition.

## Relationships

| Name | Verb | Target | Multiplicity |
|---|---|---|---|

## Invariants

The clauses the SyntheticCapability declaration enforces. Each clause owns one
`ocl` fence under its own `### <clauseId>` heading; the fence text is carried
verbatim and is never evaluated here.

### EveryChildIsRealizedByAFunction

```ocl
context SyntheticCapability
inv EveryChildIsRealizedByAFunction:
  self.realizing_function->notEmpty()
```

### ScopeIsStated

```ocl
context SyntheticCapability
inv ScopeIsStated:
  self.scope_statement->notEmpty()
```
