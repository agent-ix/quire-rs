---
id: SR-16310
title: "code-review and rust-review of the spec_validate load-failure report (quire-rs PR 599)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@fcef99e0; examples/spec_validate.rs; Makefile"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-013
    type: references
---

# Code review of the spec_validate load-failure report (quire-rs PR 599)

## Summary

Ticket: PLAT-1091. One code-review pass, with the rust-review lane, over
`git diff origin/main...fcef99e0` (6 files, +70/-5). The reviewer built the
example in its own target dir and ran `cargo test --example spec_validate`
(2 passed) and `cargo test --test it` (540 passed, tc868 included).

Root cause reproduced. `~/dev/spec-artifacts-iso` and
`~/dev/spec-artifacts-process` are on branch `bump-semantic-core-0.3.0`, and
each manifest's `data_schema` carries `{schema, digest}`. Their `origin/main`
manifests carry `{schema}` only. Run against those checkouts, the example now
exits 1 with `schema providers did not load cleanly (22 archetype
failure(s))` and one line per archetype, for example
`spec-artifacts-iso::FR (<root>/spec_artifacts_iso/manifest.yaml):
semantic.data-schema-ambiguous: data_schema carries both the { schema }
reference and schema keywords`. Run against throwaway worktrees of both
providers at `origin/main`, the result is `176 document(s), 0 failed, 43
warning(s), 0 untyped skipped, 9 Phase-7 excluded`.

Examined, clean:

- `load_provider_registry` turns any `Registry::failures()` entry into an
  `Err` that names module, archetype, path and reason. `main` prints it and
  exits 1, so the gate fails loudly instead of reporting "unknown type" for
  each document.
- The check comes after the "no modules found" check and before any
  document is validated. Nothing else in the example changes.
- The test builds two scratch providers, breaks one schema file, and
  asserts that the load is refused and names `spec-artifacts-iso::IsoOnly`.
- The Makefile comment matches `ci.yml`. The workflow checks out both
  providers at their default branch and passes them as
  `VALIDATION_PROCESS_ROOT` / `VALIDATION_ISO_ROOT`.
- No new dependencies, no `unwrap` outside tests, no new public API.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | [test-intent] `archetype_load_failure_is_reported_not_swallowed` asserts only the header text and the `module::Archetype` label. The PR exists to surface the failure reason, but no assertion covers the reason or the path. A change that dropped `f.reason` or `f.path` from the message would still pass. Fix: also assert that the error contains the broken schema's path, or a fragment of the loader's reason for the missing file. | examples/spec_validate.rs:413-443 | correct-requirement-no-evidence |
| FND-002 | low | [other] Two comments are change history, not a description of behaviour. The code comment says "instead of letting 131 documents hide it". That count belongs to one stale local setup and will go stale. The Makefile comment says "now fails with the per-archetype load reason (PLAT-1091), not with one 'unknown type' line per document". Fix: describe the current behaviour only, for example "a provider archetype that fails to load fails the gate and names the archetype and the reason". Drop the count, the ticket id and the "now ... not ..." contrast. | examples/spec_validate.rs:87-89; Makefile:328-330 | implementation-bug-despite-evidence |

## Verdict

Mergeable once the two low findings are fixed. The change is small, correct,
and fixes what PLAT-1091 left: the gate now names the real cause when the
providers do not load.
