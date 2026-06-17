# Render Parity Notes

Documents any deviations between `quire-rs::render` output and the Python Jinja2 reference renderer (spec-artifacts-iso/app/process), per [StR-002-AC-2](../stakeholder/StR-002-render-parity-with-python.md).

The render-parity test suite (`tests/render_parity/`, [FR-012](../functional/FR-012-archetype-parity-suite.md)) asserts byte equality. Any divergence MUST be entered here with rationale before the suite can be relaxed for that case.

## Known divergences

_None at v1 authoring time. Populate as the parity suite is built out._

## Whitespace handling

MiniJinja and Jinja2 have substantively the same whitespace control behavior (`{%-` / `-%}`, `trim_blocks`, `lstrip_blocks`). Empty conditional blocks (`{% if x %}{% endif %}` where x is false) MAY produce different trailing-newline behavior — flag here if encountered.

## Filter behavior

v1 ships no custom MiniJinja filters (per [FR-004](../functional/FR-004-minijinja-strict-environment.md)). Only built-in filters (`default`, `upper`, `lower`, `replace`, etc.) are available. If a template under spec-artifacts-* uses a filter MiniJinja does not implement, the divergence is flagged here and either:
- the template is rewritten to use a supported filter (preferred), or
- the gap is filed against MiniJinja upstream.

## Number formatting

Locale-aware number formatting is NOT used in spec-artifacts-* templates at the time of authoring. If introduced, behavior must be pinned to the C locale.
