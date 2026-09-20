# PLAT-843: Rust symbol-scanner AST-port differential, against PLAT-840's baseline

- Date: `2026-09-20`
- Baseline: `reports/2026-09-20-plat840-rust-scanner-baseline.md` / `.json`, corrected version (PLAT-840, PR #469) — `rust_symbols_total_all_repos: 31,781`, `abandoned_files_total_all_repos: 3`, `abandoned_files_total_rust_only: 0`, `unbacked_rows_total_all_repos: 2,408`, `status_lies_total_all_repos: 105`. The superseded first-version numbers (31,798 / 4 / 1) are **not** used here.
- Old engine: `quire-rs` pristine `main` at `08d39ea2ce50db35812f836df65cac18757daa4b` (the sha PLAT-840 itself recorded as the `quire-rs` target row) — the exact pre-PLAT-843 `src/symbols/rust.rs` line-structural scanner.
- New engine: this PR's branch (`feat/plat843-rust-ast`), tree-sitter port via `quire-code-parse` pinned at `57b83ba00431914060297bf94fcee31549c9b68b`.
- Target trees held constant: `quire-rs`/`quire-code-rs` measured against their own already-pinned local checkouts (`/home/peter/dev/quire-rs` at `08d39ea2c...`, `/home/peter/dev/quire-code-rs` at `e1b7fc303...` — both verified unchanged, still exactly PLAT-840's recorded shas); `quire-contract-ir`, `quire-protocol`, `filament-ide-rs`, `ecaz` measured against disposable clones pinned at PLAT-840's exact recorded shas. Both engines ran over byte-identical source trees per repo — every delta below is engine-only.
- Harness: `examples/plat840_rust_baseline_sweep.rs` (unmodified), run once with the new-engine binary over all six pinned trees.

## Headline

| Metric | Baseline (PLAT-840, corrected) | This PR (new engine) | Δ |
|---|---:|---:|---:|
| `rust_symbols_total_all_repos` | 31,781 | 31,695 | **−86** |
| `abandoned_files_total_all_repos` | 3 | 3 | **0** |
| `abandoned_files_total_rust_only` | 0 | 0 | **0** |
| `unbacked_rows_total_all_repos` | 2,408 | 2,411 | **+3** |
| `status_lies_total_all_repos` | 105 | 105 | **0** |

Every non-zero delta is named below by direct cause, verified with per-symbol diffs (a tool comparing old-engine and new-engine `(path, qualified_name, kind)` triples over byte-identical source), not inferred from the aggregate.

## `rust_symbols_total`: −86, fully explained by two causes

| Repo | Baseline | New | Δ | Symbols removed | Symbols added |
|---|---:|---:|---:|---:|---:|
| quire-rs | 3,403 | 3,403 | 0 | 0 | 0 |
| quire-code-rs | 429 | 429 | 0 | 0 | 0 |
| quire-contract-ir | 2,217 | 2,187 | **−30** | 33 | 3 |
| quire-protocol | 2,524 | 2,490 | **−34** | 48 | 14 |
| filament-ide-rs | 7,704 | 7,685 | **−19** | 58 | 39 |
| ecaz | 15,504 | 15,501 | **−3** | 41 | 38 |
| **all repos** | **31,781** | **31,695** | **−86** | 180 | 94 |

quire-rs and quire-code-rs are **byte-identical**: the new engine's `(path, qualified_name, kind)` list matches the old engine's exactly, symbol for symbol, over the same pinned tree. quire-rs's own tree was the proving ground for the port (see "proptest fix" below); the other four repos surfaced two further causes, both **removed false positives**, not lost real declarations:

### Cause 1: `macro_rules!` definition-template text (present in all four repos)

A `macro_rules! foo { (...) => { struct X { ... } impl X { ... } } }` **definition**'s expansion template routinely contains literal `struct`/`impl`/`fn`/`enum` text — this is the macro's own output pattern, not compiled code at that source location. The old line-structural scanner matched this text as if it were real top-level declarations, because it has no concept of "inside a macro_rules! template body." Tree-sitter correctly parses a `macro_rules!` definition as its own grammar construct (pattern/template token trees) and does not emit `function_item`/`struct_item`/etc. nodes for template contents — so the new engine, correctly, does not mint symbols there.

Confirmed by direct source inspection, one `macro_rules!` definition per removed cluster:

- `quire-contract-ir`: `crates/quire-contract-model/src/identity.rs` (`macro_rules! diagnostic_codes`, `identifier_type`, `positive_revision`), `crates/quire-contract-model/src/output_mapping.rs` (`macro_rules! raw_digest_type`, `mapping_error_codes`, `source_selection_type`, `qualified_code_type`), `src/temporal/request.rs` (`macro_rules! observation_artifact`) — the removed `DiagnosticCode`/`Wire`/`MappingRequestErrorCode` containers and their `as_str`/`deserialize`/`fmt`/`new`/`digest`/etc. template methods were never real declarations at these source locations.
- `quire-protocol`: `src/ids.rs:273` `macro_rules! identities` (`IdentityKind` enum + 11 methods, template body), and the same pattern in `src/repro.rs`, `src/claims/mod.rs`, `src/closure.rs`, `src/producer_contracts.rs`.
- `filament-ide-rs`: `crates/filament-core/src/identity.rs` (1 `macro_rules!`), `crates/filament-markups/tests/source_integrity.rs` (2 `macro_rules!`) — 6 and 12 removed methods respectively.
- `ecaz`: `src/am/ec_distann/lifecycle_state.rs:14` `macro_rules! lifecycle_state` (`allows`/`as_str`×2/`fmt`/`parse`).

This is the same class of defect CR-040 already fixed for raw-string/lifetime false positives — text that is lexically declaration-shaped but not real code at that location — now closed for `macro_rules!` template bodies too. It is **in scope** under the ticket's own rule ("free from parsing correctly"): removing it is a direct, correct consequence of parsing with a real grammar rather than text matching, not a change to what a symbol *is*.

### Cause 2: method-qualification repair (filament-ide-rs, ecaz)

The old scanner's brace-counting container stack could desync and emit a **bare, unqualified** method name (`backup`, `new`, `send_embeddings`) instead of `Type::method`, for reasons not fully root-caused (the corruption predates this port and isn't reproducible from source inspection alone — it did not correlate with any one syntactic feature across the affected files). The new engine reads the `impl_item`'s own `type` field directly from the AST, so every method is qualified under its `impl` target unconditionally, matching this ticket's own stated identity rule ("methods qualify under the for-target type"). Examples: `filament-ide-rs/crates/filament-sidecar/src/host.rs` — 24 methods, `backup`→`RealSidecarHost::backup`, etc.; `ecaz/src/am/ec_hnsw/search.rs` — 27 methods across `BeamSearch`/`VisibleFrontier`. Net symbol count is unaffected (1 removed : 1 added per method), but the **qualified name, and therefore the symbol id, changes** — named here because it is an identity change, even though it nets to zero in the headline count.

This fix is in scope for the same reason as Cause 1: it corrects the scanner's own stated, ticket-mandated behavior, it does not redefine what "qualify under the for-target type" means.

### The proptest! regression (quire-rs only, fixed before this report — see "Fixes made" below)

quire-rs's own tree required one additional fix during the port (`proptest!`-declared tests, a macro **invocation**'s custom-DSL arguments, distinct from Cause 1's macro **definition** templates) before it reached byte-identical; that fix is why quire-rs shows Δ0 above rather than a regression. See "Fixes made mid-port" for the mechanism.

## `abandoned_files`: exact match (3 / 3, 0 / 0 Rust)

No repo's abandoned-file population moved. The `check_balanced` whole-file-rejection mechanism this port replaces is what the three (TypeScript, all `filament-ide-rs`) files were rejected by; the Rust `check_balanced` code path is deleted entirely (the new engine has no whole-file gate — tree-sitter recovers per-node from a malformed subtree, see `a_truncated_file_fails_loudly_naming_a_line` in `src/symbols/rust.rs`'s test module), and zero real Rust files were resting on that gate to begin with, per PLAT-840's own headline finding. Confirms PLAT-840's prediction directly: "the differential extraction should not expect any abandoned-file recovery on the Rust side — there is nothing left to recover. If the AST rewrite's differential shows a Rust abandoned-file count above zero on either side, that is a surprise requiring its own explanation." No surprise; 0/0 both sides.

