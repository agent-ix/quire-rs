---
id: batch_scoring_on_engine
title: "BatchScoringAllocation"
type: allocation
object: allocation
---
<!-- allocation authoring skeleton (spec-objects-architecture). Fill every
     section with substantive content. Contract (manifest body_extraction
     asserts):
     - Frontmatter MUST carry id, title, type: allocation, object:
       allocation. The frontmatter `id` is the allocation's declaration key
       (QSpec FR-152, FR-154).
     - "## Allocation" (H2, required): exactly one table row with the header
       `Source | Target` — the source element (a part, a port, or an
       operation named `<interface id>/<operation>`) and the target element (a
       part). Allocation.json requires both.
     - An allocation declares structure, not data or calls: Allocation.json
       forbids `fields` and `operations`.
     - Every cell that names another declaration names it by its artifact
       `id` (its identity), never by title (QSpec FR-152: display names are
       not identities). Owners and ends are references, never part of this
       declaration's identity.
     - Keep headings unique per level. -->
# [batch_scoring_on_engine] BatchScoringAllocation

## Allocation

| Source | Target |
|---|---|
| quant_codec/score_ip_batch | scoring_engine |
