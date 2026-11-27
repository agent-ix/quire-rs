---
id: BF-901
title: "BusinessFunctionThatPerformsNothing"
type: business_function
object: business_function
expect: semantic.record-invalid
---
<!-- NEGATIVE fixture. A function is defined by what it performs, and this
     artifact declares no operation at all. BusinessFunction.json requires
     `operations` with at least one item. -->
# [BF-901] BusinessFunctionThatPerformsNothing

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| function_id | UUID | 1..1 | identity |

## Description

A function that performs nothing is a label, not a function; nothing in the
architecture graph can be realized by it.
