---
id: SR-20261113045739
title: "PLAT-1520 quire-rs PR664 spec-review integrity analysis"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs PR664 (git diff origin/main...HEAD); spec/functional/FR-084-two-way-relation-coverage-report.md; spec/functional/FR-085-cross-repository-edge-resolution.md; spec/functional/FR-058-upward-trace-completeness.md; spec/functional/index.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-084
    type: references
  - target: ix://agent-ix/quire-rs/FR-085
    type: references
---

## Summary

Ticket: PLAT-1520. Spec-only PR. Integrity pass (completeness, consistency, atomicity) over FR-084 (two-way relation coverage report), FR-085 (cross-repository edge resolution), the FR-058 cross-reference note and the index. Read against FR-024, FR-025, FR-026, FR-027, FR-050 (AC-47..51), FR-057 and FR-058 on origin/main. `quire validate` (quire 0.44.0, engine 0.59.1) passes over the changed files; the only warnings are two pre-existing `ac:non-canonical-shape` warnings on FR-058-AC-21/22, outside the diff.

Ticket acceptance cross-check: PLAT-1554 (forward view with uncovered and covered-but-unverified rows, reverse-view gaps, per-requirement verification from the matrix, chain read from `required_relations`, no hazard/threat names in the engine) is covered by FR-084-AC-1, AC-3, AC-5..7 and AC-14. Human/JSON rendering is delegated to quire-cli FR-030 (quire-cli#180), which agrees with FR-084's row fields. PLAT-1556 (cross-repo satisfied, dangling named, kind mismatch, two-repo fixture) is covered by FR-085-AC-2, AC-4, AC-6 and AC-12. No paywalled standard text is quoted. No pin, tracking table or compatibility layer is introduced, apart from the "before this FR" baseline in FND-001.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-085-AC-10 requires a one-member, identity-less corpus to produce output byte-identical to a corpus built before this FR. FR-085-AC-6 and the "Kind of a resolved target" section add a new `trace:target-kind-mismatch` finding "equally within one member". Any single-repo corpus with a wrong-kind accepted-verb edge therefore gains a finding, so the two ACs cannot both pass. AC-10 also needs a pre-FR baseline to compare against. Fix: restate AC-10 as an invariance over resolution only (resolved/dangling classification and edge sets identical to FR-026 lexical resolution), with kind-mismatch findings excluded from it. | spec/functional/FR-085-cross-repository-edge-resolution.md:88 |
| FND-002 | high | FR-085 makes document ids unique within a member but not across members, and addresses documents by `(identity, id)`. FR-025 keys the corpus by id (`HashMap<ArtifactId, usize>`), and FR-027 exposes `by_id(&str)`. Neither is amended, and FR-085 does not say what `by_id`, the id index, or FR-026's bare-id resolution return when two members hold the same id. FR-085 `extends` FR-025 but leaves its id-keying contract contradicted. Fix: state the multi-member addressing in FR-025/FR-027 (or in FR-085 as an explicit amendment), with an AC for `by_id` on a duplicated id. | spec/functional/FR-085-cross-repository-edge-resolution.md:36 |
| FND-003 | medium | FR-084 says `state` is `uncovered` when `linked` is empty and that a `dangling` entry never makes a row covered. FR-058 says that with `to: []` a dangling (cross-repo) edge satisfies the relation. A `to: []` relation whose only edge is dangling therefore raises no FR-058 finding but reports `uncovered` here, so the report and the check disagree on the same declaration. Fix: list such an edge in `linked` (or make `state` follow FR-058 satisfaction) for `to: []`, and add an AC. | spec/functional/FR-084-two-way-relation-coverage-report.md:66 |
| FND-004 | medium | FR-084 counts only `tagged` and `qualified-record` as backed, so `method-without-symbol` (a criterion whose declared method, such as Inspection or Analysis, needs no source symbol, per FR-050-AC-49) is counted as unverified (FR-084-AC-5). A requirement verified wholly by inspection always reports `unverified`, and one with a single inspection criterion can never be `verified`. The FR does not say whether this is intended. Fix: state the rule and its reason, or exclude `method-without-symbol` from the denominator as `retired` is. | spec/functional/FR-084-two-way-relation-coverage-report.md:74 |
| FND-005 | medium | FR-085 resolution only covers "a corpus where at least one member declares an identity". It does not say whether a multi-member corpus may include identity-less members, or how their own `ix://<org>/<repo>/...` self-references resolve. Read literally, adding one identified member turns every `ix://` self-link in an identity-less member to `Dangling`, because no member declares its `<org>/<repo>`. It also leaves unsaid whether two identity-less members are allowed. Fix: require every member of a multi-member corpus to declare an identity (an error otherwise, naming the member), or specify the identity-less case, with an AC. | spec/functional/FR-085-cross-repository-edge-resolution.md:42 |
| FND-006 | low | FR-085 introduces `trace:target-kind-mismatch` next to the existing `trace:reference-kind-mismatch` (FR-049-AC-12, FR-057). The FR does not distinguish the two, and the names differ by one word. A reader configuring severities cannot tell which one fires. Fix: one sentence naming the difference (relation target kind vs. trace-reference kind) and confirming the new key's `warning` default under FR-057's rule for keys added after it. | spec/functional/FR-085-cross-repository-edge-resolution.md:62 |
| FND-007 | low | FR-084-AC-12 requires "byte-identical serialized reports", but FR-084 defines no serialization: there are no key names beyond the row table, no form for `summary`, and no form for `selector`'s value as authored. Byte identity is therefore defined by whatever the implementation emits. Fix: state that the report serializes as JSON with the listed field names in a fixed key order. | spec/functional/FR-084-two-way-relation-coverage-report.md:123 |

## Verdict

Changes requested. FND-001 and FND-002 (high) are contradictions between ACs, and between FR-085 and the FR-025/FR-027 id contract. FND-003..005 (medium) are edge cases in the report and resolution rules that two implementers would read differently. FND-006..007 (low) are wording gaps. The forward/reverse row model, the derivation of verification from the matrix, the `not-tracked` aggregation, and selection/ordering are sound and cover PLAT-1554. Resolution by `(identity, id)` with no fetch, dangling-named and kind mismatch covers PLAT-1556. Both FRs keep chain names out of the engine (FR-058-CON-2/3, FR-084-AC-14).

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-008 | medium | The new Serialization section says "No other key is emitted" but leaves several shapes undefined, so the byte identity FR-084-AC-12 asks for is still open. It gives no shape for a `dangling` or `kind_mismatch` array entry. It gives no key for "the reference as authored" on a `linked` entry with `resolution: dangling`, which has no `id`, `path` or `archetype` to fill the document keys and no repo/id to order `linked` by. The obliged document is serialized as `{id, path, archetype, repo}` only, so the `backed`/`unbacked`/`not_measured` counts the verification section says "a document's entry carries" are emitted for linked documents but not for the obliged document, which is the one whose status decides a reverse-view row. Fix: give each entry shape and the dangling ordering key, and either emit `criteria` on `document` or narrow the counts sentence to linked entries. | spec/functional/FR-084-two-way-relation-coverage-report.md:108-120 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-009 | medium | The round-2 Serialization text orders resolved `linked` entries "by repo then path then id", but FR-084-AC-2 (unchanged) requires `linked` "ordered by repo then id". Within one member, path order and id order differ whenever a file's path does not sort with its id (for example `a/FR-010-x.md` and `b/FR-002-y.md`), so an implementation cannot satisfy both, and the quire-cli FR-030 byte-identity snapshot inherits whichever one the coder picks. Fix: use one key in both places, either by changing AC-2 to repo, path, id to match the row ordering, or by changing the Serialization sentence to repo then id. | spec/functional/FR-084-two-way-relation-coverage-report.md:133 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | - |
| FND-002 | fixed | - |
| FND-003 | fixed | - |
| FND-004 | fixed | - |
| FND-005 | fixed | - |
| FND-006 | fixed | - |
| FND-007 | fixed | - |
| FND-008 | fixed | - |
