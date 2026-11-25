---
id: US-021
title: "Consume Quire-RS anomaly documents in validation runs"
type: US
relationships:
  - target: "ix://agent-ix/quoin/FR-217"
    type: "traces_to"
---
# US-021: Consume Quire-RS anomaly documents in validation runs

## Story

**As a** downstream self-test integrator
**I want** to access the anomaly documents shipped with the Quire-RS library
**So that** a validation run can retain the anomaly set from the tool version that produced it.

## Context

The validation report consumes the anomaly documents carried by each tool
binary. Quire-RS already exposes its embedded self-test case files, but its
anomaly documents are not available to the linked Quire CLI self-test runner.
This story covers only access to Quire-RS-owned anomaly documents; the Quire CLI
combines them with its own records.
