---
id: SR-16496
title: "spec-review (base) of FR-062 Python and TypeScript structural implements syntax positions (quire-rs PR 628)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@8db9e04; spec/functional/FR-062-implements-relation.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-062
    type: references
---

# Spec review (base) of FR-062 Python and TypeScript syntax positions (quire-rs PR 628)

## Summary

Ticket: PLAT-866. One spec-review pass over `git diff origin/main...8db9e04` (one commit, spec only): the new FR-062 section "Declared Python and TypeScript syntax positions", FR-062-AC-18..27, FR-062-CON-5, TC-2200..TC-2209 and their AC-to-TC audit rows. Applicable assurance profile: `spec/assurance/AP-201-detection-minting.md` (`review_policy.mode: require`, `operations: [spec-review]`); this review is a direct review, not an enforced ix-flow run. Lenses: base, ears-conformance, integrity, scope-boundary, failure-domain, evidence. Criterion-strength was not run: its skill states it is blocked on the Jev client, which is not installed here; the can-fail check for each AC was done by hand in this file instead.

`quire validate --scope . spec/functional/FR-062-implements-relation.md spec/tests.md` (quire 0.36.2, engine 0.50.2) exits 0 with only the pre-existing module-load warnings.

Ids: FR-062-AC-18..27, FR-062-CON-5 and TC-2200..2209 exist neither on origin/main (highest TC there is TC-2182) nor on the only other open PR, #627 (PLAT-847), which adds TC-2183..2185 and does not touch FR-062. `git merge-tree` of this head with #627's head reports no textual conflict (both edit `spec/tests.md` in different hunks). Each new AC has exactly one TC and the audit table lists all ten.

