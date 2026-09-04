---
id: SR-075
title: "EARS conformance review of the semantic extraction boundary"
type: SpecReview
analysis: ears-conformance
scope: "US-019, FR-069, FR-070, FR-071, FR-072, NFR-021"
review_set: all
---

## Summary

Reviewed the six #388 artifacts with the engine grammar (`quire validate`, 9 `ears:*`/`quality:*` warnings on 7 statements) and a reader pass over all 44 SHALL statements and 36 acceptance criteria. No statement is ambiguous about what gets built; the dominant defects are two-SHALL bullets that pair a rule with its failure code, and refusal/severity obligations written in the indicative (`fails with …`, `the severity is error`) that the grammar cannot see. One AC pattern lacks a durable oracle: byte-identity against a "pre-change" baseline that is not a named fixture.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-420 | medium | `[ears:non-singular]` 1 engine warning (FR-069 L101 CON-2: file SHALL carry provenance and a test SHALL fail) plus 11 reader-found bullets the engine did not report, each packing two SHALLs (rule; failure code or second surface): FR-069 L24 (SHALL read and SHALL refuse), L66, L70; FR-070 L71, L76, L89; FR-071 L51, L72; FR-072 L58, L73 (Python SHALL return; WASM SHALL expose), L78. Split one requirement per SHALL so each maps to one AC row. | FR-069, FR-070, FR-071, FR-072 |
| FND-421 | low | `[ears:unclassifiable]` 2 and `[ears:missing-subject]` 2 warnings on the same two Description statements (FR-070 L21-22, FR-071 L21-22): `For an object artifact whose module carries a semantic block, the engine SHALL …`. The subject `the engine` is present but sits behind a non-EARS `For …` scope preamble, so no pattern matches. FR-069 L24 has the same intent stated as `When a module manifest carries …`, a manifest property (state), not the loader's read event. Rewrite as `When the loader reads a manifest that carries a semantic block, the loader SHALL …` / `When the engine extracts an object artifact whose module carries a semantic block, the engine SHALL …`. | FR-069, FR-070, FR-071 |
| FND-422 | low | `[quality:agentless-passive]` 4 engine warnings (FR-069 L66 `SHALL be validated`, FR-070 L51 `SHALL be recognized`, FR-071 L84 `SHALL be derived`, FR-072 L70 `SHALL be refused`) plus 1 reader-found the engine missed (FR-070 L82 `SHALL be read line by line`). The acting component is unambiguous in each file (loader in FR-069, engine elsewhere), so no allocation defect; name it. FR-072 L69-72 is also an unwanted condition and wants `If a snapshot's contractVersion is not 1.0.0, then the extraction API SHALL refuse …`. | FR-069, FR-070, FR-071, FR-072 |
| FND-423 | low | Data-as-subject: 16 Behavior statements make the artifact, cell, or heading the grammatical subject of an action the loader/engine performs (`A reference-form data_schema SHALL resolve`, `Type cells SHALL map`, `each heading SHALL own`, `availability SHALL record`): FR-069 L70, L76, L83, L86, L87; FR-070 L52, L55, L60, L71, L76, L82; FR-071 L51, L61, L65, L75; FR-072 L58. The engine accepts any noun phrase as subject, so none is flagged; the actor is clear from context, so low. Lead with `the loader SHALL` / `the engine SHALL`. | FR-069, FR-070, FR-071, FR-072 |
| FND-424 | medium | Obligations stated in the indicative with no SHALL, invisible to the grammar check and to any SHALL-keyed extraction: FR-069 L79-81 (three `fails with semantic.schema-ref-*`), L84-85 (`without a block it is silent`); FR-070 L57-59 (`fields is then unavailable`; `When the module sets legacy_forms: error, the severity is error`, a state trigger with no SHALL), CON-1..3 L97-99 (no SHALL in any constraint); FR-071 L53-54, L56-60 (five refusal/advisory outcomes), L69-71 (`dangling-clause-ref`, `duplicate-operation`); FR-072 L62-64, L71-72, L76-77, L83. Every refusal code an AC tests should be a SHALL statement. | FR-069, FR-070, FR-071, FR-072 |
| FND-425 | medium | Floating baseline in compatibility ACs: `byte-identical to the pre-change baseline` / `before and after this change` / `exactly as before` (FR-069 L31, CON-3 L102, AC-9 L117; FR-070 AC-9 L113; FR-072 L72, AC-3 L99; NFR-021 AC-3 L61). Once merged there is no "before", so the AC has no durable oracle unless the baseline is a named checked-in fixture (NFR-021 L45 comes closest by naming the fixture suites). State the fixture the bytes are compared to. | FR-069, FR-070, FR-072, NFR-021 |
| FND-426 | low | Static-boundary ACs name no oracle for the negative: FR-071 AC-6 L95 (`no clause tokenizer, parser, or evaluator symbol reachable`), NFR-021 AC-1 L59 and metric rows L43-44 (`no OCL, SysML, or FRETish parser`, `no network, git, or persistence call`). Which crate or symbol names count is unstated, so the inspection cannot fail deterministically; give the test a named denylist. | FR-071, NFR-021 |

## Coverage

- Requirement-bearing artifacts reviewed: 5/5 (FR-069, FR-070, FR-071, FR-072, NFR-021); US-019 examined for form only (Given/When/Then examples, no SHALL statements, no findings).
- SHALL statements read: 44 (FR-069 14, FR-070 10, FR-071 9, FR-072 10, NFR-021 1). Acceptance criteria read: 36.
- Scoped engine warnings: 9 (`ears:non-singular` 1, `ears:unclassifiable` 2, `ears:missing-subject` 2, `quality:agentless-passive` 4). `ears:vague-response` and `ears:non-canonical-trigger`: 0.
- Reader-found beyond the engine: 11 non-singular bullets, 1 agentless passive, 16 data-as-subject statements, 4 files with indicative obligations, 7 floating-baseline ACs, 3 oracle-less static ACs.
- Highs: 0. No statement's ambiguity changes what gets built; the FRs' ACs pin every refusal code the prose leaves indicative.

## Dispositions (applied 2026-09-03, same branch, before Plan-003)

| ID | Disposition |
| --- | --- |
| FND-420 | Fixed — one SHALL per statement; refusal codes in `If … then … SHALL` form. |
| FND-421 | Fixed — Descriptions open with `When the engine/loader …`. |
| FND-422 | Fixed — active voice with named agent. |
| FND-423 | Fixed where an obligation was hidden; descriptive mapping bullets keep data subjects by design. |
| FND-424 | Fixed — refusals and severities carry SHALL. |
| FND-425 | Fixed — baselines named as checked-in fixtures. |
| FND-426 | Fixed — crate and symbol denylist in NFR-021-AC-1/AC-2. |
