# PLAT-1774 authored inverse edge target fixture

`failure/SIT-002.md` authors `arises_from` to a `harm`, which must violate the
source's `[hazard]` target list. `control/SIT-002.md` authors the same edge to a
`hazard`, which must pass. The module intentionally declares no forward
`gives_rise_to` allow-list on `harm`; this catches inverse normalization that
would otherwise skip the authored source's target constraint.

`control-inverse/HAZ-001.md` authors the inverse label `gives_rise_to` to a
situation and verifies that the normalized forward allow-list still accepts a
valid inverse edge.

Source: Linear PLAT-1774.
