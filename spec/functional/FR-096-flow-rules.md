---
id: FR-096
title: "Flow rules and remediations, declared as module data"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-058"
    type: "extends"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-057"
    type: "requires"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-085"
    type: "requires"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-091"
    type: "references"
  - target: "ix://agent-ix/quire-rs/spec/stakeholder/StR-006"
    type: "traces_to"
---
# FR-096: Flow rules and remediations, declared as module data

Tracked by Linear PLAT-1533; amended for PLAT-1769.

## Description

The `traceability` kinds of [FR-058](./FR-058-upward-trace-completeness.md) and
[FR-091](./FR-091-ordered-field-consistency.md) cannot state a rule such as "a connection that
carries a classification holding category X **and** crosses a boundary of trust level Y, **unless**
a control with safeguard Z protects it, is denied". That rule is an AND of conditions, each one or
two edges away from the subject, each reading a **field at the other end** of the edge.
`required_relations` tests an edge, never a field of the document behind it; `where` tests one
scalar field of the `from` document only; `when_any` is an OR over edge existence.

This requirement adds **one** generic engine kind, `traceability.flow_rules`, and one vocabulary,
`traceability.remediations`. Both are module data. After they land, a new rule is a manifest entry
and needs no engine change. `flow_rules` is the only new engine kind: sinks, trust boundaries,
classifications, controls and the rules over them are archetypes and declarations of the modules
that use this engine, and the engine knows none of them.

The engine SHALL evaluate each `flow_rules` declaration of a loaded module over every document of its
`subject` archetypes and SHALL report a denial under `trace:<check>` for each subject where all
`match` conditions hold and no `unless_any` condition holds. The engine SHALL hold no archetype,
verb, field or value of its own.

A rule SHALL NOT deny or pass a subject on a value the engine inferred. Only authored frontmatter
values and resolved edges are read. The engine derives no category, trust level, agreement state or
safeguard, and it never decides that a remediation is adequate: a person accepts that.

### The `flow_rules` declaration

```yaml
traceability:
  flow_rules:
  - name: rule-a
    subject: [alpha]                       # archetypes evaluated
    match:                                 # ALL must hold
    - path: [{edge: carries, direction: outgoing, to: [beta]}]
      field: categories
      any_of: [x]
    - path: [{edge: crosses, direction: outgoing, to: [gamma]}]
      field: level
      any_of: [low, unknown]
    unless_any:                            # ANY holding clears the denial
    - path: [{edge: protects, direction: incoming, to: [delta]}]
      field: safeguards
      any_of: [fix-one]
    check: x-across-low-level              # the <check> of trace:<check>
    remediations: [fix-one, fix-two]
    rationale: "Why the rule exists, in one sentence."
```

(`alpha`, `beta`, `x` and the like stand for any archetype, field and value the module declares.)

| Field | Required | Meaning |
|---|---|---|
| `name` | yes | The rule's own name, unique across the loaded corpus members' modules, reported in the denial |
| `subject` | yes | Non-empty list of archetypes; each document of these archetypes is evaluated |
| `match` | yes | Non-empty list of conditions; **all** must hold |
| `unless_any` | no | List of conditions; **any one** holding clears the denial |
| `check` | yes | The `<check>` half of the `trace:<check>` severity key ([FR-057](./FR-057-corpus-check-severity.md)); same form as FR-058-AC-10. Several flow rules may share one `check`, and then share one severity key; no flow rule may use a `check` that another kind of declaration or the engine already reports under `trace:` |
| `remediations` | no | Names declared in `traceability.remediations` that a person may apply to clear the denial |
| `rationale` | yes | One sentence naming why the rule exists; copied into every denial |

### Condition and path

A **condition** holds a path, a field, and one test:

| Key | Meaning |
|---|---|
| `path` | A list of 0, 1 or 2 steps, naming the documents the field is read from |
| `any_path` | In place of `path`: a non-empty list of paths. The condition reads the union of the documents at the end of the paths |
| `field` | A top-level frontmatter key of the documents at the end of the path. The `id` of a document is read as the frontmatter key `id`, with no special case |
| `any_of` | A non-empty list of values |
| `absent` | `true`; in place of `any_of` |
| `present` | `true`; in place of `any_of` |

