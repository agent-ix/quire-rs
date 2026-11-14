---
id: FR-093
title: "Tag attestation preserves the manual wheel release"
type: FR
---

# FR-093: Tag attestation preserves the manual wheel release

## Description

When a `v*` tag is pushed, the Quire-rs release-CI workflow SHALL call the
shared Engineering Assurance release-attestation workflow alongside the
existing manually dispatched wheel release. The wheel-release workflow SHALL
retain its manual `workflow_dispatch` publishing path. The tag-attestation
workflow SHALL not publish wheels.

FR-093-AC-1..6 concern the Quire-rs repository's release-CI workflow only: the
tag workflow calls the shared Engineering Assurance release-attestation
workflow, while wheel publication remains manual-only. The quire-rs library
and engine do not create, sign, or retain attestations or run records.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-093-AC-1 | `.github/workflows/wheels.yml` declares `workflow_dispatch` with a `publish` input whose default is `false`. | Test (TC-2258) |
| FR-093-AC-2 | `.github/workflows/wheels.yml` accepts `workflow_call` with a Boolean `publish` input whose default is `false`. | Test (TC-2259) |
| FR-093-AC-3 | The `publish` job in `.github/workflows/wheels.yml` runs only when the event is `workflow_dispatch` and `inputs.publish` is `true`. | Test (TC-2260) |
| FR-093-AC-4 | `.github/workflows/tag-attestation.yml` is triggered only by pushes of `v*` tags. | Test (TC-2261) |
| FR-093-AC-5 | The tag-attestation workflow calls `.github/workflows/wheels.yml` with `publish: false`. | Test (TC-2262) |
| FR-093-AC-6 | The tag-attestation workflow calls the shared Engineering Assurance release-attestation workflow. | Test (TC-2263) |

> **CR-420 note (EA-110):** FR-093 adds signed-tag attestation while keeping manual wheel publication as the only publishing path. The tag caller invokes the wheel workflow with `publish: false` and the shared Engineering Assurance attestation workflow.

## Dependencies

The wheel workflow is the only publisher of Quire-rs release wheels. The
tag-attestation caller may reuse its build but SHALL not invoke the publishing
path.

## Verification Plan

| Test Case | Expected assertion |
|-----------|--------------------|
| TC-2258 | Parse `wheels.yml`; assert that `workflow_dispatch` has a `publish` input defaulting to `false`. |
| TC-2259 | Parse `wheels.yml`; assert that `workflow_call` has a Boolean `publish` input defaulting to `false`. |
| TC-2260 | Parse `wheels.yml`; assert that the publish job requires both `github.event_name == 'workflow_dispatch'` and `inputs.publish`. |
| TC-2261 | Parse `tag-attestation.yml`; assert that its only trigger is `push` for `v*` tags. |
| TC-2262 | Parse `tag-attestation.yml`; assert that its build job calls `wheels.yml` with `publish: false`. |
| TC-2263 | Parse `tag-attestation.yml`; assert that its attest job calls the shared Engineering Assurance release-attestation workflow. |
