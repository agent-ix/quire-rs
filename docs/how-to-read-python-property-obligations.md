---
title: Read Python property classification records
type: how-to
description: Inspect classified requirements and their obligation records in Python.
---

Call `classify_properties` to inspect the extracted property labels and their source positions.

## Before you start

Use the Python binding from quire-rs 0.63.5. Provide Markdown for an artifact type that has a binding-criteria table.

## Steps

1. **Classify the document.** The function returns one record for each recognized criterion row.

   ```python
   import quire

   markdown = """---
   id: FR-001
   type: FR
   ---
   ## Acceptance Criteria

   | ID | Criteria | Verification |
   |---|---|---|
   | FR-001-AC-1 | A missing key uses the declared default. | Test |
   """

   records = quire.classify_properties("iso-spec-core", "FR", markdown)
   ```

2. **Read the obligation field.** Each record includes its obligation data when the classifier can derive it; otherwise the value is `None`.

   ```python
   record = records[0]
   print(record["row_id"], record["property"], record["obligation"])
   ```

Records also carry the source line, statement, classification shape, extraction result, and signal identifiers.

## Result

Use `row_id` to identify the source criterion. The `obligation` field contains the associated record when available and is `None` otherwise.

## Next steps

- [Mint trace targets from frontmatter](how-to-mint-trace-targets-from-frontmatter.md)
