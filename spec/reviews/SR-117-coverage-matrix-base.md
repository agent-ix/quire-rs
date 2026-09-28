---
id: SR-117
title: "Base review of the computed coverage matrix spec (CR-187)"
type: SpecReview
analysis: base
scope: "agent-ix/quire-rs@bb9a280a4c4b4aa054c6cfbec2d2c5b69c114a54; spec/functional/FR-050-declarative-coverage-computation.md (CR-187, FR-050-AC-47..51); spec/functional/FR-051-source-symbol-extraction.md (FR-051-AC-27); spec/tests.md (TC-1930..TC-1941, FR-050/FR-051 coverage rows, AC to TC audit rows); spec/log.md (CR-187 entry)"
review_set: subset
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: reviews
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-051"
    type: reviews
---

## Summary

Ticket: PLAT-1077 (epic PLAT-1076). PR: agent-ix/quire-rs#494. This is the base
checklist over the CR-187 amendment. The diff adds a computed `coverage_matrix`
(FR-050-AC-47..51), a per-symbol "ignored" flag (FR-051-AC-27), and twelve
TC rows.

Several parts are sound. The status vocabulary, precedence and exhaustiveness
(AC-49) are well formed. `method-without-symbol` works against the module
data as it stands: `obligation::method_of` yields the Verification cell's
leading word, and the process module lists `Inspection`/`Analysis`/`Manual`/`Eval`
in `no_source_symbol`. No AC hard-codes an FR/AC/TC prefix; the ids are
examples only. TC ids are unique and every new AC has a TC.

Three high findings block the spec as written:

- The matrix population is every non-reference-only minted target. That
  includes the Test Matrix's own TC rows, so matrix documents are read in
  after all.
- Obligation-only ids (NFR metrics, CFG) are absent from the matrix, and a tag
  naming one still lands in `untracked_symbols`.
- The TypeScript and Python "ignored" forms name constructs the extractor
  either does not mint (`xit`, `xdescribe`) or mints as containers
  (`describe.skip`). AC-27 forbids manufacturing new symbols, so TC-1940
  cannot be met, and tests inside a skipped suite or class compute `tagged`.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | FR-050-AC-47 defines the matrix population as every `MintedTargetRecord` whose target is not `evidence: reference-only`. The ecosystem model (spec-artifacts-process `trace_targets`) declares `test-case` (archetype `TestMatrix`), `suite`, `inspection` and `stakeholder-validation-criterion` with the default `source` posture. So `spec/tests.md` becomes a `requirements[]` entry holding every TC row as a "criterion", which the epic intent forbids ("reads no TestMatrix document"). TC-1931's "carrying no additional trace targets of its own" is either vacuous (the fixture model declares no TestMatrix target) or false (it does, and the field changes). The fix must stay generic: select the population by module data, for example minted targets named by an FR-053 `obligations[].target:`, or a declared target posture. Do not select by archetype name. | FR-050-AC-47 (FR-050:149); FR-050 description lines 29-32; TC-1930, TC-1931 (tests.md:884-885) | wrong-requirement |
