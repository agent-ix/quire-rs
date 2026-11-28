---
id: negative-003
title: "TokenReuseWithIdentity"
type: audit_event
object: audit_event
expect: semantic.record-invalid
because: "occurrence identity belongs to the runtime record; AuditEvent.json admits 0 identity fields"
---
# [negative-003] TokenReuseWithIdentity

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| occurred_at | Timestamp | 1..1 | |
| event_id | UUID | 1..1 | identity |

## Event Schema

```json
{ "type": "object" }
```
