---
id: SOUP-011
title: "ExampleParserLibDisposition"
type: soup_record
object: soup_record
purl: "pkg:cargo/example-parser-lib@2.3.1"
disposition_clause: SOUP-RISK-EXAMPLE-011
---
# [SOUP-011] ExampleParserLibDisposition

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| component_id | UUID | 1..1 | identity |
| purl | String | 1..1 | minLength: 1 |

## Disposition

Worked example for a reviewer. The record above holds only the clause id
`SOUP-RISK-EXAMPLE-011`. The decision lives in a person-owned disposition
fact of the `agent-ix/quoin` FR-046 shape, keyed on that clause id. Inspect that
`attestedBy` and `authority` name a person and the authority they hold.

```json
{
  "kind": "disposition",
  "clauseId": "SOUP-RISK-EXAMPLE-011",
  "decision": "accepted_risk",
  "rationale": "The oversize-input anomaly is unreachable behind the product's 1 MiB request limit.",
  "approvalRef": "ix://agent-ix/example-product/APPROVAL-3",
  "attestation": {
    "attestedBy": "Alex Example",
    "authority": "Product security lead",
    "attestedAt": "2026-11-01T09:00:00Z",
    "expiresAt": "2027-05-01T09:00:00Z"
  }
}
```
