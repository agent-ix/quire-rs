---
id: SR-16312
title: "Spec review — PLAT-874 FR-051-AC-29 / CR-196 (quire-rs PR 596)"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-rs; spec/functional/FR-051-source-symbol-extraction.md, spec/tests.md, spec/assets/adr/0013-tree-sitter-symbol-extraction.md"
review_set: subset
relationships:
  - {target: "ix://agent-ix/quire-rs/spec/functional/FR-051", type: references}
---

## Summary

Ticket: PLAT-874. Frozen PR head: `5b129915de90eb833c198b86d87ce9288243c1cc`. Spec review of the new FR-051-AC-29 row, the CR-196 note, the edited CR-177 paragraph, the TC-2010 row and its matrix entry. Integrity and EARS conformance were checked inline. Per the brief, CR/TC number collisions with other open PRs are excluded.

## Verdict

**PASS with findings**. AC-29 is falsifiable: it states the failing case ("a zero-symbol successful report … fails this criterion"). The TC-2010 row and the matrix entries are consistent. Two defects remain. The AC has no parent SHALL clause and contradicts the unconditional statement above it. ADR-0013 still describes the removed behaviour.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | AC-29 exceeds FR-051's statement. The statement still says unconditionally that "The extractor SHALL ship per-language adapters for Rust, Python, and TypeScript", and no SHALL clause covers a build without an adapter or a coverage refusal. AC-29 therefore states behaviour no requirement asks for. The refusal half is FR-050 (coverage) behaviour, and nothing in FR-050 references it. Add an unwanted-behaviour clause to the statement ("If the build carries no adapter for a language whose source is in the tree, the extractor SHALL record it as unavailable …"), and add the refusal to FR-050 or cross-reference it there. | spec/functional/FR-051-source-symbol-extraction.md:60-79 |
| FND-002 | low | ADR-0013 still records the removed behaviour as the decision ("`src/symbols/mod.rs` falls back to a per-file build-configuration diagnostic for `SourceLanguage::Rust`" and the PLAT-868 "per-file fallback diagnostic" update) with no CR-196 supersession note. | spec/assets/adr/0013-tree-sitter-symbol-extraction.md:132-150 |

## Review Scope

| Unit | Role | Result |
| --- | --- | --- |
| FR-051-AC-29 | examined | Falsifiable, atomic enough; FND-001 on its parent statement |
| CR-196 note | examined | Accurate to the code change; supersedes the CR-177 sentence |
| CR-177 paragraph edit (feature set is part of identity) | examined | Consistent |
| FR-051-AC-9, FR-051-AC-10 | context_only | AC-9 channel is now parse-only; AC-10 wording still true |
| TC-2010 row and FR-051-AC-29 → TC-2010 matrix line | examined | Consistent |
| FR-051 statement | examined | FND-001 |
| ADR-0013 | examined | FND-002 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | high | The `FR-077-AC-7 → TC-2062` audit row was inserted into the Test Case Summary table (between TC-1893 and TC-1894) instead of the `## AC → TC Coverage Audit` table. `quire validate --scope . spec/tests.md` fails with four TestMatrix errors at line 1112, the only errors in the file, so `make validate` and `make ci` go red. The audit section also has no FR-077-AC-7 row. Move the row to follow `FR-077-AC-6` in the audit table. | spec/tests.md:1112 |

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | fixed | FR-051 statement now states the per-feature adapters and the SHALL record/refuse clause; refusal owned by FR-050-AC-58, FR-077-AC-7, FR-067-AC-8 |
| FND-002 | fixed | ADR-0013 carries a PLAT-874 / CR-201 supersession note |
| FND-003 | fixed | Stray row removed from the Test Case Summary; `FR-077-AC-7 → TC-2062` now sits in the AC → TC Coverage Audit table (spec/tests.md:2172); `quire validate spec/tests.md` reports no errors (c2f8e92) |
