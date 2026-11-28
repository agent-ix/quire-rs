---
id: negative-009
title: "DeploymentWithBadTypeToken"
type: deployment
object: deployment
expect: semantic.invalid-type-token
detail: 'type token "Integer{count}" is not an Identifier'
because: "a Type cell resolves to a KernelScalar, a bundle declaration, or an import; a token that is not an Identifier is refused at the row"
---
# [negative-009] DeploymentWithBadTypeToken

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| release_id | String | 1..1 | identity |
| replicas | Integer{count} | 1..1 | min: 1 |
