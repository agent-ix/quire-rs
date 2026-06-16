---
id: US-004
title: "Filament Editor Receives a Patch, Validates, and Re-Renders"
type: US
relationships:
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/filament-editor-app"
    type: "consumes"
    cardinality: "1:1"
---

> **RETIRED (render removal — 2026-06-04):** This user story is render-centric
> (merge → validate → **render** in one sub-millisecond call). With the render
> feature removed (no backward-compatibility layer), it is **retired**. Block edits
> are now byte-splice-only via `update_block` (FR-022); no re-render occurs. The
> render-latency NFR (NFR-001) is retired with it. Kept for history and traceability
> only; its acceptance criteria are dropped from the required-coverage tally. See
> `spec.md` §2bis.

## Story

As the **Filament editor app**, I want to receive a patch (`{ block_id, partial_data }`) over the wire, hand it to `quire-rs` to merge-validate-render against the current block, and receive back either a typed validation error or canonical markdown, so that the editor's render path completes in well under a frame (<16 ms) regardless of artifact size.

## Context

The editor today round-trips edits through a Python service. Cold-start and per-request overhead add tens of milliseconds. Bundling `quire-rs` into the editor backend collapses validate + render into a single sub-millisecond call (see NFR-001).

## Acceptance

- US-004-AC-1 (RETIRED): A test takes a baseline `FrData`, applies a partial patch (`{ title: "new title" }`), and asserts the merged-then-validated render produces markdown with the new title and the original other fields.
- US-004-AC-2 (RETIRED): A test applies a patch that makes the merged data invalid (e.g. `title: ""`) and asserts a typed validation error with the field path `data.title`, not a render error.
- US-004-AC-3 (RETIRED): A criterion benchmark `bench_patch_render_fr` averages under 1 ms median for an FR artifact of typical size (verifies NFR-001).
