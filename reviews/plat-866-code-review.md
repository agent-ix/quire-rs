---
id: SR-20261103118145
title: "Code review (with rust-review) of structural Python and TypeScript implements syntax (quire-rs PR 630)"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; git diff origin/main...0d94218 (src/coverage.rs, src/coverage/, src/symbols/, src/traceability.rs, tests/it/implements_structural.rs, tests/it/main.rs)"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/FR-062", type: references}
  - {target: "ix://agent-ix/quire-rs/FR-051", type: references}
---

## Summary

Ticket: PLAT-866. PR head `0d94218` (branch `plat-866-engine`). Code review with the rust-review lane folded in, scoped to `git diff origin/main...HEAD` (13 files, +1221/-37). The changed code was checked against FR-062 ("Declared Python and TypeScript syntax positions", AC-18..27, CON-2, CON-5), FR-051-AC-22 and FR-051-AC-51.

Evidence: `cargo test --test it implements_structural` (debug=0, `locked-build.sh`) ran in a scratch detached worktree at the same head with one added probe test. The worktree under review was not modified. All 14 PR tests pass. The probe confirmed FND-001 and cleared six other edge cases (listed in the Verdict).

## Verdict

**Pass with findings: one medium, three low.**

Answers to the dispatch questions:

1. **Id-shape check.** None is required. In FR-062, the syntax field "SHALL NOT select, replace or broaden the module's `pattern`", and "the engine SHALL NOT supply an `implements` pattern" (CON-2). Spellings are "exactly what the module's declared `pattern` matches". spec-artifacts-process FR-004-AC-31/32 put the guard on the shipped pattern: double-quoted ids, unquoted args and empty calls rejected, `\A` anchoring. ix-trace-rs FR-003 is a compile-time check in the runtime macro, not an engine rule. The engine's existing `rust_attribute` path does no shape check either (`marker_ids`). The FR-051-AC-49 kind test covers only advisory tokens (`unmatched_tags`, `mentions`). The module pattern alone therefore satisfies the spec as written.
2. **Unattached means full match only.** This reading is consistent with the spec. A structural match "SHALL cover the complete candidate". A `partial-implements-syntax-match` finding must name a "qualified owning symbol", which an unattached call does not have. A partial match on an unattached call yielding nothing breaks no AC.
3. **TypeScript decorator positions.** Method decorators are read as the preceding sibling run, skipping comments. A field or statement ends the run, and the probe confirmed a property decorator does not leak onto the next method. Class decorators are read before and after `export`, and `export default` works. The abstract-signature rejection is the existing adapter behaviour. A `function` declaration gets `position: None`, so it has no candidates and the advisory names its kind.
4. **No execution or import resolution.** Candidates are `utf8_text` of tree-sitter nodes only. No alias table, no `eval`, no second lexer.
5. **Load checks.** The `TraceMarkerForm` deserializer accepts each new value only with its own language. Anything else errors with "trace marker '<name>' has invalid syntax". `validate()` refuses all four guarded syntaxes under `trace_tags.markers`, naming the form. See FND-002 for the gap left in `validate()`.
6. **Public struct additions.** `Symbol.position`, `SourceFile.unattached_calls`, `OutsideMarkerId.position` and the new `TraceMarkerSyntax` variants break no consumer. quire-cli `origin/main` has no struct literal or exhaustive match on these types, and `gh search code --owner agent-ix TraceMarkerSyntax` finds quire-rs only.
7. **Tests.** See the gap-analysis SR (SR-20261103118146). No pins, digests or ceremony.

Also checked and found correct: the new `TraceMarkerSyntax::as_str`; the structural/position match in `bind_implements`; routing partial matches to `partial_syntax_matches`; the advisory template choice by `finding.position` (Rust stays `rust_attribute`, shell and module containers fall back to the kind); the guidance strings; both reasons added to `COVERAGE_DIAGNOSTIC_REASONS`. The `corpus_cases::tc1026` failure is expected (qa-corpus vocabulary) and is not raised here.

Probe outcomes that match the spec, so not findings:
- a destructuring declarator, a class field and `export default` wrappers each report `implements-syntax-unattached`;
- a triple-curried `implementing(A)(opts)(fn)` reports a partial match on `implementing(A)(opts)` and an unattached finding on the inner call;
- a comment before the wrapped function binds correctly;
- a Python decorator holding a `# comment` binds nothing, which is a module-pattern concern.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Two pieces of code decide which chain links are attached, and they disagree on comments. `wrapper_chain` skips `comment` nodes when it finds a link's single argument. The attached-link walk in `unattached_calls` uses `arguments.named_child(0)`, which returns the comment. Reproduced: `const x = implementing("FR-1")(/* c */ implementing("FR-2")(() => 1));` binds FR-1 and FR-2 to `x` and also reports `implements-syntax-unattached` for `implementing("FR-2")`. This breaks FR-062-AC-24: an attached link yields no finding, and an unattached call binds nothing. The same call cannot be both. Fix: build the attached set from the links `wrapper_chain` returns (have it yield the call nodes too), so one function decides chain membership. Add the case to TC-2206. | src/symbols/typescript.rs:305-313, src/symbols/typescript.rs:264-281 |
| FND-002 | low | `TraceabilityModel::validate` still refuses only `rust_attribute` on a non-Rust `implements` form. It has no matching check for `python_decorator`, `typescript_decorator` or `typescript_wrapper`. Wrong-language structural forms are refused only by the deserializer. A model built in memory or merged without passing through it (the CR-081 path) keeps, for example, a `typescript_wrapper` Python form. That form then binds nothing and nothing reports it. Extend the existing `match marker.syntax` arm to the three new values. | src/traceability.rs:1816-1826 |
| FND-003 | low | The production entry is now `parse_with_calls`. It returns an anonymous `(Vec<RawSymbol>, Vec<SourceAttribute>)` that `mod.rs` reads as `.1`/`.0`. Meanwhile `parse` and its contract doc ("A declaration-structure failure fails loudly") became `#[cfg(test)]`-only, and the `[parse]` intra-doc link on `parse_with_calls` dangles in a non-test doc build. Return a named struct, as `shell::parse` does with `Parsed`, and move the contract doc to the production function. | src/symbols/typescript.rs:180-191, src/symbols/mod.rs:582-586 |
| FND-004 | low | The `Symbol.position` doc says it is `None` for "a suite". Python test-suite classes (`is_suite`) get `Some(PythonDecorator)`, the same as any class. The code is right because a class is a Container either way, but the doc misleads. Say "a TypeScript `describe`/`suite` registration". | src/symbols/mod.rs:149-154, src/symbols/python.rs:413-418 |
