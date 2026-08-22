# quire-rs

High-performance Rust templating + parsing engine for the Filament/Quire ecosystem.

## Commands

```bash
make fmt            # format with rustfmt
make fmt-check      # verify formatting (CI gate)
make lint           # clippy with -D warnings
make test           # cargo test
make build          # release build
make clean          # cargo clean
make deny           # cargo deny check licenses
make audit-unsafe   # check that every unsafe block has a // SAFETY: comment
make audit-property # FR-052-CON-1: no GrammarFinding in the property classifier
make audit-static   # run every scripts/audits/*.sh
make check-python   # type-check --features python (own CARGO_TARGET_DIR)
make ci             # fmt-check + lint + check-python + test + deny + audit-unsafe + audit-property + audit-static
make ci-python      # build the wheel + run the PyO3 binding suite
```

`make ci` cannot run the binding suite (it needs a built wheel), so **any change
to `src/grammar/`, `src/python/`, or `tests/python/` must also pass
`make ci-python`** before merge. That suite is the only verification of the
PyO3-parity criteria (FR-042-AC-10, FR-043-AC-7, FR-047-AC-9); when nothing ran
it, TC-715 sat asserting check ids CR-014 had renamed and no gate noticed.

## Every defect lands with a fixture

A filed defect gets a **declarative case** in `tests/fixtures/corpus_cases/`,
not a new `.rs` file:

```json
{
  "name": "marker-form-mismatch",
  "issue_ref": "agent-ix/spec-artifacts-process#59",
  "tags": ["TC-992", "coverage", "binding"],
  "input":  { "module": "...", "documents": {...}, "sources": {...} },
  "expect": { "backed": 0, "diagnostic_reasons": ["no-symbol-bound"] }
}
```

`issue_ref` is **required**. A fixture whose origin is unrecorded becomes a
fixture nobody dares change, which is how a corpus rots into a set of
assertions everybody works around. `every_case_is_attributed_and_uniquely_named`
enforces it.

**Assert absence too.** `absent_diagnostic_reasons` is the half a fixture
usually forgets, and it is the half that catches a check firing on healthy
input — the failure mode that killed two diagnostics during CR-094. Where a
case is about a defect, add its **control**: the same tree without the defect,
asserting that nothing fires.

**Assert only what the case is about.** Every `expect` field is optional. A
corpus where each case pins the whole envelope fails forty cases on one
unrelated change, and is then relaxed wholesale.

Directory corpora stay for what needs real filesystem topology — the walk,
exclusion globs, symlinks. The claim is narrower: a scenario expressible as
data should not cost a file.

## Adding or improving a check

This crate ships the checks. A new one pointed at the `~/dev` corpus will fire in
the hundreds or thousands. **That is the expected result, not a signal the check
is wrong.**

A high finding count means exactly one of two things, and which is a question of
fact:

- **Bad rule** — the check misreads correct data.
- **Bad corpus** — the check reads correctly and the specs are wrong.

**Settle it by reading flagged documents**, not by preference. For each: *is the
thing the check complains about actually absent?* If the document has it in a
form the check could not read, the rule is wrong. If the document genuinely
lacks it, the finding stands. Report the split as a number — "sampled 10, 3 rule,
7 real" — because a finding count is a census, not a precision estimate. This is
the CR-014 / CR-019 / CR-022 discipline: each of those retired or rebuilt a check
*after* measuring, and CR-019 found a predicate satisfied by 99.6% of cells.

**Never widen a rule because it lowers the count.** The rule states what a good
spec looks like; it does not fit the specs that exist. A widening needs a reason
true independent of the number — "these two verbs mean the same thing in the
declared edge vocabulary" is a reason, "this drops 400 findings" is not. And when
two forms do mean the same thing, prefer **unifying the corpus on one and
flagging the rest** over accepting both: a rule that accepts every spelling
enforces nothing (see the `unify means enforce` rule).

Say which of the two you did, and why, whenever a rule changes after a
measurement.

**Advisory-first is about blast radius, not about whether findings matter.** Ship
a new corpus check at `warning` via the FR-057 severity registry so findings land
and stay visible; promotion to `error` is a separate, measured, user-gated
decision.

## Safety scaffolding

Backported from `agent-ix/rust-lib-cookiecutter` (originally from `agent-ix/ecaz`).
When upstream tightens MSRV, clippy lints, license allowlist, or audit scripts,
backport changes here via the `backport-code` skill (StR-004-AC-3). Drift is
detected on each `make ci` run via `scripts/audits/verify_cookiecutter_inheritance.sh`.

