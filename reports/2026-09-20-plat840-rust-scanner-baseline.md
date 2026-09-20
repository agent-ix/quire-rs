# PLAT-840: Rust symbol-scanner baseline, before PLAT-843

- Date: `2026-09-20`
- Measuring engine (`quire-rs`): `08d39ea2ce50db35812f836df65cac18757daa4b` (`main`)
- Traceability module: `spec-artifacts-process` at `61a20e010d5e758f52864ad3152ccdb304a39d27` (branch `epic/264-assurance-integration` — the module repo itself is not on `main` at measurement time; every measured repo below loads this exact module commit, so the module choice is held constant and reproducible even though its own branch is not `main`)
- Harness: `examples/plat840_rust_baseline_sweep.rs`, driven through the library API (`quire_rs::symbols::extract_tree_excluding`, `quire_rs::symbols::trace::bind`, `quire_rs::coverage::compute`) — never the installed `quire` CLI
- Raw output: `reports/2026-09-20-plat840-rust-scanner-baseline.json`
- Reproducibility: run twice back-to-back over the same commits; the two JSON outputs are **SHA-256 identical** (`8bd9ff7eca29a2cb46dba39253fa19053f1faefdbc95343526f54d637a464773`), confirmed a second time after fixing a git-dir resolution bug, and a third time (see "Runs" below) — not asserted, demonstrated.

This is the before-number for `src/symbols/rust.rs`, before PLAT-843 replaces it with a tree-sitter AST implementation. Once that lands there is no way to reconstruct what the line-structural scanner used to see.

## Repos measured, and why each is a `main` baseline

Ticket PLAT-840 requires "real repos, not worktrees, not eval clones," each on its own `main`. Of the six candidate repos, only `quire-rs` and `quire-code-rs` were locally checked out on `main` at measurement time; `quire-contract-ir`, `quire-protocol`, `filament-ide-rs` and `ecaz` were each on an in-progress feature branch (`codex/serde-patch-compat`, `codex/issue-10-resource-reproduction`, `analysis/pass-2-toolset-sweep`, `task231-fixed-stride-node-blocks` respectively) — other agents' live work, not to be checked out or disturbed.

This agent's sandbox also refuses to run `git` against any checkout other than its own worktree (a hard rule, not a preference), so there was no way to read `origin/main` out of those local checkouts non-destructively either.

**Resolution:** for those four, a disposable, read-only clone of `origin/main` was made with `gh repo clone agent-ix/<repo> <dir> -- --depth 1 --branch main` into this agent's own scratchpad (never touching the local `~/dev` checkout), measured there, and deleted afterward. The harness supports this directly — each target's path is overridable via `PLAT840_PATH_<REPO>` (see the module doc comment in `plat840_rust_baseline_sweep.rs`) — so a future rerun does the same thing rather than silently measuring whatever branch happens to be checked out.

All six repos in this report are therefore genuine `main`-branch measurements. The JSON's `all_repos_on_main: true` and each repo's `on_main: true` / `head_ref: "refs/heads/main"` record this explicitly; the harness prints a loud `WARNING` to stderr for any repo it measures that is *not* on `main` (there were none in this run).

| Repo | `main` sha | Source |
|---|---|---|
| `quire-rs` | `08d39ea2ce50db35812f836df65cac18757daa4b` | local checkout (already on `main`) |
| `quire-code-rs` | `e1b7fc303f3df3c9e3673358ce990525b7ff321e` | local checkout (already on `main`) |
| `quire-contract-ir` | `dfd8bd7812a571ba1f19505374460efef4a473bc` | disposable `origin/main` clone |
| `quire-protocol` | `b31e819a2c76a45c5827ecc922342a3c20e85d9d` | disposable `origin/main` clone |
| `filament-ide-rs` | `20987c81a5f8719e6e2f38db2a6f1ec8574e2d01` | disposable `origin/main` clone |
| `ecaz` | `2d7fc88aae1897fcf786f78bff75320cf8950d8c` | disposable `origin/main` clone |

Each is Rust-heavy and declares the `spec-artifacts-process` traceability model (confirmed by `type: FR` frontmatter matching the module's archetypes in every repo's `spec/functional/**`), so `Registry::traceability()` resolved to `Some` and `coverage::compute` ran for all six — no repo was silently skipped.

## Headline: symbols found, by language and `SymbolKind`

