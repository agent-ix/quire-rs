---
id: US-006
title: "LLM Patches a Single Block via Schema-Validated Merge"
type: US
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

> **RETIRED (render removal — 2026-06-04):** This user story is render-centric: its
> central flow is merge → validate → **re-render** → splice (`apply_block_patch`).
> With the render feature removed (no backward-compatibility layer), the
> render-and-splice block edit path is gone; block edits are byte-splice-only via
> `update_block` (FR-022, reframed). This story is therefore **retired**. Kept for
> history and traceability only; its acceptance and performance criteria are dropped
> from the required-coverage tally. See `spec.md` §2bis.

## Story

As an **LLM agent editing one block of an existing artifact**, I want to (1) read the block's current `(block_type, data)` from the parsed document, (2) emit a JSON merge-patch against the block-type schema surfaced via `schema_for(block_type)`, and (3) have the engine merge → validate → re-render → splice the new bytes back into the canonical markdown — so that frontmatter and every other block stay byte-identical.

## Context

The v0.2 block model (INPUT.md, FR-019..022) makes one block the natural editing unit:

- Each editable block carries a stable `{#blk-id}` Pandoc heading attribute.
- The block's JSON Schema is a thin slice of the artifact-level schema — only the fields the LLM needs to reason about.
- The merge-patch shape is the LLM's tool-call envelope.

The flow is symmetric to `US-001` but **scoped to one block**: the LLM's context window only needs the schema + current data for that one block, not the entire artifact.

## Acceptance

- US-006-AC-1 (RETIRED): A consumer parses an artifact, walks to a specific `block_id`, extracts the block's current data (frontmatter slice or extracted block fields), and feeds `schema_for(block_type)` into a tool envelope.
- US-006-AC-2 (RETIRED): `apply_block_patch(registry, doc, block_id, block_type, current_data, patch)` returns full updated markdown; the target block's bytes are the new render; all other blocks are byte-identical.
- US-006-AC-3 (RETIRED): An invalid patch (merged data fails schema) returns `SchemaViolation` with a field path the LLM can correct from.
- US-006-AC-4 (RETIRED): An unknown `block_id` returns `MissingField` without mutating the doc.

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

## Performance Criteria

Server-side measurements only — LLM round-trip latency is outside quire-rs's control.

- **US-006-PC-1**: `apply_block_patch` on a 10 KB document with a typical 5-block layout completes in p50 < 1 ms, p99 < 5 ms after `Registry::load_from` is warm. Bench: **TC-450**.
- **US-006-PC-2**: Per-call memory: one allocation for the output `String`, sized at `doc.len() + new_bytes.len()`. No retained intermediates after return. Verified via heap-profile sample in TC-450.
- **US-006-PC-3**: Inherits NFR-001 (template render <1 ms) and NFR-007 (zero disk I/O after load). Schema validation amortized to load time; per-call validator cost is JSON-walk only.
- **US-006-PC-4**: Repeated invocation against the same doc + different block_ids: linear in number of calls, no superlinear cost from re-parsing (the consumer parses once and reuses the `QuireDocument`).
