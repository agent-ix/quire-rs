---
id: negative-009
title: "AuthFlowWithDanglingPost"
type: auth_flow
object: auth_flow
expect: semantic.dangling-clause-ref
because: "a Pre:/Post: line names a clause id declared in the same artifact; this one names none"
---
# [negative-009] AuthFlowWithDanglingPost

## Flow

```mermaid
sequenceDiagram
  participant C as Client
  participant A as Authorization server
  C->>A: exchange code
```

## Operations

The exchanges this flow performs.

### exchange_code

Exchange the code for a token pair.

| Param | Type | Multiplicity | Constraints |
|---|---|---|---|
| code | String | 1..1 | minLength: 1 |

Post: NoSuchClause