"Symbols found" counts every `Symbol` `quire_rs::symbols::extract_tree_excluding` returned for that repo (code root only, `spec/` excluded), one row per distinct `(language, path, qualified_name, kind)` identity.

| Repo | Rust total | function | test_function | container | benchmark | fuzz_target |
|---|---:|---:|---:|---:|---:|---:|
| quire-rs | 3,414 | 1,563 | 1,200 | 636 | 6 | 9 |
| quire-code-rs | 429 | 205 | 153 | 71 | 0 | 0 |
| quire-contract-ir | 2,223 | 1,577 | 178 | 468 | 0 | 0 |
| quire-protocol | 2,524 | 1,705 | 270 | 549 | 0 | 0 |
| filament-ide-rs | 7,704 | 4,567 | 1,567 | 1,568 | 0 | 2 |
| ecaz | 15,504 | 10,467 | 2,717 | 2,273 | 35 | 12 |
| **all repos** | **31,798** | 20,084 | 6,085 | 5,565 | 41 | 23 |

Rust is isolated in every number above: `rust_symbols_total` per repo and `rust_rollup_by_kind` sum only `language == "rust"` symbols, never Python or TypeScript. quire-rs itself also extracted 664 Python symbols (141 container / 281 function / 242 test_function) and 186 TypeScript symbols (86 container / 17 function / 83 test_function); filament-ide-rs extracted 1,445 TypeScript symbols and 2 Python; ecaz extracted 195 Python; quire-contract-ir extracted 122 Python — all excluded from the Rust totals above and reported per-repo in the JSON's `symbols_by_language_kind`.

## Files abandoned (PLAT-163 brace-desync) — **the live Rust exposure is zero**

This is a headline finding, stated in words: **quire-rs has one abandoned file, and it is a deliberate test fixture, not real loss. quire-code-rs has zero. quire-contract-ir, quire-protocol and ecaz have zero.** Read together, current-day live exposure of this defect class to real Rust source is **nil** across every Rust repo measured.

| Repo | Abandoned files | `closes_no_block` | `blocks_left_open` |
|---|---:|---:|---:|
| quire-rs | 1 | 0 | 1 |
| quire-code-rs | 0 | 0 | 0 |
| quire-contract-ir | 0 | 0 | 0 |
| quire-protocol | 0 | 0 | 0 |
| filament-ide-rs | 3 | 3 | 0 |
| ecaz | 0 | 0 | 0 |
| **all repos** | **4** | **3** | **1** |

quire-rs's one abandoned file — `tests/fixtures/symbols/broken/truncated.rs`, reason `unbalanced braces: 2 block(s) left open` — is a checked-in fixture under `tests/fixtures/symbols/broken/`, whose entire purpose is to exercise the "a file failed to parse" diagnostic path itself (see `src/symbols/mod.rs`'s tests around `broken`). It is not production source, and it does not carry a `raw_string_prefix` (`rust_raw_string_prefix_present: false`), so it is not even an instance of the historical TC-804 shape.

**On the specific historical claim this baseline was asked to check:** `src/symbols/rust.rs`'s own `tc804_rust_lexing_is_string_and_lifetime_aware` doc comment records that *before* CR-040's lexer fix, **33 of quire-rs's own source files** — every one holding an `r#"…"#` raw-string fixture — were rejected as unbalanced, accounting for 78 of the repo's then-140 status lies. Measured today: **zero** real files are rejected for that reason, and the one abandoned file present contains no raw-string prefix at all. **CR-040 closed the Rust half of PLAT-163.** The 33-file / 78-status-lie figure is historical and must not be quoted as current loss — it is not.

**Consequence for PLAT-843:** the differential extraction should **not** expect a large abandoned-file recovery on the Rust side. If the AST rewrite recovers a meaningful number of previously-abandoned Rust files, that is a surprise requiring its own explanation, not confirmation of an assumption already priced into this baseline.

**PLAT-163's remaining live exposure is TypeScript**, all three instances in `filament-ide-rs`, every one the `a `}` closes no block` shape:

