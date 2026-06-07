# Task 018: Secondary / Fallback Locators

Status: blocked on Task 015

## Scope

Extend the DSL evaluator so a `Locator` may be a fallback chain `[primitive, primitive, ...]` evaluated in order; first non-empty wins. Emit `FallbackLocatorUsed` diagnostic when the canonical (position 0) didn't resolve.

## Subtasks

- [ ] **Locator::Fallback variant.** Already declared in Task 015's design; this task implements the evaluation order.
- [ ] **Evaluation.** Try each primitive in order; on first non-empty result, return it. On all-miss, follow `required` semantics.
- [ ] **Diagnostic.** `FallbackLocatorUsed { key, position, locator_repr }` when position > 0.
- [ ] **Domain object parity.** `domain` object_type from `spec-objects-business` uses fallback chains; parity test against filament-parser-lib reference (TC-113).

## Owns

FR-016 (4 ACs).

## Dependencies

Task 015 (foundation in place; this just extends).

## Unblocks

(Track C tail.)

## Deliverables

- Extensions to `src/extract/locator.rs`

## Primary Tests

TC-110, TC-111, TC-112, TC-113.

## Notes

- This is an independent extension of Task 015 — could be done in parallel with Task 016/017 if more agents are available.
- Reference: `~/dev/spec-objects-business/object_types/.../domain.yaml`.
