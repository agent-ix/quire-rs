# PLAT-845: nested-function identity differential, against PLAT-843's post-port engine

- Date: `2026-09-20`
- Old engine: this repo's own `origin/main` @ `456f73f557183fa38b55956c8698e97cc9fe7d79` (PLAT-843 merged, pre-PLAT-845 — the flattening bug this ticket fixes).
- New engine: this branch (`feat/plat845-nested-identity`, commit `20f8824` and its follow-up review-fix commit).
- Target trees held constant per repo, both engines run over the identical checkout: `quire-rs` = the same pinned `456f73f` checkout used to build the old engine; the other five repos are fresh `gh repo clone --branch main` disposable clones (shas below — **not** PLAT-840/843's originally-pinned shas, since each repo's `main` has moved on; both engines in *this* comparison ran against the same tree per repo regardless, so the diff is engine-only, but these numbers are not directly diffable against PLAT-840/843's historical counts without re-pinning to their exact shas first — the same caveat PLAT-843's own report carried for `ecaz`).
- Harness: `examples/plat843_audit_list.rs`, unmodified — no new extraction tooling. Run once per engine per repo (`path qualified_name kind leading_line`, sorted), diffed by exact-row multiset difference (`Counter` subtraction), not raw line diff, so a row is `old_only`/`new_only` only when its full `(path, qualified_name, kind, leading_line)` tuple has no counterpart on the other side.

| Repo | Commit scanned (target tree, held constant both sides) |
|---|---|
| quire-rs | `456f73f557183fa38b55956c8698e97cc9fe7d79` |
| quire-code-rs | `ad22335e0c86bf3afa35d7ac86c427c95f42fe32` |
| quire-contract-ir | `6b7eb8996226118c47719e552f93f2b93f890ab1` |
| quire-protocol | `0065460c69eb90479000024faa0fb7804cf3ebfc` |
| filament-ide-rs | `37c44d907ad419472faca240bd02a0fae9add7c0` |
| ecaz | `d8c72f44d814e486b8cf718796aace6241279c9a` |

## Headline: 207 changed identities, zero anomalies, verified by script

| Repo | Changed identities |
|---|---:|
| quire-rs | 23 |
| quire-code-rs | 2 |
| quire-contract-ir | 31 |
| quire-protocol | 9 |
| filament-ide-rs | 47 |
| ecaz | 95 |
| **Total** | **207** |

Total symbol *counts* are unchanged in every repo — every changed row is a rename, never an add or a remove.

**Every one of the 207 changed identities was verified programmatically** (not by inspection) to be exactly a single `::`-segment insertion into the qualified name — old and new components are identical lists except for exactly one extra component, at any position — with `path`, `kind`, and `leading_line` byte-identical between the `old_only` and `new_only` row of each pair. Zero rows fell outside this shape: no kind change, no path change, no addition or removal, anywhere in the corpus. The full 414-row (`207 old_only` + `207 new_only`) enumeration is `2026-09-20-plat845-symbol-diff.tsv`, checked in beside this file, in the identical shape as its PLAT-843 sibling (`repo`, `side`, `path`, `qualified_name`, `kind`, `leading_line`).

Representative pairs (full detail in the TSV):

- `quire-rs`, `src/traceability.rs`: `SectionNames::ScalarOrSequence` (old) → `SectionNames::deserialize::ScalarOrSequence` (new) — a struct locally declared inside a `Deserialize` impl's `deserialize` method now threads through that method, not just the impl's target type.
- `quire-rs`, five files: `tests::assert_send_sync` (old, one per file) → `tests::<its own test fn>::assert_send_sync` (new) — previously flattened to the same bare name regardless of which test declared it.
- `ecaz`, `src/tests/mod.rs`: `ScopedPgQueryCancelFlags::dlsym` appeared **twice** at different lines pre-fix (both nested inside different methods on the same impl target, both flattened to the same name) → `ScopedPgQueryCancelFlags::set_pending::dlsym` / `ScopedPgQueryCancelFlags::clear_pending_for_test::dlsym` post-fix.

## Duplicate-id sweep: zero new duplicate-id groups in this corpus, 15 pre-existing resolved

