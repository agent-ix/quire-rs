---
id: SR-072
title: "Evidence review of the semantic extraction boundary (#388)"
type: SpecReview
analysis: evidence
scope: "US-019, FR-069, FR-070, FR-071, FR-072, NFR-021, TC-1599..TC-1644 (spec/tests.md)"
review_set: all
---

## Summary

Every one of the 32 acceptance criteria and 12 constraints in scope names a verification method and is traced to at least one of TC-1599..TC-1644; `quoin advise` places every obligation (0 inconclusive) and the three Property rows (TC-1621, TC-1629, TC-1637) are property-shaped. The advisor flags two authored mismatches (FR-071-AC-6, NFR-021-AC-1) plus the four NFR-021 measurement rows; reading the grounding shows the real gaps are in which gate produces the evidence: cross-surface parity has no automated gate in either repo, `quire validate` rows point at a binary this repo does not ship, three Snapshot rows have no pre-change baseline to fail against, and one clause of FR-069-AC-8 is not observable from a hermetic test. Verdict: the plan is verifiable once FND-360..FND-364 are settled; the rest are wording and trace corrections.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-360 | medium | Authored `Inspection` on five obligations whose matrix rows are automated Static audits (`make audit-static` / `cargo test`); the catalog's `inspection` is class Inspection, evidence kind Manual, so a Static run does not discharge them. The advisor reports FR-071-AC-6 and NFR-021-AC-1 as mismatches. Set the cells to a Static-class method (`architecture-conformance`, evidence Static) and, for FR-071-AC-6, `Test` for the round-trip half that TC-1629 carries. | FR-070-CON-1, FR-071-CON-1, FR-071-AC-6, FR-072-CON-2, NFR-021-AC-1, TC-1619, TC-1627, TC-1640, TC-1641 |
| FND-361 | high | Cross-surface parity has no automated gate: `make ci` and `make ci-python` cannot see WASM, and quire-wasm's `make ci` / `ci.yml` run `wasm-pack test --node` only, never `tests/filament_core_parity.mjs` (`make test-filament`, manual). The harness also reads `../quire-rs/tests/fixtures/filament_core/graph_cases.json` by sibling path and needs an installed Python wheel, and today it does not know `extractSemantic` or `tests/fixtures/semantic/cases.json`. TC-767, the precedent, sits at "🚧 external". Name the gate that runs TC-1636 and the WASM leg of TC-1644, or mark both external as TC-767 is. | FR-072-AC-6, NFR-021-AC-4, TC-1635, TC-1636, TC-1644 |
| FND-362 | medium | `quire validate` rows name a surface this repo does not ship: quire-rs has no `[[bin]]`; the `quire` on PATH is quire-cli 0.31.0 (engine 0.46.0). In-repo the observable surface is `validate_document` (Integration); the CLI E2E evidence lives in quire-cli. TC-1634 (E2E) and the `quire validate` clause of TC-1599 should either be rephrased to `validate_document` or recorded as external to the CLI repo. | FR-069-AC-1, FR-072-AC-5, TC-1599, TC-1634 |
| FND-363 | medium | Snapshot rows assert byte-identity against a "pre-change baseline" that does not exist for the `Registry` path: `Registry` has no `Serialize` and `tests/fixtures/` holds baselines only for coverage (`coverage_baseline/expected.json`), assurance (`assurance/v1.json`) and the Filament graph (`filament_core/graph_cases.json`); properties-v1 has a schema, not a stored output. A Snapshot row can only fail against a baseline minted before the change lands, so the plan needs a task that captures the serialized archetype set and a properties-v1 output on `main` first. TC-1632, TC-1639 (coverage/assurance legs) and TC-1643 do have their baselines. | FR-069-AC-9, FR-069-CON-3, FR-070-AC-9, FR-070-CON-3, TC-1607, TC-1618, TC-1639 |
| FND-364 | medium | FR-069-AC-8's second clause ("equals the `toolchain.json` digest of `agent-ix/filament-core-data` at the recorded revision") is not observable from a hermetic test: it needs a git or network read of the upstream at `d48b8da`. The checkable form is vendored bytes == recorded provenance constant (TC-1606's first half); upstream equality is a vendoring-time procedure. Separately, the recorded path `packages/semantic-core/schemas/` does not exist at `d48b8da`; the files are `packages/semantic-core/generated/json-schema/*.json` and `generated/toolchain.json`, so a test pinned to the recorded path cannot be reproduced. | FR-069-AC-8, FR-069-CON-2, TC-1606 |
| FND-365 | low | "Arbitrary bytes" / "non-UTF-8-safe escapes" cannot reach the engine: `parse_document(markdown: &str)` takes UTF-8 by type. The Property domain for TC-1629 and the round-trip half of TC-1627 is arbitrary UTF-8 text (backticks, nested fences, CRLF); reword so the generator matches the input type. | FR-071-AC-6, TC-1627, TC-1629 |
| FND-366 | low | TC-1642 (Compile) carries two claims with different instruments: "no network, git, or persistence call" is a Static audit (`check_no_net_deps.sh` plus the module-source scan TC-1608/TC-1620/TC-1641 already use), and only "the `wasm` feature build passes" is Compile. The wasm32 build is not in quire-rs `make ci` or its workflows (no `wasm32` target anywhere); quire-wasm CI compiles it. Split the row or name the gate. | NFR-021-AC-2, TC-1642 |
| FND-367 | low | NFR-021 measurement rows M-1..M-4 are flagged by the advisor (`quantified-threshold` recommends `performance-benchmarking`). Judgement, not verdict: a target of 0 dependencies / 0 calls / 0 changed bytes / 0 mismatches is a static or byte-identity gate, not a benchmark; the authored `inspection (static)`, `unit-testing`, `integration-testing` stand, subject to FND-360 for the two inspection rows. | NFR-021-M-1, NFR-021-M-2, NFR-021-M-3, NFR-021-M-4 |
| FND-368 | low | FR-072-CON-3 (a schema generator is prohibited) is traced only to TC-1638 (Unit: schema validity + compatibility fixture), which cannot observe a generator. The prohibition is `scripts/audits/check_no_schemars.sh` (TC-062, Static); add TC-062 to the trace or extend TC-1640. | FR-072-CON-3, TC-1638, TC-1640 |
| FND-369 | low | FR-072-AC-1 places the suite at `tests/fixtures/semantic/cases.json` "in the `corpus_cases` shape with `issue_ref`", but `every_case_is_attributed_and_uniquely_named` walks `tests/fixtures/corpus_cases/` only; a sibling directory gets the shape without the attribution gate. Put the cases under `corpus_cases/` or extend the gate to the new path. | FR-072-AC-1, TC-1630 |

