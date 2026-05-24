# Task 008: schema_for — Surface On-Disk Schemas

Status: blocked on Task 005

## Scope

Expose `quire_rs::schema_for(registry, name) -> Result<&Value, QuireError>` returning the JSON Schema document that was loaded from disk — surfaced unchanged for LLM tool-call consumers and the `apply_patch` validator.

## Subtasks

- [ ] **Storage.** `CompiledArchetype` keeps the raw `serde_json::Value` alongside the compiled validator (separate fields; the raw is for surface, the compiled is for validation).
- [ ] **Surface.** `schema_for(registry, name)` returns a borrow of the raw `Value`. UnknownArchetype on miss.
- [ ] **No-schemars audit.** Confirm `schemars` is NOT a `Cargo.toml` dep (TC-062).

## Owns

FR-003 (4 ACs).

## Dependencies

Task 005 (loader stores the raw schema).

## Unblocks

Task 009 (validator uses the compiled half; this task focuses on the surfacing half), LLM tool-call consumers.

## Deliverables

- A function in `src/lib.rs` (re-exporting from `registry`)
- The `Cargo.toml` dep audit assertion

## Primary Tests

TC-009 (snapshot byte-equal to source file modulo whitespace), TC-009b (unknown archetype), TC-061 (LLM tool round-trip), TC-062 (no schemars dep).

## Notes

- "Byte-equal modulo whitespace" — `serde_json::to_string_pretty` may reorder keys vs. the source file. Either preserve order (use `serde_json::Value` parsing which is order-preserving for maps via `preserve_order` feature) or document the normalization.
- The schema returned is NOT for runtime validation — it's for surfacing. Two distinct purposes; two distinct fields.
