---
id: CG-POL-001
title: "SyntheticSecurityAccessPolicy"
type: policy
object: policy
relationships:
  - target: CG-THREAT-001
    type: governs
---
# [CG-POL-001] SyntheticSecurityAccessPolicy

## Policy

Synthetic example: privileged access is approved by an assigned owner and
reviewed quarterly.

## Invariants

### PrivilegedAccessHasAnOwner

```ocl
context SyntheticSecurityAccessPolicy
inv PrivilegedAccessHasAnOwner:
  true
```
