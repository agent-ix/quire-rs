---
id: NFR-022
title: "Skeptic oracle candidate extraction has linear scan work"
type: NFR
quality_attribute: performance_efficiency
relationships:
  - target: "ix://agent-ix/quire-rs/FR-064"
    type: "constrains"
---
# NFR-022: Skeptic oracle candidate extraction has linear scan work

## Statement

When oracle detection inspects one evidence-symbol span, the skeptic layer SHALL
bound candidate-extraction work to the span bytes and matches observed in one
forward pass per pattern, without restarting a whole-span or prefix scan for
each binding candidate.

## Scope

- Applies to the direct binding-to-assertion join and Rust helper-oracle
  candidate selection described by [FR-064](../functional/FR-064-skeptic-layer.md).
- Let `n` be the UTF-8 byte length of the inspected span, `a` its assertion
  matches, and `b` its expected/oracle binding matches. Direct candidate
  selection is bounded by `O(n + a + b)` expected work, including line lookup.
- Candidate ordering, first-match selection, expression bytes, function names,
  and reported line offsets remain the FR-064 behavior; this requirement changes
  no suspicion, diagnostic, total, or exit status.
- Regex compilation, source-symbol extraction, comment/string masking, token
  similarity, and cross-file subject resolution are outside this measurement.

## Rationale

Restarting assertion matching from byte zero for every binding makes the direct
join quadratic on multi-binding spans. Recounting newlines from byte zero for
each candidate repeats the same prefix work. Both costs occur for every test
span in a corpus run while contributing no additional evidence or behavior.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Full-span binding capture traversals per direct candidate extraction | one | at most one | unit-testing |
| Full-span assertion capture traversals per direct candidate extraction | one | at most one | unit-testing |
| Line-start index constructions per inspected evidence-symbol span | one | at most one | unit-testing |
| Candidate-specific prefix newline rescans | zero | zero | static-quality |
| Candidate and line-offset differences from the FR-064 baseline over Rust, Python, TypeScript, repeated bindings, reordered assertions, and the Rust helper path | zero | zero | property-based-testing |

## Verification

A deterministic operation-count harness varies span length and the numbers of
bindings and assertions independently, then proves that capture traversals and
line-index construction do not grow with the number of candidates. A separate
differential fixture compares the complete selected candidate and line offset
against the pre-change FR-064 behavior. A static source audit rejects a regex
capture traversal nested under a match loop and any candidate-specific newline
prefix count.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-022-AC-1 | Operation counts show at most one complete binding-pattern traversal, one complete assertion-pattern traversal, and `a + b` join operations for direct candidate extraction, regardless of the number or ordering of bindings and assertions. | Test |
| NFR-022-AC-2 | Exactly one line-start index is built per inspected evidence-symbol span, both the direct and Rust helper paths reuse it, and no candidate-specific prefix newline count remains. | Analysis |
| NFR-022-AC-3 | For Rust, Python, TypeScript, repeated binding names, assertions before or after bindings, unmatched bindings, UTF-8, CRLF, and the Rust helper path, the selected candidate fields and line offsets equal the FR-064 baseline. | Test |
| NFR-022-AC-4 | The binding-to-assertion join keeps lookup state bounded independently of `a` and `b` (the grammar admits only `expected` and `oracle`), and no output depends on lookup-container iteration order. | Analysis |

## Dependencies

- **Upstream**: [FR-064](../functional/FR-064-skeptic-layer.md) defines the
  candidate-selection and reporting behavior constrained here.
- **Implementation evidence**: `agent-ix/quire-rs#412` supplies the motivating
  defect and the preserved, not-yet-reviewed implementation checkpoint.
