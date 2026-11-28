---
id: negative_009
title: "InterfaceWithDanglingPost"
type: interface
object: interface
expect: semantic.dangling-clause-ref
because: "a Pre:/Post: line names a clause id declared in the same artifact; this one names none"
---
# [negative_009] InterfaceWithDanglingPost

## Contract

```yaml
name: InterfaceWithDanglingPost
operations:
  - name: score
    inputs: [payload]
    output: score
```

## Operations

### score

Score one payload.

| Param | Type | Multiplicity | Constraints |
|---|---|---|---|
| payload | Bytes | 1..1 | nonEmpty |

Returns: Decimal(18,9)[1..1]

Post: NoSuchClause

## Features

| Feature | Kind |
|---|---|
| score | operation |
