---
type: how-to
title: Reuse Quoin release receipts
description: Call the Quire-rs workflow from a release-tag workflow to attach test evidence to a release.
---

Use this how-to to attach a Quoin evidence receipt to a tagged release in another repository.
The reusable workflow declares the `workflow_call` trigger.

## Before you start

The caller repository needs a committed `spec/evidence/suites.md` and
`spec/evidence/bindings.json`. Its `make evidence` target runs the named suite
and records the passing result under `spec/evidence/runs/<SUITE-ID>/`.

The caller also needs to upload its release files as an Actions artifact before
the receipt job starts. The default artifact name is `release-artifacts`.

Set the repository variables `RELEASE_RECORD_PATH` and
`ALLOWED_SIGNERS_PATH` for the shared Engineering Assurance verifier.

## Steps

1. Trigger the caller's release workflow only from `v*` tags.
2. Upload release files as an Actions artifact before the receipt job.
3. Add this job and pin its reference to a Quire-rs release tag or commit SHA.

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
      suite_id: SUITE-001
      quoin_cli_version: 0.30.1
      ix_trace_run_rev: 16bd02ade6421057c1d3fb04c0ae95f35c6edd89
```

The workflow checks out the caller's tagged source, runs `make evidence`,
uploads the evidence store, then calls the shared attestation workflow. Set
`run_wheels: true` only when the caller should also use Quire-rs's wheel build.

The caller uploads release files as `release-artifacts` before the reusable
job. Set `release_artifact_name` when the caller uses another artifact name.
The `suite_id` must match a suite in the caller's registry.

## Expected output

The tag run uploads `release-evidence` and passes it to the shared attestation
workflow. Keep the `uses` reference in the caller and update its pin when
adopting a newer release. Do not copy the workflow file into the caller.
