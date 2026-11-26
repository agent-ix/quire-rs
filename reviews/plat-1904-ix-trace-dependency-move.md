# PLAT-1904 ix-trace-rs dependency move

## Revisions

- Old: tag `v0.1.2`, commit `e29cc07f70394ff018884d12406c1e150bded5f4`.
- New: `main`, resolved and locked at commit `3cd3285055e32e4c08fb124d76e489b9dd412b6b`.

This aligns Quire-RS with the existing Quoin and `filament-contract-model` source identity. Cargo locks the branch to the resolved commit.

## Upstream changed paths

The following is `git diff --name-status <old> <new>` from the cached ix-trace-rs repository:

```text
M	.github/workflows/ci.yml
M	CLAUDE.md
M	Cargo.lock
M	Cargo.toml
M	README.md
M	clippy.toml
A	ix-trace-run/Cargo.toml
A	ix-trace-run/src/cargo.rs
A	ix-trace-run/src/junit.rs
A	ix-trace-run/src/lib.rs
A	ix-trace-run/src/logfile.rs
A	ix-trace-run/src/main.rs
A	ix-trace-run/src/matching.rs
A	ix-trace-run/src/registry.rs
A	ix-trace-run/src/run.rs
A	ix-trace-run/tests/common/mod.rs
A	ix-trace-run/tests/cost_quire_rs.rs
A	ix-trace-run/tests/emit_records.rs
A	ix-trace-run/tests/fixtures/abnormal/Cargo.lock
A	ix-trace-run/tests/fixtures/abnormal/Cargo.toml
A	ix-trace-run/tests/fixtures/abnormal/src/lib.rs
A	ix-trace-run/tests/fixtures/badargs/Cargo.lock
A	ix-trace-run/tests/fixtures/badargs/Cargo.toml
A	ix-trace-run/tests/fixtures/badargs/src/lib.rs
A	ix-trace-run/tests/fixtures/basic/Cargo.lock
A	ix-trace-run/tests/fixtures/basic/Cargo.toml
A	ix-trace-run/tests/fixtures/basic/src/foo/bar.rs
A	ix-trace-run/tests/fixtures/basic/src/foo/mod.rs
A	ix-trace-run/tests/fixtures/basic/src/lib.rs
A	ix-trace-run/tests/fixtures/basic/tests/x.rs
A	ix-trace-run/tests/fixtures/dup_log/Cargo.lock
A	ix-trace-run/tests/fixtures/dup_log/Cargo.toml
A	ix-trace-run/tests/fixtures/dup_log/src/lib.rs
A	ix-trace-run/tests/fixtures/harness_false/Cargo.lock
A	ix-trace-run/tests/fixtures/harness_false/Cargo.toml
A	ix-trace-run/tests/fixtures/harness_false/src/lib.rs
A	ix-trace-run/tests/fixtures/harness_false/tests/custom.rs
A	ix-trace-run/tests/fixtures/misuse/Cargo.lock
A	ix-trace-run/tests/fixtures/misuse/Cargo.toml
A	ix-trace-run/tests/fixtures/misuse/src/lib.rs
A	ix-trace-run/tests/fixtures/rstest_tokio/Cargo.lock
A	ix-trace-run/tests/fixtures/rstest_tokio/Cargo.toml
A	ix-trace-run/tests/fixtures/rstest_tokio/src/lib.rs
A	ix-trace-run/tests/fixtures/unified/Cargo.lock
A	ix-trace-run/tests/fixtures/unified/Cargo.toml
A	ix-trace-run/tests/fixtures/unified/a/Cargo.toml
A	ix-trace-run/tests/fixtures/unified/a/src/lib.rs
A	ix-trace-run/tests/fixtures/unified/b/Cargo.toml
A	ix-trace-run/tests/fixtures/unified/b/src/lib.rs
A	ix-trace-run/tests/fixtures/unmatched/Cargo.lock
A	ix-trace-run/tests/fixtures/unmatched/Cargo.toml
A	ix-trace-run/tests/fixtures/unmatched/other/outside.rs
A	ix-trace-run/tests/fixtures/unmatched/src/lib.rs
A	ix-trace-run/tests/rstest_records.rs
A	ix-trace-run/tests/runner_basic.rs
A	ix-trace-run/tests/runner_faults.rs
A	reviews/plat-333-code-review.md
A	reviews/plat-333-gap-analysis.md
A	reviews/plat-333-pr19-code-review.md
A	reviews/plat-333-pr19-gap-analysis.md
A	reviews/plat-333-pr20-code-review.md
A	reviews/plat-333-pr20-spec-review-spec-evidence-analysis.md
A	reviews/plat-333-pr20-spec-review.md
A	reviews/plat-333-spec-review-spec-ears-analysis.md
A	reviews/plat-333-spec-review-spec-evidence-analysis.md
A	reviews/plat-333-spec-review-spec-failure-domain-analysis.md
A	reviews/plat-333-spec-review-spec-integrity-analysis.md
A	reviews/plat-333-spec-review.md
M	rust-toolchain.toml
M	spec/functional/FR-001-trace-attribute.md
A	spec/functional/FR-005-emission-mode.md
A	spec/functional/FR-006-emission-run.md
M	spec/non-functional/NFR-001-dependency-posture.md
M	spec/spec.md
M	spec/tests.md
A	src/emit.rs
M	src/lib.rs
```

## Public items and local references

The new upstream public surface is the optional `emit` Cargo feature and the `ix-trace-run` workspace package/CLI. `#[trace]` now collects validated ids and, only when `emit` is enabled, embeds the emission record. Without that feature the macro still returns the annotated item unchanged. `#[implements]` retains its marker behavior; `#[string_edge]` is unchanged. The new upstream manifest raises `rust-version` to 1.98.1, which matches this repository's toolchain.

Repository search for `#[trace]`, `#[implements]`, and `#[string_edge]` found use across `src/`, `tests/`, `tools/quire-bench/`, and `fuzz/`. Quire's engine uses `#[implements]` in `src/impact.rs` and `#[trace]` throughout implementation and test code. The benchmark has trace markers in `tools/quire-bench/tests/`; the fuzz target uses `#[trace]` in `fuzz/fuzz_targets/fuzz_validate_extract_query.rs`. No Quire manifest enables `ix-trace-rs/emit` and Quire does not depend on `ix-trace-run`.

## Focused verification

- `cargo test --locked --test corpus_concurrency` — 4 passed.
- `cargo test --locked --lib` — 813 passed, including the `implements` macro consumer in `impact` tests.
- `cargo deny --locked --workspace --all-features check bans` — bans ok; existing wildcard warnings for Git/path dependencies remain.
- `cargo metadata --locked --offline --no-deps --format-version 1` and `git diff --check` passed.
