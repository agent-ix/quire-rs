---
id: negative-002
title: "TokenReuseWithoutOccurrence"
type: audit_event
object: audit_event
expect: semantic.record-invalid
because: "AuditEvent.json requires at least one occurrence field whose type targets Timestamp"
---
# [negative-002] TokenReuseWithoutOccurrence

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| actor_reference | String | 1..1 | minLength: 1 |
| grant_family_reference | String | 1..1 | minLength: 1 |

## Event Schema

```json
{ "type": "object" }
```
