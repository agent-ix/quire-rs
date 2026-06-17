---
id: ADR-0007
title: "Internal references are relative-path links; ix:// is external-only"
type: ADR
---

# ADR 0007: Internal references are relative-path links; `ix://` is external-only

**Status**: Accepted
**Date**: 2026-06-17
**Decision authority**: kreneskyp

## Context

Authored specs reference other artifacts two ways today. Frontmatter
`relationships[].target` and body links carry the custom `ix://org/repo/name`
URI; quire-rs harvests both into the edge set (FR-026). But the overwhelmingly
common case — an FR citing a sibling `FR-002`, an `umbrella FR-008-CON-4`, the
ids in an Acceptance-Criteria table — is **bare prose text**. Those references
produce no edge.

To build the Open Knowledge Format (OKF) graph from bare prose, a consumer would
have to *scan prose at runtime* with a `FR-\d+`-style regex and guess which
tokens are references. That heuristic is fragile (false positives in fenced code,
examples, and prose like "the FR-NNN format") and lossy. We want intra-bundle
references to be **real, explicit links**, so the graph is read directly from
link destinations with no runtime guessing — improving OKF compliance.

The open question was the *form* of an internal link, given that `ix://` already
exists. `ix://` is a fully-qualified `org/repo/name` address: correct for
**external / cross-repo** references, but heavyweight and redundant for a
reference to a sibling in the same bundle.

## Decision

1. **Internal references are relative file-path Markdown links** —
   `[FR-002](./FR-002-graph-edges.md)`. They render and navigate in GitHub and
   editors today, and quire-rs resolves them to the target artifact via a
   path→id map over the loaded corpus.

2. **`ix://` is retained for external / cross-repo references only**
   (`[Order](ix://agent-ix/spec-core-service/Order)`, frontmatter
   `relationships` to other repos). Same-bundle objects/artifacts use the
   relative-path form.

3. **Relative-path links become a third edge source** in resolution (FR-026),
   alongside frontmatter `relationships` and body `ix://` links, contributing
   `references` edges. `index.md` / `log.md` are navigation documents — their
   wall-to-wall relative links do **not** flood the graph with `references`
   edges.

4. **Bare prose codes are never harvested.** We deliberately do not scan prose
   for `FR-\d+` tokens to synthesize edges — that is the heuristic this ADR
   removes. A bare code only becomes an edge after it is rewritten as an explicit
   link.

5. **Unlinked bare codes are an advisory warning, not an error** (FR-039). The
   validator classifies each bare artifact-id token into auto-fix / warn-only /
   ignore and, for the auto-fix bucket, emits the exact relative-path link it
   would apply, so an opt-in autofix can apply it deterministically. Warning
   posture (never blocking) mirrors the dangling-reference treatment under
   `Okf`.

## Consequences

- FR-026 gains a relative-path edge source and a path→id index; its dedup and
  determinism contracts extend to the new source. FR-038's existing Okf prose
  ("broken `ix://` / relative references degrade to warnings") is now backed by
  an actual relative-reference harvest rather than anticipated.
- A new FR-039 specifies unlinked-reference detection, the three-bucket
  classification, and the suggested-fix string; the apply step (byte-splice
  writeback) is surfaced by quire-cli, never auto-run.
- Authoring skeletons and skills emit relative-path links for siblings and
  `ix://` for cross-repo, so new specs are born linked.
- Migration of existing specs is via the opt-in autofix plus hand-review;
  bundles migrate organically. Cross-repo `ix://` relationships are untouched.
- Any future intra-document anchor addressing (linking `FR-024-AC-1` to a row
  anchor rather than the parent file) amends this ADR.
