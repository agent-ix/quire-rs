---
id: SR-20261112163906
title: "Code review (incl. rust-review) of PLAT-1477 FR-058 where selector (quire-rs PR 654)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@253ccad; src/corpus/required_relations.rs; src/corpus/validate.rs; src/corpus/vocabulary_coverage.rs; src/traceability.rs; tests/it/required_relations_where.rs; tests/it/required_relations.rs; tests/fixtures/traceability/required-relations-where/; CHANGELOG.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-058
    type: references
---

# Code review of PLAT-1477 FR-058 where selector (quire-rs PR 654)

## Summary

Ticket: PLAT-1477. Scope is `git diff origin/main...253ccad` (12 files, +771/-30). The code-review and rust-review lanes are in this one file.

Checks run on the review worktree with `locked-build.sh`, `-j 4` and `CARGO_PROFILE_DEV_DEBUG=0`:

- `cargo test --test it required_relations`: 23 passed. That is the 13 existing TC-898..910 tests and the 10 new TC-2248..2257 tests.
- `cargo clippy --all-targets -- -D warnings`: clean.
- Lib tests for `traceability`, `vocabulary_coverage` and `required_relations`: 17 passed. Integration tests for `vocabulary_coverage`, `audit_table` and `traceability`: 22 passed.
- `examples/spec_validate.rs` against spec-artifacts-process v0.31.0 and spec-artifacts-iso v0.25.0: 204 documents, 0 failed, 43 warnings. The only FR-058 warnings are on AC-10 and AC-11, which are already on main.

A 14-mutation sweep killed 13 of the 14 mutants. Results are in the gap-analysis artifact.

Examined and clean:

- **The engine holds no names (CON-2, CON-3).** The added `src/` lines contain no archetype, verb, field or value name: a grep for quality_attribute, security, safety, NFR, threat, hazard and mitigat finds nothing. The field and values are read from the manifest. The enum is read from the `from` archetype's frontmatter schema through the existing FR-059 `enum_at`.
- **The posture deviation is correct.** Before this PR, `validate_required_relations` routed unconfigured findings through `posture_tier(posture)`, which made them hard errors under `Strict`. That contradicts FR-058-AC-7 ("stays valid under both postures"), FR-058-AC-18 and the FR-058 Severity section. It also contradicts FR-057's rule that a corpus check introduced after FR-057 SHALL declare `warning`. The PR routes unconfigured findings as `GrammarSeverity::Warning`. FR-057 registry promotion still works: TC-903 and TC-2253 map the key to `error` and assert `!is_valid()`. No existing test asserted the old Strict behaviour, and no corpus case uses these checks. The quire-cli fixture only names the manifest key. If the old routing is restored (mutation K), TC-2253 and TC-2257 fail. The change is listed under `### Fixed` in CHANGELOG (see FND-003 on the wording).
- **The AC-20 suffix is limited to `where` relations.** It is appended only when the relation has a `where` and the document's value is a string. TC-910 still passes and TC-2255 asserts the exact pre-change sentence for a relation without `where`, so findings without `where` are unchanged byte for byte.
- **The serde handling is correct.** `deserialize_with = deserialize_present` keeps `where: null` as `Some(Null)`, and validation rejects it (TC-2254 case 2). `deny_unknown_fields` on `RequiredRelation` is kept. Unknown keys inside `where` are rejected by `WhereSelector::parse`. A load failure rejects the whole module (AC-19, "applies none").
- **The AC-22 reading fits the AC.** A field counts as "carried" when the frontmatter key is present, even with a null value. That is a fair reading of "carried by any `from` document": a misspelt manifest field is carried by no document. Ignoring `exclude` matches the AC, which says "any `from` document". The early return after a dead-field finding is equivalent to the AC. `enum_at` and `declares_property` walk the same `properties` maps, so a field the schema does not declare has no enum, and the AC leaves values unchecked in that case.
- **The selection order is `from`, then `where`, then `exclude`.** Both filters only skip documents, so an excluded document is exempt whatever `where` selects (AC-16).
- **The TC-906 and TC-907 struct literals** add `where_clause: None` and nothing else.
- **PR #655 (PLAT-1478) does not overlap.** No PR #655 and no `plat-1478` branch exist on the remote yet. This PR does not change `BundleFinding`; it only calls `BundleFinding::in_pack`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | `RequiredRelation.where_clause` is a public untyped `Option<serde_json::Value>`, parsed again by `selector()` on every call. `selector()` silently returns `None` for a malformed value, so `check_relation` needs a defensive "unreachable" early return. A validated `WhereSelector`, parsed once at load (for example through a `TryFrom<Value>` deserialize that names the relation), would encode the invariant in the type, as CLAUDE.md asks under "Design taste". | src/traceability.rs:513-538, src/traceability.rs:610-613, src/corpus/required_relations.rs:241-243 |
| FND-002 | low | `WhereSelector::selects` calls `fold_selector_value` on every `values` entry for every document, which allocates documents × values strings in a corpus sweep. Folding the entries once in `parse`, for example as a `BTreeSet<String>`, would make each document cost one fold and one lookup. | src/traceability.rs:597-606 |
| FND-003 | low | The CHANGELOG `Fixed` entry says required-relation findings are now advisory under `Strict`. The routing change also moves `cyclic-<verb>` (AC-5) and `undeclared-relation-vocabulary` (AC-11, AC-22) findings from error to warning under `Strict`, because every FR-058 finding goes through one `route` call. The entry should say "FR-058 trace findings (required relations, cycles, dead declarations)" so a Strict user can see that the build verdict changed. | CHANGELOG.md:17-20, src/corpus/required_relations.rs:74-80 |

