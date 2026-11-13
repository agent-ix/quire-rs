---
id: SR-20261113045742
title: "Code review (with rust-review lane) — PLAT-1556 cross-repository edge resolution (quire-rs PR 665)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs@fb44ef9; CHANGELOG.md, src/corpus/declared_tables.rs, src/corpus/glossary.rs, src/corpus/mod.rs, src/corpus/query.rs, src/corpus/required_relations.rs, src/corpus/resolve.rs, src/corpus/spec.rs, src/corpus/validate.rs, src/lib.rs, tests/fixtures/traceability/cross-repo/manifest.yaml, tests/it/cross_repo.rs, tests/it/main.rs"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-085", type: references}
---

## Summary

Ticket: PLAT-1556. PR head `fb44ef9` (one commit over origin/main), scoped to `git diff origin/main...HEAD` (13 files, +1624/-251). Code review with the Rust lane folded in (rust-review, rust-style loaded; repository CLAUDE.md conventions take precedence). No build or test was run by the reviewer: the brief limits verification to reading, and the coder reports focused tests, clippy, `check --features python` and `audit-static` green.

Checked and clean: the engine holds no hazard, threat or other chain-specific archetype or verb name (FR-058-CON-2); the kinds `Req`/`Haz`/`Note` and the verb `mitigates` live only in the test fixture manifest. No pin, digest, SHA, ledger, compatibility shim or fallback is introduced. No `unsafe`, no new `HashMap` in a determinism-banned module, no `unwrap` outside tests; the two `expect`s in `from_repo`/`with_bundle_package` guard an explicit invariant (a one-member build cannot fail). Resolution (`locate`) matches the FR-085 Resolution rules: identity mode looks up the authority's member with ASCII folding, an unloaded authority is Dangling and recorded in `authorities`, a loaded authority with a missing id is Dangling with no authority (so `dangling-reference` fires, FR-085-AC-17), bare ids and relative links stay in the source member, and an identity-less single member resolves lexically (AC-10). The FR-038-AC-9/AC-10 merge rule (own-repo or bare spelling plus foreign spelling of an absent id is one edge that stays dangling) is preserved because the stub key includes `target_slot`. Relations read the corpus-wide edge set through slot-keyed `edges_from`/`edges_to`, so AC-2/AC-3 hold. `trace:target-kind-mismatch` routes through the same FR-057 path as other relation findings with a `warning` default. Member views partition documents and edges correctly because `resolve` emits edges grouped by source member.

## Verdict

