---
id: negative-001
title: "ConfigurationWithIdentityRow"
type: configuration
object: configuration
expect: semantic.record-invalid
detail: '"name":"config_id"'
because: "Configuration.json admits zero identity fields; a parameter set is keyed by the object it configures, not by itself"
---
# [negative-001] ConfigurationWithIdentityRow

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| config_id | UUID | 1..1 | identity |
| storage_root | String | 1..1 | minLength: 1 |