## Verdict

No correctness defects were found. The three findings are low: one idiom, one minor efficiency issue and one CHANGELOG precision issue. The posture change is spec-correct and covered by tests.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | Fix round 1 moved validation of `where` into a `#[serde(try_from = "RawRequiredRelation")]` deserialize path, while `Serialize` stays derived on `RequiredRelation` and `WhereSelector`. No test checks that the two halves agree. I measured the current shape with a throwaway test: `where` serializes as the authored `{field, values}` (padding kept, `folded` skipped), a relation without `where` serializes with no `where` key, and both JSON and YAML round-trip back to an equal model. So nothing is broken today. The rest of `traceability.rs` pins every optional field with a serialize/deserialize test (TC-1804, the FR-050-AC-7 `exclude` tests), and this one has none. Add one assertion: a model with a `where` round-trips through `serde_yaml`, and one without serializes no `where` key. | src/traceability.rs:487-562, src/traceability.rs:572-581 |
| FND-005 | low | `WhereSelector` has public `field` and `values` beside a private `folded` cache that `selects()` reads. Code that changes `values` after load (the struct is `Clone` and reachable through the public `RequiredRelation.selector`) leaves `folded` stale, and selection silently ignores the change. Make the fields private with `field()` and `values()` accessors, or derive the folded set on demand, so the invariant cannot be broken from outside. | src/traceability.rs:572-581, src/traceability.rs:634-642 |

## Dispositions

Round 1, checked at 7a35156. Checks:

- `cargo test --test it required_relations`: 23 passed.
- `cargo clippy --all-targets -D warnings`: clean.
- Lib tests for traceability, vocabulary_coverage and required_relations: 17 passed. Integration tests for vocabulary_coverage, audit_table, traceability and bundle: 49 passed.

The mutation sweep was re-run, adding two new mutants (unfolded `selects` and `where: null` read as absent), and killed 15 of 16. The Strict-routing mutant is now also killed by TC-903.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | `RequiredRelation.selector: Option<WhereSelector>` is validated once at deserialize through `RawRequiredRelation` and `TryFrom`. `selector()`, the repeated parse and the defensive return are removed. `where: null` still fails at load, naming the relation and `where` (TC-2254 and mutation N). |
| FND-002 | fixed | `WhereSelector` folds `values` once into a `BTreeSet` in `parse`. `selects()` does one fold and one lookup per document. |
| FND-003 | fixed | The CHANGELOG `Fixed` entry names the required-relation, cyclic `acyclic_edges` and `undeclared-relation-vocabulary` findings. |
