---
id: FR-095
title: "Carry Quire-rs release test evidence with the tag attestation"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/FR-093"
    type: "references"
  - target: "ix://agent-ix/engineering-assurance/FR-069"
    type: "references"
---

# FR-095: Carry Quire-rs release test evidence with the tag attestation

## Description

Quire-rs SHALL retain the result of its declared release-verification test
suite in a Quoin evidence store and transfer that store to the shared
release-attestation workflow. The suite and binding registries are committed
with the source; each tag build creates a fresh run record and keeps it only in
the `release-evidence` workflow artifact. The shared consumer uses this store
under [Engineering Assurance FR-069](ix://agent-ix/engineering-assurance/FR-069)
and writes no Test Result when evidence is absent or unusable.

## Inputs

- A signed `v*` release tag handled by the Quire-rs tag-attestation workflow.
- The release suite `SUITE-001` declared in
  [`spec/evidence/suites.md`](../evidence/suites.md).

## Outputs

- A Quoin evidence store containing the committed suite and binding registries
  and a fresh `SUITE-001` run record.
- A workflow artifact named `release-evidence` carrying that store.
- A shared Engineering Assurance workflow call with
  `evidence_artifact` equal to `release-evidence`.

## Behavior

- For each pushed `v*` tag, the Quire-rs tag-attestation workflow SHALL run
  `make evidence` for `SUITE-001` at the tagged source revision.
- After `SUITE-001` passes and its run record is written, the Quire-rs
  tag-attestation workflow SHALL upload the committed suite and binding
  registries and fresh run record as `release-evidence`.
- The Quire-rs tag-attestation workflow SHALL pass
  `evidence_artifact: release-evidence` to the shared Engineering Assurance
  workflow.
- The Quire-rs tag-attestation workflow SHALL pass `publish: false` to the
  wheel workflow.
- The Quire-rs tag-attestation workflow SHALL preserve the manual
  `workflow_dispatch` wheel release path ([FR-093](./FR-093-tag-attestation-preserves-manual-wheel-release.md)).
- The OIDC-enabled attestation job SHALL consume the evidence artifact without
  checking out or running Quire-rs repository code, in accordance with
  NFR-011.
- If the suite fails, no run record is available, or the evidence artifact
  cannot be uploaded, the workflow SHALL fail before the reusable attestation
  call.
- The tag-attestation workflow SHALL NOT modify the committed suite or binding
  registries and SHALL NOT commit generated run records.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-095-AC-1 | A workflow test confirms that the Quire-rs `v*` tag-attestation workflow runs `make evidence` for the `SUITE-001` in its committed suite registry and uploads `spec/evidence/` as `release-evidence` only after the suite succeeds. | Test |
| FR-095-AC-2 | A workflow test confirms that the shared Engineering Assurance call receives `evidence_artifact: release-evidence`, waits for the evidence job, and cannot start after suite or upload failure. | Test |
| FR-095-AC-3 | A workflow test confirms that the OIDC-enabled job consumes the artifact and has no checkout or repository-code execution step, while the wheel workflow still receives `publish: false`. | Test |
| FR-095-AC-4 | After a tag-attestation run, the uploaded artifact contains the committed suite and binding registries plus the fresh `SUITE-001` run record; that run record is not committed to the repository. | Test |

## Verification Plan

| Test Case | Expected assertion |
|-----------|--------------------|
| TC-2229 | Parse the tag-attestation workflow and the suite registry. Confirm `SUITE-001` is the release test suite, the evidence job runs `make evidence`, uploads the root as `release-evidence`, and the reusable call receives it only after successful evidence creation. Confirm the signing job runs no repository code. |

## Dependencies

- **Upstream**: [FR-093](./FR-093-tag-attestation-preserves-manual-wheel-release.md) and [Engineering Assurance FR-069](ix://agent-ix/engineering-assurance/FR-069).
