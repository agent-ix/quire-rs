---
id: SR-2026111921333400000008
title: "code-review of PLAT-1799 hardening properties, fuzz targets and mutation oracle"
type: SpecReview
analysis: code-review
scope: "agent-ix/quire-rs; PR #711 at 666ea9a3a17b695b53433fe6f14faa7984127124; .github/workflows/fuzz.yml, Makefile, fuzz/Cargo.toml, fuzz/fuzz_targets/fuzz_assurance_export.rs, fuzz/fuzz_targets/fuzz_evidence_record.rs, fuzz/fuzz_targets/fuzz_trace_trailers.rs, src/evidence.rs, tests/it/coverage_matrix.rs, tests/it/hardening_properties.rs, tests/it/main.rs"
review_set: subset
---

## Summary

Code review with the rust-review lane, plus a diff-scoped gap check against the ticket, of quire-rs PR #711. Ticket: PLAT-1799. The PR adds two proptest round-trip properties and a cause-code bijection test, a composed mutation-oracle test, a split of `backs_minted` into two helpers with unit tests, a `MUTANTS_ARGS` pass-through for `make mutants-fr`, and three fuzz targets registered in the Makefile and the fuzz workflow.

The ticket and PR text were read as data. The mutants run is still in progress by another agent, so mutation results are judged by their absence from the diff only.

## Examined scope

- `src/evidence.rs` helper split: behaviour-preserving extraction of `declared_claim_matches_minted` and `criterion_claim_matches_minted`. Correct by reading. (examined)
- `src/evidence.rs` unit tests: the declared helper is tested for every locus component. The criterion helper omits the id component (FND-003). (examined)
- `tests/it/coverage_matrix.rs` oracle `tc1799_mutation_sensitive_retained_evidence_refusals`: calls six existing tests that share `tmpdir()` paths (FND-001). (examined)
- `Makefile` `mutants-fr` `MUTANTS_ARGS` pass-through and the documented `-- --test it tc1799` invocation (FND-002). (examined)
- `tests/it/hardening_properties.rs`: round-trip properties hold every `Option` field at `None` (FND-005). Trace tags claim AC-10 and AC-19 (FND-006). (examined)
- `fuzz/fuzz_targets/fuzz_assurance_export.rs`: no seed input reaches the accepted path (FND-004). (examined)
- `fuzz/fuzz_targets/fuzz_trace_trailers.rs`: fuzzes `extract_file` only. The `Implements:` trailer patterns are applied in `symbols::trace::bind`, which is not called (FND-007). (examined)
- `fuzz/fuzz_targets/fuzz_evidence_record.rs`: strict typed round trip. Correct. (examined)
- `fuzz/Cargo.toml`, `Makefile` `FUZZ_TARGETS`, workflow matrix: the three targets are registered consistently. The crash-only upload on `failure()` is correct. (examined)
- Ticket gap: no mutants survivor analysis or results in the diff (FND-008). (examined)

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | high | The oracle test calls six tests that use fixed per-process `tmpdir()` paths. In a normal parallel `cargo test` run they race on the same directory with their originals | tests/it/coverage_matrix.rs:1805-1813 |
| FND-002 | medium | The documented `MUTANTS_ARGS='-- --test it tc1799'` restricts cargo test to the `it` target, so the new `src/evidence.rs` unit tests that pin the split helpers never run against mutants | Makefile:238 |
| FND-003 | medium | `tc1799_criterion_target_backing_requires_exact_locus_and_selection` never varies `id`, so mutants of `entry.obligation().id == id` and of the source-name match survive it | src/evidence.rs:819-875 |
| FND-004 | medium | `fuzz_assurance_export` has no committed seed. Its healthy round-trip oracle is reached only when libFuzzer invents a valid v1 export with matching premises from nothing | fuzz/fuzz_targets/fuzz_assurance_export.rs:22-28 |
| FND-005 | medium | Both round-trip properties hold `target`, `row` and `line` at `None`, so the `skip_serializing_if` branches are never generated | tests/it/hardening_properties.rs:27,55 |
| FND-006 | medium | The JSON round-trip and cause-name tests are traced to FR-078-AC-19 and FR-078-AC-10, which they do not verify | tests/it/hardening_properties.rs:17,41,67 |
| FND-007 | medium | `fuzz_trace_trailers` never applies trailer patterns. Only `extract_file` runs, and its sole oracle compares two calls of a pure function | fuzz/fuzz_targets/fuzz_trace_trailers.rs:12-17 |
| FND-008 | medium | The ticket's survivor analysis and mutation results are not in the diff | Makefile:238 |
| FND-009 | low | The oracle test adds five trace tags while asserting nothing beyond its callees, which already carry their own tags | tests/it/coverage_matrix.rs:1796-1803 |
| FND-010 | low | The cause-code test does not bind the serde name to `as_str`, so the two wire encodings can drift | tests/it/hardening_properties.rs:69-82 |

## Verdict

Not mergeable yet. FND-001 is a real flake source in every full or `coverage_matrix` test run. FND-002 and FND-003 weaken the mutation sensitivity the PR exists to show, and FND-004 and FND-007 leave two of the three fuzz targets without a reachable healthy control. The helper split is behaviour-preserving, `fuzz_evidence_record` is sound, and the target registration and crash-only workflow upload are correct.
