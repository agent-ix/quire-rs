---
id: FR-012
title: "Ten-Archetype Render Parity Suite"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-005"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-002"
    type: "implements"
    cardinality: "1:1"
---

## Behavior

The repository SHALL ship a `tests/render_parity/` directory containing, for each archetype, at least one `(input.json, expected.md)` fixture pair. The expected output is produced by running the Python Jinja2 reference renderer in `spec-artifacts-iso` or `spec-artifacts-app` on the same input.

A single test runner `cargo test --test render_parity` SHALL:

1. Enumerate every fixture pair.
2. For each pair: deserialize `input.json` into the archetype's typed struct, run `quire_rs::render(block_type, ...)`, compare the result byte-for-byte against `expected.md`.
3. Report any mismatch with a diff.
4. Fail the test run if any mismatch exists.

The archetypes covered SHALL be exactly the ten listed in master spec § 8.3:

ISO: FR, NFR, StR, US, IT, TC, AC, CON
App: ApplicationSpec, MasterRequirements

## Acceptance

- **FR-012-AC-1**: Every one of the 10 archetypes has at least one `(input.json, expected.md)` fixture pair under `tests/render_parity/`.
- **FR-012-AC-2**: `cargo test --test render_parity` reports 10+ passing assertions.
- **FR-012-AC-3**: A regression test: temporarily mutate a template (e.g. change `{{ id }}` to `{{ id }}!`) and confirm the parity suite catches the divergence.
- **FR-012-AC-4**: The fixture-regeneration script `scripts/regenerate_parity_fixtures.sh` runs the Python reference renderer against each `input.json` and updates the `expected.md`. The script is invoked manually only when the Python reference intentionally changes.
