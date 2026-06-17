---
id: US-009
title: "LLM Creates a New Artifact From Scratch"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-001"
    type: "exercises"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-003"
    type: "exercises"
---

> **RETIRED (render removal — 2026-06-04):** This user story is render-centric:
> `schema_for` → emit whole data → **`render_by_name`** → canonical markdown. With
> the render feature removed (no backward-compatibility layer), there is no
> whole-artifact render path. New documents are now authored directly as markdown and
> checked with `validate_document` ([FR-032](../functional/FR-032-validate-document.md) / [US-014](./US-014-author-validates-markdown.md)). This story is therefore
> **retired**. Kept for history and traceability only; its acceptance and performance
> criteria are dropped from the required-coverage tally. See `spec.md` §2bis.

## Story

As an **LLM agent** producing a new artifact (e.g. drafting a brand-new FR or NFR from a user prompt), I want to call `schema_for(archetype)` once to learn the full schema, emit a complete data value, and have `render(registry, archetype, data)` produce canonical markdown in one shot, so that I get a finished artifact with no writeback and no existing doc required.

## Context

Editing ([US-006](./US-006-llm-patches-one-block.md)/007) and creation are different shapes:

- Editing operates on a parsed `QuireDocument` and writes back into it.
- Creation has no prior document; the LLM emits the artifact's whole data, and the engine renders it via the archetype-level template.

In v0.2 the whole-artifact case is treated as "one giant block whose type is the archetype". `render_by_name(registry, archetype, data)` is the canonical entry point. The output is the LLM's new `.md` file; the consumer writes it to disk.

## Acceptance

- US-009-AC-1 (RETIRED): `render_by_name(registry, "fr", data)` against a value satisfying the FR schema returns `RenderOutput { markdown, diagnostics: [] }` where `markdown` is byte-equal to the Python Jinja2 reference for the same data.
- US-009-AC-2 (RETIRED): An invalid `data` (missing required field) returns `SchemaViolation` naming the field path; no markdown is produced.
- US-009-AC-3 (RETIRED): The rendered markdown can be re-parsed by `parse_document` and the resulting `QuireDocument` round-trips to the same data fields the LLM emitted (modulo Jinja whitespace).

## Efficiency Analysis

**Round trips:** 1 (LLM → server `render_by_name` → markdown).

**LLM context cost** (input tokens):
- whole-archetype schema: 500–5,000 bytes (larger than per-block [US-006](./US-006-llm-patches-one-block.md)).
- system prompt + user prompt: variable; not quire-rs's concern.

**LLM output cost** (output tokens):
- whole-artifact data value: 500–5,000 bytes — proportional to the artifact's surface area, not template length.

**Server-side cost** per call:
- One schema validation (compiled at load time, [FR-013](../functional/FR-013-archetype-loader.md)).
- One template render. For an "FR" archetype this is typically 5–20 KB of output for a ~1 KB data payload.
- *No* writeback; no parse; no byte splice.
- Total: dominated by Jinja render time, well under [NFR-001](../non-functional/NFR-001-render-latency.md) (1 ms median per archetype).

**Comparison to [US-006](./US-006-llm-patches-one-block.md) (edit):**
- Creation skips parse + writeback (zero doc-size cost).
- Creation carries higher schema/data context (whole artifact vs one block).
- One-shot, no iterative refinement primitive — if the LLM gets a field wrong, it re-emits the whole artifact. [US-006](./US-006-llm-patches-one-block.md) is the better choice once the artifact exists.

**When to use US-009:**
- New documents only.
- Bootstrapping a corpus from a template + prompt.
- Generating fixtures (e.g. test corpus for downstream parity sweeps).

**When NOT to use US-009:**
- Editing an existing doc — even small edits — because the LLM has to re-emit the whole thing instead of a small block patch.
- Refining one block of an existing doc — use [US-006](./US-006-llm-patches-one-block.md)/007 instead.

**Failure cost:** A `SchemaViolation` on creation is the LLM's full output thrown away. Cheaper than [US-006](./US-006-llm-patches-one-block.md) in per-block latency but more costly in tokens when the LLM iterates. Worth measuring per-corpus how often the LLM gets it right on the first try.

## Performance Criteria

- **US-009-PC-1**: `render_by_name` on a 1 KB data value into a 10 KB output completes in p50 < 1 ms (inherits [NFR-001-AC-1](../non-functional/NFR-001-render-latency.md)). Bench: **TC-042** (existing per-archetype render bench).
- **US-009-PC-2**: Schema validation cost on the data value is bounded by O(data fields) — well under 100 µs for typical artifacts. Covered by the validator-choice ADR ([NFR-009](../non-functional/NFR-009-dependency-pinning.md) + TC-331).
- **US-009-PC-3**: No parse + no writeback path on the create flow; latency strictly lower than [US-006](./US-006-llm-patches-one-block.md)/007. The output `String` is the only allocation.
- **US-009-PC-4**: Round-trip self-consistency: `parse_document(render_by_name(data))` yields data fields equal to the original (US-009-AC-3). Property-test verified by TC-024 + TC-056.
