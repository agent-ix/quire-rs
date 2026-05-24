# Task 004: Parser Parity Gate (G1)

Status: blocked on Task 003

## Scope

Transliterate the TS reference test suite from `~/dev/quire/tests/` and the Python suite from `~/dev/quire-py/tests/` into Rust integration tests. **This is Quality Gate G1** — no downstream work proceeds until these pass.

## Subtasks

- [ ] **Inventory.** List every test in both reference suites; categorize by feature (frontmatter / headings / fences / slicing / IDs).
- [ ] **Fixture port.** Copy input markdown + expected output JSON to `tests/parser_parity/fixtures/`.
- [ ] **Test transliteration.** One Rust test per TS test. Each loads the fixture, parses with `parse_document`, asserts the expected `QuireDocument`.
- [ ] **Quire-py structural equivalence (TC-021).** Pick a corpus of ~50 real markdown files (sampled from `spec-artifacts-*` and `ix-spec-objects`). Run quire-py + quire-rs; assert equivalent doc structure.

## Owns

Gate G1 evidence (TC-020, TC-021).

## Dependencies

Task 003 (full parser surface). Task 001 + 002 implicit.

## Unblocks

Task 005 (loader) and everything downstream. **Until this gate passes, do not start any other task on the critical path.**

## Deliverables

- `tests/parser_parity/` directory
- `tests/parser_parity/fixtures/*.md` + `.expected.json`
- A pytest-style runner that enumerates fixtures
- Gate G1 status: **Pass** / **Fail** entry in `plan/plan.md`

## Primary Tests

TC-020, TC-021.

## Notes

- If any TS fixture doesn't transliterate cleanly (e.g. JS-specific escaping), document the divergence in `tests/parser_parity/divergences.md`. Don't silently relax.
- Performance is not checked here — this gate is correctness only.
