---
id: planner_to_scoring
title: "PlannerToScoringConnection"
type: connection
object: connection
---
<!-- connection authoring skeleton (spec-objects-architecture). Fill every
     section with substantive content. Contract (manifest body_extraction
     asserts):
     - Frontmatter MUST carry id, title, type: connection, object:
       connection. The frontmatter `id` is the connection's declaration key
       (QSpec FR-152, FR-154).
     - "## Connection" (H2, required): exactly one table row with the header
       `Source | Source Multiplicity | Target | Target Multiplicity |
       Direction` — each end names a `port` declaration and may state its
       multiplicity (leave the cell empty when the end declares none), and
       the flow direction is `source-to-target`, `target-to-source` or
       `bidirectional`. Connection.json requires `sourceEnd`, `targetEnd`
       and `flowDirection`; an end requires its port and admits an optional
       multiplicity.
     - A connection declares structure, not data or calls: Connection.json
       forbids `fields` and `operations`.
     - Every cell that names another declaration names it by its artifact
       `id` (its identity), never by title (QSpec FR-152: display names are
       not identities). Owners and ends are references, never part of this
       declaration's identity.
     - Keep headings unique per level. -->
# [planner_to_scoring] PlannerToScoringConnection

## Connection

| Source | Source Multiplicity | Target | Target Multiplicity | Direction |
|---|---|---|---|---|
| planner_query_out | 1..1 | score_query_in | 0..* | source-to-target |