## `status_lies`: exact match (105 / 105)

No repo's status-lie count moved, including quire-protocol's `+63 backed` shift (below) — none of the newly-backed rows were rows whose declared status claimed "verified" while unbacked.

## `unbacked_rows`: +3, two named per-repo moves

| Repo | Baseline unbacked | New unbacked | Δ | Baseline backed | New backed | Δ backed |
|---|---:|---:|---:|---:|---:|---:|
| quire-rs | 400 | 404 | **+4** | 1,312 | 1,309 | **−3** |
| quire-code-rs | 12 | 12 | 0 | 251 | 251 | 0 |
| quire-contract-ir | 16 | 16 | 0 | 198 | 202 | +4 |
| quire-protocol | 61 | 60 | **−1** | 175 | 238 | **+63** |
| filament-ide-rs | 1,073 | 1,073 | 0 | 1,609 | 1,609 | 0 |
| ecaz | 846 | 846 | 0 | 13 | 13 | 0 |
| **all repos** | **2,408** | **2,411** | **+3** | — | — | — |

**quire-protocol: `backed` +63, a real coverage gain.** This repo's remaining diff (Cause 1 above accounts for the count-level loss) also contains a batch of `Function` → `TestFunction` reclassifications for the same qualified names, seen while investigating Cause 1 — a direct consequence of the same more-correct attribute/leading-span reading behind the PLAT-69/PLAT-846 fixes. Where a matrix criterion's declared reference type requires `TestFunction`-kind evidence specifically, a symbol previously (mis)classified as plain `Function` could never satisfy it; correctly classified, it now can. **Not decomposed row-by-row here** — the sweep harness does not dump full `unbacked_rows`/`backed` record lists (only `unmatched_tags`/`non_binding_tags` are full dumps, per PLAT-840 F4), and reconstructing the per-row set would mean re-implementing `coverage::compute`'s internals outside the library, disproportionate for a report. The reclassification mechanism is confirmed; the exact 63 rows are not individually named.

