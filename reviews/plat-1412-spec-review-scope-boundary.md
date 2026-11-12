---
id: SR-20261103118156
title: "Scope-boundary analysis of FR-082 (quire-rs PR 645)"
type: SpecReview
analysis: scope-boundary
scope: "agent-ix/quire-rs@326eb1bd; spec/functional/FR-082-criticality-trace-column.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-082
    type: references
---

# Scope-boundary analysis of FR-082 (quire-rs PR 645)

## Summary

Ticket: PLAT-1412. This review checks how FR-082 allocates work between the engine (quire-rs) and the `quire trace` renderer (quire-cli, a separate repo). FR-077's Dependencies section already places the trace CLI surface in quire-cli. On quire-cli main there is no FR for `trace` at all: `src/commands/trace.rs` cites quire-rs FR-077 directly. The ticket lists spec-artifacts-iso, quire-rs and quoin as its repos, not quire-cli.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | An engine-repo FR places obligations on a CLI in another repo. The statement says "`quire trace` output SHALL include the column", and FR-082-AC-6 is a `quire trace` table/JSON CLI-snapshot criterion. quire-rs builds no `quire` binary, so no test in this repo can exercise or fail AC-6. A sibling requirement in quire-cli is needed, and none exists or is planned (quire-cli has no trace FR to extend). Fix: keep FR-082 to the engine field. Move AC-6 and the rendering of `unassigned` into a new quire-cli FR, either a `trace` subcommand FR, or a FR-026 `matrix` amendment if the host moves to `coverage_matrix` (SR-20261103118155 FND-001). Add quire-cli to PLAT-1412's repos. | FR-082 statement, FR-082-AC-6, spec/functional/FR-077-trace-search-index.md:93 | wrong-requirement |

## Verdict

FAIL: a sibling quire-cli requirement is required. FR-082 should own only the engine row field. The CLI column, its header, its JSON key and the `unassigned` label belong to quire-cli.
