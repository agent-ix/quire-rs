---
id: SUR-001
title: "Quire-rs release verification suite registry"
type: SuiteRegistry
---

# SUR-001: Quire-rs release verification suite registry

This registry declares the test suite represented by the Quire-rs release
Test Result. The tag-attestation workflow runs this suite at the signed tag and
transfers its fresh run record with the committed suite and binding registries.
The run record is retained in the workflow artifact and is not committed.

## Suites

| ID | Name | Command | Tool | Evidence Kind |
|----|------|---------|------|---------------|
| SUITE-001 | Rust unit and integration tests | `make evidence` | cargo-nextest | Unit |
