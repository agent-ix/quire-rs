# Task 038: validate_document PyO3 Binding

Status: complete

## Scope

Expose `validate_document` through the `quire` Python wheel (FR-032 surface) so
quire-cli, filament-parser, and other Python/wasm consumers call the same engine
function. First-party `src/python/` stays `unsafe`-free.

## Subtasks

- [ ] **Binding.** `src/python/mod.rs`: `validate_document(archetype_name: str, module_root: str, document_text: str) -> dict` returning `{is_valid: bool, errors: [{message, line, reason}]}`. Resolve the archetype from a loaded `Registry` (mirror the existing `validate`/`extract` binding pattern).
- [ ] **Error mapping.** Map `ValidationError` reasons into the dict; unknown archetype → `UnknownArchetype` exception (parity with `validate`).
- [ ] **Feature gating.** Behind the `python` feature; default build stays interpreter-free.

## Owns

FR-032 (binding surface AC — the Python entry of validate_document).

## Dependencies

Task 036 (validate_document).

## Unblocks

quire-cli `validate <md>` (consumes the wheel), filament-parser re-point.

## Deliverables

- `src/python/mod.rs` (+ stub/typing surface if the repo maintains one).

## Primary Tests

TC-533 (binding-layer happy+sad), plus a Python smoke test mirroring TC-528/TC-529.

## Notes

`maturin` build must stay green; keep first-party python `unsafe`-free
(NFR-003-AC-4). ABI3 wheel.
