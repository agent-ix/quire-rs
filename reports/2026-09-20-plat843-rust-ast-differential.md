# PLAT-843: Rust symbol-scanner AST-port differential, against PLAT-840's baseline

- Date: `2026-09-20`
- Baseline: `reports/2026-09-20-plat840-rust-scanner-baseline.md` / `.json`, corrected version (PLAT-840, PR #469) — `rust_symbols_total_all_repos: 31,781`, `abandoned_files_total_all_repos: 3`, `abandoned_files_total_rust_only: 0`, `unbacked_rows_total_all_repos: 2,408`, `status_lies_total_all_repos: 105`. The superseded first-version numbers (31,798 / 4 / 1) are **not** used here.
- Old engine: `quire-rs` pristine `main` at `08d39ea2ce50db35812f836df65cac18757daa4b` (the sha PLAT-840 itself recorded as the `quire-rs` target row) — the exact pre-PLAT-843 `src/symbols/rust.rs` line-structural scanner.
- New engine: this PR's branch (`feat/plat843-rust-ast`), tree-sitter port via `quire-code-parse` pinned at `57b83ba00431914060297bf94fcee31549c9b68b`.
- Target trees held constant: `quire-rs`/`quire-code-rs` measured against their own already-pinned local checkouts (`/home/peter/dev/quire-rs` at `08d39ea2c...`, `/home/peter/dev/quire-code-rs` at `e1b7fc303...` — both verified unchanged, still exactly PLAT-840's recorded shas); `quire-contract-ir`, `quire-protocol`, `filament-ide-rs`, `ecaz` measured against disposable clones pinned at PLAT-840's exact recorded shas. Both engines ran over byte-identical source trees per repo — every delta below is engine-only.
- Harness: `examples/plat840_rust_baseline_sweep.rs` (unmodified), run once with the new-engine binary over all six pinned trees, for the headline table. Per-symbol enumeration (`reports/2026-09-20-plat843-symbol-diff.tsv`) used `examples/plat843_audit_list.rs` (new, kept for reproducibility — prints one `path\tqualified_name\tkind\tleading_line` line per Rust symbol for a given root), run once per repo against an old-engine build (`08d39ea2c...`) and once against this branch, then identity-matched on `(path, qualified_name)`.

## Headline

| Metric | Baseline (PLAT-840, corrected) | This PR (new engine) | Δ |
|---|---:|---:|---:|
| `rust_symbols_total_all_repos` | 31,781 | 31,695 | **−86** |
| `abandoned_files_total_all_repos` | 3 | 3 | **0** |
| `abandoned_files_total_rust_only` | 0 | 0 | **0** |
| `unbacked_rows_total_all_repos` | 2,408 | 2,411 | **+3** |
| `status_lies_total_all_repos` | 105 | 105 | **0** |

Every non-zero delta is named below by direct cause, verified with per-symbol diffs (a tool comparing old-engine and new-engine `(path, qualified_name, kind)` triples over byte-identical source), not inferred from the aggregate.

**Note on shas:** all six repos are pinned to identical shas on both sides of this differential (old engine and new engine each ran against the same tree, per repo — see the per-repo table above). The harness's `on_main` check is therefore benign for this comparison, but it is worth stating explicitly that `ecaz`'s `main` has moved on since PLAT-840's baseline commit (`2d7fc88aae1897fcf786f78bff75320cf8950d8c`); every `ecaz` figure in this report, like every other repo's, is as of that pinned commit, not today's `main`.

## `rust_symbols_total`: −86, exhaustively enumerated, three named causes

**Every one of the 260 changed identities across the four moved repos is enumerated by hand, not characterized in aggregate.** The full per-symbol list — `repo`, `side` (`old_only`/`new_only`/`old_kind_changed`/`new_kind_changed`), `path`, `qualified_name`, `kind`, `leading_line` — is checked in at `reports/2026-09-20-plat843-symbol-diff.tsv` (275 lines including header). It was produced by identity-matching (`path` + `qualified_name` as key, ignoring `kind`/`leading_line`) between an old-engine run and a new-engine run, each over the same pinned tree per repo, so a reviewer can open any single row and check it against the source directly.

Matching by identity (not raw line diff) separates three distinct effects that a plain line-by-line diff conflates:

| Repo | True losses (macro_rules! template) | Qualification-repair renames (net 0 count) | `pub(in path)` recoveries | `Function`→`TestFunction` reclassifications (net 0 count) | Net Δ |
|---|---:|---:|---:|---:|---:|
| quire-contract-ir | 31 | 2 | 1 | 0 | **−30** |
| quire-protocol | 34 | 0 | 0 | 14 | **−34** |
| filament-ide-rs | 19 | 39 | 0 | 0 | **−19** |
| ecaz | 5 | 36 | 2 | 0 | **−3** |
| **total** | **89** | 77 | **3** | 14 | **−86** |

31+2+1=34≠33 for cir's raw old-only count (33) because one rename pair (`Wire`→`Clause::Wire`, `deserialize`→`Clause::deserialize`) accounts for 2 of the 33 old-only rows and both of cir's 2 non-`admit_value` new-only rows; the table's "True losses" column (31) is the remainder. Every count above is machine-verified against `2026-09-20-plat843-symbol-diff.tsv`, not eyeballed.

quire-rs and quire-code-rs are **byte-identical**: the new engine's `(path, qualified_name, kind)` list matches the old engine's exactly, symbol for symbol, over the same pinned tree (3,403 and 429 respectively, Δ0, 0 removed, 0 added). quire-rs's own tree was the proving ground for the port — see "The proptest! regression" below.

### Cause 1: `macro_rules!` definition-template text — 89 true losses, all four repos

A `macro_rules! foo { (...) => { struct X { ... } impl X { ... } } }` **definition**'s expansion template routinely contains literal `struct`/`impl`/`fn`/`enum` text — this is the macro's own output pattern, not compiled code at that source location. The old line-structural scanner matched this text as if it were real top-level declarations, because it has no concept of "inside a macro_rules! template body." Tree-sitter correctly parses a `macro_rules!` definition as its own grammar construct (pattern/template token trees) and does not emit `function_item`/`struct_item`/etc. nodes for template contents — so the new engine, correctly, does not mint symbols there.

Confirmed by direct source inspection, one `macro_rules!` definition per removed cluster — every row tagged `old_only` in `2026-09-20-plat843-symbol-diff.tsv` for these repos is one of these clusters, none unaccounted for:

- `quire-contract-ir` (31 rows): `crates/quire-contract-model/src/identity.rs` (`macro_rules! diagnostic_codes`, `identifier_type`, `positive_revision`), `crates/quire-contract-model/src/output_mapping.rs` (`macro_rules! raw_digest_type`, `mapping_error_codes`, `source_selection_type`, `qualified_code_type`), `src/temporal/request.rs` (`macro_rules! observation_artifact`) — the removed `DiagnosticCode`/`MappingRequestErrorCode` containers and their `as_str`/`deserialize`/`fmt`/`new`/`digest`/etc. template methods were never real declarations at these source locations.
- `quire-protocol` (34 rows, its entire delta): `src/ids.rs:273` `macro_rules! identities` (`IdentityKind` enum + 11 methods, template body), and the same pattern in `src/repro.rs`, `src/claims/mod.rs`, `src/closure.rs`, `src/producer_contracts.rs` — every one of the 34 `old_only` rows for this repo is inside one of these five files, matching one of these five macro definitions.
- `filament-ide-rs` (19 rows): `crates/filament-core/src/identity.rs` (1 `macro_rules!`, 6 removed methods), `crates/filament-markups/tests/source_integrity.rs` (2 `macro_rules!`, 12 removed methods, plus its own container symbol).
- `ecaz` (5 rows): `src/am/ec_distann/lifecycle_state.rs:14` `macro_rules! lifecycle_state` (`allows`/`as_str`×2/`fmt`/`parse`).

This is the same class of defect CR-040 already fixed for raw-string/lifetime false positives — text that is lexically declaration-shaped but not real code at that location — now closed for `macro_rules!` template bodies too. It is **in scope** under the ticket's own rule ("free from parsing correctly"): removing it is a direct, correct consequence of parsing with a real grammar rather than text matching, not a change to what a symbol *is*.

### Cause 2: method/local-type qualification repair — 77 rename-pairs, net 0 count (quire-contract-ir, filament-ide-rs, ecaz)

The old scanner's brace-counting container stack could desync on a specific `impl` block and emit a **bare, unqualified** name (`backup`, `new`, `Wire`) instead of `Type::name`. Identity-matched by simple name (the qualified name's last `::`-segment) within the same file, every one of these pairs a bare `old_only` row with a `new_only` row of the same simple name in the same repo — proven, not assumed, by the matching itself. The new engine reads the `impl_item`'s own `type` field directly from the AST, so every method (and every locally-scoped nested type — see the `Wire`/`Clause` example below) is qualified under its `impl` target unconditionally, matching this ticket's own stated identity rule ("methods qualify under the for-target type").

- `quire-contract-ir` (1 pair, 2 rows: `Wire`→`Clause::Wire`, `deserialize`→`Clause::deserialize`): `crates/quire-contract-model/src/identity.rs` defines `struct Wire { ... }` locally inside `fn deserialize` inside **five** separate `impl Deserialize for X { ... }` blocks (lines 398, 497, 593, 770, 952). Four of the five (`SchemaVersion::Wire`, `SourceLocation::Wire`, `SourceSpan::Wire`, `DependencyIdentity::Wire`) are qualified identically by both engines — no diff. Exactly one, the `impl Deserialize for Clause` block at line 947–952, desynced the old scanner's container tracking; the new engine correctly qualifies it `Clause::Wire`/`Clause::deserialize`.
- `filament-ide-rs` (39 pairs): e.g. `crates/filament-sidecar/src/host.rs` — 24 methods, `backup`→`RealSidecarHost::backup`, etc.
- `ecaz` (36 pairs): e.g. `src/am/ec_hnsw/search.rs` — 27 methods across `BeamSearch`/`VisibleFrontier`.

Net symbol count is unaffected (1 removed : 1 added per pair), but the **qualified name, and therefore the symbol id, changes** — named here because it is an identity change, even though it nets to zero in the headline count. In scope for the same reason as Cause 1: it corrects the scanner's own stated, ticket-mandated behavior, it does not redefine what "qualify under the for-target type" means.

### Cause 3: `pub(in path)` visibility miss — 3 genuine recoveries (quire-contract-ir, ecaz)

`pub(in crate::some::path) fn name(...)` is a real, distinct visibility-modifier syntax (path-restricted `pub`) that the old scanner's declaration-detection prefix/keyword matching did not recognize — these three functions were **entirely missing** from the old scanner's output, not misqualified. The new engine parses `function_item` regardless of its visibility modifier, since tree-sitter's grammar treats visibility as a separate optional field, not part of declaration recognition.

- `quire-contract-ir`: `crates/quire-contract-model/src/checked_package/v2/mod.rs:642` — `pub(in crate::checked_package) fn admit_value(...)`. This repo's only other `pub(in ...)` occurrence is elsewhere and had no diff; confirmed the whole repo has exactly one `pub(in `, matching exactly the one recovery (`grep -rc 'pub(in ' crates/ src/` → 1).
- `ecaz`: `src/am/ec_spire/coordinator/snapshots.rs:23,27` — `pub(in crate::am::ec_spire) fn root_control(...)` and `pub(in crate::am::ec_spire) fn object_tuple(...)`. Whole-repo `pub(in ` count: 2, matching exactly the two recoveries.
- `quire-protocol` and `filament-ide-rs` have zero `pub(in ` occurrences repo-wide, consistent with zero unexplained gains in either (filament-ide-rs's 39 `new_only` rows are all Cause-2 rename pairs; quire-protocol has 0 `new_only` rows at all).

Treated as in scope, the same class as the `unsafe impl` divergence below: the old scanner's declaration recognition had an accidental gap in a keyword/prefix list, not a deliberate design choice being revisited here.

### `Function`→`TestFunction` reclassification — 14 rows, quire-protocol only, proven by example

quire-protocol's remaining 14 changed rows (all `old_kind_changed`/`new_kind_changed` pairs, same `path`+`qualified_name`, `kind` differs) are not a loss at all — the identity persists on both sides, only `kind` (and `leading_line`) changes. Verified directly, not inferred, by opening the source at one instance:

`src/assessment.rs`, around `tc_011_assessment_bundle_refusals_are_typed_and_distinct` — old engine: `kind=Function, leading_line=3666` (the `fn` line itself, the annotation block entirely missed); new engine: `kind=TestFunction, leading_line=3654` (the doc comment two lines above `#[test]`). The actual source:

```
3654: /// Every whole-assessment refusal carries a source-bound envelope rather
3655: /// than using the absence of a bundle as its identity.
3656: #[test]
3657: #[trace(
3658:     "TC-011",
...
3665: )]
3666: fn tc_011_assessment_bundle_refusals_are_typed_and_distinct() {
```

The old scanner's leading-span/attribute-association tracking desynced across the nine-line, multi-line `#[trace(...)]` attribute block sitting between `#[test]` and the `fn`, losing track of `#[test]` entirely — so it classified the symbol as a plain `Function`, not a `TestFunction`, and also mis-set `leading_line` to the `fn` line itself with no annotation captured at all. This is PLAT-69's named defect (split/multi-line attribute truncation) directly causing a **kind** misclassification, not just a span error — the new engine's correct, whole-annotation-block leading-span read fixes both simultaneously. All 14 rows in the TSV follow the identical shape (a multi-line `#[trace(...)]` between `#[test]` and `fn`); this one was opened and verified as the representative case, not asserted as identical to the other 13 by pattern-matching alone.

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

**quire-protocol: `backed` +63, a real coverage gain, mechanism proven above.** The 14 `Function`→`TestFunction` reclassifications documented above (proven by example: `tc_011_assessment_bundle_refusals_are_typed_and_distinct`, PLAT-69's multi-line-attribute truncation causing a kind misclassification, not just a span error) are the direct cause: where a matrix criterion's declared reference type requires `TestFunction`-kind evidence specifically, a symbol previously misclassified as plain `Function` could never satisfy it; correctly classified, it now can. **The 63 backed rows are not individually enumerated here** — the sweep harness does not dump full `unbacked_rows`/`backed` record lists (only `unmatched_tags`/`non_binding_tags` are full dumps, per PLAT-840 F4), and reconstructing the per-row set would mean re-implementing `coverage::compute`'s internals outside the library, disproportionate for a report given the mechanism is already proven by direct source inspection, not inferred from the aggregate. 63 backed rows from 14 reclassified symbols is plausible on its face (one test function's `#[trace(...)]` can carry several trace ids, each a separate matrix row, as shown in the quoted example above — that one function alone carries 7).

**quire-rs: `+4` unbacked, `−3` backed — small, not fully decomposed.** quire-rs's own symbol list is byte-identical to the old engine (see above), so this shift is not a missing-or-extra-symbol effect; it must be a **span** effect — PLAT-69 (split-attribute leading-span truncation) and PLAT-846 (block-doc-comment joining) both change exactly which lines a symbol's `leading_line`/annotation-capture window covers, which can move a specific trace tag from "captured, binds" to "not captured, doesn't bind" or vice versa, on individual symbols, without changing the symbol's own identity. This is a direct, in-scope, expected consequence of the two named span fixes this ticket folds in. The net movement is small (net −7 rows across two buckets, out of 1,799 total quire-rs reference rows) and **not decomposed to specific rows** in this report — flagging that gap explicitly rather than asserting a row-level cause not actually checked.

## Fixes made mid-port (beyond the ticket's own named list)

1. **`proptest!`-declared tests** (quire-rs, not in the ticket's own enumerated defect list). `proptest! { #[test] fn name(pattern in strategy) { ... } }` is a macro **invocation** whose custom argument DSL tree-sitter tokenizes but never parses as `function_item` nodes. Left unhandled, this silently dropped every `proptest!`-declared test (34 on quire-rs's own tree, ~175 across all six repos before this fix) — including real, trace-tagged tests (`TC-890`..`TC-896`, `TC-819`, etc.). Fixed by scanning `proptest!`/`proptest::proptest!` invocation token trees for `fn NAME(...) { ... }` sequences (`collect_proptest_tests`/`scan_token_tree_for_fns`/`flat_leading_span_and_test` in `src/symbols/rust.rs`), with the same attribute/comment-aware leading-span logic used for top-level declarations. Verified fix: quire-rs's own tree went from a −175-symbol regression across the sweep to the byte-identical, Δ0 result shown above.
2. **`unsafe impl` container scoping** (no repo shows this in the six measured trees' populations, but it is a real, deliberate divergence from the pre-port scanner, named here per the ticket's instruction to flag anything touching symbol identity). The old scanner's `declaration()` check required a literal `trimmed.starts_with("impl")`, and its modifier-stripping keyword list did not include `"impl "` — so `unsafe impl Foo for Bar { ... }` was never recognized as an impl block at all; its methods stayed flat under whatever container was already in effect, with no `Foo` qualification. The new engine recognizes `impl_item` as its own grammar node regardless of an `unsafe` modifier, so `unsafe impl` methods are now qualified under `Foo`, consistent with every other impl block and with this ticket's own stated identity rule. Treated as in scope (a "free" fix, not a judgment call): the ticket's rule ("methods qualify under the for-target type") is unconditional, and the old behavior was an accidental gap in a keyword list, not a deliberate design choice being changed here.
3. **`pub(in path) fn` recognition** (Cause 3 above — quire-contract-ir's `admit_value`, ecaz's `root_control`/`object_tuple`, 3 genuine recoveries). Not part of the ticket's own enumerated defect list; found while enumerating the differential's `new_only` rows and root-caused by direct source inspection (see Cause 3).

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
