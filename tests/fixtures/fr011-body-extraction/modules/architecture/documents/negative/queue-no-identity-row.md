---
id: negative_003
title: "QueueWithoutPartitionKey"
type: queue
object: queue
expect: semantic.record-invalid
because: "Queue.json requires at least one identity field, the partition key"
---
# [negative_003] QueueWithoutPartitionKey

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| event | String | 1..1 | minLength: 1 |
| uploaded_at | Timestamp | 1..1 | |

## Message Format

```json
{ "event": "artifact.uploaded", "uploaded_at": "2026-06-10T12:00:00Z" }
```
