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
  language argument — one `path\tqualified_name\tkind\tleading_line` line per
  Python symbol, sorted — once per repo against the old-engine binary and
  once against this branch's binary, then diffed line-by-line (identity
  match on the full line, not a separate key). **`source_exclude` is read
  from the declared module** (`AUDIT_LIST_MODULE`, defaulting to
  `spec-artifacts-process/spec_artifacts_process`, PLAT-868's own fix to this
  tool — see "What this ticket also fixed" below), not hardcoded, so the
  exclusion this differential applies is guaranteed to be the same one the
  baseline used. Cross-checked with `examples/plat840_rust_baseline_sweep.rs`
  (unmodified logic) run once against the new engine over all six pinned
  trees, for the aggregate rollup and the abandoned-file/diagnostic counts.

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
| Python symbols with a `leading_line` delta | — | **5 of 975** | named below, one cause |

Every one of the 975 symbols the old engine reported is present in the new
engine's output with an **identical `(path, qualified_name, kind)` triple**
— zero symbols added, zero removed, zero reclassified, in all six repos.
This was verified by a full line-level diff of the two per-repo TSV dumps
(not a set-difference on a subset of columns), per repo:

| Repo | Python symbols (both sides) | `path`/`qualified_name`/`kind` deltas | `leading_line`-only deltas |
|---|---:|---:|---:|
| quire-rs | 656 | 0 | **5** |
| quire-code-rs | 0 | 0 | 0 |
| quire-contract-ir | 122 | 0 | 0 |
| quire-protocol | 0 | 0 | 0 |
| filament-ide-rs | 2 | 0 | 0 |
| ecaz | 195 | 0 | 0 |
| **total** | **975** | **0** | **5** |

`leading_line` is a non-identity attribute (`src/symbols/mod.rs`'s `Symbol`
doc: "1-based first line of the attached annotation block … a non-identity
attribute"), so these five rows change **no** `Symbol::compute_id` output —
`Symbol::compute_id` hashes only `(language, path, qualified_name, kind)`.
The qualified-name/container/kind non-negotiable this ticket set is
satisfied exactly: zero changes.

## The 5 `leading_line` deltas: one cause, exhaustively enumerated

All five are in `quire-rs`'s own `scripts/tests/*.py` (the local tooling test
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

Every one of these five is a `def` decorated by a multi-line
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
all five sites, not inferred from the pattern.** The old scanner's
`leading_block` walks backward *physical line by physical line*, treating a
line as part of the annotation block only if it `starts_with('@')` or
`starts_with('#')` (`src/symbols/python.rs`'s pre-port `is_annotation`). The
line immediately above each of these five `def`s is a continuation line
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

**No delta of any other shape was found.** Zero identity changes across
975 symbols in six repos of varying size and style (from `ecaz`'s "poor
linting hygiene" — Peter's own characterization, PLAT-851's report — through
`quire-rs`'s own corpus fixtures deliberately engineered to exercise every
form this extractor recognizes) is itself evidence the qualified-name
construction rules in this module's own docs (only a `class` is a scope; a
`def` is never a container for its own nested `def`s; the module is never a
name prefix) were carried over correctly, not merely asserted to be.

## `unbacked_rows`/`status_lies`: exact match, whole-repo

Cross-checked via `plat840_rust_baseline_sweep`'s aggregate fields (these are
whole-repo, all-language counts, not Python-decomposable — the same caveat
PLAT-851's own report states): `unbacked_rows_total_all_repos` 2,407,
`status_lies_total_all_repos` 105, both **identical** to PLAT-851's own
recorded baseline values (2,407 / 105) — expected, since PLAT-851 measured
these same six repos at these same shas with the pre-port Python scanner
(and the pre-port Rust/TypeScript scanners, both untouched by this ticket),
and this port changes zero Python symbol identities, so zero binding
decisions can move.

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
- `tc1031`'s successor asserts against a **named** naive single-scope
  stand-in (`let naive_single_scope = "TestParsing";
  assert_ne!(method.container.as_deref(), Some(naive_single_scope), ...)`)
  — the exact #274 defect shape (a stale scope resuming after the embedded
  string) would report `TestParsing` for both methods; `parse` reports
  `TestParsing`/`TestModification` respectively.

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
defect class this differential's 5 real-corpus deltas independently
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

- `cargo test --locked --lib symbols::` (77 tests, Rust + Python + TypeScript
  + trace.rs): pass, including the three retired-and-replaced tests, the new
  PLAT-234/unittest-classifier tests, and every pre-existing Rust/TypeScript/
  trace-seam test (unmodified, confirming the form-matching seam was not
  breached).
- `make ci`: see PR description for the full run; `check-python-symbols`
  observed both green (this branch) and red (deliberately broken, above).

## Reproducing this measurement

```bash
cd quire-rs
CARGO_TARGET_DIR=<dir> cargo build --release --example plat843_audit_list

# Old engine (a worktree checked out at bc31c2b, before this port):
git worktree add /tmp/plat868-old bc31c2b408e58b281886ec44262dd98cf6058676
CARGO_TARGET_DIR=/tmp/plat868-old-target cargo build --release --example plat843_audit_list \
  --manifest-path /tmp/plat868-old/Cargo.toml

# Per repo (six pinned trees — see PLAT-851's own report for the four
# disposable-clone shas), old and new:
<old-binary> <repo-root> python | sort > old.tsv
<new-binary> <repo-root> python | sort > new.tsv
diff old.tsv new.tsv
```