**quire-rs: `+4` unbacked, `−3` backed — small, not fully decomposed.** quire-rs's own symbol list is byte-identical to the old engine (see above), so this shift is not a missing-or-extra-symbol effect; it must be a **span** effect — PLAT-69 (split-attribute leading-span truncation) and PLAT-846 (block-doc-comment joining) both change exactly which lines a symbol's `leading_line`/annotation-capture window covers, which can move a specific trace tag from "captured, binds" to "not captured, doesn't bind" or vice versa, on individual symbols, without changing the symbol's own identity. This is a direct, in-scope, expected consequence of the two named span fixes this ticket folds in. The net movement is small (net −7 rows across two buckets, out of 1,799 total quire-rs reference rows) and **not decomposed to specific rows** in this report — flagging that gap explicitly rather than asserting a row-level cause not actually checked.

## Fixes made mid-port (beyond the ticket's own named list)

1. **`proptest!`-declared tests** (quire-rs, not in the ticket's own enumerated defect list). `proptest! { #[test] fn name(pattern in strategy) { ... } }` is a macro **invocation** whose custom argument DSL tree-sitter tokenizes but never parses as `function_item` nodes. Left unhandled, this silently dropped every `proptest!`-declared test (34 on quire-rs's own tree, ~175 across all six repos before this fix) — including real, trace-tagged tests (`TC-890`..`TC-896`, `TC-819`, etc.). Fixed by scanning `proptest!`/`proptest::proptest!` invocation token trees for `fn NAME(...) { ... }` sequences (`collect_proptest_tests`/`scan_token_tree_for_fns`/`flat_leading_span_and_test` in `src/symbols/rust.rs`), with the same attribute/comment-aware leading-span logic used for top-level declarations. Verified fix: quire-rs's own tree went from a −175-symbol regression across the sweep to the byte-identical, Δ0 result shown above.
2. **`unsafe impl` container scoping** (no repo shows this in the six measured trees' populations, but it is a real, deliberate divergence from the pre-port scanner, named here per the ticket's instruction to flag anything touching symbol identity). The old scanner's `declaration()` check required a literal `trimmed.starts_with("impl")`, and its modifier-stripping keyword list did not include `"impl "` — so `unsafe impl Foo for Bar { ... }` was never recognized as an impl block at all; its methods stayed flat under whatever container was already in effect, with no `Foo` qualification. The new engine recognizes `impl_item` as its own grammar node regardless of an `unsafe` modifier, so `unsafe impl` methods are now qualified under `Foo`, consistent with every other impl block and with this ticket's own stated identity rule. Treated as in scope (a "free" fix, not a judgment call): the ticket's rule ("methods qualify under the for-target type") is unconditional, and the old behavior was an accidental gap in a keyword list, not a deliberate design choice being changed here.

## Tests retired

Per the ticket's rule (a successor tagged and shown red against a deliberately-broken extraction before the predecessor is removed):

