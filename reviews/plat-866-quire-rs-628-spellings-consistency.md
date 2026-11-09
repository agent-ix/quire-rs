---
id: SR-16502
title: "integrity review of the FR-062 Spellings paragraph against its sibling PRs (quire-rs PR 628, head 8407145)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs@8407145; spec/functional/FR-062-implements-relation.md; cross-checked against agent-ix/spec-artifacts-process#133@0ee78cb FR-004, agent-ix/ix-trace-py#4@81e8e85 FR-002, agent-ix/ix-trace-ts#8@5c171f4 FR-002"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-062
    type: references
---

# Integrity review of the FR-062 Spellings paragraph (quire-rs PR 628, head 8407145)

## Summary

Ticket: PLAT-866. Head 8407145 adds one commit over 8db9e04. It adds a "Spellings" paragraph to the FR-062 section and one clause to FR-062-AC-19 ("a renamed import binds nothing unless the module pattern names it"). No other line changed, and TC-2201's row was not extended. `quire validate --scope . spec/functional/FR-062-implements-relation.md spec/tests.md` exits 0 at this head. The findings of SR-16496 to SR-16501 (written against 8db9e04) all still apply at this head, except that SR-16500 FND-002 is partly addressed: the paragraph states that the engine resolves no import, but CON-5 still does not, and TC-2201 does not name a renamed-import case.

The paragraph was cross-checked against the sibling heads.

- **spec-artifacts-process#133** (FR-004 Spellings bullet and AC-31 to AC-37). The paragraph agrees with it on the spellings: unqualified `implements(...)` / `implementing(...)`, qualified `ix_trace_py.implements(...)` / `ixTrace.implementing(...)`, no import resolution, a renamed import binds nothing, and the pattern must match the whole node (`@` included for decorators, the inner call for the wrapper). The TypeScript form order and templates in FR-004 match the "first templated form" rule of FR-062-AC-26. FR-004 also says "a function written as an arrow-valued binding carries the wrapper spelling instead", which confirms SR-16496 FND-003 (the advisory names the decorator for a function).
- **ix-trace-py#4** (FR-002 "Statically bound spellings"). It lists the same two spellings and the same renamed-import examples. Consistent.
- **ix-trace-ts#8** (FR-002 "Statically bound spellings"). It lists the same two spellings for both the decorator and the wrapper, documents `import * as ixTrace from "@agent-ix/ix-trace-ts"`, and says another namespace name is not bound. Consistent.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The Spellings paragraph says a renamed import "binds nothing ... and yields no finding because it matches no pattern". That is false for the TypeScript wrapper under a renamed namespace import. The wrapper candidate is the inner call with no `@` anchor, so for `import * as t` the candidate `t.implementing("FR-1")` contains `implementing("FR-1")`, which the FR-004 wrapper pattern matches. That is a proper substring, so the partial-match rule yields one `partial-implements-syntax-match` warning. The same holds for any other qualifier (`foo.ixTrace.implementing(...)`). Decorator candidates start with `@` and are unaffected. ix-trace-ts#8 says only "not bound statically", so the false claim is FR-062's alone. Fix: say that a renamed namespace qualifier on the wrapper yields the partial-match warning (arguably the useful outcome), and add that case to TC-2206/TC-2207. | spec/functional/FR-062-implements-relation.md:179-186 |
| FND-002 | medium | The removal order disagrees with FR-004 and cannot be measured. FR-062 says the comment forms are "replaced, not kept beside", with retagging done first. spec-artifacts-process#133 FR-004 keeps the Python/TypeScript line forms "until `quire coverage --json` over every repository that consumes this module reports zero `implements` relations whose form is one of those two". quire-rs reports no such thing: FR-062 states that neither relation type carries a `provenance` field and "a relation does not record how it was written (FR-051-CON-3)". So the removal condition FR-004 depends on is not observable from this engine. It is also an open-ended, all-consumers compatibility window that FR-062's "not kept beside" text does not describe. Fix: in FR-062, either delete the retag-first precondition (SR-16499 FND-001) and state that removal is not gated on consumers, or specify the per-form signal that FR-004 needs. The FR-004 side is spec-artifacts-process#133's to correct. | spec/functional/FR-062-implements-relation.md:228-241 |
| FND-003 | low | "The package-qualified one (`ix_trace_py.implements`, `ixTrace.implementing`)": `ixTrace` is not a package name. It is the namespace-import binding that ix-trace-ts#8 documents and an author may choose differently. Fix: call it "the documented namespace-import spelling (`ixTrace.implementing`)", as ix-trace-ts#8 does. | spec/functional/FR-062-implements-relation.md:184-185 |

## Verdict

Not mergeable at 8407145. The high findings of SR-16496 (FND-001, FND-002) are still open, and FND-001 and FND-002 here are medium. Apart from these, the spellings agree across all four PRs.

## Dispositions

Checked at c7eadc1 (via 592e7ad, d8a3c7a). Line numbers refer to that head.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Spellings (L178-183) requires every module pattern to be anchored at the candidate start, so `t.implementing("X")` matches nothing and yields no finding. AC-25 (L290) and TC-2207 test this, and spec-artifacts-process#133 at fac4e2e FR-004 AC-32 requires the same `\A` anchor. |
| FND-002 | fixed | L242-251 drop the retag-first precondition and state one change with no coexistence window and no precondition measurement, matching #133 at fac4e2e. The unmeasurable per-form count is no longer required by either side. |
| FND-003 | fixed | L182-183 now say `ixTrace` is the namespace-import name ix-trace-ts documents and "not a package name". |
