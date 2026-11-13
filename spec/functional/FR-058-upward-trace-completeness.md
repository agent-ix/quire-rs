---
id: FR-058
title: "Upward-Trace Completeness"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-026"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-057"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "implements"
    cardinality: "1:1"
---
# FR-058: Upward-Trace Completeness

## Description

[FR-049](./FR-049-verification-reference-integrity.md) and
[FR-050](./FR-050-declarative-coverage-computation.md) build **downward** coverage — FR → AC → TC
→ code — which answers *is what we wrote verified*. Nothing answers *is anything missing*, and
nothing operating over the existing spec text can: a requirement nobody wrote leaves no trace to
follow.

**Upward tracing is the only analysis class that finds a missing requirement.** A functional
requirement with no upstream need is a feature nobody asked for. A stakeholder requirement with no
downstream implementation is a need nobody built. Both are invisible to every check that starts at
a document and walks down, and both are ordinary findings once the graph is read in the other
direction.

This FR adds a corpus check pack over the edge set [FR-026](./FR-026-intra-spec-reference-resolution.md)
already resolves. It is **not** a new model: no new walk, no new index, no second graph.

### The declaration

Nothing about the chain is engine knowledge. `quire-rs` SHALL read the contract from the active
module's declared `traceability.required_relations`, each entry naming:

| Field | Meaning |
|---|---|
| `name` | The declaration's own name, reported in the finding |
| `from` | Archetype whose documents carry the obligation |
| `edges` | Accepted verbs — **any one** satisfies the relation |
| `to` | Accepted archetypes at the other end; empty means any document |
| `direction` | `outgoing` (the `from` document is the source) or `incoming` (it is the target) |
| `check` | The `<check>` half of the `trace:<check>` severity key ([FR-057](./FR-057-corpus-check-severity.md)) |
| `exclude` | Scope-relative globs exempt from this relation |
| `where` | Optional. Narrows the obligation to `from` documents whose frontmatter `field` holds one of the declared `values` |

**Direction is why this is one declaration and not two checks.** "Every FR traces to a need" and
"every need is implemented" are the same edge read from opposite ends, so they are one type with a
direction rather than two engine checks that can drift apart.

The engine SHALL know no archetype name, no verb, and no chain. A module that declares
`from: hazard, edges: [mitigated_by], to: [FR], direction: outgoing` gets hazard-coverage checking
from the same code — which is how `agent-ix/spec-objects-security#5` becomes manifest data instead
of a second engine FR.

`traceability.acyclic_edges` names verbs that MUST NOT form a cycle. A requirement that
transitively refines itself states nothing, and no per-document check can see it because the defect
is a property of the graph. Each cycle SHALL be reported **once**, keyed on its smallest member, so
a three-node cycle is one finding rather than three.

### Narrowing the obligation by a field value

`from` selects by kind, and some obligations belong to only part of a kind. The reverse rule "every
security- or safety-classified requirement traces to an upstream threat or hazard, else it is
unjustified scope" applies to the `NFR` documents whose `quality_attribute` is `security`
or `safety`, not to every NFR. An optional `where` on a relation expresses that:

```yaml
required_relations:
  - name: security-safety-nfr-needs-upstream-hazard
    from: NFR
    where: { field: quality_attribute, values: [security, safety] }
    edges: [mitigates]
    to: [threat, hazard]
    direction: outgoing
    check: unjustified-classified-nfr
```

`where` is a mapping of exactly two keys: `field`, one frontmatter key name, and `values`, a
non-empty list of accepted values. The engine knows neither: the manifest names the field and the
values, exactly as it names `from` and `edges`. A relation without `where` obliges every `from`
document, as before.

**How the field is read.** The engine reads the top-level frontmatter key `field` of each `from`
document, the same frontmatter source [FR-082](./FR-082-coverage-matrix-criticality.md)
reads `criticality` from. A document is **selected** when that value is a string equal to one of
`values`, comparing after trimming whitespace and ASCII case folding — the comparison
[FR-049](./FR-049-verification-reference-integrity.md)-AC-13 applies to kind values. A document is
**not selected**, and yields no finding of any kind, when the field is absent, null, empty, not a
string, or a string outside `values`. Not being selected is not a defect: a document outside `values` is correctly out of scope, and the
engine SHALL NOT call an unlisted value wrong. The engine holds no list of permitted values; the
vocabulary comes from the `from` archetype's frontmatter schema, read the way
[FR-059](./FR-059-declared-vocabulary-coverage.md) reads a field's `enum`.

