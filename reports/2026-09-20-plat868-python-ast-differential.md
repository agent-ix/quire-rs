# PLAT-868: Python symbol-scanner AST-port differential, against PLAT-851's baseline

- Date: `2026-09-20`
- Baseline: `reports/2026-09-20-plat851-python-scanner-baseline.md` — `975` Python
  symbols across six repos (`function` 512, `test_function` 276, `container`
  187), zero abandoned Python files, `excluded_source_files` 6/2/0/0/0/0
  (quire-rs/quire-contract-ir/quire-protocol/filament-ide-rs/ecaz/
  quire-code-rs).
- Old engine: `quire-rs` at `bc31c2b408e58b281886ec44262dd98cf6058676`
  (PLAT-851, the shared enabling slice this branch forked from) — the exact
  pre-PLAT-868 `src/symbols/python.rs` line/indentation-structural scanner,
  built as an isolated worktree/binary so the "old" side is never this
  branch's own code.
- New engine: this PR's branch (`feat/plat868-python-ast`), tree-sitter port
  via `quire-code-parse` (same pin as PLAT-843/851, `57b83ba0`).
- Target trees held constant, at PLAT-851's own recorded shas — **identical
  to the shas PLAT-851's own baseline report pins**: `quire-rs`
  (`08d39ea2c...`) and `quire-code-rs` (`e1b7fc303...`) against their
  already-pinned local checkouts (verified unchanged before measuring);
  `quire-contract-ir` (`6b7eb899...`), `quire-protocol` (`006546...`),
  `filament-ide-rs` (`37c44d90...`), `ecaz` (`d8c72f44...`) against disposable
  clones checked out to those exact shas. Both engines ran over byte-identical
  source trees per repo — every delta below is engine-only.
- Harness: `examples/plat843_audit_list.rs`, run with `python` as the
  language argument — one
  `path\tqualified_name\tkind\tleading_line\tline\tend_line\tcontainer` line
  per Python symbol, sorted — once per repo against the old-engine binary and
  once against this branch's binary, then diffed line-by-line (identity
  match on the full line, not a separate key). **`source_exclude` is read
  from the declared module** (`AUDIT_LIST_MODULE`, defaulting to
  `spec-artifacts-process/spec_artifacts_process`, PLAT-868's own fix to this
  tool — see "What this ticket also fixed" below), not hardcoded, so the
  exclusion this differential applies is guaranteed to be the same one the
  baseline used. Cross-checked with `examples/plat840_rust_baseline_sweep.rs`
  (unmodified logic) run once against the new engine over all six pinned
  trees, for the aggregate rollup and the abandoned-file/diagnostic counts.

  **The harness originally emitted only four fields**
  (`path`/`qualified_name`/`kind`/`leading_line`) — enough to see identity
  and one span attribute, but structurally blind to `line` and `end_line`.
  A PR review (agent-ix/quire-rs#479) measured `end_line` directly and found
  a delta this differential's first pass had not reported at all, because it
  could not have: not a smaller number found by a bigger search, a class of
  delta the tool had no column for. Extended to all six `RawSymbol` fields
  before re-running (§"The 14 `end_line` deltas" below) — this is the
  finding, not merely the fix: a differential is only as wide as the columns
  it prints, and "no delta of any other shape was found" is a true statement
  about four fields' worth of tree only, not the ones you didn't dump.

  **The `corpus` submodule trap.** A fresh `git worktree add` (or a fresh
  clone) of `quire-rs` leaves the `corpus` submodule uninitialized —
  `corpus/cases/**` is 195 of `quire-rs`'s own 656 Python symbols (30%,
  §"What this ticket also fixed" cross-references PLAT-851's own
  breakdown). The first six-field re-run of this differential silently
  measured `quire-rs` at 380 symbols instead of 656 for exactly this reason
  before `git -C <worktree> submodule update --init` was run — a
  ~40%-smaller "quire-rs" that would have read as a real extraction delta
  rather than as what it was, an empty submodule directory. Run
  `git submodule update --init` on every fresh `quire-rs` worktree/clone
  used for a measurement here, old-engine and new-engine and target-tree
  alike, and check the symbol count against PLAT-851's own recorded 656
  before trusting anything downstream of it. (Independently hit by another
  agent working a concurrent PR the same day — not a one-off.)

