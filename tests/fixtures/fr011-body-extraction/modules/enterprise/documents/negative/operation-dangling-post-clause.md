---
id: BF-902
title: "BusinessFunctionWithDanglingClause"
type: business_function
object: business_function
expect: semantic.dangling-clause-ref
---
<!-- NEGATIVE fixture. An operation's `Post:` names a clause id this artifact
     does not declare. -->
# [BF-902] BusinessFunctionWithDanglingClause

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| function_id | UUID | 1..1 | identity |

## Description

A function whose postcondition names a clause nobody declared cannot be checked
by anything downstream.

## Operations

### forecast_demand

Produce the demand forecast for one fulfillment center.

| Param | Type | Multiplicity | Constraints |
|---|---|---|---|
| center_id | UUID | 1..1 | |

Returns: JsonObject[1..1]

Post: NoSuchClauseIsDeclaredHere
