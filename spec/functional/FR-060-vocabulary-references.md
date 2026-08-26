---
id: FR-060
title: "Vocabulary References in Body-Extraction Asserts"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-033"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-054"
    type: "requires"
    cardinality: "1:1"
---
# FR-060: Vocabulary References in Body-Extraction Asserts

## Description

An archetype contract restates a vocabulary the traceability model already declares, in the same
manifest:

```yaml
column_choices:
  Type: [Unit, Integration, Property, Snapshot]     # the contract
traceability:
  vocabularies:
    test_type: [Unit, Integration, Property, Snapshot]   # the model
```

Two copies of one list, kept in agreement only by someone remembering — in this ecosystem, by a
single test in a single module (`spec-artifacts-process tests/test_manifest.py`). Since
[FR-054](./FR-054-verification-method-catalog.md) there are two more lists worth referencing,
`verification_method` and `verification_class`, both **derived** from the merged catalog. The
pressure grows rather than holds steady.

`quire-rs` SHALL accept two dereferencing assert keys, mirroring the literal forms beside them:

| Reference | Literal counterpart | Locator kinds |
|---|---|---|
| `from_vocabulary: <name>` | `choices:` | scalar (`section_body`, `heading`, `list_item`, `frontmatter_field`) |
| `column_vocabularies: {Header: <name>}` | `column_choices:` | `table_row` only |

### Where resolution happens, and why it is not a choice

References SHALL be resolved **at registry construction, after the cross-module merge**, into the
literal choices the evaluator already understands.

The location is forced, not preferred. **The vocabulary a contract names may be declared by a
different module than the archetype naming it**, so resolution cannot happen while compiling a
module — only once the merged model and the compiled archetypes are both in hand. Registry
construction is that point.

Doing it there also keeps `evaluate_assert`'s signature unchanged. The alternative — threading a
`Registry` into the per-document validation path — would change a public API every consumer of that
function inherits, in order to repeat a lookup that is constant for the life of the registry.
The evaluator never sees a vocabulary name at all.

### A reference obeys its literal's rules

A reference SHALL be legal exactly where the literal it stands in for is legal. `from_vocabulary`
is rejected on `table_row` and `code_block`, as `choices` is (CR-010); `column_vocabularies` is
rejected anywhere but `table_row`, as `column_choices` is.

This was found by a failing test rather than by design: the first implementation let
`from_vocabulary` sit on a `table_row`, where nothing would ever enforce it. **A reference that can
sit where its literal cannot is a way to smuggle a constraint into a position that ignores it**, and
a declared-but-unenforceable check is worse than a rejected one.

### An unknown name is empty, not absent

A name no loaded module declares SHALL resolve to an **empty** choice set — not to "no constraint".

The distinction is the whole safety property. Dropping the constraint would let a typo silently
**widen** the contract so every value passes, which is the same quiet-wrong-answer class as CR-075's
dead `from`. An empty set fails every cell instead: loud, and diagnosable from the first document.

### A literal beside a reference wins

When an assert carries both, the literal is kept and the reference is dropped rather than merged.
Two sources for one constraint is the duplication this FR removes; unioning them would recreate it
*inside a single assert*, where it is even harder to see than across two files.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-060-AC-1 | `column_vocabularies: {Header: name}` resolves to the declared vocabulary's values, and the reference is consumed so nothing downstream must understand it. | Test (TC-919) |
| FR-060-AC-2 | `from_vocabulary: name` resolves the scalar counterpart the same way. | Test (TC-920) |
| FR-060-AC-3 | A name no module declares resolves to an **empty** choice set, never to an absent constraint. | Test (TC-921) |
| FR-060-AC-4 | A literal `choices`/`column_choices` beside a reference wins, and the reference is dropped rather than merged. | Test (TC-922) |
| FR-060-AC-5 | An archetype naming no vocabulary is left byte-identical and is not cloned. | Test (TC-923) |
| FR-060-AC-6 | A reference is legal exactly where its literal counterpart is legal, and the load failure names the offending key. | Test (TC-924) |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-060-CON-1 | The name→vocabulary mapping SHALL exist **once**. `Registry::column_vocabulary` and the construction-time resolver read the same function; two matches would be exactly the duplication this FR removes, one level up. | Design | Inspection |
| FR-060-CON-2 | `evaluate_assert` SHALL keep its signature. The evaluator never resolves a name; every reference is literal by the time any document is validated. | Design | Inspection |
| FR-060-CON-3 | The engine SHALL name no vocabulary. `test_type`, `verification_method` and `verification_class` are the *declared* names the merged model and catalog already provide, not engine vocabulary. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-033](./FR-033-locator-assert-facet.md) (the assert facet), [FR-054](./FR-054-verification-method-catalog.md) (the named lookup this generalizes)
- **Downstream**: `spec-artifacts-process` can drop its duplicate list and the test that holds the two in agreement