Computed `(path, qualified_name, kind)` (`Symbol::compute_id`'s inputs, minus `language`, constant per repo) across all six new-engine outputs, then re-ran the identical sweep against the old-engine outputs over the same six trees before treating any count as news:

| Repo | Old-engine duplicate groups | New-engine duplicate groups | Resolved by this PR | Introduced by this PR |
|---|---:|---:|---:|---:|
| quire-rs | 0 | 0 | 0 | 0 |
| quire-code-rs | 0 | 0 | 0 | 0 |
| quire-contract-ir | 0 | 0 | 0 | 0 |
| quire-protocol | 4 | 4 | 0 | 0 |
| filament-ide-rs | 49 | 49 | 0 | 0 |
| ecaz | 136 | 121 | 15 | 0 |
| **Total** | **189** | **174** | **15** | **0** |

**Zero new duplicate-id groups were introduced by this change, in this corpus.** All 174 groups (357 symbols) remaining on the new engine already existed, byte-identically, on the old engine — root-caused to `#[cfg]`-gated dual declarations tree-sitter parses both arms of regardless of feature selection (e.g. `ecaz/src/am/common/stats.rs:164` vs `:169`, `#[cfg(feature = "pg18")]` / `#[cfg(not(feature = "pg18"))]` both declaring `fn record_bootstrap_only_scan()`), orthogonal to nesting and unaffected by this ticket's naming-scheme decision. Filed and tracked separately as **PLAT-877**.

**This PR resolves 15 pre-existing duplicate groups in ecaz**, all instances of the exact bug this ticket fixes — same-named functions nested in two different enclosing scopes, previously flattened to one id. The largest: `tests::Identity` was a **9-way collision** across `ecaz/src/quant/rabitq.rs` — nine separate `#[test]` functions each locally declaring their own `Identity` struct, all nine sharing one id pre-fix; post-fix each is distinguished by its own enclosing test (`tests::estimator_recovers_self_ip_on_sign_aligned_vector::Identity`, `tests::qbit_encoder_reduces_error_vs_binary::Identity`, etc.).

## The accepted residual: this PR trades one collision class for a narrower one — measured, not assumed

The naming scheme (plain `::`, no function-scope marker — the identical threading `mod`/`struct`/`trait`/`impl` already use) was approved conditioned on measuring its own new residual, not on assuming it rare. **This PR introduces a new, narrower collision class it did not have before:** a `mod` and a `fn` sharing a literal name in the same file, each nesting a same-named helper — `mod parse { fn helper() {} }` / `fn parse() { fn helper() {} }`, both now qualify to `parse::helper`. Before this change those two symbols had *distinct* ids (the `mod`'s helper was already `parse::helper`; the `fn`'s helper flattened to bare `helper`, PLAT-845's original bug); after, they collide. This is not a pre-existing limitation — it is a direct, named consequence of this PR's own fix, pinned as such by `a_mod_and_a_function_sharing_a_name_can_still_collide_on_nested_helpers` in `src/symbols/rust.rs`, whose own doc comment works the before/after explicitly.

**Zero real occurrences of this new class were found anywhere in the six-repo corpus** — the duplicate-id sweep above found no `mod`/`fn`-name-collision shape among the 174 remaining groups, only the `#[cfg]`-gated class (PLAT-877). Against that: the *old* flattening class this PR retires was colliding real code in this same corpus — 15 groups in `ecaz` alone, one a 9-way collision. The trade is between a collision class that was firing for real, repeatedly, in the measured corpus, and one that has fired zero times in it. A distinguishing marker that would keep both classes at zero was considered and declined, because it would make qualified names — which are user-visible, appearing in coverage output and published assurance artifacts — stop reading as real Rust paths.

## Gates

`make ci`: `fmt-check`, `lint`, `check-python`, `check-wasm` (both legs), `check-scripts`, the full `cargo test --locked`, and `deny` all clean. `audit-static` fails on `check_tool_drift.sh` (`.github/workflows/cla.yml:22` unpinned action) — confirmed pre-existing by running the same script directly against unmodified `origin/main`@`456f73f`; this PR never touches `.github/workflows/`. Now tracked as **PLAT-878**. Because `audit-static` sits before `validate`/`check-engine` in the `ci` target chain, neither ran in this composite invocation — the identical situation PLAT-843's own differential documented for `validate`.
