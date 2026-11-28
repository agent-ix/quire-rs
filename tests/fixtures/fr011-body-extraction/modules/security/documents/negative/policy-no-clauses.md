---
id: negative-006
title: "PolicyWithoutClauses"
type: policy
object: policy
expect: semantic.record-invalid
because: "Policy.json requires at least one clause; a prose-only Invariants section yields an empty array"
---
# [negative-006] PolicyWithoutClauses

## Policy

This policy describes its rules in prose and declares none, so the extracted
`clauses` array is empty.

## Invariants

The clauses are described here in prose and no `### <clauseId>` heading owns an
`ocl` fence, so nothing is declared.
