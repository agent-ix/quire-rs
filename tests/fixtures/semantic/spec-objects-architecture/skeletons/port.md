---
id: score_query_in
title: "ScoreQueryPort"
type: port
object: port
---
<!-- port authoring skeleton (spec-objects-architecture). Fill every section
     with substantive content. Contract (manifest body_extraction asserts):
     - Frontmatter MUST carry id, title, type: port, object: port. The
       frontmatter `id` is the port's declaration key; the `title` is its
       display name only (QSpec FR-152, FR-154).
     - "## Port" (H2, required): exactly one table row with the header
       `Owner | Direction | Interface | Multiplicity` — the owning part, the
       direction (`in`, `out` or `inout`), the interface type (an
       `interface` declaration) and the typed multiplicity (QSpec FR-152).
       Port.json requires all four.
     - A port declares structure, not data or calls: Port.json forbids
       `fields` and `operations`.
     - Every cell that names another declaration names it by its artifact
       `id` (its identity), never by title (QSpec FR-152: display names are
       not identities). Owners and ends are references, never part of this
       declaration's identity.
     - Keep headings unique per level. -->
# [score_query_in] ScoreQueryPort

## Port

| Owner | Direction | Interface | Multiplicity |
|---|---|---|---|
| scoring_engine | in | quant_codec | 1..1 |
