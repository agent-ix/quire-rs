---
id: SR-20261103118138
title: "PLAT-1357 quire-rs PR621 gap-analysis"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/quire-rs PR621; tests/it/bundle_validate.rs, tests/it/semantic_contract.rs, spec/tests.md, spec/functional/FR-038-okf-bundle-validation.md, spec/functional/FR-076-relationships-extraction.md"
review_set: subset
---

## Summary

Ticket: PLAT-1357. This is a planless gap pass scoped to the PR diff. `quire trace` reports one structural claim each: FR-038-AC-10 is claimed by tests/it/bundle_validate.rs:735 (TC-2151), and FR-076-AC-23 by tests/it/semantic_contract.rs:1210 (TC-2152). TC-2151 covers every FR-038-AC-10 example (X1 to X6, the mixed-case declared package, and no package declared), and it fails under the old exact or min-authority code. TC-2152 does not cover every FR-076-AC-23 example, and its foreign-package case passes for a different reason than the AC states. Plan completion: not assessed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | TC-2152 never exercises the frontmatter fold (the AC-23 example frontmatter ix://Acme/Widgets/X under package acme/widgets): every case writes the frontmatter in the package's own spelling, so the relations.rs frontmatter_edges change is untested and could be reverted with the test still green | tests/it/semantic_contract.rs:1231-1245, src/semantic/relations.rs:651-658 |
| FND-002 | low | TC-2152's foreign case ix://Other/Repo/FR-005 is refused as Malformed by the lower-case package shape check, not by the package comparison; a lower-case foreign package (ix://other/repo/FR-005) is needed to exercise UnimportedPackage. It also matches message text, not the target-not-id and declared-in-frontmatter reason tokens AC-23 names | tests/it/semantic_contract.rs:1254-1265 |
| FND-003 | medium | spec/tests.md marks TC-2152 ✅ although its ix://Acme/Widgets/FR-005 case cannot pass against the current target.rs (code-review SR-20261103118137 FND-001) | spec/tests.md:1198 |

## Examined scope

```yaml
scope:
- id: FR-038-AC-10
  path: spec/functional/FR-038-okf-bundle-validation.md
  role: examined
  excerpt: "With declared package acme/widgets: absent ix://Acme/widgets/X is dangling-reference and absent ix://Other/Repo/X is skipped. With declared package Acme/Widgets: absent ix://acme/WIDGETS/X is dangling-reference. ... own-repo ix://acme/widgets/X plus each of ix://zz/o/X, ix://Zz/o/X and ix://aaa/b/X is dangling."
- id: FR-076-AC-23
  path: spec/functional/FR-076-relationships-extraction.md
  role: examined
  excerpt: "with bundle package acme/widgets, the cells X and ix://Acme/Widgets/X both lower ix://acme/widgets/X, and with bundle package Acme/Widgets the cell ix://acme/widgets/X lowers ix://Acme/Widgets/X, and a frontmatter relationships entry ix://Acme/Widgets/X under bundle package acme/widgets compares as ix://acme/widgets/X, so a row declaring the same verb and target is declared-in-frontmatter. A target under another package without an import is still refused with semantic.invalid-model-cell and reason target-not-id."
- id: FR-038-AC-9
  path: spec/functional/FR-038-okf-bundle-validation.md
  role: context_only
  excerpt: "differs from that package under the comparison of AC-10"
- id: TC-2151
  path: spec/tests.md
  role: examined
  excerpt: "The authority-versus-package comparison folds ASCII case ... (PLAT-1357) | Integration | P0 | FR-038-AC-10 | ✅"
- id: TC-2152
  path: spec/tests.md
  role: examined
  excerpt: "A ## Relationships target under the bundle package compares ASCII case-insensitively ... (PLAT-1357) | Unit | P0 | FR-076-AC-23 | ✅"
```

## Verdict

Changes requested. TC-2151 and the FR-038 code are complete. For FR-076-AC-23, add a case with frontmatter `ix://Acme/Widgets/FR-005` under package `acme/widgets` and row cell `FR-005`, expecting `declared-in-frontmatter`. Use a lower-case unimported foreign package and assert the reason tokens. Re-mark TC-2152 only after code-review FND-001 is fixed.
