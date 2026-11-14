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
| FR-093-AC-1 | A repository-local workflow test confirms that `.github/workflows/wheels.yml` retains its `workflow_dispatch` publish input (default `false`) and permits `workflow_call` for the additive tag-attestation build, while its publish job runs only for manual dispatch with `inputs.publish` true. It confirms that `.github/workflows/tag-attestation.yml` is triggered only by pushes of `v*` tags, calls the wheel workflow with `publish: false`, and calls the shared Engineering Assurance release-attestation workflow. | Test |

> **CR-420 note (EA-110):** FR-093 adds signed-tag attestation while keeping manual wheel publication as the only publishing path. The tag caller invokes the wheel workflow with `publish: false` and the shared Engineering Assurance attestation workflow.

## Dependencies

The wheel workflow is the only publisher of Quire-rs release wheels. The
tag-attestation caller may reuse its build but SHALL not invoke the publishing
path.

## Verification Plan

| Test Case | Expected assertion |
|-----------|--------------------|
| TC-2228 | Parse both repository workflow files. Assert that `wheels.yml` retains `workflow_dispatch` with `publish` defaulting to `false`, accepts `workflow_call`, and gates its publish job on both `github.event_name == 'workflow_dispatch'` and `inputs.publish`. Assert that `tag-attestation.yml` is triggered only by `push` of `v*` tags, calls `wheels.yml` with `publish: false`, and invokes the shared Engineering Assurance release-attestation workflow. |
