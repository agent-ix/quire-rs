---
type: how-to
title: Reuse Quoin release receipts
description: Call the Quire-rs tag-attestation workflow from a release-tag workflow to validate and retain Quoin evidence.
---

Use this how-to to pass a tagged release and its Quoin evidence bundle through
the shared Engineering Assurance attestation workflow.

## Before you start

The caller repository needs a `v*` release tag and an Actions artifact with its
release files. It must also attach a `release-evidence.tar.gz` asset to the
matching GitHub Release before the Quire-rs reusable workflow's 60-minute wait
ends. The evidence archive must follow the release-evidence manifest, run
record, binding registry, suite registry, and JUnit layout accepted by the
tagged source's `scripts/verify_release_evidence.py`.

The caller source at the tag must provide that verifier and the matching
`spec/evidence/suites.md` and `spec/evidence/bindings.json` registries. The
evidence-validation job runs the verifier from the tagged source with read-only
permissions and no persisted checkout credentials. The OIDC-enabled
attestation job runs the shared Engineering Assurance workflow without
checking out or executing caller code. Set the repository variables
`RELEASE_RECORD_PATH` and `ALLOWED_SIGNERS_PATH` for the shared Engineering
Assurance verifier.

## Steps

1. Trigger the caller's release workflow only from `v*` tags.
2. Upload release files as an Actions artifact before the attestation job.
3. Attach the evidence bundle as `release-evidence.tar.gz` to the matching
   GitHub Release.
4. Call the reusable workflow by reference and pin it to a Quire-rs release
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
    with:
      run_wheels: false
      release_artifact_name: release-artifacts
      evidence_asset_name: release-evidence.tar.gz
      evidence_artifact_name: release-evidence
      ea_tooling_tag: v0.9.0
      quoin_tooling_tag: 0.30.1
```

The reusable workflow checks out the caller's tagged source as data, waits for
and verifies its release evidence asset, uploads the verified store as the
`release-evidence` Actions artifact, then calls Engineering Assurance. Set
`run_wheels: true` only if the caller should also build Quire-rs release wheels.
The workflow accepts custom names for the caller's release artifact and the
evidence asset and artifact.

## Expected output

The tagged run retains the verified Quoin evidence store and passes it to the
shared attestation workflow. Keep the `uses` reference in the caller and update
its pin when adopting a newer Quire-rs release. Do not copy the workflow file
into the caller repository.
