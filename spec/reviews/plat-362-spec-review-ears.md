---
id: SR-16361
title: "EARS review of PLAT-362 shell symbol adapter (quire-rs PR 614)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md (PR 614 diff only, head 63ac3f0)"
review_set: subset
---

## Summary

Ticket: PLAT-362. This pass checked the new Shell adapter statement and FR-051-AC-34..40 for EARS form, modality and atomicity. The opening sentence is a well-formed ubiquitous SHALL. Two normative claims after it have no modal. Three ACs bundle behaviours that have different triggers. One AC clause cannot be parsed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | Two normative claims in the Shell adapter section use no SHALL: "The adapter mints two kinds of symbol" and "An `entry_point` binds `verifies` ids ... it does not carry `implements` ... A shell function ... binds nothing". These are the rules AC-35 and AC-36 test, so they belong in the requirement's modal voice. | spec/functional/FR-051-source-symbol-extraction.md:88-104 |
| FND-002 | low | AC-38 is compound: it bundles marker-only binding, no `legacy` provenance, zero relations without a declared form, and refusal of `entry_paths` at load. The last is a module-load behaviour with a different trigger. AC-39 likewise bundles a malformed-input diagnostic with a no-execution property. Each AC should hold one behaviour. | spec/functional/FR-051-source-symbol-extraction.md:205-206 |
| FND-003 | low | AC-34's clause "a file with another extension, and a `.sh` file in a tree, never changes how a `.rs`/`.py`/`.ts` file extracts" does not parse. It also leaves open whether other shell spellings are read, such as `.bash` or an extension-less file with a `#!/bin/sh` line. | spec/functional/FR-051-source-symbol-extraction.md:201 |

## Verdict

Examined: the Shell adapter section and FR-051-AC-34..40. AC-35, AC-36, AC-37 and AC-40 use event and state forms that a test can fail. The findings are wording only. They change no behaviour.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | "The adapter SHALL mint two kinds of symbol", and the extractor "SHALL mint it without consulting module data". The rules AC-35 and AC-37 test are now modal. |
| FND-002 | fixed | Old AC-38 is split into AC-38 (eligibility), AC-39 (glob semantics), AC-42 (marker-only binding) and AC-43 (load-time refusal). Old AC-39 is split into AC-45 (unparseable-file diagnostic), and its no-execution clause moved to FR-051-CON-1. |
| FND-003 | fixed | AC-34 is rewritten as one event clause, and it states that `.bash` and extension-less `#!/bin/sh` files are not read. |
