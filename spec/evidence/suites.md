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
Terra run first refuses dirty tracked, untracked, or recursive submodule state,
then runs this suite once at the exact signed tag commit with `make evidence`
and attaches `target/release-evidence/release-evidence.tar.gz` to the matching
GitHub Release. The tag-attestation workflow validates the asset contents
against the tag SHA and retains it as the same-run `release-evidence` artifact;
it does not run the suite again or authenticate the claimed Terra producer.
Under the SWM-347 owner ruling, producer provenance is out of scope here, so the
attached evidence is unsigned. The producer uses published Quoin 0.30.1 with explicit Process v0.31.0 provider commit `28c8d2ccf03a477ac07c9955de59c7c2a873238a` to derive the six FR-095 bindings, independently of user-installed modules. The original timestamped run, generated bindings and raw JUnit bytes are hashed in the tag-bound manifest. Generated run records are never committed.

## Suites

| ID | Name | Command | Tool | Evidence Kind |
|----|------|---------|------|---------------|
| SUITE-001 | Rust unit and integration tests | `make evidence` | ix-trace-run 0.1.0+8a0ec153 | Unit |
| SUITE-004 | Local real-tool tests | `make real-tool-test` | make | Integration |
