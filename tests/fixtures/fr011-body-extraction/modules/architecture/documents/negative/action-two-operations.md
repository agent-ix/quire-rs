---
id: negative_004
title: "ActionWithTwoInvocations"
type: action
object: action
expect: semantic.record-invalid
because: "Action.json admits exactly one operation; two invocations make a process"
---
# [negative_004] ActionWithTwoInvocations

## Inputs

- `artifact_id` (uuid) — the artifact to act on.

## Operations

### verify_checksum

Recompute and compare the artifact digest.

| Param | Type | Multiplicity | Constraints |
|---|---|---|---|
| artifact_id | UUID | 1..1 | |

Returns: Boolean[1..1]

### quarantine_artifact

Move a failed artifact to quarantine — a second invocation, which is what makes
this record invalid.

| Param | Type | Multiplicity | Constraints |
|---|---|---|---|
| artifact_id | UUID | 1..1 | |
