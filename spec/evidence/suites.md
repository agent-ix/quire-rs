---
id: SUR-001
title: "Quire-rs release verification suite registry"
type: SuiteRegistry
---

# SUR-001: Quire-rs release verification suite registry

This registry declares the test suite represented by the Quire-rs release
Test Result. The committed binding registry is
[`bindings.json`](./bindings.json), a schema-version-1 Quoin evidence-store
binding file defined by [Quoin FR-030](ix://agent-ix/quoin/FR-030). It has a
top-level `schemaVersion` and `bindings` array; its binding entries identify
the obligation, suite, and supporting test symbols, and only passing test
results bind. For the release Test Result, `SUITE-001` SHALL bind each
acceptance criterion in Quire-rs FR-095 (AC-1 through AC-6). The tag-attestation
workflow runs this suite at the signed tag and uploads its fresh run record
with the committed suite and binding registries. The per-tag run record is
workflow output retained only in the `release-evidence` artifact and is not
committed.

## Suites

| ID | Name | Command | Tool | Evidence Kind |
|----|------|---------|------|---------------|
| SUITE-001 | Rust unit and integration tests | `make evidence` | cargo-nextest | Unit |
| SUITE-004 | Local real-tool tests | `make real-tool-test` | make | Integration |
