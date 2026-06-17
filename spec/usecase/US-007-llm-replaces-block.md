---
id: US-007
title: "LLM Full-Replaces a Block When Changes Are Pervasive"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-021"
    type: "exercises"
---

> **RETIRED (render removal — 2026-06-04):** This user story is render-centric: its
> flow validates `new_data` then **renders** and writes it back (`replace_block`).
> With the render feature removed (no backward-compatibility layer), the
> render-and-splice path is gone; block edits are byte-splice-only via `update_block`
> (FR-022, reframed). This story is therefore **retired**. Kept for history and
> traceability only; its acceptance and performance criteria are dropped from the
> required-coverage tally. See `spec.md` §2bis.

## Story

As an **LLM agent** making pervasive changes within a single block (rewriting a section's body, restructuring a list, changing a table's shape), I want to emit the block's *new* data wholesale rather than a merge-patch, so that the engine validates the replacement against the block-type schema and writes it back without merge-patch semantics interfering.

## Context

Merge-patch ([US-006](./US-006-llm-patches-one-block.md)) is the natural primitive when the LLM changes one or two fields. When the LLM rewrites most of a block's content, a merge-patch:

- gets large enough that it's effectively a full replacement anyway, and
- forces the LLM to reason about whether merge semantics will preserve unwanted prior state (e.g. a deep list that should be empty after the edit but a `null` patch entry on it would behave differently than an explicit `[]`).

`replace_block(registry, doc, block_id, block_type, new_data)` skips the merge step. The engine validates `new_data` directly, renders, writes back.

## Acceptance

- US-007-AC-1 (RETIRED): `replace_block` produces the same rendered bytes as running the block-type template against `new_data` directly (verified by TC-441).
- US-007-AC-2 (RETIRED): A schema violation in `new_data` returns `SchemaViolation` *before* any markdown is produced; doc is unchanged.
- US-007-AC-3 (RETIRED): Untouched blocks + frontmatter byte-identical after `replace_block`.
- US-007-AC-4 (RETIRED): An unknown `block_id` returns `MissingField` without mutating the doc.

## Efficiency Analysis

**Round trips:** 1.

**LLM context cost** (input tokens):
- block-type schema: 200–2,000 bytes (same as [US-006](./US-006-llm-patches-one-block.md)).
- current block data: optional — the LLM may not need it if the rewrite is independent of prior content. Otherwise 50–500 bytes.

**LLM output cost** (output tokens):
- Full block data: typically 200–2,000 bytes — *more* than a merge-patch.
- Break-even vs [US-006](./US-006-llm-patches-one-block.md): when ~40% of fields change, the merge-patch and the full replacement are comparable in size; beyond that, replace is competitive.

**Server-side cost** per call:
- Validation: O(new_data size) — same as [US-006](./US-006-llm-patches-one-block.md), no merge step.
- Render: identical to [US-006](./US-006-llm-patches-one-block.md) (same template).
- Writeback: identical to [US-006](./US-006-llm-patches-one-block.md) (`update_block` byte splice).
- *Saves* the deep_merge pass (small, but non-zero on large blocks).

**Comparison to [US-006](./US-006-llm-patches-one-block.md):**
- LLM cognitive load: simpler — "produce a valid block" instead of "produce a patch that, when merged, yields a valid block".
- Token cost: higher output, identical input.
- Error recoverability: easier — the LLM never has to reason about merge semantics; a schema error means re-emit the value.

**When to prefer US-007 over [US-006](./US-006-llm-patches-one-block.md):**
- The change touches > ~40% of the block's fields.
- The block contains arrays/maps where merge semantics are ambiguous (e.g. "did the LLM mean to append or replace?").
- The LLM is generating block content from scratch and doesn't need to preserve prior state.

**Failure cost:** identical to [US-006](./US-006-llm-patches-one-block.md) (validation-before-render means no half-written markdown).

## Performance Criteria

- **US-007-PC-1**: `replace_block` on a 10 KB document with a typical 5-block layout completes in p50 < 1 ms, p99 < 5 ms (within ±10% of [US-006](./US-006-llm-patches-one-block.md)-PC-1 — the deep-merge step is negligible at this scale). Bench: **TC-451**.
- **US-007-PC-2**: Per-call memory: identical envelope to [US-006](./US-006-llm-patches-one-block.md) — one allocation for the output `String`.
- **US-007-PC-3**: Inherits [NFR-001](../non-functional/NFR-001-render-latency.md) + [NFR-007](../non-functional/NFR-007-load-cost-amortized.md) (render <1 ms, zero disk I/O after load).
- **US-007-PC-4**: On larger blocks (≥ 50 KB of rendered content), `replace_block` is measurably *faster* than `apply_block_patch` because it skips the recursive `deep_merge`. The crossover where US-007 wins is documented in the TC-451 report.
