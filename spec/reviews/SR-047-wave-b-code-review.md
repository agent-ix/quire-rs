---
id: SR-047
title: "code-review of ADR-0011 Phase 2 Wave B (FR-058, CR-073..CR-075)"
type: SpecReview
analysis: code-review
scope: "src/corpus/required_relations.rs, src/traceability.rs, src/loader/mod.rs, tests/required_relations.rs, tests/fixtures/traceability/required-relations/, .gitignore"
review_set: subset
---

## Summary

Reviewed the Wave B change on `feat/fr058-required-relations` against `main` — FR-058 upward-trace
completeness (CR-073), the load-time rejection of unexecutable declarations (CR-074), and the
repository hygiene the branch carried. Five findings, two of them high. All five are fixed in this
branch; the two high findings are both instances of the same failure class the FR itself is about —
**a check that silently does nothing looks exactly like a bundle with nothing wrong**.

The headline finding was confirmed by measurement, not by reading: changing the fixture's
`from: FR` to `from: FRR` leaves a genuine orphan requirement unreported and the entire run clean.

## Verdict

**FAIL** — two `high` findings (FND-001, FND-002). Both are **fixed in this branch**; the verdict
records the state of the code as reviewed, per the same convention SR-045 used.

## Findings

| ID      | Severity | Summary                                                                                     | Refs                                            |
| ------- | -------- | ------------------------------------------------------------------------------------------- | ----------------------------------------------- |
| FND-001 | high     | A typo in a relation's `from` silently disables the check — the orphan it should find vanishes and nothing reports it | src/corpus/required_relations.rs:163             |
| FND-002 | high     | 178 files (~90k lines) of `cargo-mutants` run output committed; `.gitignore` added after the fact does not untrack   | mutants.out/, mutants.out.old/                   |
| FND-003 | medium   | `.gitignore` ignored `*.proptest-regressions`, which would silently discard every future discovered failure seed     | .gitignore:17                                    |
| FND-004 | medium   | The fixture accepted `US` in both relations' `to:` lists but never declared the archetype, so the path was untested  | tests/fixtures/traceability/required-relations/manifest.yaml:45 |
| FND-005 | low      | `check_relation`'s `has_incoming` hardcodes `Resolution::Resolved` with no comment saying why it is safe             | src/corpus/required_relations.rs:180             |

## Detail

### FND-001 — the silent twin

`check_relation` selects documents with `spec.by_type(&relation.from)`, which matches the
frontmatter `type` field. A `from` naming a kind nothing has returns **zero** documents, so the
relation checks nothing.

Measured, not inferred. With the fixture's `from: FR` changed to `from: FRR`:

```
errors=0 warnings=1
  [unsatisfied-str] 'StR-001' has no 'satisfies/satisfied_by' edge ...
```

`FR-001` is a real orphan — no upstream need at all — and it is **not in the output**. The module
loads without complaint.

This is the exact mirror of CR-074's `edges: []`, which fails *loudly* by reporting every document.
Loud is survivable: someone investigates. Silent is not.

It cannot be caught at load. `TraceabilityModel::validate` runs per module at manifest-parse time,
before the merge, and a relation legitimately names kinds another module contributes — which is the
entire point of `spec-objects-security#5`, where `from: hazard` (safety module) points
`to: [FR]` (iso module).

**Fixed as CR-075 / FR-058-AC-11**, at the first point where the merged registry and the walked
bundle are both available.

### FND-002 / FND-003 — generated artifacts

`mutants.out/` and `mutants.out.old/` were committed: 178 files, ~90k lines of per-mutant diffs and
full build logs, describing one machine at one commit. `.gitignore` was updated to exclude them, but
only *after* they were added, and gitignore does not untrack what git already tracks.

The same `.gitignore` hunk also ignored `proptest-regressions/` and `*.proptest-regressions` — which
is backwards. proptest writes those files when a property fails and re-runs those exact cases before
generating novel ones; the file **is** the mechanism by which a discovered counterexample stays
discovered, and its own header says to check it in. `tests/props_metamorphic.proptest-regressions`
survived only because it was committed before the rule was added; the next one would have been lost.

### FND-004 — an untested path the fixture documented

Both relations declared `to: [StR, US]`, and a long fixture comment explains why `US` matters: the
29148 chain is stakeholder → system → software, so an FR may hang off a use case, and declaring only
`StR` produced a 16% false-positive rate on quire-rs's own corpus. But the fixture never declared the
`US` archetype and no test used one, so the behaviour the comment describes was never exercised.
Surfaced by FND-001's fix reporting `to 'US'` as a dead kind. TC-909 covers it now.

### A rule that was narrowed, and why

The first version of the FND-001 fix also checked `edges` against the module's `edge_types`. It
immediately fired on the fixture's own **working** declaration, flagging `satisfies` and
`satisfied_by` — verbs that demonstrably resolve and find orphans correctly — because a
single-module fixture declares no vocabulary of its own.

That is a **bad rule**, not a bad fixture, and the narrowing rests on a reason independent of the
count: FR-041-AC-2 already permits verbs absent from `edge_types`, and a misspelt verb fails loudly
anyway. Verbs are now excluded by design and the FR says so.

## Coverage

`make ci` passes: `fmt-check`, `clippy -D warnings`, 524 unit + 12 `required_relations` integration
tests, `cargo deny`, and all seven static audits. The mutation-scope and Python-binding lanes were
not run — no `src/grammar/`, `src/python/` or `tests/python/` file changed.

The `rust-review` sub-skill referenced by the code-review skill's language-dispatch table
(`skills/rust-review/SKILL.md`) **does not exist in the installed skill**; the review used the
repository's own documented idioms from `CLAUDE.md` instead. Recorded so the gap is visible rather
than silently skipped.
