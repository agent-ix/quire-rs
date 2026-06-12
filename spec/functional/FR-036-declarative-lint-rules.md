---
id: FR-036
title: "Declarative Lint Rules"
artifact_type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-013"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-010"
    type: "requires"
    cardinality: "1:1"
---

## Behavior

Modules MAY declare **advisory lint rules** in `manifest.yaml` under a top-level
`lint_rules:` list. Lint is a posture distinct from structural validation
(FR-032): findings NEVER block extraction, validation, or document sync. Lint
surfaces authoring-convention drift — vocabulary discipline that is too soft to
be a structural assert but too important to leave to prose conventions.

Motivating rules (spec-objects format walkthrough, decisions #12/#14,
2026-06-11):

- An FR Acceptance Criteria table's `Verification` column SHALL use the
  ISO 29148 verification methods (`Inspection`, `Analysis`, `Demonstration`,
  `Test`), optionally annotated with test-case references — `Test (TC-035)`.
- A `Configuration` table's `Scope` column SHALL use
  `creation` / `runtime` / `session`.

### Rule shape

`lint_rules:` entries are discriminated by `type:`. v1 ships one rule type:

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
```

A cell is **valid** when it equals an allowed value, or begins with an allowed
value and the remainder (after whitespace) matches `annotation_pattern`
(anchored as a whole-remainder match). Every other data cell in the named
column produces one finding `{rule, severity, message}` naming the section,
column, 1-based row, offending value, and the allowed set.

### Loading and evaluation

- The loader (FR-013) SHALL parse `lint_rules` into a **typed** structure — the
  key is no longer inert manifest passthrough. A malformed rule fails manifest
  parse like any other shape error.
- `Registry::lint_rules()` exposes the rules aggregated across loaded modules
  in load order.
- `lint_document(rules, archetype, doc) -> Vec<LintFinding>` evaluates the
  rules against a parsed document. `archetype` (resolved by the caller, e.g.
  from frontmatter `artifact_type`) scopes filtered rules; an unresolvable
  archetype runs only unfiltered rules — never an error.
- A missing section, table, or column produces **no** findings: structural
  requirements are validation's job (FR-032), not lint's.

### Non-goals (v1)

- No `quire-rs`-side severity gating — `LintFinding.severity` is reporting
  metadata; CLI/CI consumers decide exit-code policy.
- No per-archetype `lint_rules_ref` indirection (the manifest field some
  modules carry stays inert); rules scope via the `archetypes:` filter.
- No Python binding surface (no current consumer).

## Acceptance

- **FR-036-AC-1**: A manifest declaring the `ac-verification-method` rule loads
  with `Registry::lint_rules()` returning the typed rule; a manifest with a
  malformed rule (unknown `type:`) fails module load with an
  `ArchetypeLoadFailure` naming the manifest.
- **FR-036-AC-2**: Against an FR document whose AC table holds `Test (TC-035)`,
  `Inspection`, and `Docs audit` in the Verification column, `lint_document`
  yields exactly one finding — for `Docs audit` — naming the rule id, row, and
  allowed set; severity mirrors the rule.
- **FR-036-AC-3**: A rule scoped `archetypes: [FR]` yields no findings against
  a document linted as `NFR`, and none when the archetype is unresolvable.
- **FR-036-AC-4**: A document missing the rule's section, table, or column
  yields zero findings.
- **FR-036-AC-5**: Lint evaluation never affects `extract()` /
  `validate_document()` results — a document with lint findings still extracts
  and validates exactly as without the rules loaded.
