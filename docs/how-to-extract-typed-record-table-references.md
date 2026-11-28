---
title: Extract typed references from record tables
type: how-to
description: Resolve semantic identifier cells in object record tables.
---

Declare a table property as a semantic identifier to resolve its cell value against the bundle index.

## Before you start

Use quire-rs 0.63.5 and a module with the `record-sections` mapping. The record schema must reference the semantic identifier schema and allow the target object type.

## Steps

1. **Write the record table.** The table heading names the schema property, and the column heading names the nested field.

   ```markdown
   ## Steps/Items

   | Name | Receiver~Primary |
   |---|---|
   | first | PR-1 |
   ```

2. **Type the cell in the schema.** The complete request includes the bundle entries and the allowed target type.

   ```json
   "receiver~primary": {
     "$ref": "https://schemas.agent-ix.org/semantic-core/0.1.0/SemanticId.json"
   }
   ```

   See [`typed-record-table-request.json`](examples/typed-record-table-request.json) for the full request.

3. **Extract the record.** The reference cell resolves to the target's semantic identifier.

   ```rust
   let request: serde_json::Value = serde_json::from_reader(
       std::fs::File::open("docs/examples/typed-record-table-request.json")?,
   )?;
   let extracted = quire_rs::semantic::python_entry::extract_semantic_json(&request)
       .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidData, message))?;
   let record = serde_json::to_value(extracted)?;
   assert_eq!(record["record"]["Steps/Items"][0]["receiver~primary"], "ix://acme/work/PR-1");
   ```

## Result

The record contains resolved identifiers, and `recordRelations` retains the source locations and validation paths for the links.

## Next steps

- [Extract typed relationships](how-to-model-typed-relationships.md)
- [Extract a value stream](how-to-extract-value-streams.md)
