---
title: Extract typed relationships
type: how-to
description: Turn relationship table rows into typed links between records.
---

Use a relationship table and module mapping to extract typed links from a document.

## Before you start

Use quire-rs 0.63.5 and a module that enables the `relationships` mapping. The request also needs a bundle index, a relation vocabulary, and a data schema for each typed link.

## Steps

1. **Write the relationship rows.** Keep the target identifier in the table so the extractor can resolve it against the bundle.

   ```markdown
   ## Relationships

   | Name | Verb | Target | Multiplicity |
   |---|---|---|---|
| completed | emits | event-001 | 0..1 |
   ```

2. **Declare the mapping and allowed link.** The schema gives `emits` the semantic identifier type.

   ```json
   {
     "module": {
       "package": "agent-ix/spec-objects-business",
       "exports": ["process"],
       "mappings": ["relationships"]
     },
     "relationVocabulary": {
       "objectType": "process",
       "edgeTypes": {"emits": {"category": "dataflow"}},
       "allowedLinks": {"process": {"emits": ["event"]}}
     },
     "dataSchema": {
       "type": "object",
       "properties": {
         "emits": {
           "type": "array",
           "items": {"$ref": "https://schemas.agent-ix.org/semantic-core/0.4.0/SemanticId.json"}
         }
       }
     }
   }
   ```

   The complete extraction request, including its bundle index, is in [`typed-relationship-request.json`](examples/typed-relationship-request.json).

3. **Extract the record.** The typed field contains the resolved semantic identifier. The record also retains row source information under `relationSources`.

   ```python
   import json
   import quire

   with open("docs/examples/typed-relationship-request.json") as source:
       record = quire.extract_semantic(json.load(source))

   print(record["emits"])
   # ['ix://agent-ix/shop/event-001']
   ```

The bundle must contain the target artifact and its object type must be allowed for that relationship.

## Result

The extracted record contains a typed `emits` identifier and a `relationSources` entry with the source row and multiplicity.

## Next steps

- [Build a relation report](how-to-build-a-relation-report.md)
- [Extract a value stream](how-to-extract-value-streams.md)
