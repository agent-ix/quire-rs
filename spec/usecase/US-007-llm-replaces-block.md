---
id: US-007
title: "LLM Full-Replaces a Block When Changes Are Pervasive"
artifact_type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-021"
    type: "exercises"
---

## Story

As an **LLM agent making pervasive changes within a single block** (rewriting a section's body, restructuring a list, changing a table's shape), I want to emit the block's *new* data wholesale rather than a merge-patch — so the engine validates the replacement against the block-type schema and writes it back without merge-patch semantics interfering.

## Context

Merge-patch (US-006) is the natural primitive when the LLM changes one or two fields. When the LLM rewrites most of a block's content, a merge-patch:

- gets large enough that it's effectively a full replacement anyway, and
- forces the LLM to reason about whether merge semantics will preserve unwanted prior state (e.g. a deep list that should be empty after the edit but a `null` patch entry on it would behave differently than an explicit `[]`).

`replace_block(registry, doc, block_id, block_type, new_data)` skips the merge step. The engine validates `new_data` directly, renders, writes back.

## Acceptance

- **US-007-AC-1**: `replace_block` produces the same rendered bytes as running the block-type template against `new_data` directly (verified by TC-441).
- **US-007-AC-2**: A schema violation in `new_data` returns `SchemaViolation` *before* any markdown is produced; doc is unchanged.
- **US-007-AC-3**: Untouched blocks + frontmatter byte-identical after `replace_block`.
- **US-007-AC-4**: An unknown `block_id` returns `MissingField` without mutating the doc.

## Efficiency Analysis

**Round trips:** 1.

**LLM context cost** (input tokens):
- block-type schema: 200–2,000 bytes (same as US-006).
- current block data: optional — the LLM may not need it if the rewrite is independent of prior content. Otherwise 50–500 bytes.

**LLM output cost** (output tokens):
- Full block data: typically 200–2,000 bytes — *more* than a merge-patch.
- Break-even vs US-006: when ~40% of fields change, the merge-patch and the full replacement are comparable in size; beyond that, replace is competitive.

**Server-side cost** per call:
- Validation: O(new_data size) — same as US-006, no merge step.
- Render: identical to US-006 (same template).
- Writeback: identical to US-006 (`update_block` byte splice).
- *Saves* the deep_merge pass (small, but non-zero on large blocks).

**Comparison to US-006:**
- LLM cognitive load: simpler — "produce a valid block" instead of "produce a patch that, when merged, yields a valid block".
- Token cost: higher output, identical input.
- Error recoverability: easier — the LLM never has to reason about merge semantics; a schema error means re-emit the value.

**When to prefer US-007 over US-006:**
- The change touches > ~40% of the block's fields.
- The block contains arrays/maps where merge semantics are ambiguous (e.g. "did the LLM mean to append or replace?").
- The LLM is generating block content from scratch and doesn't need to preserve prior state.

**Failure cost:** identical to US-006 (validation-before-render means no half-written markdown).