Exactly one of `path` and `any_path` is given, and exactly one of `any_of`, `absent` and `present`.

A **step** is `{edge, direction, to}`: `edge` is a verb, `direction` is `outgoing` (the current
document is the source) or `incoming` (it is the target), and `to` is a non-empty list of
archetypes. A step follows the resolved edges of the verb from the current document to documents of
one of the `to` archetypes. Edge resolution is that of FR-058, including declared inverses
(FR-058-AC-23) and edges across corpus members ([FR-085](./FR-085-cross-repository-edge-resolution.md));
an edge whose target is absent, or whose other end is outside `to`, contributes nothing. A second
step starts from the documents the first step reached, and the end of the path is the set of
documents the last step reached, each once. A path of zero steps ends at the subject itself, so a
condition on a subject's own field is `path: []`.

A path of more than two steps is refused at load. Two steps cover the longest chain the rules need:
a sink reaches its data through `{edge: delivers_to, direction: incoming, to: [connection-like]}`
then `{edge: carries, direction: outgoing, to: [classification-like]}`. When a document can also
reach data directly, the condition gives `any_path: [<two-step path>, <one-step path>]`, and holds
when the end of either path satisfies the field test.

A condition **holds** for a subject as follows. The comparison is trim plus ASCII case folding, the
same comparison as FR-058-AC-14 (`fold_selector_value`).

- `any_of`: some document at the end of the path has the top-level frontmatter `field` with a
  scalar value equal to an `any_of` entry, or a list value with a member that is. A field whose
  value is absent, null, a mapping, a number or a boolean never equals an entry; a list member that
  is not a string is skipped.
- `absent: true`: **no** document at the end of the path has the field. Where the path ends at no
  document, the condition holds. It states, for example, "no agreement stated".
- `present: true`: some document at the end of the path has the field. With `field: id` it means
  "at least one such edge exists".

### Outcome

A rule **denies** a subject when every `match` condition holds and no `unless_any` condition holds.
A rule with no `unless_any` denies on the `match` alone.

A denial is a finding under `trace:<check>` that names:

1. the rule's `name`;
2. the subject: `id` and, when its corpus member declares an identity, `repo`;
3. the **matched facts**: the documents at the end of the paths of the `match` conditions that
   held through `any_of` or `present` **and satisfy that condition's field test** (for `any_of`,
   those whose field has a value equal to an entry; for `present`, those that have the field),
   grouped by archetype, each by id (and `repo`), each once. A document at the end of a path that
   fails the field test is not named, so a connection carrying two classifications of which one holds
   the value names only that one. An `absent` condition names no document;
4. the `remediations`, in declared order, each with its declared `name`, `description`, and one
   `change` object containing exactly one of `adds`, `sets` or `removes`;
5. the `rationale`.

**Default severity is `warning` in both postures**, as [FR-057](./FR-057-corpus-check-severity.md)
requires of a corpus check introduced after it; this requirement amends neither FR-057 nor FR-058. A
denial is advisory until a module or a project promotes it. A module promotes a check by declaring
`grammar_severity: {"trace:<check>": error}` in its manifest, and a project lowers, raises or silences
one with the same registry (`--severity trace:<check>=error|warning|off`); a CLI-shaped entry wins
over a module-declared one (FR-057-AC-5), and `off` drops the denial from `errors`, `warnings` and the
flow report. A module whose rules guard a regulated model is expected to promote its checks this way;
the engine promotes none. Rules with no declaration anywhere leave output byte-identical to a build
without this requirement.

**Determinism.** Denials are ordered by `check`, then subject (`repo` then `id`, with an absent
`repo` first), then rule `name` ([NFR-006](../non-functional/NFR-006-determinism.md)). Matched
documents and members do not depend on authoring order or directory walk. Remediations retain the
rule's declared order so a caller can present the module's recommendations in their intended order.