| FND-002 | high | Obligation-only ids are not covered. Under the PLAT-1079 ruling, NFR metric ids (`{document}-M-{row}`) and `-CFG-` ids exist only as derived obligations, never as trace targets. AC-47 admits an obligation only when it shares an id with a minted target, so these ids never appear in `coverage_matrix`. `#[trace("NFR-012-M-1")]` resolves to no declared id, and `coverage/reconcile.rs:258-266` puts it in `untracked_symbols` (AC-5). The spec neither binds such a tag to its obligation (so it would compute `tagged`) nor exempts it from `untracked_symbols`. It is not in `unmatched_tags`, because the marker form binds the quoted id on that symbol. | FR-050-AC-47 (FR-050:149); FR-050-AC-5; src/coverage/reconcile.rs:258 | missing-requirement |
| FND-003 | high | FR-051-AC-27's TypeScript forms contradict existing classification. First, `xit`/`xdescribe` mint no symbol today (`TEST_NAMES = ["test","it"]`, `SUITE_NAMES = ["describe","suite"]`, typescript.rs:247,255), and AC-27 says "no new symbol ... is manufactured". So TC-1940's "`xit('t', ...)` ... marks its registered symbol ignored" cannot be satisfied. Second, `describe.skip`/`xdescribe` are suite containers (FR-051-AC-21), not test-kind symbols. The spec never says whether tests inside a skipped suite inherit "ignored". So `it('t', ...)` under `describe.skip` computes `tagged` although it never runs, which is the false green the status exists to catch. Python has the same gap: class-level `@unittest.skip` / `@pytest.mark.skip` on a TestCase class, and module-level `pytestmark`. | FR-051-AC-27 (FR-051:171); FR-051-AC-21; TC-1940, TC-1941 (tests.md:894-895) | wrong-requirement |
| FND-004 | medium | FR-051-AC-27 does not say precisely which forms count. **Rust:** it is limited to `#[test]`/`#[bench]`, while AC-3's test family also includes `#[tokio::test]`, `#[rstest]` and `#[wasm_bindgen_test]` (rust.rs:19), and `#[cfg_attr(..., ignore)]` is not addressed. **TypeScript:** it does not say whether `.skipIf(cond)` (the AC-18 curried chain), `.todo`, `xtest` or `test.fixme` count under "`.skip` anywhere in the chain". **Python:** bare `@pytest.mark.skip` without parentheses, `skipif`/`skipIf`/`xfail`, and aliased imports (`from unittest import skip`, `import pytest as pt`) are unstated, although FR-051 already bounds unittest import aliasing. **Both records:** "the reported symbol/relation gains one additional boolean" does not say which record carries it, or whether it is omitted when false. If it is always serialized, every `quire symbols` payload and FR-045 record changes, against TC-750 byte-identity. | FR-051-AC-27 (FR-051:171); FR-051-AC-3; FR-051 lines 39-46; TC-1939..TC-1941 | wrong-requirement |
| FND-005 | medium | FR-050-AC-50 is underspecified and contradicts the CR-187 note. Today `#[trace("FR-034-AC-1..FR-034-AC-5")]` mints one `verifies` relation for the literal range string (`marker_ids`, trace.rs:1540), which lands in `untracked_symbols`. AC-50 does not say whether that record stays (a double report) or goes. A legacy `// Trace: FR-034-AC-1..FR-034-AC-5` line today binds `FR-034-AC-1`, because the legacy capture stops at `..`. Applying AC-50 to it un-backs that id and changes `totals` and `unbacked_rows`, although the CR-187 note says nothing changes them. FR-051, which owns relation minting, is not amended. Ranges that are not same-prefix (`FR-034-AC-1..FR-035-AC-2`) and the short form (`FR-034-AC-1..5`) are not addressed. | FR-050-AC-50 (FR-050:152); CR-187 note (FR-050:137); src/symbols/trace.rs:1540; TC-1936, TC-1937 | wrong-requirement |
| FND-006 | medium | The FR-050-AC-48 binder carries path, line and `SymbolKind`, but not the binding symbol's qualified name and not its ignored flag. A consumer therefore cannot name the test, or see why a criterion is `tagged-by-ignored-test`. The ordering key `(path, line, symbol_kind)` is not total: two TypeScript registrations on one line (`it('a', ()=>{}); it('b', ()=>{})`) tie, so AC-51 determinism is left to the implementation. AC-47 also does not state a `requirements[]` entry's fields (path or frontmatter id) or the order of `requirements[]` and `criteria[]`. | FR-050-AC-48 (FR-050:150); FR-050-AC-47; FR-050-AC-51; TC-1932 | wrong-requirement |
| FND-007 | low | FR-050-AC-51 says the field is present "whenever the model mints at least one criterion" and never states the zero case. The FR-055-CON-3 / AC-38 precedent omits an empty additive field, and that should be said. The clause "including one whose computed `status` is `untagged` for every criterion" is garbled: "one" has no referent. | FR-050-AC-51 (FR-050:153); FR-050-AC-38 | wrong-requirement |
| FND-008 | low | The matrix is folded into the FR-050-AC-20 baseline, but AC-20's enumerated surface list is not amended to name an ignored test, a `no_source_symbol` obligation or a range tag. Its companion test ("fails if the corpus stops exercising any of that surface") would therefore not notice the baseline losing any of the four statuses. | FR-050-AC-20; FR-050-AC-51; TC-1938 | correct-requirement-no-evidence |
| FND-009 | low | The FR-050 description's report-contents list (lines 62-71) does not mention `coverage_matrix`. TC-1930's "built only from `MintedTargetRecord` and `Obligation` ... names no document reference" is a code-structure claim that no test observes; only TC-1931's differential half is observable. | FR-050 lines 62-71; TC-1930 (tests.md:884) | wrong-requirement |

