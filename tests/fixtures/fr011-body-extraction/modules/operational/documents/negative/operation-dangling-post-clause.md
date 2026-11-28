---
id: negative-008
title: "RunbookWithDanglingPost"
type: runbook
object: runbook
expect: semantic.dangling-clause-ref
detail: 'NoSuchClause'
because: "a Pre:/Post: line names a clause id declared in the same artifact; this one names none"
---
# [negative-008] RunbookWithDanglingPost

## Operations

### acknowledge_page

Acknowledge the page before anything else is changed.

| Param | Type | Multiplicity | Constraints |
|---|---|---|---|
| page_id | UUID | 1..1 | |

Post: NoSuchClause
