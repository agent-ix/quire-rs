---
id: SR-2026111921333400000007
title: "base review of FR-067-AC-9 strict assurance JSON input for PLAT-1795"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs; spec/functional/FR-067-versioned-assurance-export.md; FR-067-AC-9 (PR #707 head 69e98cd04faf604cd4f8fad6fa6579b6f629c502)"
review_set: base
relationships:
  - target: ix://agent-ix/quire-rs/spec/functional/FR-067
    type: reviews
---

## Summary

Ticket: PLAT-1795. PR: quire-rs#707 at head `69e98cd04faf604cd4f8fad6fa6579b6f629c502`, a one-line spec-only diff adding FR-067-AC-9. The criterion's intent is right and needed: the reader previously took serde_json's last-wins value for a duplicate member name, and a refusal plus a no-panic depth bound closes that. But the duplicate-key half names no error outcome, and the depth half hangs its condition on an unnamed parser limit, so the criterion fixes neither the error contract nor the threshold that #706's tests assert.

Examined: FR-067-AC-9 (the new row), plus the AC-ID sequence AC-1 to AC-9, the Verification cell, and FR-067's Behavior section and the AC-4 refusal wording for context. Clean: the ID is sequential and unique in the repo, withdrawn AC-6 is not reused, `Test` is the right verification method for both behaviours, and the criterion specifies no hash, digest, pin or tracking record. `quire validate --scope . spec/functional/FR-067-versioned-assurance-export.md` exits 0. Its only diagnostic is the existing `ac:unclassifiable` warning on withdrawn AC-6 (line 102), and the new line 105 has none.

AssuranceProfile AP-201 (`spec/assurance/AP-201-detection-minting.md`, `review_policy.mode: require`, `operations: [spec-review]`) does not apply. It governs coverage collection, trace binding, row minting, diagnostics and published schemas. AC-9 tightens how the importer parses JSON and does not change the assurance-v1 schema or anything the engine mints or reports. A direct review is therefore permitted.

Verdict: CONDITIONAL. Resolve FND-001 and FND-002 in the AC text before #706 lands against it. FND-003 and FND-004 are optional wording improvements.

## Findings

| ID      | Severity | Summary                                                                                                                                                                                                                         | Refs                                                    |
| ------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------- |
| FND-001 | medium   | The duplicate-key sentence says only "refuses" and names no error. `InvalidJson`, `SchemaViolation` or a new variant would all conform, and it does not say that no typed record is returned, so a test cannot pin the contract #706 implements. | spec/functional/FR-067-versioned-assurance-export.md:105 |
| FND-002 | medium   | The depth sentence is conditioned on "the JSON parser's supported nesting depth", an unnamed dependency default (serde_json's 128). The criterion gives no threshold, and it is vacuously satisfied if that limit is lifted (for example by serde_json `unbounded_depth`), which is exactly when a stack overflow would abort the process. | spec/functional/FR-067-versioned-assurance-export.md:105 |
| FND-003 | low      | AC-9 is compound. Duplicate-member refusal and the over-depth error are independent behaviours with separate inputs and separate tests (#706 tc2248 and tc2249), so one trace tag cannot show which half a test backs.          | spec/functional/FR-067-versioned-assurance-export.md:105 |
| FND-004 | low      | No SHALL in FR-067 Description or Behavior states the strict-input rule. The Behavior section's reader bullets cover schema validation and premise comparison only, so AC-9 verifies a requirement the FR body never makes.      | spec/functional/FR-067-versioned-assurance-export.md:59  |

## Recommendation

Split the row and name the outcome and bound:

- FR-067-AC-9: `read_assurance_export` returns `AssuranceError::InvalidJson`, naming the repeated member, and no typed record when any object at any nesting depth repeats a member name.
- FR-067-AC-10: `read_assurance_export` returns `AssuranceError::InvalidJson` without panicking or aborting for a document nested deeper than 128 arrays or objects.

Add a matching Behavior bullet: "`read_assurance_export` SHALL refuse duplicate object member names and over-deep nesting as invalid JSON before schema validation."

Cross-PR note for #706 (not a finding on this PR): the two tests carry `#[trace("FR-067-AC-4")]` and must move to the new criterion id(s). tc2248 only places the duplicate at the top level (`"format"`), so "at any nesting depth" also needs a nested-duplicate case.

## Dispositions

Round 1 was checked against PR quire-rs#707 head `88f205cddcf505bc20e21c8fd4d19403df76d5b1`. The fix commit is `88f205c` ("PLAT-1795: specify strict reader failures"), and the diff checked runs from `69e98cd` to `88f205c`, touching `spec/functional/FR-067-versioned-assurance-export.md` only. `quire validate --scope . spec/functional/FR-067-versioned-assurance-export.md` exits 0. Its only diagnostic is the existing `ac:unclassifiable` warning on withdrawn AC-6, now at line 104 because the file shifted by two lines; the new rows 107-108 have none. The fix round introduced no new finding. The CR-422 note uses the next unused CR id after CR-421, which is the repo's convention for spec amendments.

| FND     | outcome | reason |
| ------- | ------- | ------ |
| FND-001 | fixed   | AC-9 now names `AssuranceError::InvalidJson`, covers nested objects, and states that no typed record is returned. |
| FND-002 | fixed   | AC-10 and the Behavior section now give an explicit bound of more than 128 nested arrays or objects. It is consistent with serde_json's default limit and does not depend on an unnamed parser default. |
| FND-003 | fixed   | The criterion is split into AC-9 (duplicate member names) and AC-10 (nesting depth). |
| FND-004 | fixed   | The Behavior paragraph now states duplicate-member rejection as a parse stage and adds a SHALL for the over-depth `InvalidJson` result. "SHALL return no typed records until all stages succeed" binds both. |

## Machine-readable disposition record

<!-- reviewer-dispositions repo=agent-ix/quire-rs visibility=public quoin=0.36.8 module=spec-artifacts-process@v0.36.0 id=SR-2026111921333400000007 round=1 model=claude-opus-5-5 run=35ad36bf-89da-43e8-bb4c-855b5ecf2de3 -->

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |

Round 1 disposition for PR quire-rs#707. The reviewed head was
`69e98cd04faf604cd4f8fad6fa6579b6f629c502`; the fixed head is
`88f205cddcf505bc20e21c8fd4d19403df76d5b1`. Validation exits 0 with only
the existing AC-6 warning; the changed AC-9 and AC-10 rows have no diagnostics.
No new findings remain open.

+++ [reviewer data]

```yaml
dispositions:
  - fnd: FND-001
    outcome: fixed
    after_excerpt: |-
      | FR-067-AC-9 | `read_assurance_export` returns `AssuranceError::InvalidJson` for duplicate JSON object member names at any nesting depth, including names in nested objects, and returns no typed record. | Test |
  - fnd: FND-002
    outcome: fixed
    after_excerpt: |-
      | FR-067-AC-10 | JSON input with more than 128 nested arrays or objects returns `AssuranceError::InvalidJson` without panicking. | Test |
  - fnd: FND-003
    outcome: fixed
    after_excerpt: |-
      | FR-067-AC-9 | `read_assurance_export` returns `AssuranceError::InvalidJson` for duplicate JSON object member names at any nesting depth, including names in nested objects, and returns no typed record. | Test |
      | FR-067-AC-10 | JSON input with more than 128 nested arrays or objects returns `AssuranceError::InvalidJson` without panicking. | Test |
  - fnd: FND-004
    outcome: fixed
    after_excerpt: |-
      The v1 reader parses the complete JSON document while rejecting duplicate
      object member names, selects the checked-in v1 schema from `format` and
      `format_version`, validates the complete JSON value, then compares every
      exported module/version tuple with the caller's accepted set. It SHALL reject
      an unknown module as well as a known module with an unaccepted version. The
      reader SHALL return no typed records until all stages succeed. JSON input
      exceeding 128 nested arrays or objects SHALL return
      `AssuranceError::InvalidJson` without panicking.
```

+++
