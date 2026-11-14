---
id: FR-095
title: "Carry Quire-rs release test evidence with the tag attestation"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/FR-093"
    type: "references"
  - target: "ix://agent-ix/quoin/FR-030"
    type: "requires"
  - target: "ix://agent-ix/engineering-assurance/FR-069"
    type: "references"
---

# FR-095: Carry Quire-rs release test evidence with the tag attestation

## Description

FR-095 governs the tag-attestation workflow handoff; it does not change
Quire-rs product, library, CLI, or engine runtime behavior. The committed
release-evidence artifacts are the suite registry and binding registry. CI
produces a fresh per-tag run record as workflow output and retains it only in
that run's `release-evidence` artifact; it does not commit the run record. The
workflow transfers these inputs to the shared release-attestation workflow.
The shared consumer uses this store under
[Engineering Assurance FR-069](ix://agent-ix/engineering-assurance/FR-069)
and writes no Test Result when evidence is absent or unusable.

## Inputs

- A signed `v*` release tag handled by the Quire-rs tag-attestation workflow.
- The release suite `SUITE-001` declared in
  [`spec/evidence/suites.md`](../evidence/suites.md).
- The committed binding registry at
  [`spec/evidence/bindings.json`](../evidence/bindings.json), with the
  schema-version-1 Quoin evidence-store shape defined by
  [Quoin FR-030](ix://agent-ix/quoin/FR-030).

## Outputs

- A Quoin evidence store containing the committed suite and binding registries
  and a fresh `SUITE-001` run record.
- A workflow artifact named `release-evidence` carrying that store.
- A shared Engineering Assurance workflow call with
  `evidence_artifact` equal to `release-evidence`.

## Behavior

- For each pushed `v*` tag, the Quire-rs tag-attestation workflow SHALL run
  `make evidence` for `SUITE-001` at the tagged source revision.
- When `SUITE-001` passes and its run record is written, the Quire-rs
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
  checking out or running Quire-rs repository code.
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
| FR-095-AC-5 | When the shared consumer receives no `release-evidence` artifact or the artifact has no evidence root, it writes no Test Result and no release attestation, and reports an absent-evidence failure. | Test |
| FR-095-AC-6 | When the shared consumer receives an unusable evidence root, including an invalid binding registry or run record, a missing required binding, a failing suite result, or a stale run, it writes no Test Result and no release attestation, and reports the applicable failure. | Test |

## Verification Plan

| Test Case | Expected assertion |
|-----------|--------------------|
| TC-2229 | Parse the tag-attestation workflow and the suite registry. Confirm `SUITE-001` is the release test suite, the evidence job runs `make evidence`, uploads the root as `release-evidence`, and the reusable call receives it only after successful evidence creation. Confirm the signing job runs no repository code. |
| TC-2230 | Invoke the shared consumer without the `release-evidence` artifact and with an artifact lacking the evidence root. Confirm each case reports absent evidence and writes neither a Test Result nor a release attestation. Covers FR-095-AC-5. |
| TC-2231 | Invoke the shared consumer with an invalid binding registry, invalid or missing run record, missing required binding, failing suite result, and stale run. Confirm each case reports the applicable failure and writes neither a Test Result nor a release attestation. Covers FR-095-AC-6. |

## Dependencies

- **Upstream**: [FR-093](./FR-093-tag-attestation-preserves-manual-wheel-release.md), [Quoin FR-030](ix://agent-ix/quoin/FR-030) (evidence-store binding registry and run-record format), and [Engineering Assurance FR-069](ix://agent-ix/engineering-assurance/FR-069) (consumer rejection behavior).
