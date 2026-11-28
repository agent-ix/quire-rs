---
id: PROCESS-RECEIVER-SYN-001
title: Synthetic process handoff
type: process
object: process
---

# Synthetic process handoff

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| correlation_id | UUID | 1..1 | identity |

## Steps

| Step | Kind | Consumes | Emits | Receiver | Description |
|---|---|---|---|---|---|
| accept_request | command | | | ix://agent-ix/spec-artifacts-process/ROLE | Receive the synthetic handoff |
