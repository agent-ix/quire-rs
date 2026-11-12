---
id: SR-20261112155211
title: "spec-review of FR-058 where field-value selector on required_relations (quire-rs PR 652)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@b04cbdb; spec/functional/FR-058-upward-trace-completeness.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-058
    type: references
---

# Spec review of FR-058 where field-value selector (quire-rs PR 652)

## Summary

Ticket: PLAT-1475. Scope is `git diff origin/main...b04cbdb` (main is an ancestor): the new "Narrowing the obligation by a field value" section, the `where` row of the declaration table, FR-058-AC-12..20, FR-058-CON-3, TC-2248..TC-2255, the AC audit rows and the FR-058 coverage row. Gate: `examples/spec_validate.rs` with `--process-root spec-artifacts-process --iso-root spec-artifacts-iso` gives 204 documents, 0 failed, 43 warnings. The two FR-058 warnings (lines 188-189) are AC-10 and AC-11, which are on main; the new ACs raise none. The FR-050-AC-58 audit completeness check passes.

Examined and clean:

- Engine knowledge. `where` names a field and values in the manifest only; CON-3 extends CON-2 to field names and values. This is consistent with FR-058's own rule that the engine knows no archetype, verb or chain.
- Value source. The field is read from top-level frontmatter, the same source FR-082 reads `criticality` from. Absent and null mean "not selected", which matches FR-082's absent/null rule. Trim plus case-fold matches FR-049-AC-13.
- Interactions. `where` narrows only the obliged side (AC-15). The order is `from`, then `where`, then `exclude` (AC-16). The dead-vocabulary rule still tests `from` alone (AC-16). Posture routing and the FR-057 key are unchanged and advisory under Okf and Strict (AC-17). Load-time rejection lists the malformed shapes (AC-18). Finding text names the selector, and output without `where` is byte-identical (AC-19). Determinism is covered (AC-20).
- Testability. Every new AC names a concrete input and outcome and has a TC row and an audit row. CON-3 is Inspection, like CON-2.
- Public-repo hygiene. The spec text contains no roadmap or delivery-order material. The PR body names the ticket and the two public sibling repositories whose manifests use the feature.

Downstream use, walked end to end against spec-artifacts-iso, spec-objects-security and spec-objects-safety on their current main. The `NFR` archetype has `quality_attribute` as a top-level frontmatter key: `nfr-frontmatter.schema.json` has an enum that includes `security` and `safety`, and `body_extraction` reads it with `from: frontmatter_field`. `threat` (security) and `hazard` (safety) are declared kinds. Both modules already author `mitigates` from the mitigating side, so `direction: outgoing` from the NFR is the right reading. With `from: NFR`, the selector is enough to express "every security- or safety-classified NFR has a `mitigates` edge to a threat or hazard". There are two gaps. First, the worked example names a kind that does not exist (FND-001). Second, a prerequisite sits outside this PR: iso's `NFR.allowed_links` does not list `mitigates`, so an NFR authoring that edge draws a `DisallowedEdgeType` warning (FR-041-AC-2, FR-076-AC-13) until iso or the security and safety modules allow it. Also, under FR-058's dangling rule, an NFR that mitigates a hazard in another repository does not satisfy a relation with a constrained `to`. That follows from existing FR-058 behaviour, not from `where`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The worked example and the prose use `from: nfr-document`, but no loaded module declares that kind. spec-artifacts-iso declares the archetype as `NFR`, and `nfr-document` appears in no manifest. A module author who copies the canonical example gets a relation that selects zero documents and reports itself under `trace:undeclared-relation-vocabulary` (AC-11). Fix: use `from: NFR` in the YAML example and in the prose ("applies to the `NFR` documents whose ..."). | spec/functional/FR-058-upward-trace-completeness.md:69,75 |

## Verdict

Changes requested: FND-001 (medium) should be fixed in this PR. Otherwise the design is consistent with FR-058, FR-082, FR-049 and FR-057.

## Dispositions

Round 1, checked at a5fc8b0. Gate: `examples/spec_validate.rs` with the sibling process and iso roots gives 204 documents, 0 failed, 43 warnings. The FR-050-AC-58 audit check passes. The only FR-058 warnings are AC-10 and AC-11 (now lines 197-198), which are on main.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The YAML example uses `from: NFR`, and the prose says "applies to the `NFR` documents whose `quality_attribute` is ...". `NFR` is the archetype name in spec-artifacts-iso. |
