---
id: SR-20261103118133
title: "spec-review (base) — PLAT-1357 FR-038-AC-9/AC-10 (quire-rs PR 620)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs PR620 head 65437a4; spec/functional/FR-038-okf-bundle-validation.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-038
    type: references
---

## Summary

Ticket: PLAT-1357. Base spec-review over `git diff origin/main...65437a4`: FR-038-AC-9 amended from exact to ASCII case-insensitive authority-versus-package comparison, new FR-038-AC-10, new TC-2151 row and its coverage-mapping row, and a PLAT-1357 note. Checked against the code on origin/main (`src/corpus/validate.rs:317-320` compares `authority != package`; `src/corpus/resolve.rs:85-91` merges multiple foreign spellings of one absent target with a byte-order `min`), quire-cli origin/main (`src/commands/validate.rs:459` lowercases the declared package and passes it to both `with_bundle_package` and `validate_document_in_bundle`), and the semantic own-package check (`src/semantic/target.rs:104`, FR-076). FR-038 is the correct owning FR for the dangling-reference skip.

## Verdict

Changes requested (two medium). The amendment matches the ticket and the planned code (PR 621 uses `eq_ignore_ascii_case`). AC-9 and AC-10 leave the result for one absent target spelled under both the own authority and a foreign authority undefined, and the code makes that result depend on letter case. The same byte-exact own-package comparison remains in the FR-076 relationships path, which receives the same lowercased package from quire-cli.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-038-AC-9 defines the merge only for bare-id plus foreign spellings. One absent target spelled as an own-authority `ix://` link and as a foreign one from the same source is merged in `resolve.rs` by byte-order `min` of the authorities, so `ix://zz/o/X` + `ix://acme/widgets/X` (package `acme/widgets`) stays dangling while `ix://Zz/o/X` + the same own link is skipped (`'Z' < 'a'`), and `ix://aaa/b/X` + the own link is skipped at any case. AC-10's claim that case does not change the result fails on this path. Fix: state that an own-authority spelling (case-folded) claims the target local like a bare id, so the edge stays dangling. | spec/functional/FR-038-okf-bundle-validation.md:105; src/corpus/resolve.rs:85-91 |
| FND-002 | medium | The FR-076 own-package test (`src/semantic/target.rs:104`, `parsed.package() != package`) stays byte-exact, and quire-cli passes the same lowercased package into `validate_document_in_bundle`. A mixed-case own-package `## Relationships` target such as `ix://Acme/widgets/FR-001` is then refused as `target-qualified-not-imported`. After this PR, FR-038 and FR-076 define "own package" differently. Fix: add the matching FR-076 AC (or a tracked ticket) so both paths fold ASCII case. | spec/functional/FR-076-relationships-extraction.md:164-169; src/semantic/target.rs:104 |
| FND-003 | low | The `PLAT-1357 note` ("AC-10 is new ...") is change-history prose in a requirement and goes stale on the next edit. The lasting rationale (quire-cli lowercases the package) belongs in AC-10 or Rationale, not a ticket-dated note. | spec/functional/FR-038-okf-bundle-validation.md:114-116 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-038-AC-9 | examined | FND-001 |
| FR-038-AC-10 | examined | See criterion-strength SR |
| TC-2151 | examined | Matches AC-10; status planned |
| FR-038 statement | context_only | Owner of dangling-reference |
| FR-076 | context_only | FND-002 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | FR-076-AC-23 says a mixed-case own-package target "qualifies as the own package" but not which identity it lowers to. `resolve_target` keeps the cell as written (`cell.to_string()`), while a bare id lowers to `ix://{package}/{id}`. So one artifact named as `X` and as `ix://Acme/Widgets/X` would lower to two different identities under package `acme/widgets`, and two implementers could choose either spelling. Fix: state the lowered identity, e.g. `ix://<bundle package as declared>/<id>`. | spec/functional/FR-076-relationships-extraction.md:320; src/semantic/target.rs:104-114 |
| FND-005 | low | FR-076-AC-23 names the internal Rust variant `UnimportedPackage` as the refusal. The spec's observable outcome for that row is `semantic.invalid-model-cell` with reason `target-not-id` (FR-076 Refusals table; fixture case `target-qualified-not-imported`). Name the observable code and reason. | spec/functional/FR-076-relationships-extraction.md:320 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-006 | high | FR-076-AC-23 now lowers a case-matching own-package target to `ix://<declared bundle package>/<id>` in the declared spelling, "not the cell's". The FR-076 statements it verifies were not amended and still say the opposite: "The engine SHALL carry an `ix://<org>/<repo>/<id>` target verbatim when `<org>/<repo>` is the bundle's package" (exact package match, verbatim identity), and with no bundle index "an own-package `ix://<org>/<repo>/<id>` verbatim". The frontmatter-identity paragraph also keeps `ix://<org>/<repo>/<id>` as written, so a frontmatter `ix://Acme/Widgets/X` and the lowered row `ix://acme/widgets/X` no longer compare equal for `declared-in-frontmatter`. Fix: amend those statements to the case-folded match and declared-spelling identity. | spec/functional/FR-076-relationships-extraction.md:163-169,198-200,320 |

## New findings (disposition pass 3)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | low | The FR-076 summary row in `spec/tests.md` now lists AC-1..23 and TC-2152 with status "✅ Implemented", but TC-2152 (FR-076-AC-23) is planned and the code does not fold case yet (`src/semantic/target.rs:104`). The row overstates status until PR 621 lands; mark AC-23 planned (as the FR-075 row does for its planned ACs) or let 621 make it true. | spec/tests.md:115 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-038-AC-10 now states that an own-repo spelling plus any foreign spelling is one edge that stays dangling whatever the case or byte order, with the zz/Zz/aaa vectors. |
| FND-002 | fixed | New FR-076-AC-23 (TC-2152) folds ASCII case on both sides for the Relationships own-package check. |
| FND-003 | fixed | The PLAT-1357 change-history note is removed; the rationale is in FR-038-AC-10. |
| FND-004 | fixed | FR-076-AC-23 states that a matching target lowers to `ix://<declared bundle package>/<id>` in the declared spelling, with examples in both directions. |
| FND-005 | fixed | FR-076-AC-23 names the refusal as `semantic.invalid-model-cell` with reason `target-not-id`. |
| FND-006 | fixed | The three FR-076 statements now match ignoring ASCII case and lower to `ix://<bundle package>/<id>`; the only remaining "verbatim" is for imported packages. AC-23 adds the frontmatter `declared-in-frontmatter` vector. |
