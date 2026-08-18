# Changelog

All notable changes to `quire-rs` are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/) loosely. Version
numbers follow semver — pre-1.0, breaking changes may land in minor
bumps; once 1.0 ships, semver is strict.

## [0.33.0] — 2026-08-18

### Added

- **FR-060 — vocabulary references in body-extraction asserts (CR-077).**
  `from_vocabulary:` and `column_vocabularies:` name a declared vocabulary
  instead of restating it, resolved into literal choices at registry
  construction — which is where it must happen, because the vocabulary a
  contract names may be declared by a *different module* than the archetype
  naming it. A reference is legal exactly where its literal counterpart is; an
  unknown name resolves to an empty choice set rather than to "no constraint",
  so a typo cannot silently widen a contract.

- **FR-061 — combinatorial obligations from declared configuration dimensions
  (CR-078).** A module declares which columns hold dimensions, values and
  forbidden combinations; the engine mints one obligation over the interaction
  of every row, carrying `strength`, `dimensions` and `tuples` in the FR-053
  `parameters` map.

  The number is the **t-way tuple count**, deliberately not a covering-array
  size: the minimum array is NP-hard and depends on the generator, while the
  tuple count is a property of the declared space alone. Forbidden combinations
  are first-class and bite at every strength above their own width.

  Three ways to declare nothing are rejected — strength 0 at load, fewer than
  two real dimensions, and a strength above the dimension count — because each
  would read as a declared space demanding coverage of nothing, which is worse
  than absent because it looks answered.

### Fixed

- The `Constraints` tables on FR-058, FR-059 and FR-060 declared
  `ID | Constraint | Verification` where the archetype asserts
  `ID | Constraint | Type | Validation`. Three documents failing the module's
  own contract while describing checks that enforce contracts.

## [0.32.0] — 2026-08-18

### Added

- **FR-059 — declared-vocabulary coverage (CR-076).** A bundle can be 100%
  acceptance-criterion covered and still carry no requirement anywhere for
  reliability or security: every document individually fine, the *set* wrong.
  The generic primitive — *given a declared vocabulary and a declared projection
  onto it, which values does no document claim?* — with ISO 25010 characteristics
  as one instance and test-type and STRIDE coverage as others.

  **The vocabulary is read from the projected archetype's own frontmatter-schema
  `enum`, never restated in the manifest.** A second list would be free to drift
  from the first, which is the defect CR-015 closed.

  Justified absence is first-class: a value recorded in a declared "not
  applicable" field counts as covered, and that record may live on any document
  in the bundle.

  **[RAN]** over 243 `~/dev` bundles: 285/689 NFR documents (41.4%) carry
  `quality_attribute` and 0/2474 FRs do; `safety` is claimed by nothing in the
  ecosystem; and 90 bundles carry no NFR at all. That last figure changed the
  design — an empty projection is one finding rather than one per value, taking
  the sweep from 2792 findings to 1802 without any bundle ceasing to be reported.

  On this corpus the check reports labelling debt more often than quality gaps,
  and the FR says so. Advisory, per-repo settable via FR-057.

## [0.31.1] — 2026-08-18

### Fixed

- A finding for a relation declared `to: []` read "from **any any** document" —
  the article was hardcoded in the message template and already present in the
  noun phrase. Found by the first end-to-end run against `spec-objects-safety`,
  whose `hazard-has-mitigation` relation is exactly that shape. TC-910.

## [0.31.0] — 2026-08-18

The ADR-0011 Phase 2 Wave A + Wave B engine surface.

### Added

- **FR-057 — per-check corpus severity (CR-068).** Corpus findings now carry a
  `<pack>:<check>` severity key (`bundle`, `refs`, `edges`, `trace`), tunable
  through the same FR-048 registry and `--severity` layering the grammars use.
  A module can map one check `off` without touching its siblings.

- **FR-058 — upward-trace completeness (CR-073).** The first analysis class that
  finds a *missing* requirement. Downward coverage answers "is what we wrote
  verified"; nothing answered "is anything missing", and nothing operating over
  existing spec text can, because a requirement nobody wrote leaves no trace to
  follow. A module declares `traceability.required_relations` — `from`, `edges`,
  `to`, `direction`, `check` — and `traceability.acyclic_edges`. The engine holds
  no archetype name, no verb and no chain, so "every hazard must be mitigated"
  is manifest data rather than a second engine check.

- **CR-067 — an `ix://` URI grammar** replacing the previous blacklist, so a
  fully-formed reference in prose resolves whether or not it is in backticks,
  and a bare `ix://` discussing the protocol does not mint an edge.

- **CR-069 — metamorphic property suite** and two writeback fixes found by it.

### Fixed

- **CR-074 — a required relation that cannot be executed now fails at module
  load.** `edges: []` accepts no verb, so nothing satisfies the relation and
  *every* `from` document is reported — hundreds of findings against correctly
  linked documents. A `check` token containing `:` or whitespace leaves the
  relation running at a severity no `--severity` flag and no module override can
  name. A blank verb in `acyclic_edges` walks a graph no edge matches. None is
  visible in the output.

- **CR-075 — a relation naming a document kind nothing has now reports itself.**
  The silent twin of the above: a typo in `from` selects zero documents, so the
  relation checks nothing. Measured — changing `from: FR` to `from: FRR` leaves a
  genuine orphan requirement unreported and the whole run clean. Verbs are
  deliberately excluded from this rule; FR-041-AC-2 permits verbs absent from
  `edge_types`, and a misspelt verb already fails loudly.

### Repository

- `mutants.out/` and `mutants.out.old/` — 178 files of `cargo-mutants` run
  output — untracked. `.gitignore` no longer ignores `*.proptest-regressions`:
  those files are how a discovered counterexample stays discovered.

## [0.30.1] — 2026-08-17

### Fixed

