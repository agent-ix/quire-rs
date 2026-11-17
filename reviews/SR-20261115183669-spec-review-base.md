<!-- reviewer repo=agent-ix/quire-rs visibility=public quoin=0.35.0 module=spec-artifacts-process@v0.31.0 id=SR-20261115183669 method=spec-review/base lang=spec pr=quire-rs#696 model=gpt-6.1-sol run=6fbd820d-45ca-4b40-b171-6533040e6134 -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |

+++ [reviewer data]

```yaml
clean: true
scope:
  - {id: FR-050-AC-67, path: "spec/functional/FR-050-declarative-coverage-computation.md", role: examined, excerpt: "A non-ignored test source symbol carrying a direct trace marker for a live criterion binds to that criterion, whether or not a declared test-matrix row cites it; matrix rows do not gate criterion-level backing."}
  - {id: FR-050-AC-68, path: "spec/functional/FR-050-declarative-coverage-computation.md", role: examined, excerpt: "A direct criterion marker is not an unbacked claim merely because no declared matrix row cites it."}
  - {id: FR-050-AC-49, path: "spec/functional/FR-050-declarative-coverage-computation.md", role: context_only, excerpt: "one or more binders where at least one is not ignored is `tagged`."}
  - {id: FR-051-AC-49, path: "spec/functional/FR-051-source-symbol-extraction.md", role: context_only, excerpt: "An id-shaped token found by the advisory in a Rust, Python or TypeScript comment or docstring attached to a declaration yields one finding in `CoverageReport.diagnostics` under reason `trace-id-outside-marker`"}
  - {id: CR-PLAT-1767, path: "spec/functional/FR-050-declarative-coverage-computation.md", role: examined, excerpt: "Matrix-row absence is not an unbacked source claim: `tagged-without-citing-matrix-row` is removed from `unbacked_claims` and cannot cause `criterion-tagged-outside-row`; the independent `trace-id-outside-marker` claim remains diagnostic."}
findings: []
bindings:
  - {test_id: a_criterion_tag_only_backs_its_own_matrix_row, ac_id: FR-050-AC-67, trace: correct}
  - {test_id: a_criterion_tag_only_backs_its_own_matrix_row, ac_id: FR-050-AC-68, trace: correct}
```

+++
