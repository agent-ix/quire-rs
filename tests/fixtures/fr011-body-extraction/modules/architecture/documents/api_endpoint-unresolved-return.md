---
id: unresolved_001
title: "EndpointReturningUnknownRecord"
type: api_endpoint
object: api_endpoint
expect: semantic.unresolved-type
because: "the return names a token no skeleton of this module declares (FR-006-AC-5)"
---
# [unresolved_001] EndpointReturningUnknownRecord

## Endpoint

`GET /mysteries/{mystery_id}` answers with a record type this module does not
declare, so the extractor reports the token as unresolved.

## Operations

### get_mystery

Fetch one mystery by id.

| Param | Type | Multiplicity | Constraints |
|---|---|---|---|
| mystery_id | UUID | 1..1 | |

Returns: MysteryRecord[1..1]
