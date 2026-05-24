---
id: US-008
title: "Multiple Agents Edit Different Blocks of the Same Artifact"
artifact_type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-019"
    type: "exercises"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-022"
    type: "exercises"
---

## Story

As an **orchestrator running multiple LLM agents against one artifact** (e.g. a "Behavior" agent and an "Acceptance Criteria" agent both refining one FR), I want each agent's edits scoped to its own block via the stable `{#blk-id}` attribute — so concurrent edits don't collide on heading shifts, line drift, or section reordering, and last-writer-wins per block instead of per file.

## Context

Without stable block IDs, addressing "the Acceptance section" of an artifact relies on heading text + line position — both of which drift when sibling blocks change. With stable IDs:

- Each agent's tool envelope carries `block_id` as a primary key, not "second-level heading matching `Acceptance.*`".
- A patch produced by agent A at T=0 is still applicable at T=1 even if agent B inserted a paragraph above — `update_block` finds the target by ID, not by line.
- Conflicts reduce to: did two agents target the same `block_id`? If yes, last writer wins (or the orchestrator applies its own merge policy). If no, both patches compose.

`quire-rs` itself is stateless and single-threaded per call; concurrency lives in the orchestrator. The crate's job is to make per-block operations *byte-stable* across surrounding edits so the orchestrator can serialize trivially.

## Acceptance

- **US-008-AC-1**: Apply patch A to `blk-behavior`, then apply patch B to `blk-acceptance` on the result. Both patches land; frontmatter + the unmodified blocks are byte-identical to the original.
- **US-008-AC-2**: Same patches applied in reverse order produce the same final markdown (composition is commutative when targeting distinct blocks).
- **US-008-AC-3**: Two patches both targeting `blk-behavior` applied in order A→B produce the same result as if B's data had been the original patch (last-writer-wins).
- **US-008-AC-4**: Inserting a new paragraph into `blk-purpose` does NOT change the `block_id` of any other block; a patch produced before the insert still applies to the right block after it.

## Efficiency Analysis

**Concurrency model:** orchestrator-mediated serialization. quire-rs is stateless; the orchestrator decides ordering.

**Round trips per agent:** identical to US-006 (1 per block edit).

**LLM context cost per agent:** identical to US-006 — each agent only sees *its own* block's schema + data. Two agents working on the same doc consume schema budget independently, not 2× the whole-artifact context.

**Server-side cost** for N independent block edits:
- N × `apply_block_patch` calls, each O(doc size) for the byte splice.
- Total: O(N · doc_size). The doc-size factor is unavoidable (the writeback rewrites the whole string), but each call is one allocation + one pass.
- *Not* O(N · block_size · agents) — no per-agent overhead beyond the per-call cost.

**Comparison to "whole-artifact patch" (no block IDs):**
- Without `{#blk-id}`, two agents producing patches against the same artifact must either:
  - (a) work serially and re-fetch the doc between turns, or
  - (b) operate on disjoint frontmatter fields only.
- Block IDs eliminate (a) and broaden (b) to disjoint *block contents*, not just frontmatter fields.

**Failure modes:**
- Two patches same `block_id`: orchestrator's policy decides (LWW, abort, merge-patches-of-patches). quire-rs reports nothing special — each call sees the doc state passed in.
- Heading text changed in an unrelated edit: the block_id stays; targeting still works (TC-401 verifies this).

**Cost the LLM never pays:**
- Doc-wide diff reasoning. The LLM never sees "what changed elsewhere"; the orchestrator owns that.
- Conflict reconciliation. quire-rs surfaces `MissingField` only if the `block_id` literally disappeared — which can only happen if an agent deleted the heading attribute, a policy violation the orchestrator should detect.

**Limit:** quire-rs has no built-in optimistic-concurrency token (no `If-Match: <hash>`). Orchestrators wanting CAS-style updates layer that on top by hashing the block's bytes pre-patch and verifying post-parse.