**A declaration that selects nothing reports itself.** A misspelt `field` or `values` entry
selects zero documents and silently disables the relation, the same defect as a `from` naming a
kind nothing has. So a relation with `where` SHALL report under
`trace:undeclared-relation-vocabulary` when `field` is neither a property of the `from`
archetype's frontmatter schema nor carried by any `from` document, or when a `values` entry is
outside that field's schema `enum` and held by no `from` document, both comparisons using the trim and ASCII case folding rule above. Either condition alone makes
the field or entry live. When the schema declares no `enum` for the field, `values` entries are
not checked, because nothing then says what a misspelling is.

**What `where` narrows.** Only the obliged side. It selects which `from` documents carry the
obligation. It does not filter the documents at the other end of the edge, which remain governed
by `edges`, `to` and `direction` alone, and `direction: incoming` still means the selected
document is the edge's target. The order of selection is `from` kind, then `where`, then
`exclude`: a document that `exclude` exempts is exempt whether or not `where` selects it. The
dead-vocabulary rule below is unchanged, and still tests `from` alone.

**Posture and severity.** `where` adds no severity of its own. A finding from a relation with
`where` is the relation's `trace:<check>` finding, advisory by default under both `Okf` and
`Strict` like every finding here, and tunable through the same FR-057 registry key.

**The finding names the selection.** A finding from a relation with `where` names the document,
the declaration, and the selector: the field, and the value the document holds as authored. A
reader sees that an NFR is reported *because* it is classified `security`, rather than being
handed a bare "has no edge" for a document that looks unrelated to the rule.

### Authored inverse verbs

For each verb `V` in its `edges`, a relation SHALL accept an edge authored with the counterpart
label `W` of `V` from the opposite end, reading the authored edge `(b, W, a)` as `(a, V, b)` before
`direction`, `to`, `exclude` and the dangling and kind-mismatch rules of
[FR-085](./FR-085-cross-repository-edge-resolution.md) apply. The counterpart is defined entirely by
the merged registry's inverse index ([FR-041](./FR-041-authorable-inverse-edges.md)-AC-1), so its
forward-key-wins and first-wins precedence carry over: `W` is the counterpart of `V` when the index
maps `W` to `V` (the declared inverse of `V`), or maps `V` to `W` (`V` is itself an inverse label, and
gains its forward verb). A verb the index relates to nothing has no counterpart. The counterpart
is always normalized, even when the relation lists both labels of a pair, so adding a verb to `edges`
never removes an edge a relation accepted. The relation SHALL count an edge between one pair once however
it is authored or listed, and its
[FR-084](./FR-084-two-way-relation-coverage-report.md) `linked` entry SHALL carry the **listed** verb
`V`, the accepted verb, not the authored one. The edge set still records edges as authored (FR-041);
only relation matching normalizes. The manifest declares the pair once, as vocabulary; the engine
knows no verb.

The reading is by the normalized edge, so the cases FR-085 leaves to the edge's direction follow it:

- A wrong-kind normalized source under an `incoming` relation leaves the relation unsatisfied and
  yields no `trace:target-kind-mismatch` (FR-085-AC-25), whichever end authored the edge.
- Under an `outgoing` or `either` relation, a normalized target outside a non-empty `to` yields one
  `trace:target-kind-mismatch`, naming the normalized source (the obliged document) and carrying that
  document's member as `repo` (FR-085-AC-19), the target as authored, and the listed verb.
- An inverse-authored edge `(b, W, a)` whose target `a` is absent is dangling like any edge: it
  satisfies a relation on `b` whose `to` is `[]`, is named as authored in the report's `dangling`
  list, and does not satisfy a relation whose `to` is non-empty.

### Edges to a criterion

An acceptance criterion is a row, not a document, so FR-026 resolves an edge to a criterion id as
`Dangling` (measured on main: `FR-001-AC-1` as an `ix://` target is a dangling reference). A design
element still realizes a criterion by an edge directly to the criterion's own id, an id already minted
as a trace target ([FR-049](./FR-049-verification-reference-integrity.md),
[FR-050](./FR-050-declarative-coverage-computation.md)). For **relation matching only**, an edge whose
target id is not a document id of the target's member, and is an id minted from exactly one document `D` of that member by a **criterion target**, a trace target named by an `obligations:` source (the `target:` field of [FR-053](./FR-053-obligation-record.md)),