| Retired | Asserted (why it dies) | Successor(s) | Successor asserts |
|---|---|---|---|
| `tc804_lexer_counts_only_code_braces` | Internal counter state of the deleted brace-counting lexer | `tc804_delimiters_in_string_and_comment_content_do_not_move_symbols` | Outcome: a string/comment containing delimiter-shaped bytes does not shift symbol boundaries |
| `tc804_string_state_carries_across_lines` | Internal string-state-carry mechanism of the deleted lexer | `tc804_a_delimiter_carried_across_lines_does_not_move_symbols` | Outcome: a multi-line string/comment does not shift symbol boundaries |

`tc804_rust_lexing_is_string_and_lifetime_aware` (asserts outcome, not mechanism) survives unchanged. No `tc803_*` (typescript.rs) or `tc1029`/`tc1030`/`tc1031` (python.rs) tests were touched — out of scope, Phase 2 (PLAT-851).

## Boundary gate

`crates/quire-rust-extraction`'s `tests/dependency_boundary.rs` (adapted from `filament-ide-rs`'s `crates/filament-code-extraction` pattern) asserts no package named `quire-rs`/`quire-rust-extraction` depends on anything named `tree-sitter*` directly. Observed both states directly, not assumed:

- **Green** (current tree): `cargo test -p quire-rust-extraction --test dependency_boundary` — 1 passed.
- **Red** (deliberate violation): added `tree-sitter = "0.20"` directly to the root `Cargo.toml`'s `[dependencies]`, re-ran the same test — failed with `crate `quire-rs` depends on `tree-sitter` directly; the tree-sitter boundary must live behind the quire-code-parse pin in crates/quire-rust-extraction`. Reverted immediately (`Cargo.toml`/`Cargo.lock` restored via `git diff` confirmed clean against the real PLAT-843 change).

## Gates

- `make fmt-check`, `make lint`: clean.
- `make test`: exit 0, all suites pass (`cargo test --locked` over the full workspace).
- `make deny`: `licenses ok` (pre-existing unmatched-exception/allowance warnings, unrelated to this change, also present on clean `main`).
- `make audit-unsafe`, `make audit-property`: clean.
- `make audit-static` (all `scripts/audits/*.sh`): all exit 0; `check_status_agreement.sh` prints its usual advisory list (not a failure — advisory per FR-057, unrelated to this change).
- `check-wasm` (both legs: `cargo check --target wasm32-unknown-unknown --features wasm` and the wasm-feature test suite): clean. Required a fix — see "wasm feature gate" below.
- `validate`: **fails, pre-existing, verified unrelated.** Reproduced the identical failure on a clean, unmodified `quire-rs` checkout at the same commit this branch forked from — this worktree's sibling `spec-artifacts-iso` checkout doesn't match the commit pinned in `quality/validation-stack-lock.json`. Not fixed here: fast-forwarding that shared checkout risked disrupting other concurrent sessions using it, and it is unrelated to this PR's diff (touches only `src/symbols/{rust,mod}.rs`, `Cargo.toml`/`.lock`, `deny.toml`, `crates/quire-rust-extraction/**`, `spec/**` documentation). `check-engine` (which runs after `validate` in the `ci` target list and was therefore never reached in the composite run) verified independently, passing, with `QUIRE_CLI=/home/peter/dev/quire-cli`.

## Scope note: wasm feature gate (change beyond `rust.rs` itself)

`tree-sitter-rust`'s C build script cannot cross-compile to `wasm32-unknown-unknown` in this environment (no `clang`, only `gcc`/`cc`), so adding `quire-code-parse` as an unconditional dependency broke `check-wasm` — verified by reproducing a clean `check-wasm` pass on pristine `main` before this change, then reproducing the identical break with the naive unconditional-dependency version of this port. Fixed by gating the new dependency behind a `rust-symbols` optional feature (on by default; deliberately **not** added to the existing `wasm` feature's dependency set), mirroring the pre-existing `resolve-file`-vs-`wasm` gating pattern already in `Cargo.toml`. This required touching `src/symbols/mod.rs` beyond its doc comment: `#[cfg(feature = "rust-symbols")]` on `pub mod rust;`, and the `SourceLanguage::Rust` match arm in `extend_with_file` split into a real-extraction arm (feature on) and a fixed-`Err`-string arm (feature off). Verified no other file references `symbols::rust` directly (`grep -rn "symbols::rust\b" src/ examples/`). Both `check-wasm` legs pass green with this gate in place.
