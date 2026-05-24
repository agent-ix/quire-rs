# External Blockers

Tracks dependencies on external systems or repos whose state affects when `quire-rs` can ship.

## ix-cli sync counterparty

- **Status**: not yet built for the Filament → disk sync described in spec.md §17
- **Affects**: integration testing of the full Filament → ix-cli → quire-rs path
- **Workaround**: hand-author archetypes into `~/.ix/schemas/` (or git-vendor); `quire-rs` works correctly against any on-disk state regardless of how it got there
- **Resolution**: tracked separately in the ix-cli repo

## Python Jinja2 reference renderer pinning

- **Status**: spec-artifacts-iso/app/process do not pin Jinja2 version
- **Affects**: byte-parity tests (FR-012) — a future Jinja2 minor bump in spec-artifacts-iso could silently change whitespace
- **Workaround**: `scripts/parity-venv.txt` pins Jinja2 for `quire-rs`'s parity-fixture regeneration; `tests/render_parity/PROVENANCE.md` records the venv used
- **Resolution**: ideally spec-artifacts-iso pins Jinja2 in its own `poetry.lock`; until then, our pin is the local source of truth

## filament-parser-lib API stability

- **Status**: filament-parser-lib is the Python parser being eventually superseded by `quire-rs`. Recent commits ("reconcile body sections with shipped code") show ongoing schema evolution
- **Affects**: parity tests TC-040 (extract DSL parity), TC-104 (edge harvest parity)
- **Workaround**: pin to a specific filament-parser-lib commit when running parity tests
- **Resolution**: track in render-parity-notes.md if behavior diverges

## Validator crate choice

- **Status**: pending bench-driven ADR (NFR-009-AC-2; see `spec/assets/adr/0001-validator-crate.md`)
- **Affects**: NFR-001 (render <1ms), FR-002 (validation pipeline)
- **Resolution**: decided at Task 005 implementation start

## Windows support

- **Status**: declared out of scope v1 in spec.md §2.2
- **Affects**: nothing for v1
- **Resolution**: revisit for v1.1 if requested
