# PLAT-1689 controlled architecture fixture

`failure/spec/design/order_schema.md` is byte-identical to
`spec-objects-architecture/tests/fixtures/design_edges/valid/spec/design/order_schema.md`.
It retains the authored `realizes_requirement` relation, frontmatter URI,
properties table, and JSON schema. `failure/spec/functional/FR-004.md` provides
the resolved registered FR target from the valid fixture shape.

The healthy `control` retains the exact order schema and gives the resolved
target a registered `object: data_schema` while preserving `type: FR`. This
satisfies the source's synthesized `references` vocabulary and its authored
`realizes_requirement` edge. The integration test
`architecture_order_schema_synthetic_reference_is_not_checked` runs both
members against the engine and asserts that neither reports a target-kind
finding.

Language coverage: `n/a` applies because this is Markdown bundle edge resolution,
not a source-language parser. Rust, Python, and TypeScript parser paths are out
of scope because they cannot produce this Markdown relationship-resolution
behavior.