## Headline

| Metric | Baseline (PLAT-851) | This PR (new engine) | Δ |
|---|---:|---:|---:|
| `symbols_total_all_repos` (Python) | 975 | 975 | **0** |
| … `function` | 512 | 512 | **0** |
| … `test_function` | 276 | 276 | **0** |
| … `container` | 187 | 187 | **0** |
| `abandoned_files` (Python) | 0 | 0 | **0** |
| `other_diagnostics_count` (all repos, all languages) | — | 0 | **0** |
| `excluded_source_files` (all repos) | 6/2/0/0/0/0 | 6/2/0/0/0/0 | **0** |
| Python symbol identities (`path`+`qualified_name`+`kind`) changed | — | **0 of 975** | **0** |
| Python symbols with a `leading_line` delta | — | **6 of 975** | named below, one cause |
| Python symbols with an `end_line` delta | — | **14 of 975** | named below, one shared cause |

Every one of the 975 symbols the old engine reported is present in the new
engine's output with an **identical `(path, qualified_name, kind)` triple**
— zero symbols added, zero removed, zero reclassified, in all six repos.
This was verified by a full line-level diff of the two per-repo TSV dumps
(not a set-difference on a subset of columns), per repo:

| Repo | Python symbols (both sides) | `path`/`qualified_name`/`kind` deltas | `leading_line`-only deltas | `end_line`-only deltas |
|---|---:|---:|---:|---:|
| quire-rs | 656 | 0 | **6** | **1** |
| quire-code-rs | 0 | 0 | 0 | 0 |
| quire-contract-ir | 122 | 0 | 0 | **13** |
| quire-protocol | 0 | 0 | 0 | 0 |
| filament-ide-rs | 2 | 0 | 0 | 0 |
| ecaz | 195 | 0 | 0 | 0 |
| **total** | **975** | **0** | **6** | **14** |

No symbol carries both a `leading_line` delta and an `end_line` delta — the
two sets are disjoint (verified against the full seven-column diff, not
assumed from the two counts happening not to overlap).

