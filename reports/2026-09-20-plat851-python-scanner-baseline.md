# PLAT-851: Python symbol-scanner baseline, before its own AST port

- Date: `2026-09-20`
- Measuring engine (`quire-rs`): `81039d2c4e5f121bd9766340c8a002a1e01c8093` (this PR's branch — the commit containing the extended harness that produced this exact JSON)
- Traceability module: `spec-artifacts-process` at `61a20e010d5e758f52864ad3152ccdb304a39d27` (same commit PLAT-840 used, held constant for comparability)
- Harness: `examples/plat840_rust_baseline_sweep.rs`, driven through the library API (`quire_rs::symbols::extract_tree_scoped`, `quire_rs::symbols::trace::bind`, `quire_rs::coverage::compute`) — never the installed `quire` CLI. PLAT-851 extended it with generic, language-keyed rollup fields (`rollup_by_language_kind`, `symbols_total_all_repos_by_language`, `abandoned_files_total_by_language`); the walk itself is unchanged — `extract_tree_scoped` was never Rust-scoped, so Python and TypeScript figures were already being computed, just never surfaced as a rollup.
- Raw output: `reports/2026-09-20-plat851-language-scanner-baseline.json` — **shared with the TypeScript report below.** One sweep run measures every language present in a repo in a single pass, so there is one JSON for both baselines rather than two copies of the same data.
- Reproducibility: two runs back-to-back at the commit above, SHA-256 identical: `422489ef07c7b7e055de94b0680e352358886e03267780ab445306eb77dbb47b`.

This is the before-number for `src/symbols/python.rs`, before its own tree-sitter port over the boundary crate PLAT-851 widened. There was no Python (or TypeScript) pre-port baseline before this — PLAT-840 measured Rust only. Without this, "every delta has a named cause" is unenforceable for the Python port, and the port becomes unprovable the way PLAT-843's Rust port was proven.

## Repos measured, and why each is a `main` baseline

Of the six candidate repos, `quire-rs` and `quire-code-rs` were already checked out on `main` locally. `quire-contract-ir`, `quire-protocol`, `filament-ide-rs` and `ecaz` were each sitting on another agent's in-progress feature branch in their local `~/dev` checkout — not to be checked out or disturbed — so a disposable, read-only clone of `origin/main` was made for each with `gh repo clone agent-ix/<repo> <dir> -- --depth 1 --branch main` into scratch space, measured there, and deleted afterward. This is the exact resolution PLAT-840 documented when it hit the same situation, and the harness supports it the same way: each target's path is overridable via `PLAT840_PATH_<REPO>`.

All six rows are genuine `main`-branch measurements (`all_repos_on_main: true`, every repo's `on_main: true`).

| Repo | `main` sha measured | Source |
|---|---|---|
| `quire-rs` | `08d39ea2ce50db35812f836df65cac18757daa4b` | local checkout |
| `quire-code-rs` | `e1b7fc303f3df3c9e3673358ce990525b7ff321e` | local checkout |
| `quire-contract-ir` | `6b7eb8996226118c47719e552f93f2b93f890ab1` | disposable `origin/main` clone |
| `quire-protocol` | `0065460c69eb90479000024faa0fb7804cf3ebfc` | disposable `origin/main` clone |
| `filament-ide-rs` | `37c44d907ad419472faca240bd02a0fae9add7c0` | disposable `origin/main` clone |
| `ecaz` | `d8c72f44d814e486b8cf718796aace6241279c9a` | disposable `origin/main` clone |

**These are not the same shas PLAT-840 pinned for the four cloned repos.** Each repo's `main` moved between 2026-09-20's PLAT-840 measurement and this one — later the same day. `quire-rs` and `quire-code-rs` are unchanged (identical shas both times, and identical local checkouts — neither was disturbed). Do not diff this report's incidental Rust figures against PLAT-840's for `quire-contract-ir`/`quire-protocol`/`filament-ide-rs`/`ecaz` expecting a clean comparison: the trees differ, in the same way PLAT-843's differential and PLAT-845's blast radius both had to caveat. Separately, this run's Rust totals also differ from PLAT-840's for an unrelated, expected reason — this measurement runs the post-PLAT-843 tree-sitter engine (current `main`), not the pre-port line-structural scanner PLAT-840 measured, so a Rust-vs-Rust comparison to PLAT-840 is not meaningful here regardless. The Rust column below is incidental context, not this report's subject.

## ecaz's weight in every corpus-wide number

Peter, 2026-09-20: *"ecaz might not be good to test on since it has poor linting hygiene."* Not dropped — messy code is a legitimate stress case for an extractor, and dropping a repo because its numbers are inconvenient is the silent-filter failure this campaign exists to end. But it does not get to silently dominate the aggregates either: ecaz is roughly half this corpus by Rust volume, was excluded from earlier sweeps during development, and PLAT-867 separately measured it binding only 15.3% of its trace tags (vs 97.7–100% everywhere else) and tagging only 5% of candidate symbols — known-poor hygiene, not a one-off. A corpus-wide total that includes it is therefore more than half a statement about one repo.

Every rollup below is given twice: **all repos**, and **all repos excluding ecaz**. Checked, not assumed: **for Python, ecaz is not negligible** — it contributes 195 of 975 python symbols, **20.0%** of the total.

## Headline: Python symbols found, by `SymbolKind`

"Symbols found" counts every `Symbol` `quire_rs::symbols::extract_tree_scoped` returned for that repo (code root only, `spec/` excluded, plus the module's declared `source_exclude` globs applied — same model `quire coverage` uses), one row per distinct `(language, path, qualified_name, kind)` identity.

| Repo | Python total | function | test_function | container |
|---|---:|---:|---:|---:|
| quire-rs | 656 | 278 | 240 | 138 |
| quire-code-rs | 0 | 0 | 0 | 0 |
| quire-contract-ir | 122 | 78 | 29 | 15 |
| quire-protocol | 0 | 0 | 0 | 0 |
| filament-ide-rs | 2 | 1 | 0 | 1 |
| ecaz | 195 | 155 | 7 | 33 |
| **all repos** | **975** | 512 | 276 | 187 |
| **all repos, excl. ecaz** | **780** | 357 | 269 | 154 |

`quire-code-rs` and `quire-protocol` are Rust-only — 0 is a real measurement, not a gap. `quire-rs`'s own 656 breaks down, by path prefix: `corpus/cases/**` 195 (30%, dogfooded test data exercising the trace-binding grammar in every language, deliberately, per this repo's own `CLAUDE.md` convention), `scripts/tests/**` 166 (25%, the local tooling test suite), `tests/python/**` 54 (8%, the PyO3 binding suite), the remainder spread across `scripts/*.py` and `corpus/*.py` individually. `corpus/` matches none of the three declared `source_exclude` globs, so it is measured in full, same as PLAT-840 found for its own Rust corpus fixtures.

## Files abandoned (PLAT-163 brace-desync)

**Zero Python abandoned files across all six repos, ecaz included.** No Python exposure to this defect class was found in this measurement — unlike TypeScript (see the sibling report), which has three live instances, all in `filament-ide-rs`.

## Tagged-but-unbound: Python signal

`tagged_not_bound` = candidates whose annotation carries an id-shaped token (`tagged`) but which minted no `verifies` relation (`bound`). Only repos with Python **candidates** appear — `filament-ide-rs` has Python source (2 symbols, see the headline table) but zero Python candidates, so it correctly does not appear below; "source" and "candidates" are not the same population.

| Repo | Python candidates | tagged | bound | **tagged_not_bound** | Example |
|---|---:|---:|---:|---:|---|
| quire-rs | 240 | 92 | 91 | **1** | `corpus/cases/attachment/marker-form-mismatch/python/input/src/lib.py:7` `TestCoverage.test_covers_the_criterion` |
| quire-contract-ir | 29 | 29 | 29 | **0** | — |
| ecaz | 7 | 0 | 0 | **0** | — (0 of 7 candidates carry a tag at all — consistent with PLAT-867's finding that ecaz's Rust tagging is similarly sparse; not investigated further here, out of this slice's scope) |
| **all repos** | 276 | 121 | 120 | **1** | |
| **all repos, excl. ecaz** | 269 | 121 | 120 | **1** | ecaz contributes 0 to this signal — with/without split is identical |

`quire-rs`'s one `tagged_not_bound` example is its own dogfooded fixture (`corpus/cases/attachment/marker-form-mismatch/`), deliberately engineered to exercise this exact diagnostic (per this repo's `CLAUDE.md`) — not production loss, the same caveat PLAT-840 raised for its own Rust example from the identical fixture family.

## Coverage: unbacked rows and status lies (whole-repo, all languages — not Python-only)

`unbacked_rows`/`status_lies`/`backed`/`total` are per-repo counts over that repo's **whole** declared reference-row population, the same as PLAT-840 reported — not decomposable by source language, since a matrix row's evidence can come from any language a repo contains. Given here for completeness and for the ecaz split, not as a Python-specific number.

| Repo | unbacked_rows | status_lies | backed / total |
|---|---:|---:|---|
| quire-rs | 400 | 19 | 1,312 / 1,799 |
| quire-code-rs | 12 | 0 | 251 / 284 |
| quire-contract-ir | 16 | 0 | 202 / 219 |
| quire-protocol | 60 | 1 | 238 / 354 |
| filament-ide-rs | 1,073 | 55 | 1,609 / 3,701 |
| ecaz | 846 | 30 | 13 / 649 |
| **all repos** | **2,407** | **105** | 3,625 / 7,006 |
| **all repos, excl. ecaz** | **1,561** | **75** | 3,612 / 6,357 |

ecaz alone accounts for 846 of 2,407 unbacked rows (35%) and 30 of 105 status lies (29%), consistent with PLAT-867's finding that most of ecaz's Rust tests are not binding under this module's declared marker forms at all — a `trace.rs`/module-declaration matter, not this baseline's subject.

`quire-contract-ir`'s `backed` here (202) differs from PLAT-840's corrected baseline (198) and from the even-earlier installed-CLI figure (196) — expected: this measurement runs at the post-PLAT-843 engine, which includes the `backed`-affecting fixes PLAT-840's own report already characterized (`616a7e9`, `+2`) plus subsequent, unrelated main-branch movement; not re-derived here, out of this baseline's scope.

## Source-exclude sanity check (the PLAT-840 correction-table trap)

PLAT-840's own first draft was wrong twice, once from an over-matching discriminator field and once from **fixture-tree symbols counted because `source_exclude` was not applied** — worth −17 symbols and a changed headline claim on the Rust side. This baseline cannot repeat that mistake by construction: it reuses the exact same code path (`extract_tree_scoped(root, &[Path::new("spec")], &source_exclude_globs)`, one extraction call per repo, one loop over the returned symbols building every language's rollup together) that PLAT-840 fixed itself into. There is no separate Python code path that could bypass the exclusion. `excluded_source_files` per repo (files removed by the declared `tests/fixtures/**`, `tests_integration/fixtures/**`, `fixtures/**` globs before any symbol was extracted): `quire-rs` 6, `quire-contract-ir` 2, everyone else 0 — identical to what PLAT-840 reported, confirming the exclusion behaves the same way for this run.

## Reproducing this measurement

```bash
cd quire-rs
git submodule update --init
cargo build --release --example plat840_rust_baseline_sweep
PLAT840_PATH_QUIRE_CONTRACT_IR=/path/to/clean/quire-contract-ir-main \
PLAT840_PATH_QUIRE_PROTOCOL=/path/to/clean/quire-protocol-main \
PLAT840_PATH_FILAMENT_IDE_RS=/path/to/clean/filament-ide-rs-main \
PLAT840_PATH_ECAZ=/path/to/clean/ecaz-main \
./target/release/examples/plat840_rust_baseline_sweep
```

The full structured output is `reports/2026-09-20-plat851-language-scanner-baseline.json`, keyed by repo, with every number above traceable to a specific field (`symbols_by_language_kind.python`, `binding_census[language="python"]`, `abandoned_files[language="python"]`).
