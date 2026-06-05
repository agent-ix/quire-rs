# Task 012: Render Parity Gate (G2)

Status: **RETIRED** (render removal — 2026-06-04)

> The render/templating feature is removed (no backward-compatibility layer). This
> task and Quality Gate **G2** (render parity) are retired. Block round-trip
> integrity is now gated by **G4** (reframed to byte-splice-only). See `spec.md`
> §2bis and the retired FR-012. Kept for history.

Original status: blocked on Task 011

## Scope

**Quality Gate G2.** Confirm the harness from Task 011 produces byte-equal output to the Python Jinja2 reference for the FR archetype. **Until this passes, do not scale to remaining 16 archetypes (Task 013) and do not start Track C (DSL + extract).**

## Subtasks

- [ ] **Run the harness** for FR archetype. Compare byte-for-byte.
- [ ] **Diff diagnosis if fails.** Likely root causes:
  - Template parse difference (MiniJinja vs Jinja2 dialect drift)
  - Strict-undefined catching a field Python's Jinja2 silently ignored
  - JSON merge edge case (array replacement semantics)
  - Validator state coupling polluting render context
  - YAML serialization quirks (key order in frontmatter)
- [ ] **Document known divergences** in `tests/render_parity/divergences.md` if any are unavoidable. Each divergence requires an explicit StR-002-AC-2 note.
- [ ] **Update plan/plan.md Gate G2 status: Pass / Fail.**

## Owns

Gate G2 evidence (TC-030 for FR archetype).

## Dependencies

Task 011.

## Unblocks

Task 013 (full parity sweep) — if Pass. Tracks C tasks (DSL) become safe to start.

## Deliverables

- Gate G2 status update in `plan/plan.md`
- `tests/render_parity/divergences.md` (may be empty)

## Primary Tests

TC-030 (FR archetype slice).

## Notes

- This gate's purpose is to catch a structural mismatch BEFORE building 16 more archetype fixtures and the entire extract stack on top. A single archetype's worth of work is cheap; full corpus + extract is expensive.
- If MiniJinja → Jinja2 incompatibility is the root cause, document it; do not silently use Tera or another engine without ADR.
