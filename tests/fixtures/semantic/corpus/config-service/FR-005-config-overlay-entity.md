---
id: FR-005
title: ConfigOverlay Entity
object: entity
type: FR
traces:
  - US-002
status: IMPLEMENTED
---

# FR-005: ConfigOverlay Entity

## Description
The service SHALL define a `ConfigOverlay` SQLModel table with the following columns:

## Properties
| Column | Type | Constraints |
|--------|------|-------------|
| `id` | UUID | PK, default uuid4 |
| `version_id` | UUID | FK → config_versions.id |
| `priority` | int | default 0 |
| `settings` | Dict[str, Any] | JSONB column |
| `label` | str | non-empty |
| `applied_at` | datetime | default=utc_now |
| `applied_by` | str | actor identifier |
## Relationships

- `version`: many-to-one → ConfigVersion ([FR-006](./FR-006-config-version-entity.md)), lazy="noload"

## Dependencies

- [FR-006](./FR-006-config-version-entity.md) (ConfigVersion) — base entity

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-005-AC-1 | The `ConfigOverlay` model is persisted in the `config_overlays` table | Inspection |
| FR-005-AC-2 | An overlay's `priority` orders its application over its base version | Test |
| FR-005-AC-3 | The `settings` column uses JSONB for efficient querying and storage | Inspection |

### FR-005-AC-1
The `ConfigOverlay` model SHALL be persisted in the `config_overlays` table.

### FR-005-AC-2
An overlay's `priority` SHALL order its application over its base version.

### FR-005-AC-3
The `settings` column SHALL use JSONB for efficient querying and storage.
