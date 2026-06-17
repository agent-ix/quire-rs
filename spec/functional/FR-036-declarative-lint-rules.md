---
id: FR-036
title: "Declarative Lint Rules"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-013"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-010"
    type: "requires"
    cardinality: "1:1"
---

## Description

Modules MAY declare **advisory lint rules** in `manifest.yaml` under a top-level
`lint_rules:` list. Lint is a posture distinct from structural validation
([FR-032](./FR-032-validate-document.md)): findings NEVER block extraction, validation, or document sync. Lint
surfaces authoring-convention drift — vocabulary discipline that is too soft to
be a structural assert but too important to leave to prose conventions.

Motivating rules (spec-objects format walkthrough, decisions #12/#14,
2026-06-11):

- An FR Acceptance Criteria table's `Verification` column SHALL use the
  ISO 29148 verification methods (`Inspection`, `Analysis`, `Demonstration`,
  `Test`), optionally annotated with test-case references — `Test (TC-035)`.
- A `Configuration` table's `Scope` column SHALL use
  `creation` / `runtime` / `session`.

> **CR-009 note:** The `section_body_pattern` rule type (FR-036-AC-6) was added
> as a second advisory lint so a module can nudge prose conventions that live in
> a section body rather than a table cell — motivated by a `shall`-keyword nudge
> on a requirement `Statement` and an `IT-XXX-SC-NN` presence nudge. It mirrors
> the `table_column_values` philosophy exactly: advisory-only (never blocks),
> `archetypes:`-scoped, and a **missing** section produces no finding (structural
> presence is [FR-032](./FR-032-validate-document.md)'s job). The "v1 ships one rule type" framing is superseded —
> the discriminated `lint_rules` shape was always designed to grow rule types,
> and this is the first such addition.

### Rule shape

`lint_rules:` entries are discriminated by `type:`. Two rule types ship:

```yaml
lint_rules:
- type: table_column_values
  id: ac-verification-method        # stable identifier, reported per finding
  archetypes: [FR]                  # optional scope; empty/absent = all docs
  section: Acceptance Criteria      # heading owning the table
  column: Verification              # column header (case-sensitive)
  allowed: [Inspection, Analysis, Demonstration, Test]
  annotation_pattern: '\(TC-\d+(,\s*TC-\d+)*\)'   # optional trailing annotation
  severity: warning                 # warning (default) | error
- type: section_body_pattern
  id: statement-shall               # stable identifier, reported per finding
  archetypes: [FR]                  # optional scope; empty/absent = all docs
  section: Statement                # heading whose body is checked
  pattern: '\bshall\b'              # regex the body must contain (is_match)
  message: 'requirement statements should use "shall"'  # optional custom message
  severity: warning                 # warning (default) | error
```

For `table_column_values`, a cell is **valid** when it equals an allowed value,
or begins with an allowed value and the remainder (after whitespace) matches
`annotation_pattern` (anchored as a whole-remainder match). Every other data
cell in the named column produces one finding `{rule, severity, message}` naming
the section, column, 1-based row, offending value, and the allowed set.

For `section_body_pattern`, the body of the section under `section` is checked
with `is_match` (a containment match, not anchored). When the section is present
but its body does NOT match `pattern`, ONE finding `{rule, severity, message}`
is produced — `message` is the rule's custom message when set, otherwise a
default naming the section and pattern. A **missing** section produces no
finding (structural presence is validation's job, [FR-032](./FR-032-validate-document.md), not lint's). An
invalid regex is skipped without panicking.

### Loading and evaluation

- The loader ([FR-013](./FR-013-archetype-loader.md)) SHALL parse `lint_rules` into a **typed** structure — the
  key is no longer inert manifest passthrough. A malformed rule fails manifest
  parse like any other shape error.
- `Registry::lint_rules()` exposes the rules aggregated across loaded modules
  in load order.
- `lint_document(rules, archetype, doc) -> Vec<LintFinding>` evaluates the
  rules against a parsed document. `archetype` (resolved by the caller, e.g.
  from frontmatter `artifact_type`) scopes filtered rules; an unresolvable
  archetype runs only unfiltered rules — never an error.
- A missing section, table, or column produces **no** findings: structural
  requirements are validation's job ([FR-032](./FR-032-validate-document.md)), not lint's.

### Non-goals (v1)

- No `quire-rs`-side severity gating — `LintFinding.severity` is reporting
  metadata; CLI/CI consumers decide exit-code policy.
- No per-archetype `lint_rules_ref` indirection (the manifest field some
  modules carry stays inert); rules scope via the `archetypes:` filter.
- No Python binding surface (no current consumer).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-036-AC-1 | A manifest declaring the `ac-verification-method` rule loads with `Registry::lint_rules()` returning the typed rule; a manifest with a malformed rule (unknown `type:`) fails module load with an `ArchetypeLoadFailure` naming the manifest. | Test |
| FR-036-AC-2 | Against an FR document whose AC table holds `Test (TC-035)`, `Inspection`, and `Docs audit` in the Verification column, `lint_document` yields exactly one finding — for `Docs audit` — naming the rule id, row, and allowed set; severity mirrors the rule. | Test |
| FR-036-AC-3 | A rule scoped `archetypes: [FR]` yields no findings against a document linted as `NFR`, and none when the archetype is unresolvable. | Test |
| FR-036-AC-4 | A document missing the rule's section, table, or column yields zero findings. | Test |
| FR-036-AC-5 | Lint evaluation never affects `extract()` / `validate_document()` results — a document with lint findings still extracts and validates exactly as without the rules loaded. | Test |
| FR-036-AC-6 | A `section_body_pattern` rule (CR-009) produces no finding when the named section's body — its own content plus that of every subsection — matches `pattern` (`is_match`), and exactly one finding `{rule, severity, message}` when the section is present but neither it nor its subsections match — the default message names the section and pattern, a custom `message` overrides it, and the finding's severity mirrors the rule. (Matching the full subtree, not just the direct body, lets a token authored inside a subsection — e.g. an `IT-XXX-SC-NN` id under a `### Step` heading — satisfy the rule.) The rule is scoped via `archetypes:` like `table_column_values` (a non-matching or unresolvable archetype runs it only when unfiltered), and a missing section yields no finding. A YAML round-trip preserves the `type: section_body_pattern` discriminator. | Test |

## Dependencies

- **Upstream**: [FR-013](./FR-013-archetype-loader.md) (requires), [FR-010](./FR-010-query-api.md) (requires)
- **Downstream**: none
