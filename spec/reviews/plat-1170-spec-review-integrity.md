---
id: SR-16328
title: "Integrity review of PLAT-1170 FR-050-AC-61 (quire-rs PR 610)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs; spec/functional/FR-050-declarative-coverage-computation.md, spec/tests.md (PR 610 diff only); context FR-051, FR-062, src/traceability.rs, src/symbols/trace.rs"
review_set: subset
---

## Summary

Ticket: PLAT-1170. This review checked FR-050-AC-61 for consistency against FR-062 (the owner of `implements` forms), FR-051-AC-11 (legacy rewrite suggestions) and the declared trace-tag model in `src/traceability.rs`. The AC keeps binding unchanged and uses the same `CoverageReport.diagnostics` and FR-057 `warning` shape as FR-050-AC-50 and FR-062-AC-12. However, it contradicts FR-062 on provenance, and it leaves its declaration-driven rule implicit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | AC-61 says the legacy occurrence's "`legacy` provenance [is] unchanged". FR-062 states that `ImplementsRelation` carries no provenance "because there is no legacy `implements` form", and `src/symbols/trace.rs:79` has no such field. The AC asserts a property that does not exist and contradicts FR-062. FR-062 is not amended by the PR. | spec/functional/FR-050-declarative-coverage-computation.md:307, spec/functional/FR-062-implements-relation.md:58 |
| FND-002 | medium | The AC says a form "of kind `rust_attribute`", but the declared field is `syntax: rust_attribute` (FR-062 CR-191, `TraceMarkerForm.syntax`). It also defines "legacy textual implements form" only by example. The rule an implementer needs is not stated, for example: "a Rust `trace_tags.implements` form declaring no `syntax`, in a module that also declares a `syntax: rust_attribute` form". | spec/functional/FR-050-declarative-coverage-computation.md:307 |
| FND-003 | medium | `template` is optional on `TraceMarkerForm`. The AC does not say what happens when the `rust_attribute` form declares no template, or which template is used when a module declares more than one Rust `rust_attribute` form. Either case leaves the "exact replacement text" undefined. | spec/functional/FR-050-declarative-coverage-computation.md:307, src/traceability.rs:1043 |
| FND-004 | low | AC-61 adds a second legacy-migration channel (a diagnostics finding) next to the existing FR-051-AC-11 `RewriteSuggestion` channel for verifies legacy forms. It does not reference FR-051-AC-11, and it does not say whether a legacy implements use also gets a `RewriteSuggestion`. | spec/functional/FR-050-declarative-coverage-computation.md:307, spec/functional/FR-051-source-symbol-extraction.md:159 |

## Verdict

Examined: FR-050-AC-61 and the CR-207 note. The AC does not hard-code form names. It states that the finding moves no total and adds no binding path, and that a module without the attribute form, a canonical occurrence and non-Rust source yield nothing. FND-001 must be fixed by dropping the provenance clause, or by amending FR-062 in the same PR. FND-002 and FND-003 should be tightened in the AC text.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | AC-62 drops the `legacy` provenance clause and now says only "its relations and counts are unchanged". |
| FND-002 | fixed | AC-62 keys on `syntax: rust_attribute`, and defines a legacy use as a binding made by a Rust `trace_tags.implements` form of the same module that declares no `syntax`. |
| FND-003 | fixed | AC-62 uses the template of the first declared `syntax: rust_attribute` form that has one. It renders the ids as quoted literals joined by `, `, which matches `src/symbols/trace.rs` rewrite rendering. When no such template exists, no finding is emitted. |
| FND-004 | fixed | AC-62 says the finding is diagnostic-only and emits no FR-051-AC-11 `RewriteSuggestion`, and links FR-051. |