- **The Python wheel could not be built at all** since v0.29.0. FR-056 added
  `ambiguous` to `GrammarVocabularies` and two PyO3 struct literals in
  `src/python/mod.rs` were never updated, so `--features python` failed to
  compile with `missing field \`ambiguous\``. v0.29.0 and v0.30.0 both shipped
  in that state; it surfaced only when the wheel job was finally dispatched.

  `CLAUDE.md` already required it — *"any change to `src/grammar/`,
  `src/python/`, or `tests/python/` must also pass `make ci-python`"* — and
  nothing enforced it, which is the same gap that let TC-715 sit asserting
  renamed check ids.

### Changed

- **`make ci` now runs `check-python`.** `cargo check --features python` needs
  no wheel and no interpreter, so a missing field can no longer reach a tag. It
  does **not** replace `make ci-python`, which runs the binding suite and
  remains the only verification of the PyO3-parity criteria. It builds in its
  own `CARGO_TARGET_DIR`: `--features python` resolves a different feature set,
  and sharing the default target dir makes the next `cargo test` link against
  artifacts from the other set — surfacing as bogus "trait `Serialize` is not
  implemented" errors on types that plainly derive it.

## [0.30.0] — 2026-08-17

The post-merge review of the ADR-0011 P1 wave (agent-ix/quire-rs#81), landed.
Four tickets — a red release tree, and three FRs whose Test Matrix read ✅ over
behaviour the code did not have.

**The pattern worth naming.** In every one of the three, the acceptance criterion
was written at the *function* boundary and verified against the helper that
implements half of it, so the suite could not tell "implemented" from
"implemented and reachable". FR-053-AC-8's diagnostic was returned by `derive`
and dropped by its only caller; FR-054's derived vocabularies were computed
correctly and read by nothing; FR-056's collection gate admitted fewer statements
than its checks judged. Each test passed. None of them was testing the surface a
consumer reads.

### Added

- **FR-054-AC-11 — an uncatalogued verification method is reported**
  (agent-ix/quire-rs#152, **CR-064**). The catalog shipped with two vocabularies
  derived from it, `verification_method` and `verification_class`, and **nothing
  read them** — so a `Verification` cell saying `CI Gate` sat in the report
  looking exactly like one saying `Test`. Measured across the ecosystem: **55 of
  577** obligations declared a method matching no catalog entry, and quoin's
  conformance check skipped every one of them silently, which meant the
  requirements whose verification is least well defined were the ones nothing
  questioned.

  Reported as a coverage diagnostic (`uncatalogued-verification-method`), never
  a grammar finding and never an error (new CON-5) — whether the gap fails a
  build is the consuming workflow's policy. One diagnostic per distinct
  (source, method) pair with the row count and an example document, and total
  silence when no module declares a catalog, because an absent catalog cannot
  answer the question.

### Fixed

- **FR-056's quality pack reached fewer statements than it claimed, and pointed
  at the wrong line when it did** (agent-ix/quire-rs#153, **CR-065**). New
  AC-10..13 and CON-5.
  - **Table findings now carry the row's line** (TC-876). Every `Constraints`
    finding reported the section heading's line, so five flawed rows produced
    five findings at one line.
  - **All four modals are collected** (TC-877). The prose gate admitted `shall`
    and `should` while `mixed-modal` judges four, so "The parser must reject
    adequate input" reached no check at all.
  - **`by <deadline>` and `by <sort key>` no longer count as an agent**
    (TC-878). The old suppressor accepted any word after `by`, so "shall be
    written by 12:00" and "shall be sorted by name" silenced the finding. An
    agent wrapped in emphasis or a code span now *does* count, which the old
    regex rejected. New CON-5 records the direction to err in.
  - **`AmbiguityTermDef` rejects unknown fields** (TC-879), as
    `VerificationMethodDef` always did.

  **[RAN]** the fit check on both trees over the same 3,342 documents in 239
  repositories: `agentless-passive` 680 → **668**, `ambiguous-term` 146 → **161**,
  `mixed-modal` 130 → **131**, documents with ≥1 finding 675 (20.2%) → **688
  (20.6%)**. The `agentless-passive` fall is markup-wrapped agents ceasing to be
  false positives, outnumbering the sort keys and deadlines that ceased to be
  false negatives; the `ambiguous-term` rise is 15 `must`/`may` requirements no
  check had ever seen.

- **FR-053 said five things the code did not do** (agent-ix/quire-rs#151,
  **CR-063**), each ✅ in the matrix because its AC was verified against the
  helper rather than against the surface a consumer reads.
  - **AC-8's diagnostic did not exist.** `derive` returns skipped rows and
    `coverage.rs` discarded them; a row whose statement cell is empty was
    dropped silently from every payload. Skipped rows now become a
    `obligation-row-states-nothing` coverage diagnostic naming the document and
    the row ordinal (TC-870).
  - **NFC is applied for real** (TC-871). The Behavior section always said
    "Unicode NFC, then trim, then collapse"; `normalize_statement` skipped the
    NFC on a dependency argument. `unicode-normalization` is added — an editor
    rewriting a decomposed accent was otherwise indistinguishable from a
    reworded requirement, which is the false-positive suspect link the FR itself
    says gets a detector switched off.
  - **Record order is source *declaration* order** (TC-872), not source name.
    The two look identical for a single-source module, which is how the
    divergence survived review.
  - **AC-11 corrected** to match its own test, the `skip_serializing_if`
    attribute and the published schema: the empty `obligations` list is
    **absent**, which is what preserves FR-050-AC-7 byte-identity.
  - **New AC-14 — `exclude:` binds both surfaces** (TC-873).
    `classify_document_criteria` now takes the document's path and applies the
    same globs `derive` does. Before, a criterion in an excluded fixture minted
    nothing in the rollup and still carried an `obligation` in
    `properties --json` — which is what spec-correctness generates tests from,
    so it became a generated test tagged for an id nothing could ever back.

### Changed

- **Breaking (library):** `classify_document_criteria` takes a fourth argument,
  `path: Option<&Path>`. `None` preserves the old behaviour for content with no
  location (stdin); callers that read from a file should pass it so `exclude:`
  can apply.

- **The v0.29.0 tree failed two of its own gates** (agent-ix/quire-rs#150), found
  by the post-merge review of the ADR-0011 P1 wave. `TC-853` was **flaky**:
  `tests/verification_catalog.rs`'s `merged()` built its fixture in a temp
  directory keyed on the process id, and tests in one binary share a pid and run
  on parallel threads — so one test's `remove_dir_all` could race another's
  `load_from`, which is how the merge was observed to yield two methods instead
  of three. Each call now takes a per-test suffix, matching the `tmpdir(suffix)`
  convention the rest of the suite already uses. `cargo fmt --check` also
  reported drift in eight files across #144–#148, including a visibly
  mis-indented struct literal in `src/loader/mod.rs`; the tree is formatted.
  Verified: `make ci` green, `TC-853` 20/20 in isolation and 5/5 full parallel
  suite runs.

Closes agent-ix/quire-rs#150, #151, #152, #153.

## [0.29.0] — 2026-08-17

The engine half of the ADR-0011 verification program (agent-ix/quire-rs#81, P1),
released as one version because the **engine-before-module** rule means every
key below must be released before `spec-artifacts-process` may declare it — and
two releases would have blocked that module twice.

Four new FRs, 39 new acceptance criteria, all backed. **Every addition is
skip-when-empty**: a module that declares none of the new blocks produces a
byte-identical `coverage --json` payload, which the CR-057 baseline gate proves
rather than asserts.

### Added

- **FR-053 — the obligation record**, the quire↔quoin contract ADR 0011 names
  and neither side had. Requirement id + normalized statement **content hash** +
  verification method + parameters + criticality, derived from what the author
  already wrote — choosing an acceptance criterion's `Verification` method *is*
  minting the obligation. Which rows state obligations is module data
  (`traceability.obligations:`), so the engine knows the shape and never a
  column name, a method name or an archetype. Two source forms: `target:`
  inherits from a declared trace target, so an AC is not declared twice and the
  obligation id is by construction the id the rollup already keys on; and
  `archetype:` + `section:` + `id_format:` covers rows minting no id of their
  own — the NFR `Measurement and Evaluation` table, present in **19 of 19** of
  this repo's NFRs, where every row is a quantified obligation and none has an
  `ID` column. Declaring both origins or neither fails at parse.

  **The hash deliberately does not reuse the CR-017 mask**, which the ticket
  proposed. `mask_code_spans` replaces a code span's *contents* with `x` so a
  quoted keyword reads as a mention — right for grammar detection, wrong for a
  hash: ``reject a `foo` token`` and ``reject a `bar` token`` mask identically,
  and a suspect-link detector built on that stays silent through the exact
  rename it exists to catch. Normalization is NFC + trim + whitespace collapse.
  TC-831..TC-843.

- **FR-054 — the verification-method catalog.** `verification_catalog:` is a
  map of method id → {name, class, definition, evidence kind, applicability,
  tooling}, merged first-wins. Unlike the other vocabularies a re-declared id
  **is** reported (`DuplicateVerificationMethod`): two modules disagreeing about
  what `mutation-testing` means is a collision an operator must see. `class` is
  a **free string**, not a closed IADT enum, and `applicability:` is **opaque** —
  stored and surfaced, never interpreted — because deciding which requirement a
  rule matches is the advisor's judgement. `Registry::column_vocabulary` becomes
  a real named lookup answering `verification_method` and `verification_class`,
  both **derived from the merged catalog**, which is what makes it a single
  source rather than a fourth copy. TC-844..TC-853.

- **FR-055 — the published JSON output contract.** `schemas/output/coverage-v1.schema.json`
  and `properties-v1.schema.json`, hand-authored (`schemars` stays banned —
  deriving them would make the contract a shadow of the implementation, changing
  silently whenever a struct did). The version lives in the `$id` and filename,
  never in the payload (FR-008-AC-5 stands). The conformance gate validates the
  **CR-057 byte-golden baseline**, so one corpus carries both gates and a payload
  change fails both unless the schema moves with it. TC-854..TC-860.

- **FR-056 — the requirement-quality lint pack**, three checks under a new
  `quality:` grammar id so FR-048 can silence or promote 29148 quality
  independently of EARS conformance: `ambiguous-term` (closed,
  module-extensible denylist), `agentless-passive` (`shall be <participle>` with
  no `by <agent>`), `mixed-modal`. Advisory on arrival.

  **[RAN]** the CR-014 fit check *before* shipping — `cargo run --example
  fr056_fit_check` over `~/dev`, **239 repositories, 3,335 FR/NFR/StR
  documents**, worktrees deduped: `agentless-passive` 678, `ambiguous-term` 145,
  `mixed-modal` 130, and **674/3,335 = 20.2%** of documents carry at least one.
  Dogfooded at **22.4%** on this repo's own spec. Unlike the check CR-014
  retired, detection here is exact by construction — a closed list, a syntactic
  shape, a token count — so the question was the rate, not the precision.
  TC-861..TC-869.

### Changed

- `Registry` gains `verification_catalog()`, `ambiguity_terms()`,
  `ambiguity_terms_matcher()`; `column_vocabulary` answers three names where it
  matched one hardcoded string.
- `AcClassification` gains `obligation`; `CoverageReport` gains `obligations`.
  Both absent when empty.
- `GrammarVocabularies` gains `ambiguous`. Constructing one by struct literal
  needs the new field; `GrammarVocabularies::defaults()` is unchanged.

### Deferred

- **`from_vocabulary` on `LocatorAssert`** — the decision #133 left to Specify.
  Resolving a vocabulary reference inside an assert must happen after the
  cross-module merge, which means either threading a `Registry` through
  `evaluate_assert` or rewriting compiled archetypes at registry construction.
  Each is a real change with its own acceptance surface, against a duplication
  currently held honest by a passing test. Filed as agent-ix/quire-rs#146 with
  the design recorded; the *lookup* half ships here.

Closes agent-ix/quire-rs#82, #133, #134, #83.

## [0.28.0] — 2026-08-17

`archetype:` is the only trace-target origin. The `document:` path-binding form
is deleted. Cut so the ADR-0011 verification program (agent-ix/quire-rs#81) —
whose P1 track adds manifest keys under the engine-before-module rule — starts
from a released baseline rather than from unreleased `main`.

### Breaking

- **A module declaring `document:` on a trace target or a document reference no
  longer loads.** The nested structs are `deny_unknown_fields`, so the retired
  key is rejected rather than silently ignored: a module that has not migrated
  fails loudly instead of minting nothing. Migration is one line per
  declaration — replace the path with the `archetype:` that types the document,
  and scope fixture data with `exclude:`. `spec-artifacts-process` v0.14.0 ships
  the matching collapse, nine declarations to three.

### Changed

- **FR-050-AC-15 / AC-19 (CR-062)** — `archetype:` is the single required origin
  for a trace target and a document reference alike. The `document:` form
  existed for exactly one reason, recorded verbatim in `traceability.rs`:
  "`spec/tests.md` is on `DEFAULT_SKIP`, so archetype binding alone cannot see
  the file 184 repos call their Test Matrix." Type-driven corpus membership
  (#73, v0.26.0) deleted that premise, and what remained cost coverage, because
  path binding **enumerates**: the module declared three near-identical targets,
  one per filename the ecosystem happens to use, and reached nothing nested — a
  correctly authored matrix at `spec/<module>/matrix/tests.md` minted zero ids.
  TC-829 and TC-830 replace TC-802.

  **[RAN]** `scripts/sweep_coverage.py` over `~/dev`, 238 repositories,
  worktrees deduped, with the matching `spec-artifacts-process` collapse: dead
  trace tags fall from **1,401 occurrences / 1,052 distinct ids to 1,207 / 873**.
  The whole change is one repository — `filament-ide-rs`, **214 → 20** dead
  tags, rollup 17/850 → **473/2,184** rows backed — because it is the only
  repository authoring nested module matrices today, and the shape the ecosystem
  is moving toward, which is what makes enumeration the wrong contract rather
  than merely an inelegant one. Rebinding `test-case` alone leaves 49 dead tags
  there: `traces-to` and `functional-coverage` were path-bound too and could not
  read the nested matrices they describe.

- **`exclude:` is load-bearing, not optional.** Archetype binding is what lets a
  fixture matrix mint phantom ids — a fixture exercising the `TestMatrix`
  contract legitimately *is* `type: TestMatrix`. That concern kept path binding
  alive through CR-038 (67 phantom ids, 50 of them reported "backed"); it is
  answered by exclusion rather than by enumeration.

- **A mistyped matrix now mints nothing**, where under path binding frontmatter
  was irrelevant to minting. **[RAN]** 14 of 184 ecosystem matrices were untyped
  or mistyped, 6 of them real matrices carrying a Test Case Summary
  (agent-ix/quire-rs#75). Left alone this change would have taken repositories
  minting zero test-case ids from 154 to **159** — the exact regression #75
  existed to prevent. All six were corrected first and the sweep re-run:
  **153**, one better than the path-bound baseline. Zero matrices in the
  ecosystem are frontmatter-less, so the case that could have gone silently
  invisible is empty.

### Removed

- **`ScanContext::harvest`, the harvest cache, and `HarvestError`** — the
  off-corpus reader that existed only to serve path binding.
- **The `unreadable-declared-document` and `absent-declared-document`
  diagnostics.** CR-059 shipped them in v0.27.0 for the code path this release
  deletes — the right call for the interim, dead now. A minting document that
  cannot be read is the walk's `DocumentUnreadable` / `MissingUuid`, strictly
  better than the silent `None` the off-corpus reader returned.
  `archetype-matches-nothing` is the surviving reason, and a misspelled
  archetype the surviving shape of the same fault. TC-825 is retired with the
  split it pinned.

### Added

- `scripts/sweep_coverage.py` — the ecosystem dead-trace-tag sweep
  (agent-ix/quire-rs#78). It re-derives #72's numbers with a *released* engine
  and an *explicit* module path, the two things the 2026-08-14 measurement got
  wrong: an engine without CR-061 reads resolved tags as dead, and
  `~/.ix/filament/modules` lags the source tree. Dedupe rules match
  `scripts/classify_matrices.py` so the two sweeps count the same population.

Closes agent-ix/quire-rs#74.

## [0.27.0] — 2026-08-16

The three SR-007 blockers (CR-059..CR-061), reviewed as a stack by SpecReview
**SR-008**: measured against the real `spec-artifacts-process` module, diagnostics
went 6 → 0 with no status lies, TC-577 and TC-579 backed. Released so corpus
measurements run on an engine that binds leaf evidence — a sweep on 0.26.0
undercounts resolved trace tags (agent-ix/quire-rs#78).

### Added

- **FR-051-AC-17 (CR-061)** — `trace::bind` binds **leaf evidence**, not
  only test functions: `SymbolKind::Benchmark` (a `#[bench]`, or a
  function a `criterion_group!` registers) and `SymbolKind::FuzzTarget`
  (a `fuzz_target!` invocation, which declares no `fn` and so minted no
  symbol at all) now carry trace bindings. Containers and plain functions
  still bind nothing — a `mod tests` block must not inherit its members'
  markers, and production doc comments citing an acceptance criterion are
  not backing for it. TC-577 and TC-579 go 🚧 → ✅. TC-502 does **not**:
  a shell audit is never opened by the extractor, which reads
  `.rs`/`.py`/`.ts`. TC-827, TC-828.
- **FR-050-AC-13/15 (CR-060)** — a **model-level** `traceability.exclude:`,
  whose matching documents mint no trace ids, contribute no reference
  rows, and are not classified for criteria. (Document validation is
  unaffected — an excluded fixture is still schema-checked.) `exclude:` scoped the
  declarations but never the CR-028 criteria walk, which has no
  declaration to hang one on — so deliberately malformed fixture data
  inflated `totals.criteria` / `totals.property_shaped` and was
  body-parsed anyway. The new key scopes the criteria walk *and* every
  declaration, in addition to each declaration's own `exclude:`, and
  merges across modules as a union. Exclusion globs are now compiled once
  per model rather than per pattern per question. **Report change**: a
  repository declaring the new key with criteria under those paths sees
  smaller criteria totals. TC-826.

### Fixed

- **FR-050-AC-19 (CR-059)** — an **absent** declared auxiliary
  `document:` is no longer reported as an unreadable one. CR-054
  flattened `io::Error` at the point of the read, so `NotFound` — the
  ordinary case for an optional declaration a fleet module ships across
  200+ repositories — was indistinguishable from permission denied or a
  directory where a file was expected. Six such diagnostics fired on
  this repository's own spec. `absent-declared-document` is a new
  machine reason, reported only when the model minted nothing at all,
  the rule `archetype-matches-nothing` already used;
  `unreadable-declared-document` narrows to the always-wrong case and is
  still reported either way. `quire validate` reports the same two
  tokens. TC-825.

## [0.26.0] — 2026-08-16

The post-ship review of the #90 program (SpecReview **SR-006**, verdict
FAIL) landed nine changes. Two are behavior fixes on public API; the rest
are gates for claims that had none. Closes agent-ix/quire-rs#107, #108,
#109, #111, #112, #114, #115 and the engine halves of #110 and #113
(umbrella #106).

### Fixed

- **FR-005-AC-7 (CR-050)** — `parse_body` no longer panics when handed a
  `Header` parsed from a different string. `Header` is owned and stored
  beside an owned text, so the pair is constructible from safe, public,
  PyO3/wasm-reachable API, and the private body offset was sliced
  unchecked: out of bounds on a shorter string, char-boundary inside a
  multi-byte character. The offset is re-derived from the string actually
  given — one `is_char_boundary` on the correct path. TC-819.
- **FR-024-AC-12 (CR-051)** — the frontmatter-less bridge emits a
  **distinct machine reason** for the malformed flavor
  (`malformed-frontmatter`) instead of tagging both `no-frontmatter`. The
  engine had the distinction and the bridge dropped it, so only the human
  message differed. TC-820.
- **FR-050-AC-19 (CR-054)** — a declaration that selects nothing is now
  reported instead of failing open: an unreadable declared `document:`
  (previously `.ok()?`, swallowing every IO error), a declared archetype
  no document has when the model minted nothing, and a model with no
  trace targets. `CoverageReport.diagnostics` is absent when empty, so
  FR-050-AC-7 byte-identity holds. TC-822.
- **FR-044-AC-8 (CR-055)** — the glossary heading pre-filter matched the
  raw title verbatim while the lookup it gates normalizes ISO section
  numbering, so `## 3.2 Ubiquitous Language` stopped contributing terms
  and shrank the composed EARS lexicon in silence. TC-823.
- **(CR-056)** — the code walk's excluded subtree is compared by
  **identity**, not by exact path: on a case-insensitive filesystem
  `<scope>/Spec` satisfied the caller's `is_dir()` check while `==` never
  matched it, so every spec document was ingested a second time as
  source. Symlinked roots resolve too.

### Added

- **FR-005-AC-8 (CR-052)** — a checked-in golden corpus pins
  `parse_document` byte-for-byte against a snapshot captured from the
  engine **before** the CR-046 tier split. The current engine reproduces
  it exactly. TC-821.
- **FR-050-AC-20 (CR-057)** — the byte-identity property that CR-045,
  CR-047 and CR-049 rest their correctness argument on gets an actual
  gate: a fixture corpus exercising the whole reconciliation surface,
  its report checked in and byte-diffed, regenerated only by
  `make coverage-baseline-update`. TC-824.

### Changed

- **(CR-053)** — FR-024-AC-9's three compensating controls are enforced
  end to end. `check_no_shared_mutable.sh` joins the ci.yml
  `audit-static` job and `sanitize` joins `make hardening`; the audit
  matches exemptions by repo-relative path and exact source line, fails
  on a stale entry, prints every `why`, and covers `LazyLock`, `Cell`,
  `RefCell`, `thread_local!`, `static mut` and `unsafe impl Sync` across
  `src/corpus` **and** `src/python`. TC-816 widens to 8 threads × 16
  documents plus the rayon-forcing shape `python::load_repo` runs.
- **(CR-058)** — `spec/tests.md` reports zero status lies, down from ten.
  The AC→TC headline is reworded as **mapping** completeness, and
  performance criteria get one treatment across all user stories.
- Internal: `parse_header_status` removes a second copying frontmatter
  extraction from the walk, and `is_document` stops copying the body to
  answer a yes/no question (CR-046 leftovers).

## [0.25.0] — 2026-08-15

Bodies are lazy: the corpus parses headers at load and each document's
body on first touch, exactly once. Coverage parses only the archetypes
its model declares. And a frontmatter-less file inside the document root
warns instead of vanishing. Completes umbrella agent-ix/quire-rs#90.

### Added

- **FR-050-AC-18 (CR-049)** — coverage's body selection is
  declaration-driven: a corpus document whose archetype no trace target,
  document reference, or grammar binding names keeps its body
  unmaterialised through the whole rollup — selection decided on the
  header tier, never by filename, `exclude:` globs applying after
  selection — while the report stays byte-identical to a full-parse
  engine (AC-7 is the whole gate). Emergent from the CR-047 lazy tier: no
  new API, no mode flag. TC-818. (agent-ix/quire-rs#94, umbrella #90)

- **FR-024-AC-10 inverted (CR-048)** — a frontmatter-less `.md` under the
  walked root emits one non-fatal `Diagnostic::DocumentWithoutFrontmatter`
  naming its path (malformed-block flavor distinguished); exit code
  unchanged, the file still contributes nothing. CR-044's silence was
  justified only by tolerating the repo-root walk CR-045 removed — inside
  `spec/` a missing front block is almost certainly an authoring mistake,
  and silence made it a real error nobody ever saw. Never re-suppressed by
  filename. `validate_bundle` bridges the diagnostic into `BundleReport`
  warnings (reason `no-frontmatter`) in both postures, so `quire validate`
  shows it. TC-807 updated. (agent-ix/quire-rs#95, umbrella #90)

### Breaking

- **FR-025-AC-7..8, NFR-017-AC-4 (CR-047)** — `LoadedDocument.doc` (public
  field) is replaced by accessors: `raw()` (verbatim text), `frontmatter()`,
  `concept_type()`, `body()` (first-touch parse — exactly once, no filesystem
  read, concurrent first accessors receive the identical value), plus
  `from_parsed(path, id, uuid, doc)` for constructing one from an
  already-parsed `QuireDocument`. `load_repo` no longer parses bodies:
  `len`/`by_id`/`by_type`/`diagnostics` and the FR-026/027 edge queries
  complete with zero body parses. FR-024-AC-9 narrows to the walk fan-out;
  `check_no_shared_mutable.sh` widens its pattern to `OnceLock`/`OnceCell`
  and gains a named exemption list. The PyO3 `load_repo` binding is
  unchanged in shape (bodies are forced in parallel with the GIL released).
  TC-815..817. (agent-ix/quire-rs#93, umbrella #90)

## [0.24.0] — 2026-08-15

`validate_bundle` states the two roots separately.

### Breaking

- **FR-049-AC-9 (CR-045)** — `validate_bundle` takes the two roots
  separately: `document_root` (locates the root `index.md`) and
  `reference_root` (the base for model-declared `document:`/`exclude:`
  paths, which modules author against the repository scope). A corpus
  walked from `<scope>/spec` with one conflated root silently un-minted
  every path-bound trace target — 123 spurious `dangling-trace-reference`
  findings on this repo's own spec. `validate_bundle_at(root, …)` is
  unchanged (both roots = `root`). TC-814.

## [0.23.0] — 2026-08-15

The document walk is bounded to a caller-supplied root, and the parse
splits into a cheap header tier and an expensive body tier.

### Added

- **FR-005-AC-5..6 (CR-046)** — `parse_document` splits into two tiers:
  `parse_header(md) -> Option<Header>` (one frontmatter extraction, no body
  work, no input copy; `None` = not a document per the CR-044 rule) and
  `parse_body(md, &Header) -> QuireDocument`, with `parse_document`
  composing them — signature and outputs unchanged, pinned by a composition
  proptest. `Header` carries `id`/`type_`/`uuid` **and the full frontmatter
  map**. `walk::parse_one` decides membership + identity via `parse_header`,
  retiring `read_identity` and the duplicate post-parse `extract_frontmatter`
  from CR-044 — a non-document now costs one read and one failed fence check.
  TC-812, TC-813. (agent-ix/quire-rs#92, umbrella #90)

- **FR-050-AC-17 (CR-045)** — the walk is bounded to a caller-supplied
  **document root**. `quire coverage` derives two distinct roots from its one
  `--scope`: document root `<scope>/spec` for `Spec::from_path`, code root
  `<scope>` for symbol extraction, excluding the document root — documents are
  not source. New `symbols::extract_tree_excluding(root, exclude)`;
  `extract_tree(root)` is unchanged and equal to an empty exclusion. The
  phantom `[--source <DIR>]` flag is withdrawn from FR-050 — both roots derive
  from `--scope`, no manifest key, no flag. A scope with no `spec/` is a named
  diagnostic in the CLI, never a silent fallback to the wider tree. The
  repo-root crawl this fixes is what produced the 9,172 frontmatter errors
  across 223 repos that CR-044 silenced at the membership layer; they are now
  gone because the files are never visited, not because they were classified
  away. TC-809; TC-810/TC-811 land with the `quire-cli` two-root derivation.
  (agent-ix/quire-rs#91, umbrella #90)

## [0.22.0] — 2026-08-15

Corpus membership is type-driven. The filename skip list is gone.

### Breaking

- **`WalkOptions::skip_names` is removed.** It is `pub` and re-exported from the
  crate root. Nothing in `src/` constructed a non-default `WalkOptions`; the
  only in-crate consumers were `corpus/spec.rs` and `corpus/glossary.rs`, both
  using `Default`. A downstream crate that set `skip_names` should delete the
  field — the behavior it configured no longer exists.

### Added

- **FR-024-AC-10 (CR-044)** — a markdown file is a corpus document iff it
  carries a **frontmatter block**. Filename plays no part. `DEFAULT_SKIP`,
  `WalkOptions::skip_names` and `is_skipped` are deleted; a file with no
  frontmatter is dropped **silently**, with no diagnostic, which is what
  actually retires the `README.md` entry and generalizes to every stray `.md` —
  a CHANGELOG, an AGENTS file, a design note — without the engine knowing any of
  their names. Frontmatter present but naming an unregistered type is still a
  document, triaged by validation as before (error under `Strict`, warning under
  `Okf`). TC-807.

  The constant was never a decision. It began in
  `filament-parser-lib/filament_parser/loader.py` as a **graph-ingestion**
  filter — commit `1d17b6f`: the listed files *"validate via quire as their own
  archetypes but are **not graph nodes**"* — and quire-rs `8dc32a5` copied it
  into `load_repo`, a **validation** loader, where *"not a graph node"* became
  *"not a document."* The engine could not load the canonical instance of
  `TestMatrix`, a type its own module registers.

  **[RAN]** `scripts/classify_matrices.py` over `~/dev`, worktrees and
  `-task<N>` copies deduped: of 184 matrices at a bound path, **0 carry no
  frontmatter block**; 170 are typed `TestMatrix` and are unaffected; 14 are
  mis-typed (10 declaring `type: index` — those documents saying they are not
  matrices), of which 6 mint rows today. Against that, **20 real matrices across
  9 repos become visible for the first time**, 12 of them minting, in filename
  conventions no enumeration covered — `spec/test-matrix.md`,
  `spec/test_matrix.md`, `spec/traceability_matrix.md`, `spec/*/matrix/tests.md`.

- **FR-024-AC-11 (CR-044)** — `glossary_terms_from_path` applies the same
  membership rule. It scans raw text rather than building a `Spec` and inherited
  the skip through `discover_files`, so its scope would have silently widened to
  every stray `.md`, letting a file that is not a document define a repository's
  ubiquitous language. The rule now lives once, in `walk::is_document`. TC-808.

### Changed

- **`NON_ARTIFACT_FILES` reduced to `{index.md, log.md}`.** The `README.md`
  entry is permanently dead under the frontmatter rule. The `tests.md` entry
  would have become a live suppression of a genuine index gap: a `TestMatrix` is
  an artifact and an index that omits it is incomplete. **[RAN]** 4 of 180 repos
  with a `spec/tests.md` already list it in `spec/index.md`, so **172 now report
  `index-incomplete`** — authoring debt the suppression was hiding. FR-038 and
  FR-038-AC-5 updated to match.
- `tests/spec_dogfood.rs::spec_documents` goes through `load_repo` instead of a
  hand-rolled `read_dir` recursion. It walked by hand precisely so TC-794 could
  reach `spec/tests.md` — the type-driven rule, implemented in a test, as a
  workaround for the engine not implementing it. Third independent markdown
  walker in the tree; now the second.

Closes agent-ix/quire-rs#63, #73, #76, #77. Review: `SR-005`
(`reviews/2026-08-15-cr044-type-driven-membership.md`).

## [0.21.0] — 2026-08-14

A legacy trace comment carrying a list binds every id it carries.

### Added

- **FR-051-AC-16 (CR-043)** — a legacy textual form mints one `verifies`
  relation per trace id its match carries. `marker_ids` already comma-split a
  canonical marker's argument list, so `#[trace("TC-001", "FR-007-AC-1")]` bound
  both ids; `legacy_id` returned capture group 1 whole, so
  `// Trace: FR-001-AC-1, FR-001-AC-2` bound the first and silently dropped the
  rest. **[RAN]** 98 such lines across `~/dev`, worktrees and `-task<N>` copies
  excluded: **205 ids binding to nothing across 17 repos**, spanning every
  declared legacy shape and all three languages.

  A form declaring `id_format` is unchanged — `TC-{1}` renders over a function
  name, which cannot carry a list. One match is one authored line, so a listed
  match yields one rewrite suggestion naming all its ids rather than N
  conflicting single-id rewrites.

  **The engine half alone converts nothing.** `Trace:\s*(ID)` matches once and
  stops at the comma, so capture group 1 is already a single id. The declared
  patterns must widen their id group to a list — that half lands in
  `spec-artifacts-process` and is what turns the 205 into coverage.

  Closes agent-ix/quire-rs#68.

## [0.20.0] — 2026-08-14

Phase D groundwork: the symbol adapters stop losing whole files, and the
traceability model learns two things it could not express.

### Added

- **FR-050-AC-15 (CR-038)** — declared **path scoping**. A trace target or
  document reference may carry `exclude:` globs, and may declare `archetype`
  and `document` together. Scanning `spec-artifacts-process` by archetype had
  been minting 67 test-case ids from deliberately malformed fixtures and reading
  50 of them as *backed*, because a fixture reusing `TC-017` collides with the
  real one.
- **FR-050-AC-16 (CR-041)** — module-declared `no_source_symbol` verification
  methods. A row verified by an agent-behaviour eval or by inspection cannot
  carry a trace tag, so reporting it as a status lie asserts something its own
  declared method makes impossible. The exemption changes the verdict and never
  the facts: the row stays in `unbacked_rows` and the counts are untouched.
- **FR-051-AC-14 (CR-039)** — one lexer pass per file in the TypeScript adapter,
  replacing three functions that each re-derived comment/string/template state.
- **FR-051-AC-15 (CR-040)** — the same for Rust, taught raw strings (`r#"…"#`),
  lifetimes (`&'a str`), character literals and **nesting** block comments.
- **FR-019, FR-020, FR-022 (CR-042)** — the v0.2 block model is authored as
  requirements for the first time. It shipped without documents, which is how
  10 matrix rows kept claiming `apply_block_patch`, an API the render removal
  deleted.

### Fixed

- 33 of this crate's own source files — every one holding a `r#"…"#` JSON
  fixture — were rejected as `unbalanced braces` and yielded **zero** symbols,
  so every trace tag in them bound to nothing. Skipped files: 33 → 1.
- `flatten_into_registry` merged `vocabularies` field by field, silently
  dropping any key added after `test_type`.

### Removed

- **FR-021** (block edit API) is retired. Its whole surface was render-dependent;
  US-006/US-007's acceptance criteria had already been retired for that reason.

### Measured

On this crate's own matrix, with no row edited to get there:

```
status lies   140 → 7
backed rows   144/907 → 358/926
```

The 7 remaining are untaggable rather than overclaiming — four verified in other
repos, a criterion bench, a fuzz target, and one asserting an absence.

Matrix at 488/488. TC-801..TC-805 added.

## [0.18.0] — 2026-08-08

Phase B of the acceptance-criteria property-testing program (#17, #20).

### Added

- **Acceptance-criteria property classification** (FR-052): a second,
  orthogonal shape axis over the same `ac` binding. A closed
  `PropertyShape` enum under one fixed precedence, `{domain, precondition,
  oracle}` spans that are statement-relative and carry both byte offsets
  and their own text, `row_id` and a `signals` audit trail on each record,
  and a module `property_idioms` registry demoted to a **booster** so
  CON-4 keeps extraction coverage independent of it. Never a
  `GrammarFinding`, never addressable by a `grammar_severity` key, so
  `--strict` immunity holds by construction (CON-1). New exports:
  `classify_document_properties`, `classify_document_criteria`,
  `AcClassification`, `PropertyShape`, `PropertyIdioms`, `AcPropertyCounts`.
- **`Extraction`, the three-valued outcome** (FR-052-AC-16/17, CR-033):
  `extractable | candidate | not-extractable`, derived from `(property,
  extractable)` and feeding back into neither. `candidate` names a
  metamorphic label the structural pass did not corroborate — a generator
  MAY emit and MUST mark the test as requiring review. Closes #46.
- **Coverage criteria counts** (FR-050-AC-13): `CoverageReport.criteria`
  plus two `CoverageTotals` counts, emitted as an all-or-nothing pair so a
  JSON consumer never sees one without the other.
- **Recall widening** (CR-030): the universal determiner is read at two
  further bounded subject positions. One of three candidate widenings
  cleared the ≥85% precision gate fixed in advance; the other two were
  deleted rather than narrowed.
- **PyO3 `classify_properties`** with full field parity.
- **Dogfood gate** (FR-048-AC-11): this repo's own `spec/` is judged under
  the severity promotion its published module ships.

### Fixed

- Four `ac` checker defects the CON-1 promotion sweep exposed
  (CR-024/025/026): the pair idiom tied to its separator, `Then` counted
  outside a Given/When/Then criterion, a vacuous predicate firing on a
  common noun, and a backtick run masking only to its first tick.

### Changed

- `ac:vacuous-outcome` and `ac:non-singular` are promoted to `error` in
  the `spec-artifacts-iso` manifest (CR-027). **`DEFAULT_SEVERITY` is
  unchanged** — the engine still ships every `ac` check advisory.

## [0.1.0] — unreleased

The initial implementation pass. Lays down every layer of the v1 spec
(parse → query → load → render → extract → harvest) plus the hardening
scaffolding (fuzz, miri, mutants, audit, perf bands, parity).

### Added

- **Parser** (FR-005/006/007/008/009): `parse_document`,
  `extract_frontmatter`, `QuireDocument` / `QuireSection`. BOM-strip,
  CRLF-tolerant frontmatter, fence-aware ATX heading walk, byte-exact
  section slicing (no `.strip()`), ASCII-only slug-line IDs.
- **Query** (FR-010): `section`, `sections`, `parse_table`,
  `parse_tables`, `table_from_section`, `parse_bullet_list`,
  `extract_diagrams`, `search`. Regex driver compiles once via
  `OnceLock`. TS parity for case-insensitive heading match +
  section-number prefix stripping.
- **Loader** (FR-013/014): filesystem-first archetype loader.
  `IX_SCHEMA_PATH` env-var resolution, tilde expansion, canonical-path
  dedup, symlink-loop guard, file-not-dir / permission-denied
  diagnostics. `Registry::{load_from, load_strict, from_env,
  from_default}`. First-wins archetype + module collisions with
  shadow-queryable `archetype_in_module`. Per-archetype failures
  aggregate without aborting the load.
- **Schema validation** (FR-002): `apply_patch(archetype, current,
  patch)` deep-merges then validates the merged result. JSON
  Pointer → dotted field path conversion for NFR-005 error shape.
- **Render** (FR-001/004/017): strict `minijinja::Environment`,
  `{% include %}` / `{% extends %}` rejected at load time, `render`
  returns `RenderOutput { markdown, diagnostics }`, `render_by_name`
  + `render_with_env` entry points.
- **Schema surface** (FR-003): `Registry::schema_for(name)` returns
  the loaded JSON Schema verbatim. No `schemars` dep.
- **Extract / DSL** (FR-011/016/018): six Locator primitives,
  fallback chains via `Locator::Fallback`, single-yield + multi-yield
  evaluators, `emit_edges`. `per_match` Locators evaluate against
  the iteration unit's local scope. DSL structural validation at
  load time (mutually-exclusive `match`/`iterate_over`, unknown keys,
  missing `from:`).
- **Edge harvesting** (FR-015): `harvest_edges` walks structured
  `relationships:` block + 6 sugar fields in canonical order + DSL
  `emit_edges`. Targets normalized via `RelationshipResolver`
  (Identity / Mock / Ix-Uri reference impls). Dedup by
  `(source, type, target)` first-wins.
- **Error shape** (NFR-005): `QuireError` with 13 variants. Display
  strings carry variant name + load-bearing identifier; never leak
  serde / validator internal debug forms. `format_violation` truncates
  the observed preview at 80 chars on a char boundary.
- **Diagnostics** (FR-017): non-fatal `Diagnostic` enum + collector;
  `Diagnostics::by_kind` filter. Surfaced from `Registry`,
  `ExtractionResult`, `EdgeHarvest`, `RenderOutput`.
- **Tracing** (NFR-008): feature-gated `tracing` spans at every hot
  entry (`parse`, `render`, `apply_patch`, `extract`, `harvest_edges`,
  `load`). Zero cost when the feature is off.
- **Determinism** (NFR-006): proptest no-panic harness for
  `parse_document` (10 000 cases) and `apply_patch` (10 000 cases);
  byte-exact slice round-trip proptest (10 000 cases); 64-thread
  cross-thread `render` / `parse` / `harvest_edges` determinism tests.
- **Parity** (FR-012, StR-002, StR-003):
  - Parser parity: 88 tests transliterated from the TS + Py
    reference suites.
  - Render parity: 10 cases (8 ISO archetypes + 2 demo) compared
    byte-exact against Python+Jinja2 reference via the regen pipeline
    in `scripts/regenerate_parity_fixtures.sh`. CI fails on drift
    between regen and committed expecteds.
  - Real-document parser sweep: 61 markdown files across 4 corpora,
    asserting no panic + byte-exact stitch + well-formed slug IDs.
- **Perf gates** (NFR-001/002/007): criterion benches for render,
  parse, load, validator; `scripts/check_perf_regression.sh`
  enforces a 10 % band against the stored baseline (CI caches the
  baseline across runs).
- **Hardening** (NFR-011/012/013/014): 6 cargo-fuzz targets covering
  parse, frontmatter, apply_patch, DSL, manifest, schema. Weekly
  miri, mutants, fuzz; daily cargo-audit + on every PR. 6 static
  audit scripts (`check_no_net_deps`, `check_no_schemars`,
  `check_no_shellout`, `check_dep_pins`, `check_hashmap_audit`,
  `verify_cookiecutter_inheritance`).
- **Validator-crate ADR** (NFR-009): `spec/assets/adr/0001-validator-crate.md`
  decides `jsonschema ~0.18` with rationale and bench-baseline pointer.

### Notes for downstream

- Registry-shared clones: `Registry::clone()` is `Arc<Inner>`-cheap.
- `RelationshipResolver` is a trait object friendly bound; consumers
  can ship their own (e.g. an ix-cli-aware) impl.
- The crate ships zero `unsafe` blocks (NFR-003); the `audit-unsafe`
  baseline is empty.
- No network deps in `Cargo.lock` (NFR-013 / FR-013-AC-6, enforced by
  `check_no_net_deps.sh`).