SHALL be read as an edge to `D`, keeping the criterion id attached to the edge. A document id wins
over a minted criterion id of the same spelling; a criterion id minted by two documents of one member
stays dangling. The target's member is chosen as FR-085 chooses it, so `ix://o/b/FR-001-AC-1` is read
against the criteria member `o/b` minted. The engine parses no id shape and derives no document from a
prefix: the owner is the document the minted obligation belongs to. Resolution, the edge set and the
`Dangling` classification are unchanged, and a relation report does not list a read edge as
`dangling`. Which trace targets count is the module's declaration, not an engine name: only a target
named by an `obligations:` source, so an id minted by a trace target no such source names (for example a
test-case or requirement-document target) is never read this way. With no `obligations:` source no id
is minted for this purpose and every such edge stays dangling.

> **CR-416 note:** the first section and FR-058-AC-23 to AC-26 were added, and AC-2 now reads "a verb
> the declaration does not list, or the counterpart of one it lists". The original text matched
> only the verb as authored, so a requirement that authored the inverse of a listed verb toward a
> design element was reported as having no edge ([FR-086](./FR-086-design-element-join.md)).
> [FR-084](./FR-084-two-way-relation-coverage-report.md) follows unchanged: its `state` equals FR-058
> satisfaction, and its `linked` `verb` is the accepted (listed) verb it already names.
>
> **CR-417 note:** the second section and FR-058-AC-27 to AC-29 were added. Design elements realize
> criteria, and the only way to address a criterion is its own id, which resolution cannot find.
> The minting set is defined by the `obligations:` source's `target:` field because trace targets carry no kind and the engine may not name one. `#fragment` addressing was rejected because it would change FR-026 resolution for every
> fragment-bearing reference in the ecosystem.

### Severity

Findings ship **advisory**, tunable per declaration through the FR-057 registry — the `check` token
is chosen by the module precisely so `trace:orphan-fr` and `trace:unimplemented-str` are separate
keys. Promotion to `error` requires a corpus sweep and is user-gated, as every check before it.

### What a dangling edge cannot do

An edge to a repository loaded into the same corpus is not dangling: [FR-085](./FR-085-cross-repository-edge-resolution.md) resolves it, and a resolved target outside `to` is a kind mismatch there rather than a silent non-match.

An edge whose target is absent from the bundle SHALL NOT satisfy a relation whose `to` is
constrained. The target is not loaded, so nothing can say what archetype it is, and accepting it
would let a typo satisfy the very requirement it broke. With `to: []` the verb alone is the
contract and a cross-repo edge counts — which is a reading a module opts into deliberately.

### Why a bad declaration must fail at load

Every other declaration in this model fails at load when it cannot be executed, and this one has a
sharper reason to: **its failure modes are quiet and plausible rather than obviously broken.**

A relation with `edges: []` accepts no verb, so nothing can satisfy it and *every* `from` document
is reported. On a real repository that is hundreds of findings against documents that are correctly
linked — a report that reads as a corpus-wide defect and sends someone to fix the specs. The mirror
image is just as bad: a `check` token that cannot form a `trace:<check>` severity key leaves the
relation running with a severity no `--severity` flag and no module override can ever name, so it
cannot be tuned or switched off. A blank verb in `acyclic_edges` walks a graph no edge matches, and
the cycle check silently covers nothing while reading as declared.

None of these is visible in the output. A check reporting everything and a check reporting nothing
both look like "no bug here" from the outside, and the only place they can be reported *against the
declaration that caused them* rather than against innocent documents is module load.

The `check` token is validated with `grammar::is_severity_key` — the same predicate the `--severity`
CLI parser uses — so the manifest and the command line accept exactly one vocabulary rather than
drifting into two.

### A declaration that names a kind nothing has

`edges: []` fails loudly. The opposite typo fails **silently**, and it is the worse of the two.

`from` selects documents by kind. A `from` naming a kind nothing has selects **zero** documents, so
the relation checks nothing at all. Measured on the fixture: changing `from: FR` to `from: FRR`
leaves a genuine orphan requirement unreported and the run comes back clean. One character disables
the check, and the result is indistinguishable from a bundle with nothing wrong.

So a relation naming a kind that **no loaded module declares and no document in the bundle is**
SHALL report itself. Either condition alone makes the kind live: a declared kind with no documents
yet is a contract waiting for content, and an undeclared kind that documents do use is a different
defect other checks already report.

This cannot be a load-time rule. `TraceabilityModel::validate` runs per module at manifest-parse
time, before the merge, and a relation legitimately names kinds another module contributes —
`from: hazard` in `spec-objects-safety` pointing `to: [FR]` from `spec-artifacts-iso`. The merged
registry and the walked bundle are both in hand only at validation.

