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
Quire-rs product, library, CLI, or engine runtime behavior. For a `v*` tag,
the tag workflow checks out the exact tagged source and runs `make evidence`
once. That target verifies the clean checkout and tag identity, executes
`SUITE-001`, packages and validates its generated Quoin store, then restores
the committed binding registry. The workflow uploads the verified store as a
same-run `release-evidence` Actions artifact and transfers it to the shared
release-attestation workflow. The shared consumer uses this store under
[Engineering Assurance FR-069](ix://agent-ix/engineering-assurance/FR-069)
and writes no Test Result when evidence is absent or unusable.
The tag-attestation workflow is also reusable by another repository's
tag-release workflow through `workflow_call`, with a documented example.

## Inputs

- A signed `v*` release tag handled by the Quire-rs tag-attestation workflow.
- The release suite `SUITE-001` declared in
  [`spec/evidence/suites.md`](../evidence/suites.md).
- The committed binding registry at
  [`spec/evidence/bindings.json`](../evidence/bindings.json), with the
  schema-version-1 Quoin evidence-store shape defined by
  [Quoin FR-030](ix://agent-ix/quoin/FR-030).
- The exact tagged source tree and its initialized recursive submodules.

## Outputs

- A verified Quoin evidence store containing the committed suite and binding
  registries, generated bindings, raw JUnit results, and a fresh `SUITE-001`
  run record.
- A workflow artifact named `release-evidence` carrying that store.
- A shared Engineering Assurance workflow call with
  `evidence_artifact` equal to `release-evidence`.
- A reusable `workflow_call` interface and a short example for calling the
  tag-attestation workflow by repository reference.

## Behavior

- For each pushed `v*` tag, the Quire-rs tag-attestation workflow SHALL check
  out the exact tag commit with recursive submodules and run `make evidence`
  once before uploading release evidence.
- `make evidence` SHALL refuse tracked or untracked changes in the repository
  checkout or any initialized recursive submodule before running the suite.
  It SHALL run `SUITE-001` only when `HEAD` is exactly the requested
  `vMAJOR.MINOR.PATCH` tag's commit. It SHALL write the fresh Quoin
  run record, restore the committed binding registry in the source worktree,
  and package the committed suite and binding registries with the generated
  bindings and run record as `target/release-evidence/release-evidence.tar.gz`.
- `make evidence` SHALL stream-validate its archive with a
  maximum of six regular files, 20,000,000 compressed bytes, and 50,000,000
  aggregate decompressed bytes. It SHALL validate the safe file set, manifest
  tag, full commit SHA, suite and run record; verify manifest SHA-256 hashes of each payload file; compare the suite and
  committed binding registries byte-for-byte with the tagged source; and
  reject stale, empty, failed, or wrong-commit evidence before extracting the
  verified store. The tag-attestation workflow SHALL upload that store as the
  same-run `release-evidence` artifact.
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
- The tag-attestation workflow SHALL expose typed `workflow_call` inputs for
  the wheel-build choice and release artifact names. Evidence validation and
  attestation SHALL run only for `v*` tag refs. A reusable caller SHALL build
  wheels only when explicitly requested.
- The tag-attestation workflow SHALL NOT modify the committed suite or binding
  registries and SHALL NOT commit generated run records.

> **CR-426 note (2026-11-26, PLAT-1934):** The tag workflow originally reran `SUITE-001`
> on a GitHub-hosted runner after the required Terra release run, producing two
> full-suite runs for one release. The approved FR-095 resolution makes the
> tagged workflow the single producer: it runs `make evidence` once at the
> exact tag commit and passes the verified store to the existing fail-closed EA
> consumer.

> **CR-427 note (2026-11-26, PLAT-1934):** Review requires the release run to
> refuse dirty tracked, untracked, or recursive submodule state before tests
> execute. `make evidence` checks that condition before invoking the suite and
> verifies the bounded evidence archive before workflow upload.

> **CR-429 note (2026-11-26):** Published Quoin 0.30.1 records timestamped runs without a commit field. The producer selects the exact run path from the recorder JSON response and preserves those bytes. The release manifest binds the clean exact-tag source commit to SHA-256 hashes of the run, binding registries, suite registry and raw JUnit results. Producer authentication remains outside scope.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-095-AC-1 | A workflow test confirms that the `v*` tag job checks out the exact tag commit with recursive submodules, runs `make evidence` once, and uploads the verified evidence store as the `release-evidence` artifact. | Test |
| FR-095-AC-2 | A workflow test confirms that `make evidence` verifies the generated archive's required files, hashes, tag, commit, registries, and fresh passing SUITE-001 run record before extraction; the upload and attestation jobs do not run after generation or verification fails. | Test |
| FR-095-AC-3 | A workflow test confirms that the OIDC-enabled job consumes the artifact and has no checkout or repository-code execution step, while the wheel workflow still receives `publish: false`. | Test |
| FR-095-AC-4 | After a tag-attestation run, the uploaded artifact contains the committed suite and binding registries, the generated binding registry, and the fresh `SUITE-001` run record from the exact tag commit; the source worktree does not commit generated records. | Test |
| FR-095-AC-5 | If the tag workflow cannot generate or upload the `release-evidence` artifact, it does not call the shared Engineering Assurance workflow. | Test |
| FR-095-AC-6 | When the release evidence verifier receives an unusable archive, including an invalid binding registry or run record, a missing required binding, a binding without passing test evidence, a failing suite result, or a stale run, it rejects the archive before extraction or artifact upload. | Test |
| FR-095-AC-7 | A workflow test confirms that the tag-attestation workflow exposes typed `workflow_call` inputs for the wheel-build choice and release artifact names, runs evidence validation and attestation only for `v*` tag refs, and builds wheels from a reusable call only when explicitly requested. | Test |
| FR-095-AC-8 | A documentation test confirms that the adoption how-to calls the reusable workflow by repository reference from a tag-only caller and provides the release artifact inputs without vendoring the workflow. | Test |

## Verification Plan

| Test Case | Expected assertion |
|-----------|--------------------|
| TC-2229 | Parse the tag-attestation workflow and suite registry. Confirm the workflow checks out the exact tag with recursive submodules, runs `make evidence`, uploads the verified output as `release-evidence`, and calls EA only after upload. |
| TC-2230 | Confirm the tag-attestation job requires successful evidence generation and upload before invoking EA, and a failed evidence job prevents attestation. Covers FR-095-AC-5. |
| TC-2231 | Run packaging and archive verification against invalid bindings, unsupported test symbols, an invalid or missing run record, a failing suite result, and a stale run. Confirm each case fails before packaging or extraction. Covers FR-095-AC-6. |
| TC-2232 | Verify a synthetic evidence archive for the exact tag SHA and matching committed registries; refuse a wrong SHA, changed registry, failed or stale run before extraction. |
| TC-2235 | Verify a valid evidence archive and confirm the original timestamped run path and bytes are preserved. |
| TC-2236 | Refuse an evidence archive whose full commit SHA differs from the triggering tag SHA. |
| TC-2237 | Refuse an evidence archive whose committed suite registry differs from the tagged source. |
| TC-2238 | Refuse an evidence archive containing a failed SUITE-001 run record. |
| TC-2239 | Refuse an evidence archive whose run timestamp does not postdate the annotated tag creation time. |
| TC-2240 | Refuse an archive with an unsafe member path or without the exact SUITE-001 run record. |
| TC-2241 | Refuse `make evidence` before test execution when tracked, untracked, or recursive submodule state is dirty. |
| TC-2242 | Stream an archive with more than six entries, more than 20,000,000 compressed bytes, or more than 50,000,000 decompressed bytes; refuse each before extraction. |
| TC-2243 | Parse the reusable workflow declaration and confirm typed inputs and release-tag guards; verify a push-triggered caller with wheel building disabled skips the wheel build and upload. Covers FR-095-AC-7. |
| TC-2244 | Read the adoption how-to and confirm it calls the reusable workflow by reference from a tag-only caller without vendoring the workflow. Covers FR-095-AC-8. |

## Dependencies

- **Upstream**: [FR-093](./FR-093-tag-attestation-preserves-manual-wheel-release.md), [Quoin FR-030](ix://agent-ix/quoin/FR-030) (evidence-store binding registry and run-record format), and [Engineering Assurance FR-069](ix://agent-ix/engineering-assurance/FR-069) (consumer rejection behavior).