Examined: the new section in full; FR-062-AC-18 to AC-27; FR-062-CON-5; TC-2200 to TC-2209; the audit rows. Read for context: FR-062 CR-191 section, AC-12..17 and CON-2/CON-4; FR-051-AC-22, AC-49 (main and #627); FR-050-AC-62; `src/symbols/typescript.rs` (`mint_arrow_const_declarators`, the `method_definition` arm); `src/symbols/python.rs` (function kind classification); `src/symbols/trace.rs` (`carries_implements`); tree-sitter-typescript 0.23.2 `class_body`, `class_declaration`, `abstract_class_declaration`.

Public-repo check: the section names PLAT-866 and three public sibling repositories (ix-trace-py, ix-trace-ts, spec-artifacts-process, all PUBLIC per `gh repo view`) and states a this-feature release precondition. No ecosystem roadmap, delivery order across programs or private research appears. Clean.

Replacement choice: the section, AC-26 and TC-2208 state one consistent position: the engine keeps generic declared-text forms (FR-062-CON-2), the module drops the Python/TypeScript textual forms, and a comment then binds nothing and is reported by the FR-051 advisory. FR-062-AC-16 ("Python/TypeScript declared comment forms retain their existing relations") is consistent with AC-26's last sentence. The rendering rule that names the replacement conflicts with FR-051-AC-22 (FND-002) and picks an impossible construct for TypeScript functions (FND-003).

Can-fail check by hand: AC-18, AC-19, AC-24, AC-25 and AC-27 can each fail on a concrete input. AC-20, AC-21, AC-22 and AC-23 each carry a clause that cannot be met or cannot be constructed as written (FND-001, FND-004, FND-005, FND-006).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-062-AC-23 binds "the symbol that declarator mints" for `const f = implementing("X")(<function>)`, but FR-051's TypeScript extraction mints a declarator symbol only when the value is a parenthesized `arrow_function` (`mint_arrow_const_declarators`, FR-051 PLAT-882 note). A wrapped initializer is a `call_expression`, and a function-expression value mints nothing today. No AC in this PR extends extraction, so under the spec as written every wrapper falls under AC-24's "initializer of a declarator that mints no symbol" and warns: AC-23 and AC-24 contradict. Fix: add an AC (FR-051 or here) that a `const`/`let`/`var` declarator whose initializer is `callee(...)(arrow_function or function_expression)` mints a Function symbol named by the declarator, with the same qualification, span and container rules, and say whether a bare single-parameter arrow inside the wrapper qualifies. | spec/functional/FR-062-implements-relation.md (AC-23, AC-24); src/symbols/typescript.rs:614-660 |
| FND-002 | high | FR-062-AC-26 says the FR-051 advisory names `@implements(...)` / `@implementing(...)` for a Python/TypeScript production function, rendered from the first structural form of the file's language. FR-051-AC-22 (main) renders only from the first `syntax: rust_attribute` form, and #627 narrows it to a `rust_attribute` form "whose language is the symbol's own", which for Python/TypeScript is never present, so it names the kind alone (TC-2185). The two requirements give different replacement text for the same finding, and AC-26 amends FR-051 behaviour without amending FR-051-AC-22. Fix: amend FR-051-AC-22 in this PR to render from the first `implements` form of the symbol's language that declares a `syntax` and a `template`, and reconcile with #627's wording in whichever PR merges second. | spec/functional/FR-062-implements-relation.md (AC-26); spec/functional/FR-051-source-symbol-extraction.md (AC-22) |
| FND-003 | medium | AC-26 and the section render the TypeScript replacement from "the first declared `implements` form of the file's language that declares a `syntax` and a `template`", regardless of the symbol. The section itself says a TypeScript `function` declaration and an arrow-valued `const` admit no decorator, yet if the decorator form is declared first the advisory tells the author to write `@implementing("FR-007-AC-2")` on a function, which does not compile. Fix: select the form by the symbol's position (class or method: `typescript_decorator`; declarator-minted function: `typescript_wrapper`; `function` declaration: the wrapper with the rewrite stated, or the kind alone). | spec/functional/FR-062-implements-relation.md (AC-26) |
| FND-004 | medium | AC-21 requires a decorator on an `abstract` method to bind. TypeScript rejects a decorator on a method with no body, tree-sitter-typescript 0.23.2 admits a decorator in `class_body` only before `method_definition` (not before `abstract_method_signature`), and the adapter mints methods only from `method_definition`. No valid test input exists. Fix: drop `abstract` from the method list; if the intent was `abstract class`, say so (`abstract_class_declaration` takes decorators and is minted as a container). | spec/functional/FR-062-implements-relation.md (AC-21) |
| FND-005 | medium | AC-20 says the same decorator "on a test function or fixture binds nothing". The Python adapter has no fixture concept: `@pytest.fixture def db():` is a `Function`, a production kind, and the binder chooses by kind (`carries_implements`), so `@implements(...)` on it binds. The clause fails against the extraction FR-051 specifies. Fix: drop "or fixture", or specify fixture classification in FR-051. | spec/functional/FR-062-implements-relation.md (AC-20); src/symbols/python.rs:433-450 |
| FND-006 | medium | Two clauses cannot be constructed: AC-22 "a `typescript_decorator` form on a test registration or on a `describe`/`suite` container" (a decorator cannot attach to a call expression, so there is no such input), and AC-21 "a decorator on a class that is not a production container" (every TypeScript class declaration is minted as a `Container`, so no such class exists). A test for either passes vacuously. Fix: remove them or name the concrete input meant. | spec/functional/FR-062-implements-relation.md (AC-21, AC-22) |

## Verdict

Not mergeable: FND-001 and FND-002 (high) block the PR, and FND-003 to FND-006 (medium) should be fixed in the same round. The id allocation, AC-to-TC mapping, public-repo content and the replacement position are clean.

## New findings (disposition pass 1)

Checked at c7eadc1 (via 592e7ad, d8a3c7a). Line numbers refer to that head.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | medium | FR-062-AC-21 and the section (L196-206) say a decorator on an `abstract` method signature "SHALL bind nothing and SHALL yield no finding". Measured with quire 0.36.2: `export abstract class A { @implementing("FR-1") abstract foo(): void; bar(): number { return 1; } }` makes the TypeScript adapter reject the whole file (`line 3: unresolvable declaration structure (column 14)`) and the file yields no symbols at all, so `bar` and every other symbol in the file are lost and an FR-051-AC-9 per-file diagnostic is emitted. TC-2203's "yield no finding" cannot pass. Fix: state that such a file takes FR-051's per-file failure (with a diagnostic), or drop the case. | spec/functional/FR-062-implements-relation.md:196-206, 286; spec/tests.md:1195 |
| FND-008 | medium | The section (L196-206) and FR-062-AC-21 say an accessor decorator binds nothing because "the extractor mints no symbol for it". tree-sitter parses `get g()` as a `method_definition`, and the adapter mints it as a Function (`A.g` in `quire symbols` at 0.36.2). So by the table's own rule (a decorator attached to a method definition) an accessor decorator is eligible and binds, which contradicts the AC. Fix: let accessors bind as methods (the ix-trace-ts decorator accepts any member), or exclude `get`/`set` method definitions by name and say the extractor still mints them. | spec/functional/FR-062-implements-relation.md:158-160, 196-206, 286 |
| FND-009 | low | FR-062-AC-26, FR-051-AC-22 and TC-2208 name `implementing(...)` for a "`const`-bound TypeScript function", but FR-051-AC-51 and the arrow-valued rule also mint `let` and `var` declarators. The replacement text for a `let`/`var`-bound function is unstated. Fix: say "declarator-bound" (`const`, `let` or `var`). | spec/functional/FR-062-implements-relation.md:291; spec/functional/FR-051-source-symbol-extraction.md:228 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | FR-051-AC-51 (L258, TC-2210) mints a Function symbol for a declarator whose initializer is a wrapper chain, by shape alone. FR-062-AC-23 (L288) binds that symbol and AC-24 (L289) keeps "a declarator that is not a chain" as the unattached case, so the two ACs agree with each other and with AC-51. |
| FND-002 | fixed | This PR's FR-051-AC-22 (L228) keeps #627's own-language filter and renders from the structural form whose `language` is the symbol's own and whose position fits. Its verification lists TC-2185 and TC-2208. FR-062-AC-26 (L291) cites FR-051-AC-22. #627 is merged (7602ea4), and merge-tree against origin/main conflicts on that AC-22 row and on the `spec/tests.md` AC-22 audit row (TC-2185 vs TC-2208). The resolution is this PR's AC-22 text plus the union `TC-1044, TC-1046, TC-1081, TC-2165, TC-2185, TC-2208`. A rebase is needed before merge. |
| FND-003 | fixed | FR-051-AC-22 and FR-062-AC-26 choose by position: `@implementing` for a class or method, `implementing(...)` for a declarator-bound function, the kind alone for a `function` declaration. TC-2208 covers each case. |
| FND-004 | fixed | `abstract` is removed from the methods that bind (AC-21 L286, TC-2203). The replacement clause about abstract signatures is a new defect, FND-007. |
| FND-005 | fixed | AC-20 (L285) and TC-2202 now say a `@pytest.fixture` function binds, because it is a production Function symbol, which matches the extractor. |
| FND-006 | fixed | AC-22 (L287) replaces the impossible case with a class inside a `describe` callback, which binds, and the `describe` itself, which binds nothing. AC-21 drops "a class that is not a production container". Both inputs can be written. |
