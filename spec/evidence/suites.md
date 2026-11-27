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
workflow checks out the exact signed tag and recursive submodules, supplies the
pinned Process and ISO validation providers for corpus integration tests, then
runs this suite once with `make evidence`. The target refuses dirty source and
submodule state, verifies the exact tag and generated run record, restores the
committed binding registry, and writes `target/release-evidence/verified/`.
The workflow uploads that verified store as the same-run `release-evidence`
artifact; the shared Engineering Assurance workflow consumes that artifact and
does not rerun the suite. The evidence package records the original run,
generated bindings and raw JUnit bytes. Generated run records are never
committed. The evidence producer uses published Quoin 0.30.1 with Process
provider commit `28c8d2ccf03a477ac07c9955de59c7c2a873238a` to derive FR-095
bindings; this producer provider pin is separate from the Process and ISO
validation provider snapshots used by the corpus integration tests.

## Suites

| ID | Name | Command | Tool | Evidence Kind |
|----|------|---------|------|---------------|
| SUITE-001 | Rust unit and integration tests | `make evidence` | ix-trace-run 0.1.0+8a0ec153 | Unit |
| SUITE-004 | Local real-tool tests | `make real-tool-test` | make | Integration |
| SUITE-005 | FR-051 cross-language E-10 emitter fixtures | `make test-fr051-e10` | pinned ix-trace emitters + Quoin | Integration |

`SUITE-005` is a local-only, reproducible conformance harness for
FR-051-AC-33 and FR-051-AC-48. It installs Quoin from the public npm registry,
installs `ix-trace-run` from the pinned public Git revision, installs the
Python pytest emitter from its pinned public Git revision, and installs the
TypeScript emitter from npmjs. The harness runs passing and intentional-failure
fixtures for Rust, Python and TypeScript, validates the JUnit symbol, file and
trace fields, and records each result into a fresh temporary Quoin evidence
store. It uses task-local caches and configuration, does not use npm.ix, and
does not change the committed evidence bindings.
