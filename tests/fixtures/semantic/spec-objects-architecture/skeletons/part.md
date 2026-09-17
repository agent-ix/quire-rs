---
id: scoring_engine
title: "ScoringEngine"
type: part
object: part
---
<!-- part authoring skeleton (spec-objects-architecture). Fill every section
     with substantive content. Contract (manifest body_extraction asserts):
     - Frontmatter MUST carry id, title, type: part, object: part. The
       frontmatter `id` is the part's declaration key; the `title` is its
       display name only (QSpec FR-152, FR-154).
     - "## Part" (H2, required): exactly one table row with the header
       `Owner | Declared Type | Multiplicity` — the owning composite type,
       the part's declared type, and its typed multiplicity (QSpec FR-152).
       Part.json requires all three.
     - A part declares structure, not data or calls: Part.json forbids
       `fields` and `operations`, so there is neither a "## Properties" nor
       an "## Operations" section.
     - Every cell that names another declaration names it by its artifact
       `id` (its identity), never by title (QSpec FR-152: display names are
       not identities). Owners and ends are references, never part of this
       declaration's identity.
     - Keep headings unique per level. -->
# [scoring_engine] ScoringEngine

## Part

| Owner | Declared Type | Multiplicity |
|---|---|---|
| search_service | quant_scoring_engine | 1..1 |