### Load-time validation

A `flow_rules` or `remediations` entry that cannot be evaluated does not load, and the load failure
names the rule (or remediation) and the offending entry. Only that entry is dropped: every other rule
and remediation, every `required_relations` entry and every other declaration of the module and of
the other selected modules loads ([FR-058](./FR-058-upward-trace-completeness.md), "A declaration that
cannot load drops itself"), and validation reports `trace:declared-model-dropped` for it. The load
fails on:

- a `subject` that is empty, or a `subject` or step `to` naming an archetype no loaded module declares;
- an `edge` that is not a declared verb ([FR-058](./FR-058-upward-trace-completeness.md)), or a
  `direction` other than `outgoing` or `incoming`;
- a `field` that no archetype at the end of the path declares as a property of its frontmatter schema
  (`id` is always admitted), so a misspelt field refuses the module instead of silently disabling the
  rule. With several end archetypes it is refused only when none of them declares it;
- a `check` equal to a `trace:` check the engine reports itself or that another declaration of the
  loaded modules names (FR-058-AC-10), so one severity key never governs two kinds of finding;
- an `any_of` value outside the `enum` of that `field` in the frontmatter schema of the archetypes
  at the end of the path (the same lookup as FR-058-AC-22, taking the `enum` of the field or of its
  array items). Enum lookup follows local JSON Schema `$ref` values from the field property or its
  array items through schemas in the loaded module; remote references are not fetched. A missing,
  cyclic, or otherwise unresolved reference fails closed with a diagnostic located at the schema
  reference and naming the flow rule and field. The value is compared after trim and ASCII fold.
  With several end archetypes it is refused only when no end archetype that declares an `enum` for
  the field admits it; a field whose schema declares no `enum` leaves its values unchecked;
- a `remediations` entry that is not declared, that cannot clear the denial, or whose `field` or
  `value` its archetype's schema does not admit (below);
- a duplicate rule name, or a duplicate remediation name;
- an empty `match`, an empty or blank `check` or `rationale`, a `check` that cannot form a
  `trace:<check>` key, or a blank entry anywhere in a list;
- a path of more than two steps, a condition with both `path` and `any_path` or with neither, or with
  none or more than one of `any_of`, `absent` and `present`, or `absent` or `present` set to anything
  but `true`;
- any key outside those listed in this requirement.

### Remediations

`traceability.remediations` is the vocabulary a rule may offer. Each entry:

```yaml
remediations:
- name: fix-one
  description: "Protect the connection with a control whose safeguards include fix-one."
  adds: {edge: protects, direction: incoming, from: delta, field: safeguards, value: fix-one}
```

| Field | Meaning |
|---|---|
| `name` | Unique token; the vocabulary of the rule's `remediations` |
| `description` | One sentence stating the model change a person makes |
| exactly one of `adds`, `sets`, `removes` | The model change |

- `adds: {edge, direction, from, field, value}`: a document of archetype `from` with `field` equal
  to `value` is linked to the subject by `edge` in `direction` (as seen from the subject).
- `sets: {field, value}`: the subject's own frontmatter `field` is set to `value`.
- `removes: {edge, direction}`: **every** edge of the verb `edge` between the subject and a document
  of the first step, in `direction` as seen from the subject, is deleted. Which document an author
  edits depends on the direction. For `outgoing` the subject is the source, so the author edits the
  **subject** and deletes its own `edge` entries. For `incoming` the edge is authored by the
  first-step endpoint and points to the subject, so the author edits each **first-step document**,
  deleting its `edge` entries that target the subject; the subject itself is not edited. The
  remediation `description` states the edge and which document is edited. An edge starting at a
  second step is neither the subject's nor an endpoint's edge to the subject and is never removed.

A rule may name only a remediation whose model change would clear **that rule's** denial; naming a
fix that cannot is refused at load. A remediation **matches** a rule when:

- `adds`: some `unless_any` condition has a `path` of one step (or an `any_path` entry of one step)
  with the same `edge` and `direction`, `from` among the step's `to`, the same `field`, and `value`
  among its `any_of` (compared after trim and ASCII fold);
- `sets`: some `unless_any` condition has an empty `path` (or an `any_path` entry of zero steps), the
  same `field`, and `value` among its `any_of`;
- `removes`: some `match` condition tested with `any_of` or `present` has **every one of its paths
  beginning** with a first step of that `edge` and `direction`, so removing the subject's edges of that
  verb (at either endpoint, per the direction) leaves every path ending at no document and the condition fails. A path of zero steps, a path
  beginning with another step, or an `any_path` holding any such path does not qualify, because the
  condition would still hold through that path after the removal.

A remediation entry is itself refused at load when it holds none or more than one of
`adds`/`sets`/`removes`, names an undeclared verb or archetype, names a `field` that the schema of its
archetype does not declare, or gives a `value` outside the `enum` of its `field` by the lookup above.
The archetype is `from` for `adds`. For `sets` it is each `subject` archetype of every rule that names
the remediation, and the entry is refused when any of them fails.

The engine applies no remediation and writes nothing: a remediation is a name, a sentence and a
stated change, for a person to carry out and for a module to offer.

### The flow report (`quire.flow-report/v1`)

The engine SHALL build a serializable flow report over a loaded corpus and its resolved edges, with
the denials above and the facts the rules read. The `quire-cli` `relations --flow` subcommand
renders it; the engine writes no QSL, and the lowering of the report into a solver model belongs to
`quire-spec-language`. The JSON form has the keys in this order:

```json
{"schema": "quire.flow-report/v1", "rules": [...], "vocabularies": {...}, "facts": {...}, "denials": [...], "remediations": [...]}
```

- **`rules`**: the declared rules in declaration order across members, each as authored, normalised
  to the full form: `name`, `subject`, `match`, `unless_any` (always present, possibly empty),
  `check`, `remediations`, `rationale`. A condition keeps `path` or `any_path` as declared and
  exactly its one test key.
- **`vocabularies`**: an object keyed by archetype, then by field, holding the `enum` members (in
  schema order) of every field some condition reads at the end of a path, where the archetype's
  schema declares an `enum`. A field with no `enum` is absent. It is the closed set a consumer
  lowers to an enumeration.
- **`facts`**: an object keyed by archetype name, in sorted order, holding one array per archetype
  named in any rule's `subject` or step `to`. The keys are exactly the archetype names the loaded
  rules declare; the engine fixes no key, and a corpus whose rules name other archetypes has other
  keys. Each entry is a document:
  `{"id", "repo", "path", "fields", "edges"}`, with `repo` omitted when the member has no identity.
  `fields` holds only the top-level frontmatter keys some condition reads at that archetype, with
  each value as authored, keys sorted; a key the document does not state is omitted, never
  defaulted. `edges` holds each resolved edge a rule step follows from this document, as
  `{"edge", "direction", "target", "archetype"}` (target by id, and `repo` when present), sorted by
  edge, direction, then target. Entries are ordered by `repo` then `id`.
- **`denials`**: one object per denial in the order above:
  `{"rule", "check", "subject", "matched", "remediations", "rationale", "severity"}`. `subject` is
  `{"id", "repo", "archetype"}`; `matched` is an object keyed by archetype, each an array of
  `{"id", "repo"}` sorted by repo then id; `remediations` is an array in the rule's declared order,
  each `{"name", "description", "change"}` where `change` contains exactly one of `adds`, `sets` or
  `removes` using the declared remediation action shape; `severity` is the resolved FR-057 severity
  of the key. The validation finding includes the same detailed remediation objects.
- **`remediations`**: the declared remediations, in declaration order across members, each
  `{"name", "description", "adds" | "sets" | "removes"}` with exactly its one change key. It is the sixth
  and last top-level key (quire-cli FR-044 renders the document key for key).

No timestamp, run-specific value or path outside the corpus appears. Arrays are always present, empty
when empty. A corpus declaring no `flow_rules` serializes the report with empty `rules`,
`vocabularies`, `facts`, `denials` and `remediations`.

The report lists every denial the engine produced, of any severity including `warning`; a denial dropped
by `off` is not in `denials`. Whether a caller exits non-zero on a denial, in strict mode or not, is the
caller's rule and belongs to `quire-cli`; this requirement defines no exit code.

### Lowering contract for the solver (informative)

The report is shaped so that a lowering can use only constructs quire-analyze already specifies:
`vocabularies` lower to enum sorts (quire-analyze FR-020), each `facts` entry to a record of its
`fields` (FR-020 datatypes), and "for every subject" to a quantifier over a finite collection
(FR-031, VER-90). No arithmetic is needed. The lowering is delivered by PLAT-1541, which **waits for
QSL-1900**, the production step of the contract package (`quire.contract-package/v1`) in
`quire-spec-language`. This requirement imposes nothing on QSL and the engine does not depend on it.

### Out of scope

The shape of any particular object type (a sink, a trust boundary, a classification, a control) and
any starter rule set belong to the modules that ship them. Records of processing, threat-model
completeness rules beyond `present`/`absent`, runtime monitors and authorization-policy analysis are
other work. The CLI flag, its human rendering and exit codes belong to `quire-cli`.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-096-AC-1 | Seeded model: connection `CONN-1` carries a classification whose `regulated_categories` list holds `health` and crosses a boundary whose `trust_level` is `untrusted`, and no control protects it. The module's rule, which reads `regulated_categories` and `trust_level` as declared module data (the engine holds neither name), yields exactly one denial under `trace:health-across-untrusted-boundary` naming `CONN-1`, the classification id, the boundary id and the remediation `encrypt-in-transit`. | Test |
| FR-096-AC-2 | The AC-1 model plus a control with `safeguards: [encrypt-in-transit]` that `protects` `CONN-1` yields no denial. | Test |
| FR-096-AC-3 | A sink with `kind: llm_prompt` and `processor_agreement: not_covered`, fed by a connection through `delivers_to` that carries `health`, is denied under the sink rule that matches `kind` and a `processor_agreement` outside `covered`/`not_applicable`; the same sink with `processor_agreement: covered` is not denied, and with `unknown` it is denied. | Test |
| FR-096-AC-4 | A sink whose data arrives by both the two-step path (`delivers_to` incoming, then `carries` outgoing) and its own `carries` edge is read through `any_path`: the condition holds when either path ends at a document holding the value, and the denial names the documents from both paths once each. | Test |
| FR-096-AC-5 | A rule whose `any_of` names a value outside the `enum` of that field in the end archetype's schema (for example `medical` where the schema admits `health`, `personal`, `financial`) fails module load with an error naming the rule and the value; the same rule with an in-`enum` value loads, and a value differing from a member only by case or surrounding whitespace loads. | Test |
| FR-096-AC-6 | Each of these fails module load naming the rule: an unknown archetype in `subject`; an unknown archetype in a step `to`; an unknown edge verb; a `remediations` entry that is not declared; a duplicate rule name; an empty `match`; a path of three steps; a condition holding both `path` and `any_path`, or neither; a condition with two of `any_of`/`absent`/`present`; a blank `check`. | Test |
| FR-096-AC-7 | A remediation whose `adds` matches none of the rule's `unless_any` conditions (wrong `edge`, `direction`, `from`, `field` or `value`) fails load when the rule names it, naming the rule and the remediation; the same remediation unnamed by any rule loads. A `sets` that matches no zero-step condition fails likewise, and so does a `removes` whose edge and direction do not begin every path of one `match` condition: an edge only at a second step, or an `any_path` holding one path that does not begin with it, is refused. An `incoming` `removes` is valid on the same terms. A `removes` whose edge and direction begin every path of an `any_of` or `present` condition (for example an `any_path` of two paths both beginning with the removed step) loads. | Test |
| FR-096-AC-8 | A corpus with no document carrying the facts a starter-style rule reads (no connection-like document with a classification-like edge) yields no finding from any `flow_rules` declaration. | Test |
| FR-096-AC-9 | `absent: true` holds when no document at the end of the path has the field and when the path ends at no document; `present: true` with `field: id` holds exactly when at least one edge of the step exists; `absent` and `present` are exact complements for one path and field. | Test |
| FR-096-AC-10 | Comparison is trim plus ASCII case folding: a document stating ` Health ` matches `any_of: [health]`; a list value holding a matching member matches; a value differing by more than case and whitespace does not; a number, mapping, boolean, null or missing field never equals an entry, and none of them raises an error or warning. | Test |
| FR-096-AC-11 | A denial names the rule, the subject id (and `repo`), the matched documents grouped by archetype, the `remediations` in declared order with each remediation's name, description and exact declared change under `change`, and the `rationale`; the validation finding includes those same details. An `absent` condition names no document. A connection carrying two classifications of which only one holds the tested category names only that classification, in the denial and in the report's `matched`. | Test |
| FR-096-AC-12 | With no registry entry the severity of a denial is `warning` in both postures, so a bundle whose only fault is a denial stays valid. A module declaring `grammar_severity` `trace:<check>` as `error` makes the same denial an error and the bundle invalid; `--severity trace:<check>=warning` demotes it again and wins over the module entry; `off` drops exactly the denials of that check from `errors`, `warnings` and the report. | Test |
| FR-096-AC-13 | Denials are ordered by `check`, subject, then rule name, identical across two runs and for documents or members supplied in reverse order. | Test |
| FR-096-AC-14 | A fixture whose archetypes, verbs, fields and values are neutral (for example `alpha`, `beta`, `links_to`, `rank_field`) and share no name with this specification yields the same denials, so the engine holds no name. | Test |
| FR-096-AC-15 | The flow report for a corpus serializes with keys in the order `schema`, `rules`, `vocabularies`, `facts`, `denials`, `remediations`; `schema` equals `quire.flow-report/v1`; two runs over one corpus are byte-identical. | Test |
| FR-096-AC-16 | `facts` is keyed by exactly the archetype names the loaded rules declare, with no fixed or domain-specific key, and holds, for each such archetype, one entry per document with only the fields a condition reads at that archetype and the edges a step follows from it; a field the document does not state is omitted, an unread field appears nowhere, and entries are ordered by repo then id. | Test |
| FR-096-AC-17 | `vocabularies` holds the schema `enum` members, in schema order, for each archetype and field a condition reads, including enums reached through local `$ref` values on the field property or its array items; it omits a field whose schema declares no `enum`, and does not fetch remote references. | Test |
| FR-096-AC-18 | A corpus declaring no `flow_rules` yields empty `rules`, `vocabularies`, `facts`, `denials` and `remediations`, and the FR-058 and FR-084 relation output over it is byte-identical to a build without this requirement. | Test |
| FR-096-AC-19 | A rule's `denials` entry equals the finding the validation path reports for the same subject: the same `check`, subject, matched documents and severity. | Test |
| FR-096-AC-20 | The report's `denials` lists every denial of every severity: a denial resolved to `warning` is present with `severity` `warning`, a denial resolved to `off` is absent, and a corpus with no denial has an empty `denials`. The report is the same whatever exit code a caller derives from it. | Test |
| FR-096-AC-21 | Reading a rule never infers a value: with a `connection` carrying no `carries` edge the engine adds no category, and the same corpus with the field a condition reads removed from the only classification holding the tested value yields no denial from that rule and no such key in the `facts` entry of that classification. | Test |
| FR-096-AC-22 | The engine source names no archetype, verb, field or value of a data-flow, privacy or security model. | Inspection |
| FR-096-AC-23 | `flow_rules` is the only engine kind this requirement adds; `remediations` is a vocabulary read by it, and no check, relation or report outside this FR and FR-084's flow section reads either. | Inspection |
| FR-096-AC-24 | A condition whose `field` is declared by no end archetype's frontmatter schema (for example `regulated_categores` for `regulated_categories`) fails module load naming the rule and the field; the correct spelling loads, and `field: id` loads. A remediation whose `field` is not declared by its archetype's schema fails load likewise. | Test |
| FR-096-AC-25 | A rule whose `check` equals a `trace:` check the engine reports (for example `dangling-trace-reference`) or one named by a `required_relations` entry fails module load naming the rule and the check; two flow rules sharing one new `check` load, and one `--severity` entry for that key governs both. | Test |
| FR-096-AC-26 | For a condition field whose schema property refers locally to a schema with an `enum`, every declared enum value is accepted in `any_of` and an out-of-enum value is refused at module load naming the rule, field, and value; the same behavior holds when the local `$ref` is on the field's array `items`. | Test |
| FR-096-AC-27 | A missing, cyclic, or remote `$ref` encountered while resolving a flow-rule enum is not fetched or treated as an unconstrained field: module load fails closed with a diagnostic identifying the schema reference location and the flow rule and field. | Test |
| FR-096-AC-28 | A field with an inline enum, including an inline array-item enum, continues to accept every declared `any_of` value and reject an out-of-enum value at module load; its `vocabularies` members and order remain unchanged. | Test |
| FR-096-AC-29 | A module with an unexecutable flow rule (a `subject` naming an undeclared archetype) selected beside a module declaring `required_relations` loads the relations, records one failure naming the rule, and still evaluates the module's other rules; `validate_bundle` reports `trace:declared-model-dropped` naming the rule (FR-058-AC-38, FR-058-AC-41). | Test |

## Constraints

| ID | Constraint | Type | Validation |
|----|-----------|------|------------|
| FR-096-CON-1 | Every field of `TraceabilityModel` added here SHALL survive both `is_empty` and the cross-module merge (FR-058-CON-1). | Design | Test |
| FR-096-CON-2 | The engine SHALL hold no archetype, verb, direction, field name or field value for `flow_rules` or `remediations`; the manifest declares all of them and the vocabulary a value is checked against comes from the archetype's frontmatter schema. | Design | Inspection |
| FR-096-CON-3 | A rule SHALL read only authored frontmatter values and resolved edges. No code path derives a category, trust level, agreement state, safeguard or remediation adequacy. | Design | Inspection |
| FR-096-CON-4 | The engine SHALL NOT apply a remediation, write a document, or compile QSL. | Design | Inspection |
| FR-096-CON-5 | Evaluation SHALL be a pure function of the loaded corpus and its modules, with no shared mutable state ([NFR-006](../non-functional/NFR-006-determinism.md)). | Design | Inspection |

> **CR-001 note (2026-11-16):** PLAT-1479 extends denial remediation output to include the declared description and exact change object, both in the flow report and the validation finding. The report's top-level remediation vocabulary remains unchanged; a denial includes only the remediation names declared by its rule and preserves that rule's order.

> **CR-421 note (2026-11-17, PLAT-1769):** Flow-rule enum lookup follows local JSON Schema references from a field property or its array items so that referenced closed vocabularies constrain `any_of` values and populate the flow report. Missing, cyclic, and remote references fail closed with a located diagnostic; enum lookup does not fetch remote schemas. Inline enum behavior is unchanged.

## Dependencies

- **Upstream**: [FR-058](./FR-058-upward-trace-completeness.md) (edge resolution, `trace:<check>`, the load-time pattern and the enum lookup of FR-058-AC-22), [FR-057](./FR-057-corpus-check-severity.md) (severity), [FR-085](./FR-085-cross-repository-edge-resolution.md) (edges across members)
- **Related**: [FR-091](./FR-091-ordered-field-consistency.md) (the field-rule declaration pattern this declaration follows; nothing is read from it)
- **Downstream**: [FR-084](./FR-084-two-way-relation-coverage-report.md) (the relation report carries this report as its `flow` section); `quire-cli` FR-044 (`relations --flow`); the modules that declare sinks, controls, rules and remediations (PLAT-1535, PLAT-1537, PLAT-1540, PLAT-1479); PLAT-1541 (the solver lowering, which waits for QSL-1900, the production contract-package step of `quire-spec-language`)