| File | Reason class | Cause (inspected directly) |
|---|---|---|
| `ui/tests/e2e/tc-784-786-789-project-switcher.spec.ts` | `closes_no_block` | Regex literals with escaped braces, matching PLAT-163's named shape exactly: `/openDiscoveredProject:\s*\(([^)]*)\)\s*=>[^\n]*__TAURI_INVOKE\([^)]*\{([^}]*)\}/` (line 197) and `/export type DiscoveredProjectPayload = \{([\s\S]*?)\};/` (line 211) |
| `ui/tests/native/it-019-sync-native.spec.ts` | `closes_no_block` | Same shape: `bindings.match(/export type TokenStatusDto = \{([^}]*)\}/)` (line 226) |
| `ui/tests/mocks/handlers.ts` | `closes_no_block` | Same diagnostic class, but **not** the escaped-brace-regex shape — no regex literal containing a brace was found in this file by direct inspection. Raw character-level brace counts are exactly balanced (372 `{` / 372 `}`), which rules out a genuinely malformed file and confirms the scanner is miscounting rather than reporting a real defect. The specific triggering construct was not isolated within this measurement's scope; named here as unmeasured rather than guessed at. |

Because TypeScript (`typescript.rs`) is explicitly untouched by PLAT-843 (Rust-only, per the plan), this TypeScript exposure is Phase 2 territory, not PLAT-843's.

## Where the real Rust signal is: tagged-but-unbound symbols

The abandoned-file count is not where PLAT-843 will move the needle — that count is already at its floor for Rust. The signal PLAT-843's rewrite has to explain is `tagged_not_bound`: evidence symbols whose annotation block carries an id-shaped token (`tagged`) that nonetheless minted no `verifies` relation (`bound`). `BindingCensus` in `src/symbols/trace.rs` computes `candidates`/`tagged`/`bound` per language already; this harness adds `tagged_not_bound = tagged - bound` and surfaces the one concrete pointer the library retains per language (`unmatched_example` / `unbound_example` — the library keeps one example, not the whole population, by design, so this is a sample, not an enumeration).

| Repo | Rust candidates | tagged | bound | **tagged_not_bound** | Example (one, from the library) |
|---|---:|---:|---:|---:|---|
| quire-rs | 1,215 | 887 | 867 | **20** | `corpus/cases/attachment/marker-form-mismatch/rust/input/src/lib.rs:5` `tests::covers_0` |
| quire-code-rs | 153 | 121 | 121 | **0** | — |
| quire-contract-ir | 178 | 178 | 177 | **1** | `tests/kani_shared.rs:203` `a_proved_run_with_zero_success_checks_settles_inconclusive_as_vacuous` |
| quire-protocol | 270 | 263 | 263 | **0** | — |
| filament-ide-rs | 1,569 | 1,411 | 1,387 | **24** | `crates/filament-backend/src/adapters/search.rs:859` `tests::a_document_is_not_folded_into_its_requirement` |
| ecaz | 2,764 | 137 | 21 | **116** | `fuzz/fuzz_targets/parse_text_structured.rs:6` `fuzz_target` |

