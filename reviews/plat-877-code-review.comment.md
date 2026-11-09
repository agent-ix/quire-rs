<!-- reviewer repo=agent-ix/quire-rs visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-16326 method=code-review lang=rust pr=quire-rs#607 model=claude-opus-5-5 run=82f9e1bd-94fd-4d3c-aa25-cb3fe7bd92c4 -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | low | code-bug | Raw attribute text is embedded verbatim, so a multi-line attribute puts newlines in `reason`; non-cfg attributes are listed too |
| FND-002 | low | other | No CHANGELOG entry for the new `symbol-id-collision` diagnostic class (symbols JSON, coverage/symbols/assurance stderr) |

Verified:
- One diagnostic per same-id group per file. The output is deterministic (BTreeMap, then a (line, column) sort, then a reason sort).
- No cross-file collision is possible, because the id hashes the path.
- No fixture, baseline, Python or wasm change. Extraction diagnostics reach only `SymbolTableReport` and quire-cli stderr, and exit codes are unchanged.
- quire-rs `src` self-extraction now emits 2 diagnostics (`From` impl pairs in evidence.rs). `tests/` emits 0.
- Grouping costs O(n log n) per file.
- 142 `symbols::` lib tests and 55 focused `it` tests pass. clippy is clean under all-features and under no-default-features.

+++ [reviewer data]

```yaml
clean: false
scope:
  - {id: "src/symbols/mod.rs#extend_with_file", path: src/symbols/mod.rs, role: examined, excerpt: "self.diagnostics.extend(id_collision_diagnostics(path, &file_symbols)); self.symbols.extend(file_symbols);"}
  - {id: "src/symbols/mod.rs#id_collision_diagnostics", path: src/symbols/mod.rs, role: examined, excerpt: "fn id_collision_diagnostics(path: &str, symbols: &[Symbol]) -> Vec<SymbolDiagnostic>"}
  - {id: "src/symbols/mod.rs#tc2082", path: src/symbols/mod.rs, role: examined, excerpt: "tc2082_rust_cfg_gated_duplicates_are_reported / tc2082_python_redefinition_is_reported / tc2082_typescript_duplicate_title_is_reported"}
  - {id: src/symbol_table.rs, path: src/symbol_table.rs, role: context_only, excerpt: "diagnostics: extraction.diagnostics.iter().map(|d| FileDiagnostic { path: d.path.clone(), reason: d.reason.clone() })"}
  - {id: "quire-cli/src/commands/coverage.rs", path: "quire-cli/src/commands/coverage.rs", role: context_only, excerpt: "for d in &extraction.diagnostics { io::emit_diagnostic(ctx.diagnostics, \"SymbolExtraction\", ...) }"}
findings:
  - fnd: FND-001
    severity: low
    confidence: medium
    method: code-review
    check: code-bug
    artifact_id: "src/symbols/mod.rs#id_collision_diagnostics"
    related_ids: [FR-051-AC-31]
    path: src/symbols/mod.rs
    lines: "588-597"
    excerpt: |-
      let attrs = s.attributes.iter().map(|a| a.text.as_str()).collect::<Vec<_>>().join(" ");
      if attrs.is_empty() { format!("{path}:{}", s.line) } else { format!("{path}:{} [{attrs}]", s.line) }
    finding: "Raw attribute source text is embedded verbatim, so a rustfmt-wrapped multi-line attribute (#[cfg(all(\\n ...))]) puts newlines into reason and breaks one-line stderr; every non-cfg attribute (#[inline], #[trace], #[test]) is also listed. 0 of 121 on ecaz, but unguarded. Collapse whitespace."
  - fnd: FND-002
    severity: low
    confidence: medium
    method: code-review
    check: other
    artifact_id: CHANGELOG.md
    related_ids: [FR-051-AC-31]
    path: CHANGELOG.md
    lines: "8"
    excerpt: |-
      ## [Unreleased]
    finding: "No [Unreleased] entry for the new symbol-id-collision diagnostic class that now appears in quire symbols JSON diagnostics and on coverage/symbols/assurance stderr (2 new lines on quire-rs's own tree)."
bindings: []
```

+++
