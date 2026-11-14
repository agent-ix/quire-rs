---
id: FR-093
title: "Tag attestation preserves the manual wheel release"
type: FR
---

# FR-093: Tag attestation preserves the manual wheel release

## Description

Quire-rs SHALL add release-tag attestation alongside its existing manually
dispatched wheel release. Adding the tag-attestation caller SHALL leave the
manual release controls intact, and the attestation build SHALL be unable to
publish wheels.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-093-AC-1 | A repository-local workflow test confirms that `.github/workflows/wheels.yml` retains its `workflow_dispatch` trigger and `publish` input gate on all wheel publication steps, while `.github/workflows/tag-attestation.yml` handles `v*` tag pushes by calling the wheel workflow with `publish: false`. | Test |

## Dependencies

The wheel workflow is the only publisher of Quire-rs release wheels. The
tag-attestation caller may reuse its build but SHALL not invoke the publishing
path.

## Verification Plan

| Test Case | Expected assertion |
|-----------|--------------------|
| TC-2228 | Parse both repository workflow files. Assert that manual dispatch and its `publish` input gate remain on the wheel workflow's publication steps, and that the tag-attestation workflow passes `publish: false` to the reusable wheel workflow. |
