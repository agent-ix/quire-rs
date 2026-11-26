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
Quire-rs product, library, CLI, or engine runtime behavior. Terra runs
`SUITE-001` once at the exact release tag commit and packages the committed
suite registry, committed and generated binding registries, and fresh run
record as `release-evidence.tar.gz`. That archive is attached to the matching
GitHub Release. The tag workflow does not execute the suite; it validates the
attached bundle against the tag and source commit, retains it as a same-run
`release-evidence` Actions artifact, and transfers that artifact to the shared
release-attestation workflow. The shared consumer uses this store under
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
- A `release-evidence.tar.gz` asset attached to the matching GitHub Release,
  produced by one successful `make evidence` run on Terra at the tag commit.

## Outputs

- A Quoin evidence store containing the committed suite and binding registries
  and a fresh `SUITE-001` run record.
- A workflow artifact named `release-evidence` carrying that store.
- A shared Engineering Assurance workflow call with
  `evidence_artifact` equal to `release-evidence`.

## Behavior

- For each pushed `v*` tag, the Quire-rs tag-attestation workflow SHALL wait
  for `release-evidence.tar.gz` to be attached to the matching GitHub Release;
  it SHALL fail closed if the asset is absent after a 60-minute bounded wait.
- `make evidence` SHALL run `SUITE-001` only when `HEAD` is exactly the
  requested `vMAJOR.MINOR.PATCH` tag's commit. It SHALL write the fresh Quoin
  run record, restore the committed binding registry in the source worktree,
  and package the committed suite and binding registries with the generated
  bindings and run record as `target/release-evidence/release-evidence.tar.gz`.
- The tag-attestation workflow SHALL validate the archive's safe file set,
  manifest tag, full commit SHA, suite and run record; compare the suite and
  committed binding registries byte-for-byte with the tagged source; and
  reject stale, empty, failed, or wrong-commit evidence before uploading it as
  the same-run `release-evidence` artifact.
- The tag-attestation workflow SHALL NOT run `make evidence` or otherwise
  execute `SUITE-001`; the Terra run is the one full suite for the release.
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

> **CR-426 note (2026-11-26, PLAT-1934):** The tag workflow originally reran `SUITE-001`
> on a GitHub-hosted runner after the required Terra release run, producing two
> full-suite runs for one release. The tag workflow now waits for the single
> Terra-produced bundle attached to the matching GitHub Release, binds it to
> the tag's full commit SHA, and republishes only the verified evidence store
> as a same-run Actions artifact for the existing fail-closed EA consumer.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-095-AC-1 | A workflow test confirms that `make evidence` requires the exact `v*` tag commit and creates a bundle from one successful `SUITE-001` run, while the tag-attestation workflow waits for the attached release asset and does not rerun the suite. | Test |
| FR-095-AC-2 | A workflow test confirms that the tag-attestation job rejects a missing, malformed, failed, or wrong-commit asset; uploads verified evidence as `release-evidence`; and the shared Engineering Assurance call waits for that upload. | Test |
| FR-095-AC-3 | A workflow test confirms that the OIDC-enabled job consumes the artifact and has no checkout or repository-code execution step, while the wheel workflow still receives `publish: false`. | Test |
| FR-095-AC-4 | After a tag-attestation run, the uploaded artifact contains the committed suite and binding registries, the generated binding registry, and the fresh `SUITE-001` run record from the exact tag commit; the source worktree does not commit generated records. | Test |
| FR-095-AC-5 | When the shared consumer receives no `release-evidence` artifact or the artifact has no evidence root, it writes no Test Result and no release attestation, and reports an absent-evidence failure. | Test |
| FR-095-AC-6 | When the shared consumer receives an unusable evidence root, including an invalid binding registry or run record, a missing required binding, a failing suite result, or a stale run, it writes no Test Result and no release attestation, and reports the applicable failure. | Test |

## Verification Plan

| Test Case | Expected assertion |
|-----------|--------------------|
| TC-2229 | Parse the tag-attestation workflow and suite registry. Confirm the workflow does not run the suite, waits for the release asset, validates it against the tag SHA and committed registries, uploads it as `release-evidence`, and calls EA only after validation. |
| TC-2230 | Invoke the shared consumer without the `release-evidence` artifact and with an artifact lacking the evidence root. Confirm each case reports absent evidence and writes neither a Test Result nor a release attestation. Covers FR-095-AC-5. |
| TC-2231 | Invoke the shared consumer with an invalid binding registry, invalid or missing run record, missing required binding, failing suite result, and stale run. Confirm each case reports the applicable failure and writes neither a Test Result nor a release attestation. Covers FR-095-AC-6. |
| TC-2232 | Verify a synthetic Terra bundle for the exact tag SHA and matching committed registries; refuse a wrong SHA, changed registry, failed or stale run before extraction. |
| TC-2235 | Verify a valid synthetic Terra bundle and confirm the extracted run path uses the expected commit prefix. |
| TC-2236 | Refuse a Terra bundle whose full commit SHA differs from the triggering tag SHA. |
| TC-2237 | Refuse a Terra bundle whose committed suite registry differs from the tagged source. |
| TC-2238 | Refuse a Terra bundle containing a failed SUITE-001 run record. |
| TC-2239 | Refuse a Terra bundle whose run timestamp does not postdate the annotated tag creation time. |
| TC-2240 | Refuse an archive with an unsafe member path or without the exact SUITE-001 run record. |

## Dependencies

- **Upstream**: [FR-093](./FR-093-tag-attestation-preserves-manual-wheel-release.md), [Quoin FR-030](ix://agent-ix/quoin/FR-030) (evidence-store binding registry and run-record format), and [Engineering Assurance FR-069](ix://agent-ix/engineering-assurance/FR-069) (consumer rejection behavior).