## Advisor Output

`quoin advise --repo /home/peter/dev/quire-rs --json`, filtered to the 44 obligations in scope: 0 inconclusive, 0 uncatalogued, 6 mismatches (FR-071-AC-6, NFR-021-AC-1, NFR-021-M-1..M-4). The `sca-sbom` recommendations on FR-069-AC-8, FR-070-AC-1/2, FR-071-AC-1/5, FR-072-AC-5 and NFR-021-AC-2 match on `third-party-dependency` (the vendored bundles) and are satisfied by TC-1606's provenance-digest check; the `property-based-testing` recommendations on the "every case" criteria (FR-070-AC-4..7, FR-072-AC-1, FR-069-AC-3..6) are covered by the table-driven Unit rows plus TC-1621. No `fault-detection-*` characteristic matched: nothing in scope is bound to a run yet.

## Evidence Strategy

| Requirement | Method | Artifact | Gate |
|---|---|---|---|
| US-019 | demonstration through EX-1..4 | TC-1610, TC-1612, TC-1613, TC-1600 | `make ci` |
| FR-069 | test + static audit + snapshot | `tests/semantic_contract.rs`, `scripts/audits/`, Registry baseline (FND-363) | `make ci` (`audit-static`) |
| FR-070 | test + property + static audit | `tests/semantic_properties.rs`, vendored quoin mapping fixtures | `make ci` |
| FR-071 | test + property + static audit | `tests/semantic_clauses.rs`, `operations-cases.json` | `make ci` |
| FR-072 | test + snapshot + parity | `tests/semantic_surface.rs`, `tests/fixtures/semantic/cases.json`, `tests/python/test_bindings.py`, quire-wasm `tests/filament_core_parity.mjs` | `make ci`, `make ci-python`; WASM leg ungated (FND-361) |
| NFR-021 | static audit + byte-identity + parity | TC-1641..TC-1644 | `make ci`; wasm32 build and WASM parity ungated in this repo (FND-361, FND-366) |

Per-obligation outcomes belong in each requirement's `Verification` cell; this review changes no spec file.

## Dispositions (applied 2026-09-03, same branch, before Plan-003)

| ID | Disposition |
| --- | --- |
| FND-360 | Fixed — all static-audit obligations are `Test`; FR-071-AC-6 split into a Test and the audit under CON-1. |
| FND-361 | Fixed — Python leg under `make ci-python` (TC-1635/1644); WASM leg external in `agent-ix/quire-wasm#3` with a CI parity gate as its deliverable. |
| FND-362 | Fixed — rows and ACs name `validate_document`; CLI E2E belongs to quire-cli. |
| FND-363 | Fixed — baselines named (`tests/fixtures/semantic/baseline/*.json`), minted on `main` first (plan task). |
| FND-364 | Fixed — vendored bytes vs recorded constant; path corrected. |
| FND-365 | Fixed — domain is arbitrary UTF-8 text. |
| FND-366 | Fixed — TC-1642 static, TC-1649 compile in `make ci`. |
| FND-367 | Accepted — methods stand. |
| FND-368 | Fixed — FR-072-CON-3 traced to the schemars audit (TC-1650). |
| FND-369 | Fixed — attribution test governs `tests/fixtures/semantic/cases.json` (FR-072 Inputs, TC-1630). |
