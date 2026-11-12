---
id: SR-20261112155212
title: "failure-domain review of FR-058 where field-value selector (quire-rs PR 652)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/quire-rs@b04cbdb; spec/functional/FR-058-upward-trace-completeness.md; spec/tests.md"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-rs/FR-058
    type: references
---

# Failure-domain review of FR-058 where selector (quire-rs PR 652)

## Summary

Ticket: PLAT-1475. This review looked for silent failure modes, identity confusion and unstated edge cases in the `where` selector.

Covered and clean:

- Document-side values. A field that is absent, null, empty or not a string, or whose string is outside `values`, selects nothing and yields no finding (AC-12, AC-13). The field's own schema catches a misspelt value in a document. This matches FR-049, which leaves misspelt Kind cells to the module grammar.
- Excluded documents. A document that `where` selects and `exclude` exempts is not reported (AC-16).
- Incoming direction. `where` applies to the obliged side only (AC-15).
- YAML coercion. A `values` entry that YAML reads as a bool or a number is refused at load as a non-string entry (AC-18).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | A misspelt `where` declaration disables the relation with no output. If `values` contains `saftey`, or `field` is `quality_atribute`, every document passes the schema and is simply not selected, so the run comes back clean. That is the "one character disables the check" failure that FR-058 itself treats as must-report for `from` (AC-11 and the section "A declaration that names a kind nothing has"). The text says "the field's own schema is where an unknown enumeration member is caught", but that schema validates documents, not the manifest's `values` list, so nothing catches the typo. The precedents call for a diagnostic. FR-049 reports a misspelt declaration (`kind-column-matches-nothing`, FR-049-AC-18) and leaves only document-side values to the grammar. FR-059-AC-2/AC-7 already read a field's `enum` from the projected archetype's frontmatter schema with no engine knowledge, and report a declaration whose field yields no enum (`trace:undeclared-coverage-vocabulary`). The claim that the engine "cannot call an unlisted value wrong" is therefore inaccurate, because FR-059 does that generically. Fix: add a declaration-side check in the AC-11 style. A `where.field` that the `from` archetype's frontmatter schema does not declare and that no `from` document carries, or a `values` entry that is outside that field's schema `enum` (when one exists) and that no document holds, reports itself under a named `trace:` key, naming the relation and the entry. Add an AC and a TC. Keep CON-3 (the vocabulary comes from the schema, not the engine). | spec/functional/FR-058-upward-trace-completeness.md:93-97 |
| FND-002 | low | Case folding is not specified. AC-14 and the prose say "folding case". FR-049, which the prose cites as the same comparison, says "ASCII case folding". Unicode and ASCII folding differ (for example `ß`, dotted `İ`), so two implementers could disagree on a non-ASCII value. Fix: say "ASCII case folding" in the prose and in AC-14. | spec/functional/FR-058-upward-trace-completeness.md:91, AC-14 |
| FND-003 | low | AC-18 does not say whether a whitespace-only `values` entry (`"  "`) counts as "empty", or what happens to two entries that are equal after trim and fold (`[security, Security]`). For `kind_targets` keys, FR-049 rejects both "empty after trimming" and a key that "duplicates another key under the matching rule". Fix: state the same here, an entry empty after trimming or duplicated under the matching rule is rejected at load, and add both to TC-2253. | spec/functional/FR-058-upward-trace-completeness.md AC-18 |

## Verdict

Changes requested: FND-001 (medium) should be fixed in this PR. FND-002 and FND-003 (low) are wording fixes.
