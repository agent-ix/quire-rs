---
id: CONN-PORTAL-LAB
title: "PortalLabConnection"
type: connection
object: connection
relationships:
  - target: "CLASS-PHI"
    type: "carries"
  - target: "CLASS-PUBLIC"
    type: "carries"
  - target: "BOUND-INTERNET"
    type: "crosses"
---
# [CONN-PORTAL-LAB] PortalLabConnection

## Connection

| Source | Source Multiplicity | Target | Target Multiplicity | Direction |
|---|---|---|---|---|
| phi_link_out | 1..1 | phi_link_in | 1..1 | source-to-target |
