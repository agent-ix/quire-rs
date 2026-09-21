# PLAT-851: TypeScript symbol-scanner baseline, before its own AST port

- Date: `2026-09-20`
- Measuring engine (`quire-rs`): `81039d2c4e5f121bd9766340c8a002a1e01c8093` (this PR's branch — the commit containing the extended harness that produced this exact JSON)
- Traceability module: `spec-artifacts-process` at `61a20e010d5e758f52864ad3152ccdb304a39d27` (same commit PLAT-840 used, held constant for comparability)
- Harness: `examples/plat840_rust_baseline_sweep.rs`, driven through the library API — never the installed `quire` CLI. See the Python report (this pair's sibling) for the full description of PLAT-851's extension; it is identical for both languages.
- Raw output: `reports/2026-09-20-plat851-language-scanner-baseline.json` — **shared with the Python report.** One sweep run measures every language in one pass.
- Reproducibility: two runs back-to-back at the commit above, SHA-256 identical: `422489ef07c7b7e055de94b0680e352358886e03267780ab445306eb77dbb47b`.

This is the before-number for `src/symbols/typescript.rs` — the largest of the three grammars PLAT-851's boundary-crate widening prepares for by adapter surface (~1,118 lines: vitest/jest registration grammar, curried modifiers, string-literal titles), per PLAT-851's own scope note. There was no TypeScript (or Python) pre-port baseline before this slice.

## Repos measured

Identical set, identical shas, identical run as the Python report — see that report's table. `filament-ide-rs` is effectively where TypeScript lives in this corpus; see the ecaz note below for why the other four contribute nothing.

**These are not the same shas PLAT-840 pinned** for `quire-contract-ir`/`quire-protocol`/`filament-ide-rs`/`ecaz` — each moved `main` since PLAT-840's measurement, later the same day. Do not diff this report's incidental Rust figures against PLAT-840's expecting a clean comparison; this measurement also runs the post-PLAT-843 tree-sitter engine, not the pre-port scanner PLAT-840 measured, so Rust is not this report's subject.

## ecaz's weight — and here it contributes nothing

Same corpus-dominance caveat as the Python report: ecaz is roughly half this corpus by Rust volume, was excluded from earlier sweeps during development, and PLAT-867 measured it binding only 15.3% of its trace tags (vs 97.7–100% elsewhere) — known-poor hygiene, not incidental. Every rollup below is still given twice, all repos and all repos excluding ecaz, per that standing instruction.

**Checked, not assumed: for TypeScript, ecaz contributes exactly zero.** It is a `pgrx` Postgres extension with no TypeScript source at all (`symbols_by_language_kind` for `ecaz` has no `typescript` key). Every with/without-ecaz split below is therefore identical — stated once here rather than repeated per table.

## Headline: TypeScript symbols found, by `SymbolKind`

Same measurement definition as the Python report (`extract_tree_scoped`, code root only, `spec/` and the module's declared `source_exclude` globs excluded).

| Repo | TypeScript total | function | test_function | container |
|---|---:|---:|---:|---:|
| quire-rs | 165 | 12 | 73 | 80 |
| quire-code-rs | 0 | 0 | 0 | 0 |
| quire-contract-ir | 0 | 0 | 0 | 0 |
| quire-protocol | 0 | 0 | 0 | 0 |
| filament-ide-rs | 1,445 | 697 | 431 | 317 |
| ecaz | 0 | 0 | 0 | 0 |
| **all repos** | **1,610** | 709 | 504 | 397 |
| **all repos, excl. ecaz** | **1,610** | 709 | 504 | 397 |

`quire-rs`'s 165 is **165/165 `corpus/cases/**`** — unlike Python's 656 (spread across `corpus/cases`, `scripts/tests`, `tests/python` and more, see the Python report), there is no TypeScript binding suite to contribute a second source; `quire-rs` has no TypeScript bindings. `corpus/` matches none of the declared `source_exclude` globs, so it is measured in full. `filament-ide-rs` dominates for the obvious reason: it is the one TypeScript application in this corpus.

## Files abandoned (PLAT-163 brace-desync) — TypeScript is where the live exposure is

Unlike Python (zero abandoned files, see the sibling report), TypeScript has **3 live instances, all in `filament-ide-rs`**, all the same `a `}` closes no block` diagnostic shape. This reproduces PLAT-840's own findings exactly — same three files, same shas' worth of investigation, re-verified here rather than re-derived:

| File | Cause (PLAT-840's own inspection, re-confirmed present here) |
|---|---|
| `ui/tests/e2e/tc-784-786-789-project-switcher.spec.ts` | Escaped-brace regex literals: `/openDiscoveredProject:\s*\(([^)]*)\)\s*=>[^\n]*__TAURI_INVOKE\([^)]*\{([^}]*)\}/` and `/export type DiscoveredProjectPayload = \{([\s\S]*?)\};/` |
| `ui/tests/native/it-019-sync-native.spec.ts` | Same shape: `bindings.match(/export type TokenStatusDto = \{([^}]*)\}/)` |
| `ui/tests/mocks/handlers.ts` | Same diagnostic class, but **not** the escaped-brace-regex shape — PLAT-840 found brace counts exactly balanced (386/386) here and left the triggering construct unmeasured rather than guessed at. Not re-investigated in this slice; still unmeasured. |

This is **PLAT-163's remaining live exposure**, entirely TypeScript, entirely `filament-ide-rs` — the exact fixture the eventual TypeScript port (over this slice's widened boundary crate) needs a regression case for (see PLAT-851's own acceptance criterion: *"A TypeScript file containing a regex literal with `{` binds every tag in the file — previously abandoned all of them. Fixture required."*, which belongs to the port itself, not this enabling slice).

| Repo | Abandoned files | `closes_no_block` |
|---|---:|---:|
| filament-ide-rs | 3 | 3 |
| everyone else | 0 | 0 |
| **all repos** | **3** | **3** |
| **all repos, excl. ecaz** | **3** | **3** |

## Tagged-but-unbound: TypeScript signal

| Repo | TypeScript candidates | tagged | bound | **tagged_not_bound** | Example |
|---|---:|---:|---:|---:|---|
| quire-rs | 73 | 66 | 66 | **0** | — |
| filament-ide-rs | 431 | 394 | 385 | **9** | `ui/tests/e2e/graph-monitor.spec.ts:83` `shows the debugging hint when no sync has run (TC-1038)` |
| **all repos** | 504 | 460 | 451 | **9** | |
| **all repos, excl. ecaz** | 504 | 460 | 451 | **9** | ecaz contributes 0 — with/without split is identical |

## Coverage: unbacked rows and status lies (whole-repo, all languages — not TypeScript-only)

Identical table to the Python report — coverage is not decomposable by source language. Reproduced here for a reader of this report alone.

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

`filament-ide-rs`'s 1,073 unbacked rows / 55 status lies are the repo with essentially all the TypeScript in this corpus, but this total is not TypeScript-attributable on its own — it is a repo-wide figure covering that repo's Rust too.

## Source-exclude sanity check

Same construction as the Python report: one `extract_tree_scoped` call per repo, `source_exclude_globs` applied before any symbol (of any language) is extracted, no separate TypeScript code path that could bypass it. `excluded_source_files`: `quire-rs` 6, `quire-contract-ir` 2, everyone else 0 (including `filament-ide-rs`, where the TypeScript volume lives) — identical to PLAT-840's own figures.

## Reproducing this measurement

Identical command to the Python report — one run produces both baselines:

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

The full structured output is `reports/2026-09-20-plat851-language-scanner-baseline.json`.
