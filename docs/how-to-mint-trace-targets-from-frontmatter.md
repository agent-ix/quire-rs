---
title: Mint trace targets from frontmatter
type: how-to
description: Use a document's frontmatter identifier as its trace target.
---

Configure a trace target with `from: frontmatter_id` to mint the document's own identifier without reading a table.

## Before you start

Use a traceability module with a target archetype and document references. The referenced documents must carry an `id` in frontmatter.

## Steps

1. **Declare the target source.** Exclude fixtures or other documents that should not contribute targets.

   ```yaml
   traceability:
     trace_targets:
       - name: requirements
         archetype: FR
         from: frontmatter_id
         exclude:
           - fixtures/**
   ```

2. **Add a document with its own identifier.** The identifier becomes the minted target.

   ```markdown
   ---
   id: FR-029
   type: FR
   ---
   # Requirement
   ```

3. **Link references to that target.** A document-reference declaration can point its pattern matches at the named target source.

   ```yaml
   document_references:
     - name: verification
       archetype: FR
       section: Acceptance Criteria
       column: Verification
       row_id_column: ID
       pattern: '(TC-[0-9]+)'
       targets: [requirements]
   ```

`frontmatter_id` reads the document identifier directly. It does not use a section or an identifier column.

## Result

The coverage report includes the document identifier as a target for matching references.

## Next steps

- [Inspect a coverage matrix](https://github.com/agent-ix/quire-cli/blob/main/docs/how-to-inspect-a-coverage-matrix.md)
