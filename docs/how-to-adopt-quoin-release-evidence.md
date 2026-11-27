---
type: how-to
title: Reuse Quoin release receipts
description: Call the Quire-rs tag-attestation workflow from a release-tag workflow to generate and retain Quoin evidence.
---

Use this how-to to pass a tagged release and its Quoin evidence bundle through
the shared Engineering Assurance attestation workflow.

## Before you start

The caller repository needs a `v*` release tag and an Actions artifact with its
release files. Its tagged source must include the `make evidence` target,
`SUITE-001`, and Quoin suite and binding registries. The reusable workflow
checks out that exact tag with recursive submodules, runs `make evidence`, and
uploads the verified store to the same workflow run.
It also supplies the pinned Process and ISO schema providers required by the
Quire-rs corpus integration tests.

The caller must provide the `REGISTRY_TOKEN` secret for build-time access to
Agent-IX schema packages. The example uses `secrets: inherit`; callers can
instead pass that secret explicitly.

The evidence job uses read-only permissions and disables persisted checkout
credentials. The OIDC-enabled attestation job runs the shared Engineering
Assurance workflow without checking out or executing caller code. Set the repository variables
`RELEASE_RECORD_PATH` and `ALLOWED_SIGNERS_PATH` for the shared Engineering
Assurance verifier.

## Steps

1. Trigger the caller's release workflow only from `v*` tags.
2. Upload release files as an Actions artifact before the attestation job.
3. Call the reusable workflow by reference and pin it to a Quire-rs release
   tag or commit SHA.

```yaml
on:
  push:
    tags:
      - 'v*'

permissions: {}

jobs:
  attest-release:
    needs: [build, upload-release-artifact]
    permissions:
      contents: read
      id-token: write
      attestations: write
    uses: agent-ix/quire-rs/.github/workflows/tag-attestation.yml@vX.Y.Z
    secrets: inherit
    with:
      run_wheels: false
      release_artifact_name: release-artifacts
      evidence_artifact_name: release-evidence
      ea_tooling_tag: v0.9.0
      quoin_tooling_tag: 0.36.10
```

The reusable workflow checks out the caller's tagged source, generates and
verifies its release evidence, uploads the store as the `release-evidence`
Actions artifact, then calls Engineering Assurance. Set
`run_wheels: true` only if the caller should also build Quire-rs release wheels.
The workflow accepts custom names for the caller's release and evidence
artifacts.

## Expected output

The tagged run retains the verified Quoin evidence store and passes it to the
shared attestation workflow. Keep the `uses` reference in the caller and update
its pin when adopting a newer Quire-rs release. Do not copy the workflow file
into the caller repository.
