---
id: SR-16303
title: "PLAT-1236 wheel checkout code-review"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; .github/workflows/wheels.yml; PLAT-1236; PR565"
review_set: subset
---

## Summary

Reviewed PR565 frozen144d3b544610359d619bb52b253cf397a8014f56 against c0fa4135c754437ed8aa4370f0faf62cbb2a09eb. The diff only changes .github/workflows/wheels.yml: initialize existing submodules and add examples/tools to its sparse directory set. No Rust, manifest, lock, version, credential, publication-policy or tag changes occur. No Cargo runs or source edits were performed by this reviewer.

The recorded action-equivalent checkout experiment applies the actual sparse directory list before submodule initialization, then runs locked cargo metadata --no-deps and discovers all seven workspace packages. Its assessment explicitly invalidates the first failed experiment that listed the uninitialized corpus gitlink as a cone directory. Cargo metadata requires corpus/Cargo.toml, tools/quire-bench/Cargo.toml and examples/spec_validate.rs, which the old checkout misses. The corrected sparse set and post-checkout submodule initialization provide them. Both normal-library schema includes remain selected. The pinned QA tree and added directories have no Windows-invalid paths or case collisions in the inspected census.

Publication still requires a real hosted wheel build and smoke imports after this fix. Metadata success is not wheel compilation, ABI qualification or a successful registry upload. A new dispatch must use the fixed workflow descendant while leaving v0.51.0 immutable at the released Rust source; parent must verify runtime/package source equivalence before that dispatch. The publish job only downloads produced wheels and is unchanged.

## Assurance Context

AP-201 concerns changes to coverage collection, trace binding, row minting, diagnostics and schemas. This scoped distribution-checkout fix changes none of those computations; the workflow remains GOVERNED_EVIDENCE=false and cannot mint governed QA credit. No measurement qualification or policy exception is claimed.

## Examined Units

- **FR-023-AC-1** (examined): cargo build (no features) and cargo build --features python both succeed; the no-feature build feature resolution shows no pyo3 linkage.
- **FR-023-AC-2** (context_only): After maturin build --features python and install, import quire; quire.parse_document(text) returns an object whose frontmatter, section headings, and block ids match the Rust parse_document output for the same input.
- **FR-023-AC-6** (context_only): A single abi3 wheel built once imports successfully under two different CPython 3.x minor versions in CI.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

PASS for the scoped checkout fix. Runtime language reviews and spec-authoring sub-analyses are not applicable to this YAML-only change. Real hosted wheel compilation, smoke imports and publication remain pending and are not claimed by this review.
