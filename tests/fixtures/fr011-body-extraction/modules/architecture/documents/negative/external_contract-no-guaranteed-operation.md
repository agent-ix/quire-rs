---
id: negative_006
title: "ContractWithoutGuaranteedOperation"
type: external_contract
object: external_contract
expect: semantic.record-invalid
because: "ExternalContract.json requires at least one operation carrying a post clause: this kind exists to pin guarantees"
---
# [negative_006] ContractWithoutGuaranteedOperation

## Contract

The service consumes a third-party geocoder and states no `Post:` guarantee on any
operation, which is what makes this record invalid.

## Operations

### geocode

Resolve a postal address to coordinates.

| Param | Type | Multiplicity | Constraints |
|---|---|---|---|
| address | String | 1..1 | minLength: 1 |

Returns: JsonObject[0..1]
