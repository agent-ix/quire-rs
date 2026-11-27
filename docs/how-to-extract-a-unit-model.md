---
title: Extract a unit model from Markdown
type: how-to
description: Add unit tables to a semantic module and extract their dimension, target, scale, and offset as typed data.
---

With a semantic module and Python caller in place, add a `unit` table locator and pass the artifact's bundle index to extract a unit record.

## Before you start

You need a module with semantic extraction enabled and the artifact index for its dimension and unit types.

## Steps

1. Add the unit locator to the module's `unit` object type. Keep `Target` optional so a base unit can omit its reference unit.

   ```yaml
   object_types:
     - name: unit
       body_extraction:
         yield_pattern:
           match:
             unit:
               from: table_row
               under_section: Scale
               assert:
                 columns: [Dimension, Target, Scale, Offset]
                 optional_columns: [Target]
   semantic:
     contract_version: "1.0.0"
     package: acme/physical-models
     exports: [unit]
     targets: [json-schema, markdown]
   ```

2. Add a scale table to the unit document. The `Dimension` cell names a dimension artifact. The optional `Target` cell names another unit artifact.

   ```markdown
   ## Scale

   | Dimension | Target | Scale | Offset |
   |---|---|---:|---:|
   | Time | ms | 1/1000 | 0 |
   ```

   Write scale and offset as integers or reduced fractions. Use `0` for zero.

3. Pass the Markdown, locator, and bundle index to `quire.extract_semantic`. The bundle must include each referenced artifact with its `dimension` or `unit` object type.

   ```python
   body_extraction = {
       "yield_pattern": {
           "match": {
               "unit": {
                   "from": "table_row",
                   "under_section": "Scale",
                   "assert": {
                       "columns": ["Dimension", "Target", "Scale", "Offset"],
                       "optional_columns": ["Target"],
                   },
               }
           }
       }
   }
   bundle = {
       "package": "acme/physical-models",
       "objects": [
           {"id": "Time", "names": ["Time"]},
           {"id": "ms", "names": ["ms"]},
       ],
       "artifacts": [
           {"id": "Time", "object": "dimension", "operations": []},
           {"id": "ms", "object": "unit", "operations": []},
       ],
   }
   record = quire.extract_semantic({
       "markdown": document,
       "module": {
           "contractVersion": "1.0.0",
           "package": "acme/physical-models",
           "exports": ["unit"],
           "imports": {},
       },
       "path": "spec/units/Millisecond.md",
       "sourceIdentity": "ix://acme/physical-models/spec",
       "bundle": bundle,
       "bodyExtraction": body_extraction,
   })
   ```

## Result

The extraction record carries resolved identities in `model.unit`:

```json
{
  "model": {
    "unit": {
    "dimension": { "target": { "kind": "SemanticId", "value": "ix://acme/physical-models/type/Time" } },
    "target": { "target": { "kind": "SemanticId", "value": "ix://acme/physical-models/type/ms" } },
    "scale": "1/1000",
    "offset": "0",
    "sourceSpan": {
      "sourceIdentity": "ix://acme/physical-models/spec",
      "path": "spec/units/Millisecond.md",
      "startLine": 12,
      "startColumn": 1,
      "endLine": 12,
      "endColumn": 27
    }
    }
  }
}
```