`leading_line` and `end_line` are both non-identity attributes
(`src/symbols/mod.rs`'s `Symbol` doc: "…a non-identity attribute" on each),
so these twenty rows change **no** `Symbol::compute_id` output —
`Symbol::compute_id` hashes only `(language, path, qualified_name, kind)`.
The qualified-name/container/kind non-negotiable this ticket set is
satisfied exactly: zero changes.

## The 6 `leading_line` deltas: one cause, exhaustively enumerated

All six are in `quire-rs`'s own `scripts/tests/*.py` (the local tooling test
suite already reflected in PLAT-851's own report, "scripts/tests/\*\* 166",
25% of quire-rs's Python total):

| File | Symbol | Old `leading_line` | New `leading_line` |
|---|---|---:|---:|
| `scripts/tests/test_dep_pins.py` | `test_deprecated_yaml_packages_in_fuzz_graph_fail_closed` | 104 | 101 |
| `scripts/tests/test_dep_pins.py` | `test_each_fuzz_yaml_pin_mutation_fails_closed` | 91 | 73 |
| `scripts/tests/test_dep_pins.py` | `test_each_load_bearing_pin_mutation_fails_closed` | 130 | 119 |
| `scripts/tests/test_dep_pins.py` | `test_required_yaml_lock_packages_fail_closed` | 149 | 140 |
| `scripts/tests/test_measurement_export.py` | `test_attestation_drift_fails_closed` | 175 | 152 |
| `scripts/tests/test_tool_drift.py` | `test_each_drift_class_fails_closed` | 212 | 97 |

Every one of these six is a `def` decorated by a multi-line
`@pytest.mark.parametrize(\n    (...),\n    [\n        ...\n    ],\n)` whose
argument list wraps across several physical lines — e.g.
`test_dep_pins.py:101-104`:

```python
@pytest.mark.parametrize(
    ("package", "version"), [("serde_yaml", "0.9"), ("unsafe-libyaml", "0.2")]
)
def test_deprecated_yaml_packages_in_fuzz_graph_fail_closed(
```

**Cause: PLAT-234's defect class, confirmed by direct source inspection at
all six sites, not inferred from the pattern.** The old scanner's
`leading_block` walks backward *physical line by physical line*, treating a
line as part of the annotation block only if it `starts_with('@')` or
`starts_with('#')` (`src/symbols/python.rs`'s pre-port `is_annotation`). The
line immediately above each of these six `def`s is a continuation line
(`)`, or a tuple/list literal) that starts with neither, so the walk stops
immediately and `leading_line` is left at the `def`'s own line — the
decorator is entirely missed. The new engine's `leading_span` walks
`decorated_definition`'s preceding **sibling nodes**, and
`decorated_definition`'s own span already includes every decorator
regardless of how any one of them wraps (tree-sitter tokenizes the whole
`@pytest.mark.parametrize(...)` call as one node), so `leading_line`
correctly reaches the decorator's own first line.

This is exactly the delta class the ticket's own PLAT-234 closure names —
demonstrated here on `@pytest.mark.parametrize`, a different decorator than
PLAT-234's own `@pytest.mark.trace`, which is a stronger, unplanned
confirmation that the fix is general rather than narrowly targeted at one
decorator spelling. Per PLAT-843's cause-based rule ("free from parsing
correctly → fix here and name the delta; requires changing what a symbol
*is* or how it is *named* → a separate ticket"): this is free. It changes a
non-identity span attribute only, in the direction of *more* of the true
annotation block being captured, never less — no symbol's `qualified_name`,
`kind`, or `container` moved.

**Zero identity changes** across 975 symbols in six repos of varying size and
style (from `ecaz`'s "poor linting hygiene" — Peter's own characterization,
PLAT-851's report — through `quire-rs`'s own corpus fixtures deliberately
engineered to exercise every form this extractor recognizes) is itself
evidence the qualified-name construction rules in this module's own docs
(only a `class` is a scope; a `def` is never a container for its own nested
`def`s; the module is never a name prefix) were carried over correctly, not
merely asserted to be. There is one more delta shape, enumerated next.

## The 14 `end_line` deltas: one shared cause, exhaustively enumerated

Invisible to this differential's first pass (four-field harness, no
`end_line` column); found once the harness was widened to all six fields
(see "Harness" above). One in `quire-rs`, thirteen in `quire-contract-ir`:

| File | Symbol | Old `end_line` | New `end_line` |
|---|---|---:|---:|
| `scripts/tests/test_gap_census.py` | `repo_fixture` | 31 | 71 |
| `tests/test_assurance_ordering.py` | `AssuranceOrderingTests` | 12 | 47 |
| `tests/test_assurance_ordering.py` | `AssuranceOrderingTests.test_ordering_is_enforced_with_and_without_optimization` | 12 | 47 |
| `tests/test_matrix_status.py` | `MatrixStatusTests` | 23 | 407 |
| `tests/test_matrix_status.py` | `MatrixStatusTests.test_main_reads_a_real_tree_and_fails_closed` | 75 | 116 |
| `tests/test_matrix_status.py` | `MatrixStatusTests.test_main_ties_the_coverage_verdict_to_the_exit_code` | 130 | 154 |
| `tests/test_matrix_status.py` | `MatrixStatusTests.test_rejects_complete_rows_backed_by_planned_tests` | 23 | 43 |
| `tests/test_matrix_status.py` | `MatrixStatusTests.test_rejects_functional_rows_that_cite_zero_criteria` | 241 | 272 |
| `tests/test_matrix_status.py` | `MatrixStatusTests.test_rejects_non_functional_rows_that_cite_a_retired_criterion` | 284 | 297 |
| `tests/test_matrix_status.py` | `MatrixStatusTests.test_rejects_policy_acceptance_citation_without_executable_test` | 47 | 64 |
| `tests/test_matrix_status.py` | `MatrixStatusTests.test_rejects_rows_that_omit_a_live_acceptance_criterion` | 183 | 229 |
| `tests/test_matrix_status.py` | `MatrixStatusTests.test_retired_heading_matches_case_and_trailing_text` | 336 | 362 |
| `tests/test_matrix_status.py` | `MatrixStatusTests.test_retired_section_exclusion_is_position_independent` | 376 | 407 |
| `tests/test_native_orchestration.py` | `exercise` | 51 | 82 |

**Cause: the same `#274` defect family the retired `tc1029`/`tc1030`/`tc1031`
already pin, confirmed by reading every site, not inferred from the shape.**
Each of these fourteen symbols' body contains a triple-quoted (`"""` or
`'''`) multi-line string literal — a markdown-table fixture
(`test_matrix_status.py`), a rendered spec document
(`test_gap_census.py::repo_fixture`), a Python-source-as-string fixture
(`test_assurance_ordering.py`), a shell/JSON double fixture
(`test_native_orchestration.py::exercise`). The pre-port scanner's
hand-rolled `Quoting`/`block_end` string-state tracking misread content
inside these literals as ending the enclosing suite early, truncating
`end_line` well before the declaration's real close — `MatrixStatusTests`'s
384-line understatement (23 vs. 407) is the same family of defect as the
already-documented "10 of 21 classes" misattribution `tc1031`'s own retired
predecessor was written against, just manifesting as a truncated span
instead of a misattributed container. tree-sitter's grammar makes a string
literal's content structurally un-mistakable for suite-ending syntax, so the
new engine reports the declaration's true extent regardless of what any
embedded string contains — the identical structural fix already documented
above for the false-positive-symbol shape of `#274`, now shown to also fix
its false-truncation shape, for free, by the same mechanism.