**Verbs are deliberately excluded from this rule.** A first attempt checked `edges` against the
module's `edge_types` and immediately fired on the fixture's own working declaration, because a
single-module fixture declares no vocabulary of its own while still using `satisfies` correctly —
and [FR-041](./FR-041-authorable-inverse-edges.md)-AC-2 already permits verbs absent from
`edge_types`. That is a bad rule, not a bad fixture. A misspelt verb also fails loudly, which is the
case that needs no help.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-058-AC-1 | A document of the declared `from` archetype with no accepted edge is reported, naming the document and the declaration that asked for it; one that has the edge is not reported. | Test |
| FR-058-AC-2 | Any one of the declared `edges` satisfies the relation; a verb the declaration does not list, and is not the counterpart of one it lists, does not. | Test |
| FR-058-AC-3 | `direction: incoming` reports a document nothing points at over the accepted verbs — the same declaration read the other way. | Test |
| FR-058-AC-4 | A **dangling** edge does not satisfy a relation whose `to` is non-empty, so a typo'd target cannot satisfy the requirement it broke. | Test |
| FR-058-AC-5 | A cycle over a declared `acyclic_edges` verb is reported exactly once, naming the path; the finding is keyed on the cycle's smallest member so rotations collapse. | Test |
| FR-058-AC-6 | Each declared relation carries its own `trace:<check>` severity key, so mapping one `off` leaves its siblings reporting and mapping one `error` promotes only it. | Test |
| FR-058-AC-7 | Findings are advisory by default: a bundle whose only faults are upward-trace findings stays valid under both postures unless a module says otherwise. | Test |
| FR-058-AC-8 | A module declaring neither `required_relations` nor `acyclic_edges` produces byte-identical output to one that never heard of this FR. | Test |
| FR-058-AC-9 | Findings are ordered by document then id, and identical across runs over one bundle ([NFR-006](../non-functional/NFR-006-determinism.md)). | Test |
| FR-058-AC-10 | A declaration that cannot be executed SHALL fail at module load, naming the offending entry: no accepted `edges`, an empty `from`/`check`/`edges`/`to` entry, a duplicate relation name, a `check` token that cannot form a `trace:<check>` severity key, an uncompilable `exclude` glob, or a blank verb in `acyclic_edges`. | Test |
| FR-058-AC-11 | A relation naming a document kind that no loaded module declares **and** no document in the bundle is SHALL report itself under `trace:undeclared-relation-vocabulary`, naming the kind and the declaration. Verbs are deliberately **not** checked this way — an absent `edge_types` entry is legal (FR-041-AC-2) and a misspelt verb already fails loudly. | Test |
| FR-058-AC-12 | A relation with `where: {field: F, values: [V1, V2]}` reports a `from` document whose frontmatter `F` is `V1` or `V2` and which has no accepted edge; it does not report a selected document that has the edge, and it does not report a `from` document whose `F` is a string outside `values`, with or without the edge. | Test |
| FR-058-AC-13 | A `from` document whose field `F` is absent, null, empty, a number, a list or a mapping is not selected and yields no finding, and the run neither errors nor warns about it. | Test |
| FR-058-AC-14 | The field value matches after trimming whitespace and ASCII case folding, so ` Security ` selects against `values: [security]`; `values` entries match the same way, and a value that differs by more than case and surrounding whitespace is not selected. | Test |
| FR-058-AC-15 | `where` narrows only the obliged `from` side: with `direction: outgoing` and with `direction: incoming`, an edge to or from a document whose own `F` is outside `values` still satisfies the relation, and `to` is constrained exactly as without `where`. | Test |
| FR-058-AC-16 | A document that matches `where` and an `exclude` glob is not reported; a document that matches `where` and no glob is. | Test |
| FR-058-AC-17 | A relation naming a document kind that no module declares and no document is still reports under `trace:undeclared-relation-vocabulary` whether or not it carries `where`. | Test |
| FR-058-AC-18 | A relation with `where` reports under its own `trace:<check>` key, advisory by default: a bundle whose only faults are such findings stays valid under both `Okf` and `Strict`, and mapping the key to `error` or `off` through the FR-057 registry promotes or drops exactly those findings. | Test |
| FR-058-AC-19 | Module load rejects a `where` that cannot be executed, naming the relation and `where`, and applies none of the module's declaration: `where` not a mapping, `field` missing, blank or not a string, `values` missing, empty, not a list, or holding a blank or non-string entry, two `values` entries equal after trimming and ASCII case folding, and any key other than `field` and `values`. | Test |
| FR-058-AC-20 | A finding from a relation with `where` names the document, the declaration, the selector `field`, and the document's field value as authored; a finding from a relation without `where` is byte-identical to the pre-change text. | Test |
| FR-058-AC-21 | A relation without `where` selects exactly the documents it selected before this amendment, and over one bundle repeated runs of a relation with `where` yield identical findings in document-then-id order ([NFR-006](../non-functional/NFR-006-determinism.md)). | Test |
| FR-058-AC-22 | A relation with `where` reports under `trace:undeclared-relation-vocabulary`, naming the declaration and the entry, when `field` is neither a property of the `from` archetype's frontmatter schema nor carried by any `from` document, or when a `values` entry is outside that field's schema `enum` and held by no `from` document, where "outside" and "held" both compare after trimming and ASCII case folding as in FR-058-AC-14, so a `values` entry differing from an `enum` member only by case or surrounding whitespace is not reported; a field or entry that is live by either route yields no such finding, and a field whose schema declares no `enum` has its `values` unchecked. | Test |
| FR-058-AC-23 | Given a verb `V` declaring `inverse: W` and a relation listing `V` with `direction: incoming`, an edge authored `(a, V, b)` and an edge authored `(b, W, a)` each satisfy the relation for `b`; a document with neither is reported, and the same relation with `direction: outgoing` is satisfied for `a` by either spelling. | Test |
| FR-058-AC-24 | A relation listing only verbs the inverse index relates to nothing is byte-identical in output to before this amendment. A relation listing a verb that is itself an inverse label (`satisfies`, inverse of `satisfied_by`) gains its forward verb as counterpart; where two forward verbs declare one inverse, the index's first-wins entry is the only counterpart; a label that is a forward key is not read as an inverse. | Test |
| FR-058-AC-25 | A pair authored both ways yields one edge and one `linked` entry carrying the listed verb `V`. A relation listing only `V` and the same relation listing `V` and `W` accept the same edges and report the same documents: a document satisfied by `(b, W, a)` under `incoming` `V` stays satisfied after `W` is added to `edges`, and a pair counts once under either listing. | Test |
| FR-058-AC-26 | For an inverse-authored edge: under an `incoming` relation a wrong-kind normalized source yields no `trace:target-kind-mismatch` and leaves the relation unsatisfied; under an `outgoing` relation a normalized target outside a non-empty `to` yields one finding naming the normalized source, its `repo`, the target as authored and the listed verb; an edge to an absent document satisfies a relation with `to: []` (named in `dangling`) and does not satisfy one with a non-empty `to`. | Test |
| FR-058-AC-27 | Given a trace target named by an `obligations:` source that mints `X-AC-1` from document `X`, an edge whose target is `X-AC-1` satisfies a relation on `X` as an edge to `X`, keeps `X-AC-1` attached, is not listed as `dangling`, and leaves FR-026's classification of the edge `Dangling`. | Test |
| FR-058-AC-28 | A document whose id equals a minted criterion id takes precedence; a criterion id minted by two documents of one member stays dangling and is named as authored; an id minted by a trace target no `obligations:` source names (a test-case target, say) is never read as an edge to its document; with no `obligations:` source every such edge stays dangling. | Test |
| FR-058-AC-29 | In a two-member corpus, `ix://o/b/X-AC-1` is read against the criteria member `o/b` minted and is dangling when `o/b` holds no such criterion, even if member `o/a` mints it; the engine reads no id prefix (a fixture with neutral id spellings behaves identically). | Test |
## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-058-CON-1 | Every field of `TraceabilityModel` SHALL survive both `is_empty` and the cross-module merge. Neither is an exhaustive `match`, so a field added to the struct and to nothing else compiles and is silently dropped — which is exactly what happened while building this FR, twice in one change. | Design | Test |
| FR-058-CON-2 | The engine SHALL contain no archetype name, verb, or direction belonging to any particular traceability chain. A second chain is a manifest entry, never a second check. | Design | Inspection |
| FR-058-CON-3 | The engine SHALL contain no frontmatter field name and no field value for `where`. The manifest declares both, and the engine holds no enumeration of permitted values: any vocabulary it checks `values` against comes from the `from` archetype's frontmatter schema. | Design | Inspection |

## Dependencies

- **Upstream**: [FR-026](./FR-026-intra-spec-reference-resolution.md) (the edge set this reads), [FR-057](./FR-057-corpus-check-severity.md) (the severity knob it ships behind), [FR-050](./FR-050-declarative-coverage-computation.md) (the declaration model it extends)
- **Downstream**: `spec-objects-security#5` bidirectional hazard/threat coverage, which becomes declarations against this FR rather than engine code; [FR-084](./FR-084-two-way-relation-coverage-report.md) (the coverage report over these declarations); [FR-085](./FR-085-cross-repository-edge-resolution.md) (edges between repositories loaded into one corpus); [FR-086](./FR-086-design-element-join.md) (the design-element join over named relations)
