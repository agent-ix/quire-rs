---
id: SR-20261112205307
title: "spec-review of the FR-049 / FR-069 edits for PLAT-1491 and PLAT-1484 (quire-rs PR 660)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@a1203093; spec/functional/FR-049-verification-reference-integrity.md (Several tables under one declared section, AC-23..26); spec/functional/FR-069-semantic-module-contract-at-load.md (record_validation null paragraph, AC-29); spec/tests.md (TC-2318..TC-2322, audit rows)"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-049", type: references}
  - {target: "ix://agent-ix/quire-rs/FR-069", type: references}
---

## Summary

Tickets: PLAT-1491 and PLAT-1484. This spec-review covers integrity and EARS on the edited requirement text. The repo's own spec gate was run on the PR head: `examples/spec_validate` with spec-artifacts-process and spec-artifacts-iso on `main`. It reports `205 document(s), 1 failed`. The one failure is FR-049, at the new AC-23 row. With FR-049 restored to main and the rest of the head kept, the only errors are the expected missing-AC audit rows, so the failure comes from this PR. The new ids are unique: TC-2318..2322, FR-049-AC-23..26 and FR-069-AC-29 appear in no other open quire-rs PR and not on main.

## Verdict

**FAIL.** The PR leaves the spec gate red (FND-001). Apart from that, the decision text is clear: why a reference declaration reads every table while FR-076 refuses a second one. Each AC is singular and testable. The AC→TC audit rows exist for every new AC (FR-050-AC-58). The FR-069 paragraph keeps the refusal order explicit. The false "remaining optional objects" sentence is filed as FND-001 of the gap-analysis SR and is not duplicated here.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The FR-049-AC-23 row embeds unescaped pipes: `K-1 \| north \| L-1`, `K-2 \| north \| R-1` and `K-3 \| north \| L-7` written raw. The acceptance-criteria table row therefore parses as 9 cells against a 3-column header. `spec_validate` fails with "FR-049: line 183: [FR] 'acceptance_criteria_table': FR-049-AC-23: table row has 9 cell(s) but the header declares 3", so `make validate` (part of `make ci`) is red. The row also renders broken on GitHub. Fix: escape the pipes (`\|`) or describe the rows without pipe characters, then re-run `make validate`. | spec/functional/FR-049-verification-reference-integrity.md:183 |