**Directly measured: zero bindings were minted or moved by any of these
fourteen span expansions.** A wider `end_line` is a wider `Symbol::attached_source`
in `trace.rs`'s own binder — a real risk that a span correction could
silently mint a `verifies`/`implements` relation nobody intended (the exact
failure mode a 2026-09-20 review of this PR, agent-ix/quire-rs#479, raised).
Measured directly rather than argued: a disposable harness (`extract_tree`
over the real new-engine tree, clone the extraction, override each of the
fourteen symbols' `end_line` back to its old, shorter value, run
`trace::bind` — same `tests/fixtures/traceability/iso` model
`tests/trace_dogfood.rs` already uses — on both the real and the
span-truncated extraction, diff `verifies`/`implements` per symbol) found
**`DELTA=false` for all fourteen**. The newly-included lines often do
contain id-shaped substrings (`TC-021`, `FR-099-AC-1`, and similar, visible
in the raw diff) — but none of them sit in a form the declared `trace_tags`
grammar recognizes (a canonical marker, a `Trace:`/comment-id legacy line);
they are ordinary prose mentioning ids, which is exactly the population
`unmatched_tags`/`non_binding_tags` already exist to report on the ids that
*do* have a recognized adjacency, not something this span-widening changed.
Reproduce with:

```bash
# Extracted from the real new-engine tree, symbols' end_line overridden back
# to the OLD (pre-port) value for a same-source-file A/B comparison:
cargo run --release --example plat868_span_bind_probe -- \
  <target-tree-root> tests/fixtures/traceability/iso \
  <path> <qualified_name> <old_end_line> [...]
```

(the probe itself is a throwaway ~70-line example — build one from
`tests/trace_dogfood.rs`'s own `graph_for` helper plus a clone-and-override
loop over `SymbolExtraction.symbols`; not committed, since it exists to
answer one question once, not to become a permanent tool.)

## `unbacked_rows`/`status_lies`: not an isolated delta — see the dogfood test instead

Cross-checked via `plat840_rust_baseline_sweep`'s aggregate fields, with
`PLAT840_PATH_QUIRE_RS` pointed at this branch and the other five repos left
at their current (moved-forward, **not** re-pinned to PLAT-851's shas)
checkouts: `status_lies_total_all_repos` **105**, identical to PLAT-851's
own recorded baseline (105). `unbacked_rows_total_all_repos` is **not**
comparable the same way (1,536 vs. PLAT-851's 2,407) because the other five
repos' own row populations moved between the two measurements — that
difference is real repo drift elsewhere, not a Python signal, and reporting
it as one would overclaim.

**This is not, and must not be read as, an isolated before/after delta for
this branch's own spec edits** — the other five repos were not held
constant the way the six-field differential above holds all six target
trees constant, so a status-lies count that happened to match does not by
itself prove this branch introduced no new lie. It was originally going to
be re-run in a controlled form (quire-rs alone swapped between its old pin
and this branch, everything else identical) specifically to catch the one
defect class this rollup cannot see by construction: a row this branch
itself flips to ✅ (`TC-1880`, `FR-051-AC-25`) reads as one more correctly-
green row to an aggregate count, whether or not it actually binds — the
count cannot distinguish "backed" from "typo'd 🚧 to ✅ and nothing checks
it," which is exactly what F1 (agent-ix/quire-rs#479 review) found had
happened here.

**That defect is independently, and more strongly, closed by
`tests/trace_dogfood.rs::tc1880_the_python_decorator_wrap_tests_actually_bind`**,
added in response to F1: it runs the real binder against this repo's own
`src/symbols` tree and asserts `TC-1880` is present in `graph.verifies` for
each of its three nominated tests, by `(symbol, trace_id)` — confirmed red
first (stripped the `#[trace(...)]` markers, reran, `TC-1880` absent from
`backed_trace_ids()`'s full output) before being trusted green, and it now
runs on every `cargo test`. A per-symbol assertion that fails on exactly the
condition it exists to catch is stronger evidence than a whole-repo rollup
that happens to match — the controlled sweep was not rerun, because there is
no remaining gap for it to close.

## Retired tests and their successors

`tc1029`/`tc1030`/`tc1031` are retired, per the ticket's own ring-fence
(PLAT-843 comment thread: "`tc1029`/`tc1030`/`tc1031` are this ticket's to
retire — PLAT-843 ring-fenced them") and per this ticket's own instruction
("each with a named successor"). They keep their ids and
`FR-051-AC-20` bindings (`spec/tests.md`; `CR-176` already reworded AC-20's
text to the *property* these rows pin, not the deleted scanner's own
mechanism — see the CR-179 spec note).

| Retired | Asserted (why it dies) | Successor | Successor asserts |
|---|---|---|---|
| `tc1029_a_triple_quote_is_tracked_wherever_it_opens` | The hand-rolled `Quoting` state machine's own opener/closer tracking across six string forms | `tc1029_a_string_embedded_declaration_is_structurally_unreadable` | Outcome: none of six embedded-declaration string forms mint a symbol, and the real test after them all is still seen |
| `tc1030_a_delimiter_in_a_string_or_a_comment_does_not_toggle` | `scan_line`'s own per-line return value, asserted directly (`assert_eq!(scan_line(...), Quoting::Code)`) | `tc1030_a_delimiter_in_a_string_or_comment_never_affects_the_tree` | Outcome: the same six delimiter-in-string/comment shapes produce no phantom declaration and no toggled state to observe |
| `tc1031_the_scope_stack_survives_an_embedded_string` | The hand-rolled scope *stack*'s own resume behaviour after an embedded string | `tc1031_the_true_container_survives_an_embedded_string` | Outcome: the class following an embedded string is seen, and its method's `container` is that class, not the class open when the string began |

**Each successor confirmed red first, against a stand-in that reproduces the
retired defect's own shape**, embedded directly in the test (not toggled and
reverted by hand, so the proof is reproducible from the test file alone,
`git blame`-visible forever rather than asserted once in a PR description):

- `tc1029`'s and `tc1030`'s successors assert their fixture premise against
  `naive_line_scan_finds_declaration` — a substring/prefix scanner with no
  notion of a string literal at all (the retired tests' own defect class:
  reading a `class`/`def`-shaped line without regard to what encloses it).
  Run against the fixtures used here, this naive scanner *does* report every
  phantom declaration (`Phantom`, `raw_phantom`, `f_phantom`, `rb_phantom`,
  `inline_phantom`, `phantom_in_doc`) — confirmed by the test's own
  `assert!(naive_line_scan_finds_declaration(...))` premise assertions,
  which fail loudly if the fixture ever stops being adversarial. `parse`
  itself reports none of them, which is the property under test.
- `tc1031`'s successor asserts against a naive single-scope stand-in
  computed **from the fixture itself**, not hand-typed
  (`naive_never_popped_scope(source)`, which reads the first `class NAME`
  line out of `source`) — the exact #274 defect shape (a scope stack that
  never pops, so it always reports whichever class came first) would report
  the same first class for both methods; `parse` reports
  `TestParsing`/`TestModification` respectively. This replaces an earlier
  draft that asserted against a bare string literal
  (`let naive_single_scope = "TestParsing";`): a bare literal stays in sync
  with the fixture only by luck, so renaming the fixture's first class could
  make the assertion vacuous while still passing. Computing the stand-in
  from the fixture ties the two together structurally (PLAT-868 PR #479
  review, F8).

`tc800_wrapped_signature_span_reaches_the_docstring` (CR-037) needed no
rewrite — it already asserted outcome, and passes unchanged against the new
engine (`function_definition`'s own node span already covers a multi-line
signature); its doc comment now also names `TC-1880` for the "a `def` whose
signature spans lines" clause of that TC's compound description.

`TC-1880`'s row (`spec/tests.md`) describes three sub-cases, each with the
single-line spelling as its control: (1) a wrapped `@pytest.mark.trace(...)`
directly above `def`, (2) a wrapped `@pytest.mark.trace(...)` separated from
`def` by a second, also-wrapped decorator, and (3) a `def` whose own
signature spans lines. (3) is `tc800` above. (1) and (2) are new tests:
`plat234_a_black_wrapped_multiline_decorator_reaches_leading_line` (the
ticket's own PLAT-234 closure, now table-driven over the wrapped form and a
single-line control) and
`a_second_wrapped_decorator_between_the_tag_and_def_does_not_move_leading_line`
(a `@pytest.mark.parametrize(...)` wrapped and interposed before `def`).
Both assert `leading_line == 1` and that the joined span still contains the
tag string, for the wrapped form and its single-line control alike.

Confirmed red against the retired (pre-port) implementation directly: the
second-wrapped-decorator source was run through the pre-port `parse` at
`bc31c2b` (a disposable probe test, reverted immediately after, never
committed) and reported `leading_line == 9` (the `def` line itself) instead
of `1` — the pre-port scanner's physical-line walk resynced only once it hit
a line starting with `@` or `def`, so a *second* wrapped decorator pushed
`leading_line` all the way to the `def`, losing both decorators' lines. The
new engine reports `leading_line == 1` for the same source. This is the same
defect class this differential's 6 real-corpus leading_line deltas independently
demonstrate, now also isolated in a unit test.

## What this ticket also fixed (per the linked PLAT-868 comment thread)

1. **CI leg compiling the grammar feature.** `make check-python-symbols`
   (`cargo check --locked --no-default-features --features python-symbols`,
   its own `CARGO_TARGET_DIR`, mirroring `check-python`) is added to `make
   ci` and to `.github/workflows/ci.yml`. Confirmed to fail when the feature
   is broken: temporarily changed `mod.rs`'s `SourceLanguage::Python` arm to
   call a nonexistent `python::parse_broken` — both `make check-python-symbols`
   and a plain default `cargo check --locked` failed identically (an
   unresolved-name compile error naming `src/symbols/mod.rs:381`), because
   `python-symbols` is promoted to `default` by this port (see "What this
   ticket also fixed" below), so the default build now exercises the same
   code the isolated leg does. This makes the isolated,
   `--no-default-features` leg's own distinct value **feature isolation**
   rather than reachability: it proves `python-symbols` compiles *without*
   `rust-symbols`/`resolve-file` also enabled, which a default build (both
   on) cannot distinguish — the exact blind spot
   `crates/quire-rust-extraction/tests/dependency_boundary.rs`'s own
   `cargo_metadata()` doc comment names for the equivalent Rust case.
   Reverted before committing.
2. **License coverage for the grammar.** `make deny-grammars` (`cargo deny
   --locked --features typescript-symbols check licenses`) is added
   alongside the existing default-features `make deny`/`licenses` CI job.
   `python-symbols` is now default (see below), so plain `make deny` already
   covers `tree-sitter-python`; `deny-grammars` additionally activates
   `typescript-symbols` so `tree-sitter-typescript` — already a locked
   `Cargo.lock` entry, still uncompiled by any gate pending PLAT-869 — is
   license-checked too, rather than merely present and unexamined.
   Deliberately **not** `cargo deny --all-features`: that also activates
   this crate's unrelated `python` (PyO3) feature, which pulls in
   `pyo3-build-config` → `target-lexicon` (`Apache-2.0 WITH LLVM-exception`)
   — a real, pre-existing license gap this crate's `deny.toml` doesn't cover,
   unrelated to either grammar. Observed directly: `--all-features` fails on
   exactly that package (`licenses FAILED`, naming `target-lexicon`); the
   narrower `--features typescript-symbols` invocation does not (`licenses
   ok`) — confirming the narrower form is both necessary (broad form fails
   for an unrelated reason) and sufficient (it does activate and therefore
   check both new grammars).
3. **`source_exclude` read from the module, not hardcoded.**
   `examples/plat843_audit_list.rs` now loads
   `Registry::load_module(...).traceability().source_exclude` (same
   `AUDIT_LIST_MODULE` env-var-override convention
   `examples/plat840_rust_baseline_sweep.rs` already uses for its own
   `PLAT840_MODULE`), rather than three hardcoded glob strings. This
   differential's own reproduction is therefore guaranteed to apply the same
   exclusion the baseline did, rather than trusting that a hardcoded copy
   happened to still agree with the module's declaration.

## Gates

Exact counts, not assumed (re-run after rebasing onto `origin/main`
`bd5ce9a`, PR review agent-ix/quire-rs#479 F10/F11):

- `cargo test --locked --lib symbols::`: **72** at `bc31c2b` (pre-port) →
  **88** on this branch, post-rebase. Of that `+16`, **`+8` are this PR's
  own** (`src/symbols/python.rs` alone: 5 pre-port tests → 13, net `+8`:
  three retired and rewritten keeping their ids (`tc1029`/`tc1030`/`tc1031`,
  net 0), `tc800` kept unchanged (net 0), `paren_depth_ignores_quotes_and_comments`
  retired with no direct successor — it asserted the now-deleted `paren_delta`
  helper's own internal return values, not an outcome, so nothing needed
  porting; its outcome-level coverage (a signature's parens never move the
  span) is `default_argument_parens_do_not_move_the_span`, one of the nine
  new tests, net `-1`) and nine new (net `+9`, net total `-1+9=+8`) and
  **`+8` are unrelated**, already on `main` before this branch rebased onto
  it (PLAT-844/#477's `trace_search` module, `src/symbols/mod.rs`).
  `tests/trace_dogfood.rs`: 2 → 3 (the `TC-1880` regression pin, F1).
- `make ci`: **fully green, both times it was required** (before opening the
  PR and again before merge, per this repo's own testing cadence) — the
  first run (pre-rebase, `bc31c2b` fork point) passed through
  `audit-property` and was expected to truncate at `audit-static`
  (PLAT-878's unpinned-CLA-action gap, pre-existing, unrelated to this PR);
  the second run, **after rebasing onto `bd5ce9a`** (which includes
  PLAT-878's fix, #475), ran the full chain for the first time this PR has
  ever reached it: `validate` (**172 documents, 0 failed, 41 warnings** —
  identical warning count to `main`) and `check-engine` (**OK**; one
  advisory, `quire-cli` pinned 27 commits behind `HEAD` — informational per
  this repo's own "pins vs. ceremony" convention, not a gate failure) both
  pass. `check-python-symbols` observed both green (this branch) and red
  (deliberately broken, above).

## Reproducing this measurement

```bash
cd quire-rs
# Fresh worktree/clone of quire-rs itself (as an engine build, and as one of
# the six *target* trees, `quire-rs` in the per-repo table above) MUST
# initialize the corpus submodule — see the harness note above. Skipping
# this silently undercounts quire-rs's own Python symbols by ~40% (380 vs.
# the true 656) and reads as an extraction delta rather than as what it is.
git submodule update --init
CARGO_TARGET_DIR=<dir> cargo build --release --example plat843_audit_list

# Old engine (a worktree checked out at bc31c2b, before this port):
git worktree add /tmp/plat868-old bc31c2b408e58b281886ec44262dd98cf6058676
git -C /tmp/plat868-old submodule update --init
# bc31c2b's own copy of the audit tool still emits only four fields — copy
# this branch's six-field version over it before building, so both sides
# emit the same columns to diff:
cp examples/plat843_audit_list.rs /tmp/plat868-old/examples/plat843_audit_list.rs
CARGO_TARGET_DIR=/tmp/plat868-old-target cargo build --release --example plat843_audit_list \
  --manifest-path /tmp/plat868-old/Cargo.toml

# Per repo (six pinned trees — see PLAT-851's own report for the four
# disposable-clone shas; quire-contract-ir, quire-protocol, filament-ide-rs,
# and ecaz need no submodule init, only quire-rs does), old and new:
<old-binary> <repo-root> python | sort > old.tsv
<new-binary> <repo-root> python | sort > new.tsv
diff old.tsv new.tsv
```