## Verdict

**Not mergeable as is.** FND-001 to FND-003 are high. FND-001 and FND-002 decide
the population the matrix computes over. FND-003 makes TC-1940 unsatisfiable
and leaves the headline false green (tests in skipped suites or classes
reported `tagged`). The medium findings are precision gaps that an implementer
would otherwise settle by guessing. Spec-only; no code changed.

## New findings (disposition pass 1)

Reviewed at `agent-ix/quire-rs@2d79eb4f3f1769182a20f3f4827ddef1d1419e8e`.

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-010 | medium | The behavior changes R2 and R4 introduced are not reconciled with the surrounding text. (1) The CR-187 note still says "Nothing here changes `unbacked_rows`, `status_lies`, `no_symbol_rows` or `totals`", while FR-050-AC-50 now says "`totals`, `unbacked_rows` and `backed` counts can therefore change". (2) Description item 3 and FR-050-AC-5 still define an untracked symbol as a tag resolving to "no declared target or row", so AC-5 still requires an obligation-only id tag to be reported, which contradicts AC-47's R2 clause. (3) The note and AC-47 cite a "combinatorial `-CFG-` id (FR-061)", but FR-061 defines no `-CFG-` shape, and no source or spec file mentions one. | FR-050:147; FR-050:68; FR-050-AC-5 (FR-050:191); FR-050:165; FR-050-AC-47 (FR-050:180) | wrong-requirement |
| FND-011 | medium | The FR-050-AC-48 binder is still "one per distinct `(path, symbol)` `verifies` relation". Under FR-051-AC-21 a TypeScript registration's qualified name ignores its enclosing suite, so `it('works')` in two different `describe` blocks of one file are two symbols with one `(path, symbol)` key. They collapse into a single binder, and its line, column and `ignored` are undefined. If one is skipped, the status is ambiguous. The new `column` (author judgment call 1) makes the sort key total but cannot help, because the dedup runs first. `column` also appears in neither the FR-051-AC-23 symbol record nor any adapter AC, and its unit (byte, char or UTF-16) is unstated. Fix: key binder identity on `(path, line, column)` or the AC-23 identity digest, and specify `column` in FR-051. | FR-050-AC-48 (FR-050:181); FR-051-AC-21; FR-051-AC-23 (FR-051:166); TC-1932 | wrong-requirement |
| FND-012 | low | FR-051-AC-27 widens the base-name sets of AC-3, AC-18 and AC-21 inline (author judgment call 2) without amending those ACs. AC-3 still reads "TypeScript `test`/`it` registrations classify as test symbols", and AC-21 still names only `describe`/`suite`. AC-27 also still says "no new symbol ... is manufactured", although `xit`/`xtest`/`xdescribe` now mint symbols. The direction is right; the owning ACs should state their own sets. | FR-051-AC-3 (FR-051:147); FR-051-AC-21; FR-051-AC-27 (FR-051:171) | wrong-requirement |

## Dispositions

| FND | Outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 2d79eb4 |
| FND-002 | fixed | 2d79eb4 (the residual AC-5 and description wording is tracked as FND-010) |
| FND-003 | fixed | 2d79eb4 |
| FND-004 | fixed | 2d79eb4 |
| FND-005 | fixed | 2d79eb4 (the stale CR-187 note sentence is tracked as FND-010) |
| FND-006 | fixed | 2d79eb4 (the dedup-key collision is tracked as FND-011) |
| FND-007 | fixed | 2d79eb4 |
| FND-008 | fixed | 2d79eb4 |
| FND-009 | fixed | 2d79eb4 |