**Changes requested: one medium, six low.** The core resolution, member-view partitioning and relation changes are correct and idiomatic. FND-001 attributes no-frontmatter warnings to members by a path-prefix guess that silently drops them when the root spellings differ. The rest are robustness or API-hygiene issues on paths FR-085 does not exercise. The spec and test findings are in the gap analysis SR-20261113045743.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | In a member view, `DocumentWithoutFrontmatter` diagnostics (held corpus-wide) are attributed to a member by `path.starts_with(document_root)`. That is a lexical prefix test on two independently spelled paths. If a caller loads a member as `spec` and passes `./spec`, or loads it by an absolute path and passes a relative root, every no-frontmatter warning for that member is silently dropped. A member root nested under another member's root would also be reported twice. No test puts a no-frontmatter file in a multi-member corpus. Fix: keep each member's load diagnostics with the member (a range, as is done for slots and edges) and report the view's own, then add a test with one such file in each member. | src/corpus/validate.rs:345-352, src/corpus/spec.rs:167-200 |
| FND-002 | low | `check_relation` iterates `by_type` documents and re-derives the slot with `spec.slot_of(&doc.id)`. On an unscoped multi-member `Spec` handed to the public `validate_bundle`, `slot_of` returns `None` for any id held by two members, so that document is silently skipped by every required relation (and its kind mismatches are never reported). Fix: iterate the slots `by_type_slots` already produces instead of round-tripping through the id. | src/corpus/required_relations.rs:256-266 |
| FND-003 | low | `assurance.rs` still looks things up by id on the corpus edge set. `project_relations` calls `spec.by_id(&edge.source)`, which returns `None` for an id shared by two members, so exporting a multi-member corpus fails with `MissingSourceBytes`. `required_relation_satisfied` judges the target kind through `by_id(other)` rather than `target_slot`, so it can disagree with `required_relations.rs`. Fix: use `edge.source_slot`/`target_slot` through the crate-internal `doc()`, or document and reject a multi-member corpus at the exporter boundary. | src/assurance.rs:507-512, src/assurance.rs:678-690 |
| FND-004 | low | `with_bundle_package` now clones every `LoadedDocument` and rebuilds the corpus, redoing indexing, duplicate detection and full resolution. This doubles resolution work on every validate path that declares a package (NFR-001/NFR-015 are perf-gated). On a corpus of two or more members it silently returns `self` unchanged, so the identity the caller asked for is ignored without any signal. Fix: re-run only `resolve` over the existing documents (the documents and id index do not change). Make the multi-member case visible: return a `Result`, or document it and add a test. | src/corpus/spec.rs:262-280 |
| FND-005 | low | `Edge` (all-`pub` before this PR) gains `pub(crate) source_slot`/`target_slot`. Downstream code can no longer construct or exhaustively destructure it. The derived `PartialEq` now compares corpus slots too, so two edges that agree on every public field compare unequal across corpora that differ only in member order. Fix: implement `PartialEq` over the public fields (or move the slots into a side table), and mark `Edge` `#[non_exhaustive]` deliberately. | src/corpus/resolve.rs:25-53 |
| FND-006 | low | The CHANGELOG puts several behaviour changes for existing callers under `### Added`, none marked **Breaking** as the PLAT-1501 entry is: `with_bundle_package` changes resolution, so a foreign-authority `ix://` to a locally held id becomes Dangling and stops satisfying constrained relations; `by_id`/`outgoing`/`referencing` return nothing for an id shared across members; and `Edge` gains non-public fields. Fix: add a `**Breaking, PLAT-1556:**` entry listing them. | CHANGELOG.md:10-23 |
| FND-007 | low | `Spec::from_members(vec![])` returns `Ok` with an empty corpus, but FR-085 says a corpus is built from one or more members. `validate_corpus` then reports nothing. Fix: add a `CorpusError::NoMembers` variant (or state in FR-085 that zero members is a valid empty corpus) and test it. | src/corpus/spec.rs:143-170 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| src/corpus/resolve.rs (Index, locate, resolve, build_path_index, Edge) | examined | Correct against FR-085 Resolution; FND-005 |
| src/corpus/spec.rs (CorpusMember, CorpusError, build, with_bundle_package, views, by_qualified_id) | examined | FND-004, FND-007 |
| src/corpus/query.rs (slot-keyed queries, member-scoped edges) | examined | Correct |
| src/corpus/required_relations.rs (kind mismatch, dangling naming, acyclic over corpus) | examined | FND-002 |
| src/corpus/validate.rs (validate_corpus, validate_view, dangling-reference) | examined | FND-001 |
| src/corpus/declared_tables.rs, glossary.rs, mod.rs, lib.rs | examined | Mechanical; correct |
| CHANGELOG.md | examined | FND-006 |
| src/assurance.rs | context_only | Unmigrated id-keyed consumer; FND-003 |
| FR-058-CON-2 / CON-3 (no chain names in engine) | examined | Clean |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Commit 148abdc stores each member's load-diagnostics range (`Member::load_diagnostics`), and `validate_view_findings` reads `spec.load_diagnostics()` with the path-prefix filter removed. Test FR-085-AC-24 spells the roots `<DIR>/spec/../spec`, which the old filter would not match, and expects one `no-frontmatter` per member with its `repo`. It passes at 148abdc. |
| FND-002 | fixed | Commit 148abdc makes `check_relation` iterate `spec.by_type_slots(&relation.from)` and drops the `slot_of(&doc.id)` round-trip. Test `an_id_shared_by_two_members_is_still_checked_in_each` expects 2 `req-no-haz` findings from `validate_bundle` over an unscoped two-member corpus; it passes. |
| FND-003 | fixed | Commit 148abdc makes `build_assurance_export` return `AssuranceError::MultiMemberCorpus` for more than one member, so the id-keyed exporter only ever sees one member. Tested by `the_assurance_export_refuses_a_multi_member_corpus`. |
| FND-004 | fixed | Commit 148abdc: `with_bundle_package` returns `Result` (`CorpusError::IdentityFixed` on several members). It reuses the inner corpus through `Arc::try_unwrap`, falling back to a clone when the corpus is shared, and re-runs only `resolve_edges`. No in-repo non-test caller breaks; quire-cli and quoin do not call it. |
| FND-005 | fixed | Commit 148abdc makes `Edge` `#[non_exhaustive]` with a hand-written `PartialEq`/`Eq` over the public fields only. AC-11's test now asserts that every edge of the a,b corpus is `contains`-equal to an edge of the b,a corpus. |
| FND-006 | fixed | Commit 148abdc adds `### Changed` with three `**Breaking, PLAT-1556:**` entries: `with_bundle_package` now returns `Result` and changes resolution; `Edge` is non-exhaustive with slot-free equality; `by_id`/`outgoing`/`referencing` return nothing for a shared id, the exporter refuses a multi-member corpus, and `BundleFinding` gains `repo`. These cover every public break. |
| FND-007 | fixed | Commit 148abdc adds `CorpusError::NoMembers`, returned by `build` for an empty member list. FR-085 now states that zero members is an error, and test FR-085-AC-22 asserts it. |
