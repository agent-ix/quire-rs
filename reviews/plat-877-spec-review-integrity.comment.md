<!-- reviewer repo=agent-ix/quire-rs visibility=public quoin=0.29.1 module=spec-artifacts-process@b82e42e893612b4b4ee94127714b4577ee893502 id=SR-16328 method=spec-review/integrity lang=spec pr=quire-rs#607 model=claude-opus-5-5 run=82f9e1bd-94fd-4d3c-aa25-cb3fe7bd92c4 -->

| FND | Severity | Check | Summary |
| --- | --- | --- | --- |
| FND-001 | medium | ambiguous | AC-31 states no severity (AC-28 pins FR-057 `warning`). quire-cli therefore renders every collision in the error form on coverage/symbols/assurance, though exit codes are unchanged |
| FND-002 | low | other | The AC-31 examples and the CR-206 note cover only cfg variants and branch redefinitions. The main non-cfg hits (Rust trait-impl methods, Python nested helpers) are not described |

CR-205 and TC-2081 are skipped on purpose because PR #606 claims them. The AC-31, CR-206 and TC-2082 ids are fresh. The matrix range AC-1..31 and the audit row are present.

+++ [reviewer data]

```yaml
clean: false
scope:
  - {id: FR-051-AC-31, path: spec/functional/FR-051-source-symbol-extraction.md, role: examined, excerpt: "When two or more symbols extracted from one file share a symbol id (for example two mutually exclusive `#[cfg]`-gated declarations of one Rust function, a Python function defined in both branches of an `if`, or two TypeScript registrations with one title), extraction reports one per-file diagnostic per colliding id, reason `symbol-id-collision`, naming the kind, the qualified name, and every colliding declaration by `path:line` with its outer attributes (...). Symbol ids are unchanged, and a file with no collision yields no such diagnostic (CR-206, PLAT-877)."}
  - {id: CR-206, path: spec/functional/FR-051-source-symbol-extraction.md, role: examined, excerpt: "AC-31 is new. The id hashes language, path, qualified name and kind and nothing evaluates `#[cfg]`, so two cfg-gated variants of one item mint one id and consumers saw one symbol where two declarations exist. The ruling is to keep identity and report the collision instead of resolving it; per-build-configuration extraction is not decided here."}
  - {id: FR-051-AC-28, path: spec/functional/FR-051-source-symbol-extraction.md, role: context_only, excerpt: "One finding is reported per occurrence, in `CoverageReport.diagnostics` under reason `range-in-trace-tag` at FR-057 severity `warning` exactly as FR-050-AC-50 states it"}
  - {id: TC-2082, path: spec/tests.md, role: examined, excerpt: "Two declarations in one file that share a symbol id (Rust cfg-gated variants, Python branch redefinition, TypeScript duplicate title) yield one `symbol-id-collision` diagnostic naming both by path:line with their outer attributes; ids are unchanged and distinct declarations yield none."}
findings:
  - fnd: FND-001
    severity: medium
    confidence: medium
    method: spec-review/integrity
    check: ambiguous
    artifact_id: FR-051-AC-31
    related_ids: [FR-051-AC-28, FR-057]
    path: spec/functional/FR-051-source-symbol-extraction.md
    lines: "179"
    excerpt: |-
      When two or more symbols extracted from one file share a symbol id (...), extraction reports one per-file diagnostic per colliding id, reason `symbol-id-collision`, naming the kind, the qualified name, and every colliding declaration by `path:line` with its outer attributes (...) where the adapter records them. Symbol ids are unchanged, and a file with no collision yields no such diagnostic (CR-206, PLAT-877).
    finding: "AC-31 states no severity (AC-28 pins FR-057 warning); SymbolDiagnostic has no severity field and quire-cli prints every extraction diagnostic via emit_diagnostic (error form, not warning:), so benign pairs (quire-rs evidence.rs From impls, quire-cli cfg-gated libc_eloop) show as errors on every coverage/symbols/assurance run. Exit codes unchanged. State the severity."
  - fnd: FND-002
    severity: low
    confidence: medium
    method: spec-review/integrity
    check: other
    artifact_id: CR-206
    related_ids: [FR-051-AC-31]
    path: spec/functional/FR-051-source-symbol-extraction.md
    lines: "181-187"
    excerpt: |-
      AC-31 is new. The id hashes language, path, qualified name and kind and nothing evaluates `#[cfg]`, so two cfg-gated variants of one item mint one id and consumers saw one symbol where two declarations exist. The ruling is to keep identity and report the collision instead of resolving it; per-build-configuration extraction is not decided here.
    finding: "Examples and note cover cfg variants/branch redefinitions only; measured non-cfg hits are distinct functions the identity scheme merges: Rust trait-impl methods qualified under the type (all 2 in quire-rs src, ecaz psql.rs ConnectTarget x3), Python nested helpers qualified under the class (auth-fastapi, 5 groups). A reader cannot tell these are expected."
bindings: []
```

+++
