---
id: negative_007
title: "OrderRepositoryWithDanglingPost"
type: repository
object: repository
expect: semantic.dangling-clause-ref
because: "a Pre:/Post: line names a clause id declared in the same artifact; this one names none"
---
# [negative_007] OrderRepositoryWithDanglingPost

## Operations

The operations this repository exposes.

### save

Persist the whole aggregate atomically.

| Param | Type | Multiplicity | Constraints |
|---|---|---|---|
| order | Order | 1..1 | |

Post: NoSuchClause
