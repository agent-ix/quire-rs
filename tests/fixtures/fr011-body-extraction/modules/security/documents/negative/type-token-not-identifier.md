---
id: negative-010
title: "AssetWithBadTypeToken"
type: asset
object: asset
expect: semantic.invalid-type-token
because: "a Type cell holds a kernel scalar or an Identifier naming another declaration"
---
# [negative-010] AssetWithBadTypeToken

## Description

An asset whose Type cell is not an Identifier.

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| asset_id | UUID | 1..1 | identity |
| classification | Data Restricted | 1..1 | |
