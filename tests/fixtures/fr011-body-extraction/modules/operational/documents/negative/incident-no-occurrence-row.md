---
id: negative-006
title: "IncidentWithoutOccurrenceTime"
type: incident
object: incident
expect: semantic.record-invalid
detail: '"name":"incident_id"'
because: "Incident.json requires at least one Timestamp field; an observed execution that records no time of occurrence is a standing definition, not an incident"
---
# [negative-006] IncidentWithoutOccurrenceTime

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| incident_id | UUID | 1..1 | identity |
| severity | String | 1..1 | enumValues: sev1\|sev2\|sev3 |
