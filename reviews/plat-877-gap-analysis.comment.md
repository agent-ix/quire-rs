<!-- reviewer repo=agent-ix/quire-rs visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-16327 method=gap-analysis lang=rust pr=quire-rs#607 model=claude-opus-5-5 run=82f9e1bd-94fd-4d3c-aa25-cb3fe7bd92c4 -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | low | coverage | The AC-31 clause "naming the kind" is asserted by no test, so dropping the kind from the message still passes TC-2082 |

Plan completion: not assessed (planless). All three adapter tests are tagged TC-2082/FR-051-AC-31 and pass. Rust and Python each have a separate clean control file. TypeScript has an inline distinct title under the `len == 1` guard. The Rust test asserts the ids are unchanged. No stubs found and no coverage inflation.

+++ [reviewer data]

```yaml
clean: false
scope:
  - {id: FR-051-AC-31, path: spec/functional/FR-051-source-symbol-extraction.md, role: examined, excerpt: "When two or more symbols extracted from one file share a symbol id (...), extraction reports one per-file diagnostic per colliding id, reason `symbol-id-collision`, naming the kind, the qualified name, and every colliding declaration by `path:line` with its outer attributes (...) where the adapter records them. Symbol ids are unchanged, and a file with no collision yields no such diagnostic (CR-206, PLAT-877)."}
  - {id: TC-2082, path: spec/tests.md, role: examined, excerpt: "Two declarations in one file that share a symbol id (Rust cfg-gated variants, Python branch redefinition, TypeScript duplicate title) yield one `symbol-id-collision` diagnostic naming both by path:line with their outer attributes; ids are unchanged and distinct declarations yield none."}
findings:
  - fnd: FND-001
    severity: low
    confidence: high
    method: gap-analysis
    check: coverage
    artifact_id: FR-051-AC-31
    related_ids: [TC-2082]
    path: src/symbols/mod.rs
    lines: "788-806"
    excerpt: |-
      When two or more symbols extracted from one file share a symbol id (...), extraction reports one per-file diagnostic per colliding id, reason `symbol-id-collision`, naming the kind, the qualified name, and every colliding declaration by `path:line` with its outer attributes (...) where the adapter records them. Symbol ids are unchanged, and a file with no collision yields no such diagnostic (CR-206, PLAT-877).
    finding: "The 'naming the kind' clause is asserted by no TC-2082 test (none checks for `function`), so removing g[0].kind.as_str() from the message would still pass."
bindings:
  - {test_id: tc2082_rust_cfg_gated_duplicates_are_reported, ac_id: FR-051-AC-31, trace: correct}
  - {test_id: tc2082_python_redefinition_is_reported, ac_id: FR-051-AC-31, trace: correct}
  - {test_id: tc2082_typescript_duplicate_title_is_reported, ac_id: FR-051-AC-31, trace: correct}
```

+++
