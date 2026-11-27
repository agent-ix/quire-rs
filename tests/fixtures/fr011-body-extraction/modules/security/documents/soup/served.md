---
id: SOUP-011
title: "ExampleParserLib"
type: soup_record
object: soup_record
purl: "pkg:cargo/example-parser-lib@2.3.1"
name: example-parser-lib
version: "2.3.1"
supplier: unknown
serves:
  - ix://agent-ix/spec-objects-security/FR-100
known_anomalies:
  - statement: "Rejects inputs over 16 MiB without an error code."
    source_ref: ix://agent-ix/example-product/ISSUE-7
support_status: community_supported
end_of_support: 2027-06-30
disposition_clause: SOUP-RISK-EXAMPLE-010
---
# [SOUP-011] ExampleParserLib

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| component_id | UUID | 1..1 | identity |
| purl | String | 1..1 | minLength: 1 |
| version | String | 0..1 |  |
