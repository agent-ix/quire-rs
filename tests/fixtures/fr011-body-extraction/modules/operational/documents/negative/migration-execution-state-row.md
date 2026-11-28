---
id: negative-010
title: "MigrationWithAppliedAt"
type: migration
object: migration
expect: semantic.record-invalid
detail: '"name":"applied_at"'
because: "Migration.json admits zero occurrence fields; an applied-at Timestamp is execution state, and a migration that records when it ran is an observed execution wearing a standing definition's schema"
---
# [negative-010] MigrationWithAppliedAt

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| migration_version | String | 1..1 | pattern: /^[0-9]{14}$/ |
| applied_at | Timestamp | 1..1 | |

## Invariants

### BackfillPrecedesConstraint

```ocl
context MigrationWithAppliedAt
inv BackfillPrecedesConstraint:
  self.migration_version->notEmpty()
```
