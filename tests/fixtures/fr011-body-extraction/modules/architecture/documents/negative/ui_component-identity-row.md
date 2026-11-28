---
id: negative_005
title: "ComponentWithIdentifiedProp"
type: ui_component
object: ui_component
expect: semantic.record-invalid
because: "UiComponent.json admits zero identity fields; props are not identified"
---
# [negative_005] ComponentWithIdentifiedProp

## Properties

| Field | Type | Multiplicity | Constraints |
|---|---|---|---|
| row_id | UUID | 1..1 | identity |
| loading | Boolean | 1..1 | |

## Props

- `row_id` — an identity-flagged prop, which a UI component may not declare.
- `loading` — whether the skeleton row state is rendered.
