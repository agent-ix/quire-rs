---
id: AN-002
title: "CI workflow is dispatch-only"
type: Anomaly
status: identified
safety_impact: unassessed
---
# [AN-002] CI workflow is dispatch-only

## Description

The 2026-11-23 regulated standards review reports that Quire-rs CI is
dispatch-only. The checked-in `.github/workflows/ci.yml` declares only
`workflow_dispatch` under `on`; it has no push, pull-request, or schedule
trigger. This record does not claim that a CI run succeeded or failed.

## Source

2026-11-23 regulated standards review, lane-D, “High: no executed verification
of the hardening wave” and “P1 behavioral mutation” findings.
