---
id: negative-007
title: "SecretWithEmbeddedMaterial"
type: secret
object: secret
expect: semantic.unknown-constraint-keyword
because: "a secret is named by its locator; `default:` is outside the closed constraint keyword set, so the Markdown path cannot express embedded material at all"
---
# [negative-007] SecretWithEmbeddedMaterial

## Rotation

Rotated on demand.

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| secret_id | UUID | 1..1 | identity |
| locator | String | 1..1 | default: vault://atlas/oidc |
