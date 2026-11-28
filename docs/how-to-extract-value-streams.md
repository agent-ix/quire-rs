---
title: Extract a value stream
type: how-to
description: Extract ordered stages, bindings, and outcomes from value-stream tables.
---

Use three tables to extract a value stream with ordered stages, typed capability and process bindings, and stage outcomes.

## Before you start

Use quire-rs 0.63.5 and a module that exports `value_stream`. The extraction request needs a bundle index and allowed links for the capability and process records.

## Steps

1. **Add the stage table.** Each stage has an identifier, order, name, and value.

   ```markdown
   ## Stages

   | Stage ID | Order | Name | Value |
   |---|---:|---|---|
   | intake | 1 | Intake | Capture the request |
   | fulfilment | 2 | Fulfilment | Deliver the order |
   ```

2. **Add bindings and outcomes.** A binding row names bundle objects; an outcome row belongs to a stage.

   ```markdown
   ## Stage Capability Bindings

   | Stage ID | Capability | Process |
   |---|---|---|
   | fulfilment | Order fulfilment | Fulfil Orders |

   ## Stage Outcomes

   | Stage ID | Outcome ID | Name | Description |
   |---|---|---|---|
   | fulfilment | delivered | Delivered | Order reaches the customer |
   ```

3. **Run extraction.** This request fixture includes the module, table locators, bundle index, and allowed links.

   ```rust
   use std::{error::Error, fs::File};

   fn main() -> Result<(), Box<dyn Error>> {
       let request: serde_json::Value = serde_json::from_reader(
           File::open("docs/examples/value-stream-request.json")?,
       )?;
       let extracted = quire_rs::semantic::python_entry::extract_semantic_json(&request)
           .map_err(std::io::Error::other)?;
       let record = serde_json::to_value(extracted)?;
       println!("{}", record["model"]["stages"][0]["id"]);
       Ok(())
   }

   // output: intake
   ```

Stages are returned in their declared order. Bindings resolve to semantic identifiers from the bundle.

## Result

The model contains ordered stages, outcomes attached to their stage, and typed capability and process bindings.

## Next steps

- [Extract typed relationships](how-to-model-typed-relationships.md)
- [Extract a unit model](how-to-extract-a-unit-model.md)
