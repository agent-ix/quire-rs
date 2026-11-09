---
id: SR-20261103118137
title: "PLAT-1357 quire-rs PR621 code-review (Rust lane)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs PR621; src/corpus/resolve.rs, src/corpus/validate.rs, src/semantic/relations.rs, src/semantic/target.rs, tests/it/bundle_validate.rs, tests/it/semantic_contract.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1357. Reviewed `git diff origin/main...HEAD` at head 07cbfcb against FR-038-AC-9/AC-10 and the FR-076 statements and AC-23. Rust lane (rust-review, rust-style loaded) folded in. Not built by the reviewer (build lock saturated); the lead's focused run had not started when this review was written. Findings are traced by reading the code.

The corpus side (resolve.rs, validate.rs) is correct: a bare id clears the authority set, `ix://` spellings accumulate into a `BTreeSet` (sorted, deduplicated), `validate_bundle` skips only when every authority differs from the declared package under `eq_ignore_ascii_case`, and with no package declared every miss stays dangling as before. The TC-2151 test fails under the old code for X1, X5 and X6.

The semantic side has a blocking defect. `resolve_target` calls `semantic_identity` before the new case-folded package comparison, and `semantic_identity` accepts only lower-case ASCII letters in the org and repo. So `ix://Acme/Widgets/X` is refused as `Malformed` (`target-not-id`) before the fold is reached. The second loop case of TC-2152 asserts exactly that cell, so the test as written should fail.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | semantic_identity rejects upper-case org/repo before the folded own-package check, so the AC-23 cell ix://Acme/Widgets/X under package acme/widgets is refused target-not-id instead of lowering to ix://acme/widgets/X; the TC-2152 case asserting it should fail | src/semantic/target.rs:79-88, src/semantic/target.rs:103-104 |
| FND-002 | medium | A mixed-case declared package (Acme/Widgets) lowers to ix://Acme/Widgets/X, which the semantic-v1 schema's semanticId pattern (lower-case org) rejects. Bare ids already did this before the PR, but AC-23 and TC-2152 now require it | src/semantic/target.rs:113, schemas/output/semantic-v1.schema.json:67 |
| FND-003 | low | The ix_authority doc comment still says the authority is "compared exactly"; the comparison now folds ASCII case | src/corpus/resolve.rs:331-334 |
| FND-004 | low | The shared resolve_target fold also changes FR-075 model reference cells (target.rs resolve_reference) and systems-model cells (systems.rs:343): an own-package ix:// cell that differs from the package only in case now resolves instead of being refused. FR-075 does not state this and no test covers it | src/semantic/target.rs:294, src/semantic/systems.rs:343 |

## Examined scope

```yaml
scope:
- id: resolve.rs add_target
  path: src/corpus/resolve.rs
  role: examined
  excerpt: "let merged = match (o.get_mut().take(), authority) { (Some(mut a), Some(b)) => { a.extend(b); Some(a) } _ => None };"
- id: validate.rs cross_repo
  path: src/corpus/validate.rs
  role: examined
  excerpt: "spec.bundle_package().is_some_and(|package| { !edge.authorities.is_empty() && edge.authorities.iter().all(|authority| !authority.eq_ignore_ascii_case(package)) })"
- id: target.rs resolve_target
  path: src/semantic/target.rs
  role: examined
  excerpt: "let parsed = semantic_identity(cell, alphabet).ok_or(Unresolved::Malformed)?; if !parsed.package().eq_ignore_ascii_case(package) { ... } (parsed.path(), format!(\"ix://{package}/{}\", parsed.path()))"
- id: target.rs semantic_identity
  path: src/semantic/target.rs
  role: examined
  excerpt: "let package_ok = |s: &str| { s.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit()) && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || \"._-\".contains(c)) };"
- id: relations.rs frontmatter_edges
  path: src/semantic/relations.rs
  role: examined
  excerpt: "let owner = if parsed.package().eq_ignore_ascii_case(package) { package } else { parsed.package() }; format!(\"ix://{owner}/{}\", parsed.last_segment())"
- id: relations.rs DeclaredInFrontmatter compare
  path: src/semantic/relations.rs
  role: examined
  excerpt: ".any(|(verb, fm_target)| verb == row.verb && fm_target == identity)"
- id: TC-2151
  path: tests/it/bundle_validate.rs
  role: examined
  excerpt: "authority_comparison_folds_ascii_case"
- id: TC-2152
  path: tests/it/semantic_contract.rs
  role: examined
  excerpt: "relationship_own_package_check_folds_ascii_case"
```

## Verdict

Changes requested (high). resolve.rs and validate.rs are correct and idiomatic. There are no panics and no unsafe code. Every other site that compares an authority or package was grepped; the four changed sites are all of them. Note that `Edge.authorities` replaces the public `Edge.authority` field, which is acceptable before release.

FND-001 suggested fix: in `resolve_target`, parse with `IxRef::parse`, do the folded own-package match first, and run the lower-case shape check only on the foreign or import path, or after lowering. FND-002 is spec-level and needs a decision: either fold the declared package to lower case when building identities, or refuse a mixed-case bundle package.