- **`#![cfg_attr(not(feature = "python"), forbid(unsafe_code))]`** at the crate root is the safety guarantee: the default build makes any first-party `unsafe` a **compile error** (NFR-003-AC-5). It's scoped off for `--features python` because PyO3 macros expand to `unsafe` in-crate. **The Miri job is retired (ADR 0006)** — with zero first-party `unsafe` enforced at compile time there's no first-party UB surface; dependency advisories are covered by `cargo-audit` (NFR-014), the concurrency surface by `loom` (NFR-017).
- `clippy.toml` pins MSRV to `1.75` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Retained because it covers the **`python`** build, where `forbid(unsafe_code)` is scoped off — every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt` (currently empty). Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses 100-char width and `StdExternalCrate` import grouping. CI fails on drift.
- `rust-toolchain.toml` pins to stable + rustfmt + clippy.

## Design taste

Write idiomatic Rust. Lean on the type system to encode invariants rather
than re-checking them at runtime: prefer enums + exhaustive `match` over
booleans + flags, `Option`/`Result` over sentinel values, newtypes over
raw strings/ints when the meaning matters, and `&str`/borrowed slices over
allocating clones on hot paths. Where the TS/Py references rely on dynamic
shapes (e.g. JSON `Value` blobs), the Rust port should look for a stronger
typed representation when it doesn't break parity. "Stringly-typed" code
and `unwrap()`s outside of tests are smells — surface errors via
`thiserror`-derived enums per NFR rather than panicking.

## Code quality & references

Write code that is **safe, idiomatic, performant, and well-spec'd**, and reach for the
authoritative references rather than guessing.

**Performant.** Latency/throughput are normative (NFR-001/002/007/015/016), CI-gated by
`scripts/check_perf_regression.sh` (10% band over a stored criterion baseline). Determinism
(NFR-006) is enforced by `scripts/audits/check_hashmap_audit.sh`, which **bans `HashMap`** in
`src/parser`, `src/merge.rs`, `src/extract`, `src/validate_document.rs` — use `BTreeMap`/`IndexMap`
anywhere iteration order is observable. Prefer borrowed slices over clones on hot paths; do work
once at load time (compile-once, parse/validate-fast). The render/templating feature is removed.

**Well-spec'd.** FRs precede code. `spec/tests.md` must stay at 100% AC→TC coverage (a `grep`
integrity check is wired into that file). Honor the gates in `plan/tasks/README.md` — don't start
gated work early. **Never silently edit `spec/` from an implementation branch**: a spec error or
change requires a CR note (see CR-002 in FR-024 for the pattern).

**v0.3 invariants (corpus + bindings).**
- `load_repo`'s parallel parse **fan-out** is data-parallel with no shared mutable state —
  `par_iter().map().collect()` of owned results, diagnostics gathered after the parallel region
  (FR-024-AC-9 as amended by CR-047; this is what the NFR-017 loom check proves). Interior
  mutability elsewhere in `src/corpus` exists **only** as a named, justified exemption in
  `scripts/audits/check_no_shared_mutable.sh` (whose pattern also catches `OnceLock`/`OnceCell`):
  currently the FR-025 lazy body cell (`body_cache.rs`) and the compile-once regexes in
  `declared_tables.rs`. Anything else fails `make ci`.
- The corpus (`Spec`) is immutable, `Arc<Inner>`-wrapped, `Send + Sync` — **mirror `Registry`**
  (`src/registry.rs`), don't invent a new lifecycle. Headers (path/id/uuid/frontmatter map/verbatim
  text) are **eager** at construction; bodies are **lazy** behind a per-document once-cell
  (first touch parses exactly once, no filesystem read; concurrent first accessors get the
  identical value — FR-025-AC-7/8, loom TC-815 + TSAN TC-816). External immutability is
  unchanged: no query ever returns a different answer twice.
- Document identity is **read, never derived**: `id` = human artifact id (resolution key), `uuid` =
  durable UUID7 from frontmatter (catalog id). No path/content derivation, no file mutation at load.
- First-party `src/python/` stays **`unsafe`-free**; PyO3 macro-generated unsafe is upstream and
  doesn't count (NFR-003-AC-4). The `python` feature is gated so the default build stays
  interpreter-free.

**References.**
- `spec/spec.md` §19 — hardening posture (which ECAZ tools are adopted/skipped and why).
- `spec/assets/adr/` — 0001 (validator crate: `jsonschema ~0.18`), 0002 (three-layer pipeline:
  quire-rs per-doc/per-spec ← filament-parser-lib orchestration ← service layer graph).
- Upstream parity: `agent-ix/quire` (TS parser), `agent-ix/quire-py`; the Python reference renderer
  in `spec-artifacts-*`. Crates: [MiniJinja](https://docs.rs/minijinja),
  [`jsonschema`](https://docs.rs/jsonschema), [`rayon`](https://docs.rs/rayon),
  [`ignore`](https://docs.rs/ignore), [`uuid`](https://docs.rs/uuid) (v7),
  [`pyo3`](https://pyo3.rs).

## Layout

```
src/lib.rs             # crate root
src/parser/            # parse_document → QuireDocument (TS/Py parity)
src/loader/, registry.rs  # archetype Registry (compiled schema only)
src/validate.rs, validate_document.rs, writeback.rs  # schema/markdown validate + byte-splice edit
src/extract/           # body-extraction DSL evaluator
src/corpus/            # v0.3: load_repo + Spec corpus + resolution + query
src/python/            # v0.3: PyO3 bindings (feature = "python")
tests/                 # integration + parity + determinism + dogfood
benches/               # criterion benchmarks (harness = false)
fuzz/                  # cargo-fuzz targets
spec/, plan/           # requirements artifacts + execution plan
scripts/               # local tooling + CI audits
```
