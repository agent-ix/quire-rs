---
id: SR-20268
title: "gap-analysis of PLAT-212 FR-069 AC-16..20 (quire-rs PR 638)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@7f2eaf60; spec/functional/FR-069-semantic-module-contract-at-load.md; spec/tests.md; src/loader/mod.rs; src/semantic/contract.rs; src/semantic/resolver.rs; tests/it/semantic_contract.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-069
    type: references
---

# Gap analysis of PLAT-212 FR-069 AC-16..20 (quire-rs PR 638)

## Summary

Ticket: PLAT-212. Diff-scoped gap analysis of FR-069-AC-16..20 against
TC-2214..TC-2218 and the code in `git diff origin/main...7f2eaf60`. Done as a
manual AC-to-test-to-code check over the diff; the full planless
`quoin matrix` audit was not run because the brief scopes review to the PR
diff and forbids rerunning gates. Plan completion: not assessed.

Examined, clean:

- FR-069-AC-16 -> TC-2214 `import_types_shape`: absent key loads and records
  nothing; a two-name list is recorded; all six refusal shapes plus the
  non-object form assert `semantic.invalid-value` at the specified locus and
  that the module does not load.
- FR-069-AC-17 -> TC-2215 `import_types_checked_against_the_loaded_provider`:
  exported list loads; unexported `Ghost` fails naming package and type at
  `semantic.import_types.agent-ix/prov`; provider absent, at another version,
  and refused each leave the entry unchecked with `semantic.import-unresolved`.
- FR-069-AC-18 -> TC-2216 `unused_import_warns`: warning without a `$ref`,
  silent with one.
- FR-069-AC-19 -> TC-2217 `declared_import_ref_is_opaque`: the opaque `$ref`
  accepts number, null and object while the root `type: object` still
  rejects a string; the ref is recorded with package, type name (`.schema`,
  `.json` and fragment dropped) and from-type; the undeclared package fails
  `semantic.schema-ref-unshipped` naming the URL; at loader level an
  unexported type fails at `object_types[entity].data_schema.schema` and an
  exported one loads.
- FR-069-AC-20 -> TC-2218
  `import_cycles_are_named_over_types_only_when_type_edges_cycle`: type cycle
  named over `#` nodes; no-ref, one-sided-ref and non-cycling-ref package
  cycles each refused and named over packages.
- `spec/tests.md` status flips for TC-2214..TC-2218 and the FR-069 row match
  the tagged tests. Every new production branch in the diff is owned by one of
  AC-16..20; no orphan code.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - | correct-requirement-no-evidence |

## Verdict

Clean. Every AC in scope has a tagged test that can fail on its stated
behaviour. The self-import validation gap is recorded as SR-20267 FND-001
(code-review); its fix should carry the test named there.
