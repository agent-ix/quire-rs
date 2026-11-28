---
id: negative-005
title: "RunbookWithoutOperations"
type: runbook
object: runbook
expect: semantic.record-invalid
detail: 'at operations: [] has less than 1 item'
because: "Runbook.json requires at least one operation; a prose-only Operations section yields an empty array"
---
# [negative-005] RunbookWithoutOperations

## Operations

This runbook describes its actions in prose and declares none, so the
extracted `operations` array is empty.
