# Render Parity Divergences (FR-012 / Gate G2)

Intentional byte-level differences between `quire_rs::render` output and
the Python Jinja2 reference renderer. Each entry needs an explicit
StR-002-AC-2 note before the parity suite can be relaxed for that case.

## Status: v1 target zero divergences

None documented at v1 authoring time. The bootstrap `demo` archetype
(see `modules/demo/`) is hand-authored so the harness has something to
chew on; adding an entry here requires that:

- The divergence is *demonstrably* a Python/MiniJinja dialect issue,
  not a quire-rs bug.
- A test case captures the divergence in `cases/` with a `divergent-`
  filename prefix.
- The CI parity job skips the prefix-`divergent-` cases by default,
  but the divergent case still runs in `--nocapture` mode in a
  dedicated weekly job.
