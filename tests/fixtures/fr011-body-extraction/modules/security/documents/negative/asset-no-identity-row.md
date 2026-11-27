---
id: negative-001
title: "AssetWithoutIdentity"
type: asset
object: asset
expect: semantic.record-invalid
because: "Asset.json requires at least one identity field"
---
# [negative-001] AssetWithoutIdentity

## Description

An asset that names no identity of its own.

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| name | String | 1..1 | minLength: 1 |
| steward | String | 1..1 | minLength: 1 |
