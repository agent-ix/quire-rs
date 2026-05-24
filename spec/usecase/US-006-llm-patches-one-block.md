---
id: US-006
title: "LLM Patches a Single Block via Schema-Validated Merge"
artifact_type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-019"
    type: "exercises"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-021"
    type: "exercises"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-022"
    type: "exercises"
---

## Story

As an **LLM agent editing one block of an existing artifact**, I want to (1) read the block's current `(block_type, data)` from the parsed document, (2) emit a JSON merge-patch against the block-type schema surfaced via `schema_for(block_type)`, and (3) have the engine merge → validate → re-render → splice the new bytes back into the canonical markdown — so that frontmatter and every other block stay byte-identical.

## Context

The v0.2 block model (INPUT.md, FR-019..022) makes one block the natural editing unit:

- Each editable block carries a stable `{#blk-id}` Pandoc heading attribute.
- The block's JSON Schema is a thin slice of the artifact-level schema — only the fields the LLM needs to reason about.
- The merge-patch shape is the LLM's tool-call envelope.

The flow is symmetric to `US-001` but **scoped to one block**: the LLM's context window only needs the schema + current data for that one block, not the entire artifact.

## Acceptance

- **US-006-AC-1**: A consumer parses an artifact, walks to a specific `block_id`, extracts the block's current data (frontmatter slice or extracted block fields), and feeds `schema_for(block_type)` into a tool envelope.
- **US-006-AC-2**: `apply_block_patch(registry, doc, block_id, block_type, current_data, patch)` returns full updated markdown; the target block's bytes are the new render; all other blocks are byte-identical.
- **US-006-AC-3**: An invalid patch (merged data fails schema) returns `SchemaViolation` with a field path the LLM can correct from.
- **US-006-AC-4**: An unknown `block_id` returns `MissingField` without mutating the doc.

## Efficiency Analysis

**Round trips:** 1 (LLM → server `apply_block_patch`).

**LLM context cost** (input tokens):
- block-type schema: typically 200–2,000 bytes (one block, not whole artifact).
- current block data: 50–500 bytes (JSON of one block's fields).
- *not loaded into LLM context:* frontmatter, sibling blocks, untouched body.

**LLM output cost** (output tokens):
- JSON merge-patch payload: typically 20–500 bytes (only changed fields).
- Cheaper than US-007 (full-replace) when the change touches few fields.

**Server-side cost** per call:
- Schema validation: O(merged JSON size) — pre-compiled validator amortized to load-time (FR-013 + NFR-007).
- Template render: O(block template size) ≪ O(whole-artifact template).
- Writeback: O(doc size) byte splice — single allocation, one pass.
- No disk I/O after `Registry::load_from` (NFR-007 audited).

**Comparison to US-001** (whole-artifact patch):
- US-006 sends ~10× less context to the LLM (1 block vs 1 artifact).
- US-006 re-renders ~10× less template surface per call.
- US-006 is composable: edit a sequence of blocks in N calls without re-rendering N-1 untouched blocks.

**Failure cost:** A `SchemaViolation` ends the loop with a field path; the LLM retries with corrected fields. No half-written markdown returned (validation precedes render).
