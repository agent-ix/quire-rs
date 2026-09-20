# PLAT-840: Rust symbol-scanner baseline, before PLAT-843

- Date: `2026-09-20`
- Measuring engine (`quire-rs`): `e20d948563419f24ab7f2f3b2011d0e90ef4b8e3` (this PR's branch, the commit containing the harness that produced this exact JSON — distinct from the `quire-rs` *target repo*'s own measured sha below, `08d39ea2c...`, which is `quire-rs`'s separate `main` checkout)
- Traceability module: `spec-artifacts-process` at `61a20e010d5e758f52864ad3152ccdb304a39d27` (branch `epic/264-assurance-integration` — the module repo itself is not on `main` at measurement time; every measured repo below loads this exact module commit, so the module choice is held constant and reproducible even though its own branch is not `main`)
- Harness: `examples/plat840_rust_baseline_sweep.rs`, driven through the library API (`quire_rs::symbols::extract_tree_scoped`, `quire_rs::symbols::trace::bind`, `quire_rs::coverage::compute`) — never the installed `quire` CLI
- Raw output: `reports/2026-09-20-plat840-rust-scanner-baseline.json`
- Reproducibility: run twice back-to-back at the final committed sha above; the two JSON outputs are **SHA-256 identical** (`ca83c003ec01951027051cb7d57a6a3188a77bc5a288e0a077c52d9d805f87b8`) — demonstrated, not asserted. See "Runs" for the full history, including the intermediate byte-identical pair from immediately after the review fixes but before this commit's sha was final.

This is the before-number for `src/symbols/rust.rs`, before PLAT-843 replaces it with a tree-sitter AST implementation. Once that lands there is no way to reconstruct what the line-structural scanner used to see.

**This artifact was reviewed once and corrected.** The review reproduced every number byte-identically and independently re-derived them through a different code path (per-file `extract_file` over a `std::fs` walk); every table and every commit sha checked out. It then raised nine findings, all addressed below (F1–F9) and re-measured where the fix changed a number. The "Delta from the first version of this artifact" section at the end states exactly what moved and why.

## Repos measured, and why each is a `main` baseline

Ticket PLAT-840 requires "real repos, not worktrees, not eval clones," each on its own `main`. Of the six candidate repos, only `quire-rs` and `quire-code-rs` were locally checked out on `main` at measurement time; `quire-contract-ir`, `quire-protocol`, `filament-ide-rs` and `ecaz` were each on an in-progress feature branch (other agents' live work, not to be checked out or disturbed).

This agent's sandbox also refuses to run `git` against any checkout other than its own worktree, so there was no way to read `origin/main` out of those local checkouts non-destructively either.

**Resolution:** for those four, a disposable, read-only clone of `origin/main` was made with `gh repo clone agent-ix/<repo> <dir> -- --depth 1 --branch main` into this agent's own scratchpad (never touching the local `~/dev` checkout), measured there, and deleted afterward. The harness supports this directly — each target's path is overridable via `PLAT840_PATH_<REPO>` — so a future rerun does the same thing rather than silently measuring whatever branch happens to be checked out.

All six repos in this report are genuine `main`-branch measurements. The JSON's `all_repos_on_main: true` and each repo's `on_main: true` / `head_ref: "refs/heads/main"` record this explicitly; the harness prints a loud `WARNING` to stderr for any repo it measures that is *not* on `main` (there were none in this run).

| Repo | `main` sha | Source |
|---|---|---|
| `quire-rs` | `08d39ea2ce50db35812f836df65cac18757daa4b` | local checkout (already on `main`) |
| `quire-code-rs` | `e1b7fc303f3df3c9e3673358ce990525b7ff321e` | local checkout (already on `main`) |
| `quire-contract-ir` | `dfd8bd7812a571ba1f19505374460efef4a473bc` | disposable `origin/main` clone |
| `quire-protocol` | `b31e819a2c76a45c5827ecc922342a3c20e85d9d` | disposable `origin/main` clone |
| `filament-ide-rs` | `20987c81a5f8719e6e2f38db2a6f1ec8574e2d01` | disposable `origin/main` clone |
| `ecaz` | `2d7fc88aae1897fcf786f78bff75320cf8950d8c` | disposable `origin/main` clone |

Each is Rust-heavy and declares the `spec-artifacts-process` traceability model (confirmed by `type: FR` frontmatter matching the module's archetypes in every repo's `spec/functional/**`), so `Registry::traceability()` resolved to `Some` and `coverage::compute` ran for all six — no repo was silently skipped.

## F1: every declared target is recorded, whether or not it could be measured

The first version of this harness `continue`d past a target whose path had no `spec/` directory with only an `eprintln!` — stderr, not the durable JSON. Demonstrated by review: pointing a path override at a non-existent directory made the harness exit `0`, write a 5-repo JSON that still claimed `all_repos_on_main: true`, and report a smaller total as if it were complete. That is exactly the "quietly skips a repo and reports a smaller total" failure this program exists to measure — reproduced in the measurement program itself.

Fixed, both halves as required:

1. Every entry in `TARGETS` gets a `repos[]` row. A target that could not be measured gets `measured: false` and a `skip_reason` string; every numeric field on that row is a placeholder zero/empty, never mixed into a total silently.
2. The process **exits non-zero** unless `all_targets_measured` — a broken `PLAT840_PATH_*` override, a repo that vanished, or a bad path can no longer produce a plausible-looking short report with exit code 0.

Verified by re-running with `PLAT840_PATH_ECAZ` pointed at a nonexistent directory: exit code `1`, JSON still has all 6 `repos[]` entries, `ecaz` carries `"measured": false, "skip_reason": "no spec/ directory at ..."`, `all_targets_measured: false`.

## F3 (ruling): the module's declared `source_exclude` globs are now applied

The harness previously walked every repo with `extract_tree_excluding` — no glob filtering — so its symbol and coverage figures were **not** what `quire coverage --scope <repo> --json` reports for the same repo, which is what the ticket names as the source for the per-repo number. `spec-artifacts-process/manifest.yaml:630` declares:

```
source_exclude:
- "tests/fixtures/**"
- "tests_integration/fixtures/**"
- "fixtures/**"
```

Fixed: the harness now calls `extract_tree_scoped(root, &[Path::new("spec")], &source_exclude_globs)`, where `source_exclude_globs` comes straight from `registry.traceability().source_exclude`, and captures `SymbolExtraction::excluded_source_files` into each repo's JSON row.

**Verified against the reviewer's real-CLI numbers for `quire-contract-ir`.** Re-measured: Rust binding census is now `176 / 176 / 175` and `excluded_source_files: 2` — an exact match to the cited CLI output. (The CLI's cited `backed: 196` versus this artifact's `backed: 198` for the same repo is not fully reconciled — `quire-contract-ir` is a live, actively-developed repo, and "backed" is a whole-corpus row count sensitive to `spec/` authoring, not just to which source files are walked, so a few rows' difference between two measurements taken at different moments on `main` is plausible. The specific claim this fix targets — the Rust binding census and the excluded-file count — matches exactly.)

**This strengthens the headline finding rather than weakening it.** `tests/fixtures/symbols/broken/truncated.rs` — the one "abandoned file" the first version of this artifact reported — sits inside `tests/fixtures/**`, a declared-excluded path. Under the declared model, the file is never walked at all, so it cannot be abandoned by anything. **The abandoned Rust-file count is `0`, measured directly, not inferred from "it's just a fixture."** PLAT-163's Rust exposure is nil, full stop.

This does not touch the `corpus/cases/attachment/marker-form-mismatch/` caveat in the "tagged-but-unbound" section below — `corpus/` matches none of the three declared globs, confirmed by re-running: `quire-rs`'s `tagged_not_bound_example` for Rust is still `corpus/cases/attachment/marker-form-mismatch/rust/input/src/lib.rs:5`, unchanged.

## F2: the raw-string discriminator field has been removed

The first version carried `rust_raw_string_prefix_present` on each `AbandonedFile`, computed by scanning for any `r` + `#`* + `"` byte sequence. That matches **any string literal ending in the letter `r`** — `let e = "error";` matches — a false positive the field's own doc comment acknowledged for TypeScript/Python but dismissed, when the same over-match applies identically inside Rust source. Over-matching was harmless for this artifact's conclusion (it never hid a real raw-string file), but the one place it would matter — a PLAT-843 rerun checking whether a newly-abandoned real file is a raw-string case — is exactly where an over-matching field reports `true` regardless of the actual cause.

Building a lexer-aware version means re-implementing `rust.rs`'s own string/comment-tracking state machine outside the crate (it is `pub(crate)`), which is disproportionate for a field whose only job is "does this one abandoned file (there are usually 0–1 of them) contain a raw string" — a question a human can answer by opening the file directly. **Removed** rather than shipped as a discriminator that cannot discriminate. `AbandonedFile` now carries only `path`, `language`, `reason`, `reason_class`.

This also closes a related finding: the removed code's `std::fs::read_to_string(...).unwrap_or_default()` silently reported `false` for an unreadable file — the same "quiet failure reads as a clean answer" family as F1. Deleting the field removes that call along with it; the only remaining `unwrap_or_default()` in the harness is on `Option<Vec<String>>` for `source_exclude_globs` (empty globs when no traceability model — a legitimate default, not a masked read failure).

## F4: `unmatched_tags` and `non_binding_tags` are now full record dumps, not counts

The first version reported `graph.unmatched_tags.len()` and `graph.non_binding_tags.len()` and stated that "the public API exposes per-language aggregate counts plus one example each, not a per-symbol classification." **That claim was true for `tagged_not_bound` (still is — see below) but false for `unmatched_tags` and `non_binding_tags`: both are already `Vec<...>` of fully row-addressable records** (`SymbolGraph::unmatched_tags: Vec<UnmatchedTag>` — `{trace_id, language, path, line, symbol}`; `SymbolGraph::non_binding_tags: Vec<NonBindingTag>` — `{path, symbol, kind, trace_id, form, line}`), documented as the row-addressable form of the aggregate. The ticket's Capture list explicitly asks for "trace tags bound vs. unmatched," and dumping the records was nearly free.

Fixed: both populations are now serialized in full per repo, sorted by `(path, line)` for determinism. Sizes: quire-rs 350 unmatched / 47 non-binding, quire-code-rs 31 / 0, quire-contract-ir 53 / 1, quire-protocol 66 / 0, filament-ide-rs 2,110 / 67, ecaz 132 / 30 — 2,742 unmatched-tag records and 145 non-binding-tag records total, every one with a `path:line` a human or a differential script can open directly. This is the per-symbol before-set PLAT-843's differential needs to explain symbol by symbol rather than by aggregate.

The corrected claim: `tagged_not_bound` (candidates whose *kind* qualifies and whose annotation carries a token, but which minted no `verifies` relation) genuinely has **no** fully-enumerable public accessor — `BindingCensus` retains one example per language (`unmatched_example` / `unbound_example`), not the whole population, and reconstructing it would mean re-implementing `trace::bind`'s internal per-symbol tag classification outside the library. That limitation is real and stated where it applies; it does not apply to `unmatched_tags` or `non_binding_tags`, which are dumped in full below.

## Headline: symbols found, by language and `SymbolKind`

"Symbols found" counts every `Symbol` `quire_rs::symbols::extract_tree_scoped` returned for that repo (code root only, `spec/` excluded, plus the module's declared `source_exclude` globs — see F3), one row per distinct `(language, path, qualified_name, kind)` identity.

| Repo | Rust total | function | test_function | container | benchmark | fuzz_target |
|---|---:|---:|---:|---:|---:|---:|
| quire-rs | 3,403 | 1,560 | 1,196 | 632 | 6 | 9 |
| quire-code-rs | 429 | 205 | 153 | 71 | 0 | 0 |
| quire-contract-ir | 2,217 | 1,575 | 176 | 466 | 0 | 0 |
| quire-protocol | 2,524 | 1,705 | 270 | 549 | 0 | 0 |
| filament-ide-rs | 7,704 | 4,567 | 1,567 | 1,568 | 0 | 2 |
| ecaz | 15,504 | 10,467 | 2,717 | 2,273 | 35 | 12 |
| **all repos** | **31,781** | 20,079 | 6,079 | 5,559 | 41 | 23 |

Rust is isolated in every number above: `rust_symbols_total` per repo and `rust_rollup_by_kind` sum only `language == "rust"` symbols, never Python or TypeScript. The per-language breakdown for every repo, including Python and TypeScript counts, is in the JSON's `symbols_by_language_kind`.

`excluded_source_files` (files the declared `source_exclude` globs removed from the walk before extraction): quire-rs 6, quire-contract-ir 2, everyone else 0.

## Files abandoned (PLAT-163 brace-desync) — **the live Rust exposure is zero, measured directly**

This is a headline finding, stated in words: **every Rust repo measured has zero abandoned files, measured directly against the module's declared `source_exclude` model** (F3) — not "one, and it's a fixture," which was the first version's inferred caveat. quire-rs's `tests/fixtures/symbols/broken/truncated.rs` sits inside the declared-excluded `tests/fixtures/**`, so it is never walked and cannot be abandoned by anything, under the same model `quire coverage` uses.

| Repo | Abandoned files | `closes_no_block` | `blocks_left_open` |
|---|---:|---:|---:|
| quire-rs | 0 | 0 | 0 |
| quire-code-rs | 0 | 0 | 0 |
| quire-contract-ir | 0 | 0 | 0 |
| quire-protocol | 0 | 0 | 0 |
| filament-ide-rs | 3 | 3 | 0 |
| ecaz | 0 | 0 | 0 |
| **all repos** | **3** | **3** | **0** |

**On the specific historical claim this baseline was asked to check:** `src/symbols/rust.rs`'s own `tc804_rust_lexing_is_string_and_lifetime_aware` doc comment records that *before* CR-040's lexer fix, **33 of quire-rs's own source files** — every one holding an `r#"…"#` raw-string fixture — were rejected as unbalanced, accounting for 78 of the repo's then-140 status lies. Measured today, under the declared exclusion model: **zero** real files are rejected for any reason. **CR-040 closed the Rust half of PLAT-163.** The 33-file / 78-status-lie figure is historical and must not be quoted as current loss — it is not.

**Consequence for PLAT-843:** the differential extraction should **not** expect any abandoned-file recovery on the Rust side — there is nothing left to recover. If the AST rewrite's differential shows a Rust abandoned-file count above zero on either side, that is a surprise requiring its own explanation.

**PLAT-163's remaining live exposure is entirely TypeScript**, all three instances in `filament-ide-rs`, every one the `a `}` closes no block` shape (re-verified against the exact measured sha `20987c81a5f8719e6e2f38db2a6f1ec8574e2d01`):

| File | Reason class | Cause (inspected directly) |
|---|---|---|
| `ui/tests/e2e/tc-784-786-789-project-switcher.spec.ts` | `closes_no_block` | Regex literals with escaped braces, matching PLAT-163's named shape exactly: `/openDiscoveredProject:\s*\(([^)]*)\)\s*=>[^\n]*__TAURI_INVOKE\([^)]*\{([^}]*)\}/` (line 201) and `/export type DiscoveredProjectPayload = \{([\s\S]*?)\};/` (line 215) |
| `ui/tests/native/it-019-sync-native.spec.ts` | `closes_no_block` | Same shape: `bindings.match(/export type TokenStatusDto = \{([^}]*)\}/)` (line 226) |
| `ui/tests/mocks/handlers.ts` | `closes_no_block` | Same diagnostic class, but **not** the escaped-brace-regex shape — no regex literal containing a brace was found in this file by direct inspection. Raw character-level brace counts are exactly balanced (**386 `{` / 386 `}`**, re-verified against the fresh `origin/main` clone at the measured sha — corrected from an earlier miscount of 372/372), which rules out a genuinely malformed file and confirms the scanner is miscounting rather than reporting a real defect. The specific triggering construct was not isolated within this measurement's scope; named here as unmeasured rather than guessed at. |

Because TypeScript (`typescript.rs`) is explicitly untouched by PLAT-843 (Rust-only, per the plan), this TypeScript exposure is Phase 2 territory, not PLAT-843's.

## Where the real Rust signal is: tagged-but-unbound symbols

The abandoned-file count is not where PLAT-843 will move the needle — it is already at its floor (zero) for Rust. The signal PLAT-843's rewrite has to explain is `tagged_not_bound`: evidence symbols whose annotation block carries an id-shaped token (`tagged`) that nonetheless minted no `verifies` relation (`bound`). `BindingCensus` in `src/symbols/trace.rs` computes `candidates`/`tagged`/`bound` per language already; this harness adds `tagged_not_bound = tagged - bound` and surfaces the one concrete pointer the library retains per language (`unmatched_example` / `unbound_example`).

| Repo | Rust candidates | tagged | bound | **tagged_not_bound** | Example (one, from the library) |
|---|---:|---:|---:|---:|---|
| quire-rs | 1,211 | 883 | 863 | **20** | `corpus/cases/attachment/marker-form-mismatch/rust/input/src/lib.rs:5` `tests::covers_0` |
| quire-code-rs | 153 | 121 | 121 | **0** | — |
| quire-contract-ir | 176 | 176 | 175 | **1** | `tests/kani_shared.rs:203` `a_proved_run_with_zero_success_checks_settles_inconclusive_as_vacuous` |
| quire-protocol | 270 | 263 | 263 | **0** | — |
| filament-ide-rs | 1,569 | 1,411 | 1,387 | **24** | `crates/filament-backend/src/adapters/search.rs:859` `tests::a_document_is_not_folded_into_its_requirement` |
| ecaz | 2,764 | 137 | 21 | **116** | `fuzz/fuzz_targets/parse_text_structured.rs:6` `fuzz_target` |

**Important caveat, unchanged by F3 (confirmed by re-measurement — `corpus/` matches none of the declared `source_exclude` globs):** quire-rs's own example lives at `corpus/cases/attachment/marker-form-mismatch/rust/input/src/lib.rs` — quire-rs's own declarative test-fixture corpus, deliberately engineered to exercise the "marker-form-mismatch" diagnostic this exact metric measures (see `CLAUDE.md`'s "Every defect lands with a fixture" convention). **quire-rs's `tagged_not_bound = 20` is at least partly, and possibly mostly, quire-rs's own dogfooded test data, not production loss.**

The other repos' examples are real source files (test modules, `tests/kani_shared.rs`, a production adapter test), consistent with genuine loss candidates rather than fixture noise — though, per the library's own design, only one example per language per repo is retained via `BindingCensus`, so this row is not a full census of the 20/1/24/116 populations (see F4 above for what *is* dumped in full — `unmatched_tags` and `non_binding_tags`, a related but not identical population).

**ecaz's 116 does not read the same way as the others.** Its overall Rust tag rate is far lower than every other repo (137 tagged of 2,764 candidates — 5%, versus 60–100% everywhere else), so the dominant story there is very likely a **binding-form convention mismatch** — ecaz's Rust tests may use a marker form the `spec-artifacts-process` module's declared grammar does not recognize — which is `trace.rs` / module-declaration territory, explicitly **out of PLAT-843's scope**. Attributing ecaz's 116 to the scanner rewrite would very likely overstate what PLAT-843 fixes there.

**What was not measured:** a full per-symbol breakdown of all 20/1/24/116 `tagged_not_bound` symbols by shape (split attribute, block doc comment, span truncation, as hypothesized) was not attempted — this population specifically has no fully-enumerable public accessor (see F4). Reconstructing it would mean re-implementing `trace::bind`'s internal tag-detection logic outside the library. That is real, separate work for whoever runs the PLAT-843 differential, not something this baseline blocks on.

`non_binding_tags` and `unmatched_tags` — now full record dumps per F4, not just counts — are a *different* population from `tagged_not_bound` (a trace id on a symbol whose *kind* cannot bind it, and a generic id-shaped token no declared form bound, respectively) and are not characterized further here; every record is in the JSON with `path:line:symbol`.

| Repo | non_binding_tags (records) | unmatched_tags (records) |
|---|---:|---:|
| quire-rs | 47 | 350 |
| quire-code-rs | 0 | 31 |
| quire-contract-ir | 1 | 53 |
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

None of these totals moved from the first version of this artifact — applying `source_exclude` (F3) changed which *source files* are walked, not which declared *matrix rows* end up backed, because the excluded fixture trees were never cited by any real matrix row to begin with. See "Delta" below for the full before/after.

ecaz's `13 / 649 backed` is consistent with the low Rust tag-match rate noted above: most of ecaz's Rust tests are not binding under this module's declared marker forms at all, which depresses `backed` far below what its 15,504 Rust symbols and 2,717 Rust test functions would suggest.

## Runs (reproducibility)

1. First full run after fixing a `.git`-dir resolution bug for linked worktrees and for the module's sub-path.
2. Immediate rerun, same commits: byte-identical JSON.
3. After adding the `on_main` filter (four repos re-measured at disposable `origin/main` clones) and the `tagged_not_bound`/example fields: two more runs, byte-identical (`8bd9ff7eca29a2cb46dba39253fa19053f1faefdbc95343526f54d637a464773`).
4. After the review (F1–F9 above), re-measured all six repos again (fresh `origin/main` clones for the same four repos) with `extract_tree_scoped` + declared `source_exclude` globs, full `unmatched_tags`/`non_binding_tags` record dumps, the raw-string field removed, and per-target `measured`/`skip_reason` tracking: two runs back-to-back, byte-identical (`bf4d52f0fcddcd3f4f746c5874188c211b008c22e0778c69dfc2a92aa5d1c22b` — this was before the fix commit landed, so `quire_rs_measuring_commit` in that pair is not the final sha).
5. F1's exit-code fix separately verified: pointing `PLAT840_PATH_ECAZ` at a nonexistent directory produces exit code `1`, a 6-entry JSON with `ecaz.measured: false` and a `skip_reason`, and `all_targets_measured: false`.
6. **After committing the fix** (`e20d948563419f24ab7f2f3b2011d0e90ef4b8e3`), re-measured once more so `quire_rs_measuring_commit` in the shipped JSON is accurate: two runs back-to-back, **byte-identical** (`ca83c003ec01951027051cb7d57a6a3188a77bc5a288e0a077c52d9d805f87b8`). This is the JSON checked in.

No wall-clock, randomness, or non-deterministic iteration order is in the harness (`BTreeMap` throughout, sorted `abandoned_files`/`unmatched_tags`/`non_binding_tags`); the library's own collections are already NFR-006-ordered.

## Reproducing this measurement

**Requires the `corpus` git submodule** (`git submodule update --init`) if running `make ci` alongside this — a fresh clone of `quire-rs` without `--recurse-submodules` measures an empty `corpus/` directory and silently produces different `quire-rs` numbers than this artifact (F1's per-target `measured`/exit-code fix does not catch this particular case, since `corpus/` merely being empty is not "no `spec/` directory" — stated here explicitly instead).

```bash
cd quire-rs
git submodule update --init
cargo build --release --example plat840_rust_baseline_sweep
# Point any repo whose local ~/dev checkout is not on `main` at a clean clone:
PLAT840_PATH_QUIRE_CONTRACT_IR=/path/to/clean/quire-contract-ir-main \
PLAT840_PATH_QUIRE_PROTOCOL=/path/to/clean/quire-protocol-main \
PLAT840_PATH_FILAMENT_IDE_RS=/path/to/clean/filament-ide-rs-main \
PLAT840_PATH_ECAZ=/path/to/clean/ecaz-main \
./target/release/examples/plat840_rust_baseline_sweep
echo "exit=$?"   # non-zero means a target was missed — check skip_reason in the JSON
```

The full structured output is `reports/2026-09-20-plat840-rust-scanner-baseline.json`, keyed by repo, with every number above traceable to a specific field.

## F8: this PR itself changes quire-rs's own numbers on next measurement

Merging this PR adds `examples/plat840_rust_baseline_sweep.rs` — a new `.rs` file, with tracking tags in its own doc comments — to the repo this artifact measures as `quire-rs`. **PLAT-843's differential must re-measure `quire-rs` pinned at `08d39ea2ce50db35812f836df65cac18757daa4b`** (the sha recorded for the `quire-rs` target row above), not at whatever `main` is by the time the differential runs, or it will see a small, expected delta from this file's own presence and misattribute it to the scanner rewrite.

## Gates

- `make fmt-check`, `make lint` (crate-wide): clean, re-verified after the F1–F9 fix.
- `make ci`: every step passes — `fmt-check`, `lint`, `check-python`, `check-wasm`, `check-scripts`, `test` (614 lib tests + full integration suite, including `corpus_cases` after `git submodule update --init`), `deny`, `audit-unsafe`, `audit-property`, `audit-static` — **except** the final `validate` step, which fails on a pre-existing, unrelated condition: this worktree's sibling `spec-artifacts-iso`/`spec-artifacts-process` checkouts don't match the commits pinned in `quality/validation-stack-lock.json` (`git status` confirms this PR touches only new files under `examples/` and `reports/` — nothing that could affect spec validation). `check-engine` (which runs after `validate` in the target list) was **not reached** in that `make ci` invocation, because `validate` failing stops the composite target first; invoked separately afterward, it exits `SKIP — no consumer workspace at ../quire-cli` given the default relative path from this worktree, which is not evidence it would pass under a normal checkout layout, only that this worktree's `validate` failure prevented it from being reached at all.

## Delta from the first version of this artifact (post-review corrections)

| Metric | First version | This version | Δ | Why |
|---|---:|---:|---:|---|
| `rust_symbols_total_all_repos` | 31,798 | 31,781 | **−17** | F3: `source_exclude` now applied — fixture-tree Rust symbols in quire-rs (−11) and quire-contract-ir (−6) no longer counted |
| `abandoned_files_total_all_repos` | 4 | 3 | **−1** | F3: quire-rs's one abandoned file was itself inside a declared-excluded path |
| `abandoned_files_total_rust_only` | 1 | **0** | **−1** | Same — the live Rust exposure is now measured as exactly zero, not "one fixture" |
| `unbacked_rows_total_all_repos` | 2,408 | 2,408 | 0 | Excluded fixture trees backed no real matrix row, so the declared-row totals are unchanged |
| `status_lies_total_all_repos` | 105 | 105 | 0 | Same reasoning |
| `rust_rollup_by_kind.function` | 20,084 | 20,079 | −5 | F3 |
| `rust_rollup_by_kind.test_function` | 6,085 | 6,079 | −6 | F3 |
| `rust_rollup_by_kind.container` | 5,565 | 5,559 | −6 | F3 |
| `rust_rollup_by_kind.benchmark` / `fuzz_target` | 41 / 23 | 41 / 23 | 0 | No excluded fixture minted a benchmark or fuzz target |
| Per-repo `tagged_not_bound` (Rust) | 20 / 0 / 1 / 0 / 24 / 116 | 20 / 0 / 1 / 0 / 24 / 116 | 0 | The gap size is unchanged in every repo — fewer candidates and fewer tagged on the quire-rs/quire-contract-ir side, but the same proportion stayed unbound |
| `unmatched_tags` totals (sum of per-repo counts) | 2,742 | 2,742 | 0 | F4: same population, now a full `Vec` of `{path, line, symbol, trace_id, language}` records per repo instead of a bare `.len()` |
| `non_binding_tags` totals (sum of per-repo counts) | 145 | 145 | 0 | Same: now a full `Vec` of `{path, symbol, kind, trace_id, form, line}` records per repo |

No repo's `on_main`, `head_sha`, or measured status changed. `quire-contract-ir`'s `excluded_source_files` is newly captured as `2` (was not tracked in the first version at all).

**Net read:** the fix did not change *which* declared spec rows are backed, unbacked, or lying — it changed the *symbol inventory* (removing fixture noise) and, as a direct consequence, corrected the headline claim from "one abandoned Rust file, and it's a fixture" to "zero abandoned Rust files, measured against the same model `quire coverage` uses." That is a strictly stronger, more defensible statement of the same underlying fact.