**Important caveat, found while sampling rather than assumed:** quire-rs's own example lives at `corpus/cases/attachment/marker-form-mismatch/rust/input/src/lib.rs` — a path under `corpus/cases/attachment/`, which is quire-rs's own declarative test-fixture corpus, deliberately engineered to exercise the "marker-form-mismatch" diagnostic this exact metric measures (see `CLAUDE.md`'s "Every defect lands with a fixture" convention). This means **quire-rs's `tagged_not_bound = 20` is at least partly, and possibly mostly, quire-rs's own dogfooded test data, not production loss.** The harness intentionally does not filter fixtures out of the walk — the PLAT-843 differential needs the exact same file set old-vs-new to be a fair diff — but a reader comparing this number against a post-PLAT-843 rerun of quire-rs should expect the fixture-derived share of `tagged_not_bound` to move only if PLAT-843 also changes how `mod tests`/fixture files are walked, which it should not.

The other repos' examples are real source files (test modules, `tests/kani_shared.rs`, a production adapter test), consistent with genuine loss candidates rather than fixture noise — though, per the library's own design, only one example per language per repo is retained, so this is not a full census of the 20/1/24 populations.

**ecaz's 116 does not read the same way as the others.** Its overall Rust tag rate is far lower than every other repo (137 tagged of 2,764 candidates — 5%, versus 60–100% everywhere else), so the dominant story there is very likely a **binding-form convention mismatch** — ecaz's Rust tests may use a marker form the `spec-artifacts-process` module's declared grammar does not recognize — which is `trace.rs` / module-declaration territory, explicitly **out of PLAT-843's scope** (PLAT-843 only replaces `rust.rs`'s symbol extraction, not the marker-form grammar `trace.rs` matches against). Attributing ecaz's 116 to the scanner rewrite would very likely overstate what PLAT-843 fixes there.

**What was not measured:** a full per-symbol breakdown of all 20/1/24/116 `tagged_not_bound` symbols by shape (split attribute, block doc comment, span truncation, as hypothesized) was not attempted — the public API exposes per-language aggregate counts plus one example each, not a per-symbol classification, and reconstructing that would mean re-implementing `trace::bind`'s internal tag-detection logic outside the library. That is real, separate work for whoever runs the PLAT-843 differential, not something this baseline blocks on.

`non_binding_tags` (a trace id written on a symbol whose *kind* cannot bind it — a container or plain function, not the tagged-candidate population above) and `unmatched_tags` (a generic id-shaped token that matched no declared form) are reported per repo in the JSON for completeness; they are a different population from `tagged_not_bound` and not characterized further here.

| Repo | non_binding_tags | unmatched_tags |
|---|---:|---:|
| quire-rs | 47 | 351 |
| quire-code-rs | 0 | 31 |
| quire-contract-ir | 1 | 56 |
| quire-protocol | 0 | 66 |
| filament-ide-rs | 67 | 2,110 |
| ecaz | 30 | 132 |

## Coverage: unbacked rows and status lies

`unbacked_rows` = `CoverageReport::unbacked_rows.len()` (declared reference rows with no backing `verifies` relation from any source symbol). `status_lies` = `CoverageReport::status_lies.len()` (a subset of unbacked rows whose declared status claims it is verified). Both are per-repo counts over that repo's whole declared reference-row population (`totals.total`), not just Rust.

| Repo | unbacked_rows | status_lies | backed / total |
|---|---:|---:|---|
| quire-rs | 400 | 19 | 1,312 / 1,799 |
| quire-code-rs | 12 | 0 | 251 / 284 |
| quire-contract-ir | 16 | 0 | 198 / 219 |
| quire-protocol | 61 | 1 | 175 / 354 |
| filament-ide-rs | 1,073 | 55 | 1,609 / 3,701 |
| ecaz | 846 | 30 | 13 / 649 |
| **all repos** | **2,408** | **105** | — |

ecaz's `13 / 649 backed` is consistent with the low Rust tag-match rate noted above: most of ecaz's Rust tests are not binding under this module's declared marker forms at all, which depresses `backed` far below what its 15,504 Rust symbols and 2,717 Rust test functions would suggest.

## Runs (reproducibility)

Three full sweeps were run over the identical commit set (the same `main` shas throughout — the disposable clones were made once and reused):

1. First full run after fixing a `.git`-dir resolution bug for linked worktrees (`quire-rs`'s own sha was initially misread as "unknown" because a worktree's `.git` is a file, not a directory) and for the module's sub-path (`spec_artifacts_process` is a subdirectory of its own git checkout).
2. Immediate rerun, same commits: **byte-identical JSON** (`sha256sum` match).
3. After adding the `on_main`-filter fix (pointing the four non-`main` repos at disposable `origin/main` clones) and the `tagged_not_bound`/example fields: rerun twice back-to-back — **byte-identical JSON** (`8bd9ff7eca29a2cb46dba39253fa19053f1faefdbc95343526f54d637a464773`, both runs).

No wall-clock, randomness, or non-deterministic iteration order is in the harness (`BTreeMap` throughout, sorted `abandoned_files`); the library's own collections are already NFR-006-ordered.

## Reproducing this measurement

```bash
cd quire-rs
cargo build --release --example plat840_rust_baseline_sweep
# Point any repo whose local ~/dev checkout is not on `main` at a clean clone:
PLAT840_PATH_QUIRE_CONTRACT_IR=/path/to/clean/quire-contract-ir-main \
PLAT840_PATH_QUIRE_PROTOCOL=/path/to/clean/quire-protocol-main \
PLAT840_PATH_FILAMENT_IDE_RS=/path/to/clean/filament-ide-rs-main \
PLAT840_PATH_ECAZ=/path/to/clean/ecaz-main \
./target/release/examples/plat840_rust_baseline_sweep
```

The full structured output is `reports/2026-09-20-plat840-rust-scanner-baseline.json`, keyed by repo, with every number above traceable to a specific field.
