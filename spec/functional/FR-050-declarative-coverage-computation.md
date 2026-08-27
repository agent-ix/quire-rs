---
id: FR-050
title: "Declarative Coverage Computation"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-017"
    type: "implements"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-051"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-027"
    type: "requires"
    cardinality: "1:1"
---
# FR-050: Declarative Coverage Computation

## Description

`quire-rs` SHALL compute a deterministic requirement-coverage rollup — the
FR→AC→TC→test reconciliation the `gap-analysis` workflow currently performs by
grepping — and `quire-cli` SHALL expose it as `quire coverage`.

The engine SHALL NOT hardcode FR/AC/TC semantics. Coverage is driven by a
**declarative traceability model** that a Filament module declares in its
`manifest.yaml` under a `traceability:` section — the same
spec-semantics-as-module-data pattern as `body_extraction`, `lint_rules`, and
`grammar_ref`. The model SHALL declare:

- **Trace targets** — which archetype + section + table + id column mints
  trace ids (e.g. the FR `Acceptance Criteria` `ID` column via its existing
  `id_pattern`), including targets minted by an auxiliary trace source outside
  the corpus walk (e.g. Test Matrix rows in `spec/tests.md`).
- **Document references** — which columns or annotations reference which
  target kinds (e.g. the `Verification` cell annotation `Test (TC-nnn)` → TC
  targets; the matrix `Traces To` column → requirement/AC targets). These
  declarations also drive [FR-049](./FR-049-verification-reference-integrity.md).
  A declaration MAY additionally request two normalizations of the cell before
  ids are extracted, both off by default (CR-015): `expand_ranges` turns a
  same-prefix range (`FR-001..FR-006`) into its concrete ids, and
  `strip_annotations` drops parenthetical spans so a qualifier
  (`FR-022-AC-5 (superseded by FR-030)`) contributes one reference, not two.
- **Status vocabulary** — the matrix status column and which values class as
  `complete`, `pending`, `failed`, or `retired`. A cell SHALL class by its
  **leading marker**, so an authored qualifier (`✅ Complete`,
  `⚠️ scale evidence deferred`) classes correctly while keeping the note that
  carries why (CR-015).
- **Column vocabularies** — a `vocabularies:` block declaring the values a
  matrix column admits (first entry: `test_type`). The engine SHALL treat these
  as the single source for both validation and the rollup, so a module's
  contract and its coverage computation cannot disagree about what a column
  means (CR-015).
- **Trace-tag grammar** — a reference to the source-tag patterns
  ([FR-051](./FR-051-source-symbol-extraction.md)) that bind source symbols to
  trace ids.

Given a loaded `Spec` corpus, a `Registry` with a declared model, and a symbol
graph from [FR-051](./FR-051-source-symbol-extraction.md), the engine SHALL
reconcile declared targets, declared references, and scanned source tags
generically, and SHALL emit a machine-readable report containing:

1. **Unbacked rows** — declared reference rows (e.g. matrix test cases) whose
   trace target has no backing `verifies` relation from any source symbol.
2. **Status lies** — rows whose status classes as `complete` while their
   target has no backing source symbol.
3. **Untracked symbols** — source symbols carrying a trace tag that resolves
   to no declared target or reference row.
4. **Per-target-group counts** — for each minting document (e.g. each FR),
   backed and total trace-target counts.

### Two roots, one scope (CR-045)

`quire coverage --scope <DIR>` SHALL derive **two distinct roots** from its
single scope and never interchange them:

- **document root** — `<scope>/spec`, the only tree walked for corpus
  documents (`Spec::from_path`); the walk never leaves it ([FR-024](./FR-024-parallel-repo-walk.md)).
- **code root** — `<scope>`, the tree scanned for source symbols
  ([FR-051](./FR-051-source-symbol-extraction.md)), **excluding the document
  root** — documents are not source.

A scope with no `spec/` directory SHALL exit with a diagnostic naming the
missing document root — never a silent fallback to walking the scope itself,
which is how the repository-wide crawl survived. `spec/` is convention, not
configuration: no manifest key and no flag relocates it. Report paths remain
relative to `<scope>`, so minted `document:` paths keep their `spec/` prefix
and reports over a compliant repo are byte-identical across the split.

### Declaration-driven body selection (CR-049)

The `traceability:` model is a projection stated before the walk begins —
*these archetypes, these sections, these columns*. The engine SHALL honor it
as a **bound on what is parsed**, not just a filter on what is reported:
during coverage computation, a corpus document whose archetype no trace
target, document reference, or grammar binding names has its body left
unmaterialised ([FR-025](./FR-025-spec-corpus-model.md) lazy tier). Selection
is decided on the header tier (frontmatter `type`), **never by filename**
(CR-044), and `exclude:` globs apply *after* archetype selection, not
instead of it. A declared archetype whose document lacks the declared
section (e.g. a root index-of-matrices `TestMatrix` with no
`## Test Case Summary`) is legal and simply mints nothing. A caller that
needs every body — `quire validate`'s structural pass — asks for every
body; the point is that it *asks*.

`quire coverage [PATHS] --scope <DIR>` SHALL print the
report as JSON on stdout; repeated runs over identical inputs SHALL emit
byte-identical output (NFR-006 ordering discipline). When the active modules
declare no traceability model, the command SHALL exit with a distinct
diagnostic instead of an empty report.

A module other than `spec-artifacts-iso` obtains coverage by declaring its own
model; the engine knows nothing of "AC" or "TC" as concepts.

The coverage report is an input to the ecosystem partition specified by
[FR-066](./FR-066-gap-disposition-census.md). That census deliberately uses
authored rows rather than `totals.total` as its denominator, so a declaration
that failed to mint cannot erase its own gap from the population.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-050-CON-1 | Verdict policy (PASS/CONDITIONAL/FAIL), review gating, and SpecReview authoring SHALL remain outside quire — the `gap-analysis` workflow consumes the report and owns judgment. | Architecture | Inspection |
| FR-050-CON-2 | The coverage computation SHALL perform no network or service I/O; inputs are the corpus, the registry, and local source trees. | Architecture | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-050-AC-1 | A manifest `traceability:` section declaring trace targets, document references, a status vocabulary, and a trace-tag grammar loads, and the `Registry` exposes the declared model. | Test (TC-732) |
| FR-050-AC-2 | A malformed `traceability:` section fails module load like any other manifest shape error; an absent section loads and marks the model undeclared. | Test (TC-733) |
| FR-050-AC-3 | A reference row whose trace target has no backing `verifies` relation appears in the report's unbacked rows with the row id and its target id. | Test (TC-734) |
| FR-050-AC-4 | A row whose status classes as `complete` with no backing source symbol appears in the report's status lies; the same row with a backing symbol does not. | Test (TC-735) |
| FR-050-AC-5 | A source symbol whose trace tag resolves to no declared target or row appears in the report's untracked symbols with its file and symbol name. | Test (TC-736) |
| FR-050-AC-6 | The report carries per-minting-document backed/total counts, and their sum equals the bundle-wide totals. | Test (TC-737) |
| FR-050-AC-7 | Repeated `quire coverage` runs over identical corpus, model, and source inputs emit byte-identical JSON. | Test (TC-738) |
| FR-050-AC-8 | A fixture module with a non-ISO vocabulary (different archetype, id pattern, and status values) obtains a correct rollup from its own declaration, with no engine change. | Test (TC-739) |
| FR-050-AC-9 | When no active module declares a traceability model, `quire coverage` exits non-zero with a diagnostic naming the missing declaration. | Test (TC-740) |
| FR-050-AC-10 | A status cell carrying a trailing note (`✅ Complete`) classes by its leading marker, and a value declared in the `retired` class classes as retired rather than unknown. | Test (TC-758) |
| FR-050-AC-11 | A declared `vocabularies.test_type` is exposed on the `Registry` as the core values plus the module's extensions, and is the same list a matrix contract validates against. | Test (TC-759) |
| FR-050-AC-12 | With `expand_ranges` declared, `FR-001..FR-003` resolves as three references; with `strip_annotations` declared, `FR-022-AC-5 (superseded by FR-030)` resolves as one. Both are off unless declared. | Test (TC-760) |
| FR-050-AC-13 | A report over a corpus whose documents carry criteria contains a `criteria` entry per contributing document and the two new totals; a corpus whose documents carry none contains an empty `criteria` list and serializes byte-identically to a report from an engine that predates the field. A document the model-level `exclude:` matches contributes no entry, is counted in neither total, and is not body-parsed. | Test (TC-788, TC-826) |
| FR-050-AC-14 | A declared model that mints zero trace targets is reported distinctly from full coverage — never as `100%` — and `quire coverage --strict` exits non-zero on it. | Test (TC-797) |
| FR-050-AC-15 | A trace target or document reference declares exactly one origin, `archetype:`, and MAY declare `exclude:` path globs; excluded documents mint no ids and contribute no reference rows. The model MAY additionally declare a model-level `exclude:` whose matching documents mint no trace ids, contribute no reference rows, and are not classified for criteria; it scopes every declaration in addition to that declaration's own, merges across modules as a union, has its patterns compile-checked like any other, and leaves the model undeclared when it is all a module declares. Document validation is unaffected. | Test (TC-801, TC-826, TC-829) |
| FR-050-AC-16 | A module MAY declare which test-type values mint no source symbol; an unbacked row carrying one is reported as a no-symbol row rather than a status lie, stays listed as unbacked, and a module declaring none reports exactly as before. | Test (TC-805) |
| FR-050-AC-17 | The two roots derive from one `--scope` and stay distinct: repo-root files (`README.md`, `CHANGELOG.md`, `plan/*.md`) are never read as documents, the code walk never enters the document root, the minted-id set over a compliant repo is byte-identical to a pre-split run, and a scope with no `spec/` directory exits with a diagnostic naming the missing root (CR-045). | Test (TC-809, TC-810, TC-811) |
| FR-050-AC-18 | During coverage computation, a corpus document whose archetype no trace target, document reference, or grammar binding names has its body left unmaterialised; a declared archetype's body is parsed; selection is decided on the header tier and never by filename; a module declaring no `traceability:` model still errors (`ModelUndeclared`) before any selection; and the report is byte-identical to a full-parse engine's (CR-049). | Test (TC-818, TC-738) |
| FR-050-AC-19 | A declaration that selects nothing is reported in `CoverageReport.diagnostics` and as a `quire validate` warning, never in silence: a declared `archetype:` no corpus document has is reported when the model minted no id at all, and a model declaring no `trace_targets` is reported as minting nothing. `quire coverage` and `quire validate` report the same machine token for the same finding. The list is empty — and the key absent — for a model whose declarations select, so FR-050-AC-7 byte-identity holds (CR-054, amended CR-059, narrowed CR-062). | Test (TC-822) |
| FR-050-AC-20 | The byte-identity property is gated by a checked-in baseline, not by inspection: a fixture corpus exercising minted ids, an auxiliary matrix, an `exclude:` glob, all three status classes, an undeclared status value (CR-083), the `no_source_symbol` exemption, an untracked symbol, a dangling reference, an undeclared archetype and criteria classification has its report stored as `tests/fixtures/coverage_baseline/expected.json` and byte-diffed on every test run. Regeneration is a deliberate act (`make coverage-baseline-update`) whose diff is reviewed, and a companion test fails if the corpus stops exercising any of that surface (CR-057). | Test (TC-824) |
| FR-050-AC-21 | A reference row whose authored status value the declared `traceability.status` vocabulary classes as none of `complete`, `pending`, `failed` or `retired` is reported in `CoverageReport.undeclared_statuses` with the declaration, the document, the row id and the authored value verbatim — whether or not the row is backed. A corpus whose every status value is declared reports an empty list, the key is absent from the JSON, and the payload is byte-identical to a report from an engine predating the field. The list does not affect `totals`, and `--strict` does not gate on it (CR-083). | Test (TC-941, TC-942, TC-946) |
| FR-050-AC-22 | The model MAY declare `source_exclude:` path globs under the **code** root; a source file matching one yields no symbols and no trace bindings, a non-matching glob leaves the extraction byte-identical, and the document walk's `groups` and `totals` are unaffected either way. The key merges across modules as a union, has its patterns compile-checked at module load like every other glob list, and leaves the model undeclared when it is all a module declares. It can only subtract: a `source_exclude` of `spec/**` neither un-excludes the document root nor admits anything under it (CR-085). | Test (TC-944, TC-945, TC-949) |
| FR-050-AC-23 | A trace id that is the row id of a **status-carrying** reference row and is bound by more than one distinct source symbol — distinctness is the `(path, symbol)` pair — is reported in `CoverageReport.shared_trace_ids` with the id and every binding symbol, deterministically ordered by id and inside each record by `(path, symbol)`. An id whose rows carry no status (an acceptance criterion verified by several tests) is never reported. A corpus whose every status-row id is uniquely bound reports an empty list, the key is absent from the JSON, and the payload is byte-identical to a report from an engine predating the field. The list does not affect `totals`, and `--strict` does not gate on it (CR-087). | Test (TC-950, TC-951) |
| FR-050-AC-24 | The number of source files a declared `source_exclude:` glob removes from the symbol walk is counted and carried into the coverage report as `excluded_source_files`; a report over a model declaring no `source_exclude`, or one whose globs match nothing, omits the key entirely — never `0` — and serializes byte-identically to a report from an engine predating the field (CR-088). | Test (TC-952, TC-953) |
| FR-050-AC-25 | A `source_exclude` list containing a pattern that does not compile never partially filters: module load rejects it with an error naming `source_exclude` as the key at fault, and an extraction invoked with globs that bypassed model validation applies **no** glob at all and surfaces a diagnostic naming the offending pattern (CR-088). | Test (TC-954, TC-945) |
| FR-050-AC-26 | Every row-shaped coverage record — `unbacked_rows`, `status_lies`, `no_symbol_rows`, `undeclared_statuses` — carries the 1-based document line (frontmatter included, the numbering `validate` findings use) of the matrix row it came from, with two unbacked rows in one document reporting different lines; `untracked_symbols` carries the tagged symbol's declaration line. The contract's `line` keys are optional and omitted — never `null` — when unrecovered, so a payload from an engine predating them still conforms and a conformant reader of the prior schema is unbroken (CR-089). | Test (TC-955, TC-956, TC-957) |

| FR-050-AC-27 | The coverage report carries the FR-051-AC-19 binding census as `binding_census`, **unconditionally** — present whenever the code walk found at least one evidence symbol, whether or not anything bound, so a reader can tell a healthy premise from a hollow one without waiting for a failure. A language with candidates and zero bound is additionally reported in `diagnostics` under `no-symbol-bound`; a language binding a smaller fraction of its candidates than MP-201's observation boundary is reported under `low-symbol-binding` with both counts and explicit uncertainty rather than a diagnosis. Both records name the language in `value`, declare `traceability.trace_tags`, and name every form that was consulted. A language at or above the boundary is reported in the census and in no diagnostic. Neither record affects `totals`, and `--strict` does not gate on them (CR-093, CR-135). | Test (TC-983, TC-984) |

| FR-050-AC-28 | The criteria rollup carries the FR-052-AC-18 split: `totals.specific_shaped` alongside `criteria` and `property_shaped` as an all-or-nothing triple, a per-document `specific_shaped` count absent when zero, and a per-document `grounding` map giving, per shape label, how many of its records carry `domain` / `precondition` / `oracle` and how many carry all three. `coverage.specific_shaped` is emitted as its own FR-063 metric over the same denominator as `coverage.property_shaped`, so the two are comparable and the honest figure is findable by name (CR-095). | Test (TC-989) |
| FR-050-AC-29 | A declarative corpus case is data: `{name, issue_ref, tags, input, expect}`, where `input` is a whole miniature repository (module manifest, spec documents, source files) and `expect` names only the facts the case is about. One parameterized test runs every case; each case is uniquely named, carries at least one tracking id, and carries an `issue_ref` naming the filing it regresses. Two runs of a case produce byte-identical reports. `expect` can assert diagnostics that must be **absent** as well as present. | Test (TC-992..TC-996) |
| FR-050-AC-30 | A corpus benchmark scores declared metrics against checked-in baselines with **ratchet** semantics: better rewrites the baseline, equal holds, worse fails naming the metric and both values. Every metric declares unit / population / method / direction, and a number outside the dictionary is refused. A `gate-zero` metric never ratchets — it is a gate with no tolerance. A corpus entry that cannot be read is **skipped loudly**, and a metric the payload cannot supply is **omitted with its reason**, never scored zero. A tier-2 entry declares a pinned SHA and a run against a different tree is refused. Reports are deterministic. | Test (`scripts/tests/test_bench.py`) |
| FR-050-AC-31 | A cross-corpus sweep snapshots per-repository numbers across the enumerated ecosystem and compares two snapshots as a **distribution**: improved / regressed / unchanged counts, the net gain, and the fraction of that gain contributed by a single repository. A repository the engine cannot read is recorded as unreadable rather than scored zero; a population that moved between snapshots is reported, and the comparison is over the intersection. Gains and regressions are counted separately and never netted into one number. | Test (`scripts/tests/test_overfit_check.py`) |
| FR-050-AC-32 | The benchmark corpus spans **every language the binder binds**, and a corpus entry may declare which metrics it is scored on so it can be carried for language coverage without its content churn thrashing a ratchet. `skeptic.suspicion_rate` — suspicions over evidence symbols examined — is scored per entry: a rate near 100% is a rule misreading a language, not a corpus full of vacuous tests. | Test (`scripts/tests/test_bench.py`) |
| FR-050-AC-33 | A **trace target** that selects a document by archetype and then reads nothing out of it is reported per document, never in silence, under two distinct machine tokens: `section-matches-nothing` when the declared `section:` heading is absent, and `id-column-matches-nothing` when the section is found and the declared `id_column` is not among the table's headers. Each record names the document in `path` and names, in its message, both the value **found** and the value **declared**; the section record additionally names the `id_column` it could **not** check, because the absent heading strands the table before the column is read. Neither is gated on whether any other declaration minted, so a model minting its criteria normally still reports a stranded matrix. A reference declaration — whose section is legitimately optional — and a document carrying the declared heading and column both report neither (CR-117). | Test (TC-1033, TC-1034, TC-1035) |
| FR-050-AC-34 | A declaration's `section:` accepts **one heading name or several**, on that one key: a scalar as before, or a sequence. Every named section of a document contributes its own table, in **document order**, and each is checked for the declared `id_column` separately. An entry containing `*` matches any run of characters, including none; an entry containing none is the heading exactly, matched as `query::section` has always matched it — and no other metacharacter is introduced. A one-name declaration selects the same headings it always did, with one measured exception: where a document repeats that heading, every occurrence now contributes where the first alone used to — `tables_of` walks all matching sections while `query::section` returned the first. Measured: **0 of 393** TestMatrix documents repeat the ecosystem declared heading, so the ecosystem is unaffected, but it is a behaviour change and is stated as one rather than as an identity. An empty sequence, or a blank entry, fails module load naming the declaration. When no named section is present the `section-matches-nothing` record names **every** section the declaration tried, and a one-name declaration round-trips back out as the scalar it was authored as (CR-118). | Test (TC-1037, TC-1038) |
| FR-050-AC-36 | A declared **trace target** whose archetype names no document in the corpus is reported as `archetype-matches-nothing`, naming the declaration and the archetype, **whenever it happens** — never suppressed because a different declaration minted. The record names no document, because there is no file to open. A **reference** declaration is not reported: its section is legitimately optional, the same distinction that keeps `section-matches-nothing` off healthy repositories. The payload shape is otherwise unchanged — a target that matched nothing is still absent from `groups` rather than present with a zero total. | Test (TC-1048, TC-1049) |
| FR-050-AC-37 | An id in `untracked_symbols` and an id in `unbacked_rows` that differ only in **zero-padding, letter case or separator** are reported as one `untracked-id-near-miss`, naming **both** spellings and both loci. Zero-padding, case and separator are one class and collapse onto one key, so `TC-1`, `tc_001` and `TC-001` compare equal while `TC-001` and `TC-010` do not. An **exact** match is not reported — the id bound and its row went unbacked for some other reason — and an id matching no row at all is not reported either. No count moves: the two halves were already in the payload, and what is new is the join. | Test (TC-1050, TC-1051) |
| FR-050-AC-38 | The report carries one `minted_targets` record per minted row with its id, target declaration, document, 1-based row line and backed state, deterministically ordered by target, document, id and line. The record count equals `totals.total` and the records whose state is backed equal `totals.backed`. An empty minted population omits the additive field under FR-055-CON-3 rather than inventing rows; a current CLI advertises the `minted_targets` capability so a census can refuse an older payload instead of inferring row state from `unbacked_rows`, which contains only document-reference rows (#361). | Test (TC-1073) |
| FR-050-AC-39 | The report carries one `unmatched_tags` record for every generic trace-id token in an evidence symbol's attached annotation block that no declared form bound **on that symbol**. Each record names the trace id, language, repo-relative path, 1-based annotation line and qualified symbol, ordered deterministically by language, path, line, symbol and id. A bound id is absent while an unmatched sibling id on the same otherwise-bound symbol remains; id-shaped text appearing only inside the symbol body is absent. An empty population omits the additive field, and a current CLI advertises the `unmatched_tags` capability so a consumer can join authored-but-unread ids to matrix rows without reimplementing the engine's annotation parser (#362). | Test (TC-1074) |
| FR-050-AC-40 | A trace target MAY declare `evidence: reference-only`; omission is exactly the existing `source` posture. A reference-only target still scans and registers every id for declared-reference resolution and dangling-reference checks, but its rows enter no coverage group, total or `minted_targets` record, and a source tag naming one cannot manufacture backed coverage. The posture is module data rather than an engine-known archetype or id prefix, an unknown value fails module load, and the default serializes as it did before the field existed (#363). | Test (TC-1075) |
| FR-050-AC-35 | Where every declared section a document **has** holds no table, the declaration reports `section-holds-no-table` naming the document, the sections it matched and the sections it declares. One table-less section among others is not reported — a parent heading whose rows live under its sub-headings is ordinary. The record exists because that shape mints nothing while both sibling diagnostics stand down: the section **was** found, so `section-matches-nothing` cannot fire, and there is no table, so `id-column-matches-nothing` has no headers to read (CR-120). | Test (TC-1041) |




> **CR-117 note (2026-08-24):** AC-33 is new — the archetype matched and the
> declared table did not. `agent-ix/quire-rs#270`; epic
> `agent-ix/quire-rs#264`.
>
> **The dominant ecosystem failure had no token at all.** `ScanDiagnostic`
> carried exactly one reason, `archetype-matches-nothing`, for the case where a
> declaration names an archetype no document has. The far more common case —
> the archetype matches, the document is selected, and the declared `section:`
> is one word off — went through `rows_of`'s `let Some(sec) = … else { return
> Vec::new() }` and produced **nothing**: no finding, no path, no message. The
> only symptom was a smaller denominator, which reads as a repository with
> fewer tests.
>
> **[RAN]** across 239 repositories: **3,514 TC ids in 88 repositories** mint
> nothing for this reason. Those repositories report **6.77%** of rows backed
> against **32.55%** for repositories whose heading matches — so the defect
> does not merely hide ids, it makes the repositories carrying it look like the
> repositories doing the least testing.
>
> **Two tokens, not one, and the reason is measured.** The two faults were
> first reported as producing indistinguishable payloads. Diffed key by key,
> they differ in exactly one field: a wrong **section** strands the whole table
> and `unbacked_rows` is empty, while a wrong **id column** reads the table and
> mints a row whose `row_id` is `null`. `totals`, `groups`, `diagnostics`,
> `binding_census`, `metrics` and `criteria` are byte-identical. The one field
> that differs is not on any coverage summary. So a single "something matched
> nothing" sends a reader of `agent-ix/identity` — where both faults sit on one
> document — to edit the heading, and leaves all 606 ids stranded.
>
> **Scoped to trace targets, and that is load-bearing.** A *reference*
> declaration's section is legitimately optional: the ecosystem's
> `functional-coverage` reads `## Functional Requirement Coverage`, which the
> matrix template emits only when it has content. Diagnosing its absence would
> fire on every well-formed Test Matrix in the corpus — the failure mode that
> killed two diagnostics during CR-094. A trace target's section is not
> optional: it is the whole of what the declaration selects the document for.
>
> **Not gated on `minted_anything` — and since CR-135 neither is
> `archetype-matches-nothing`.** A model-wide gate suppresses one declaration's
> finding because a *different* declaration succeeded, which is exactly
> `agent-ix/identity`: its FR criteria mint normally while 606 TC ids strand.
> The gate was written because a model legitimately declares archetypes an
> individual repository has no instance of; measured across 245 repositories,
> that reasoning cost the `test-case` signal in 57 of them to spare noise in
> the rest, and the noise is itself a declaration-side fact
> (`agent-ix/spec-artifacts-process#75`) rather than an engine one.
>
> **The section message names the column it could not check.** The wrong
> heading strands the table before the `id_column` is read, so on a document
> carrying both faults the column fault is *unobservable*. A reader told only
> about the heading fixes it, re-runs, and meets a second fault that was there
> all along. Naming the unchecked column is what makes that one pass instead of
> two.

> **CR-118 note (2026-08-24):** AC-34 is new — one heading name reached one
> heading. `agent-ix/quire-rs#272`; epic `agent-ix/quire-rs#264`.
>
> `TraceTarget.section` was `String`. A matrix that groups its rows under
> several headings minted only the group under the one declared name, so the
> denominator silently became *the rows under one heading* rather than *the
> rows this document declares* — and a repository in that state reports
> flawless minting health over the fraction it can see.
>
> **A pattern, not a list of names, and the reason is measured.** **[RAN]**
> over the 393 `type: TestMatrix` documents in `~/dev`: 434 test-case ids sit
> in a `Test ID` table the ecosystem's `test-case` target cannot reach, and
> **306 of them are under a heading that CONTAINS `Test Case Summary`** —
> `Test Case Summary (plugin scope)`, `Phase 4 Test Case Summary`,
> `Test Case Summary — packages/elements`. Those qualifiers are per-repository
> and per-phase, so the enumeration `#272` sketched
> (`["Test Case Summary", "Test Cases", "Integration Test Matrix"]`) reaches 14
> of the 434 and goes stale the day somebody writes `Phase 5`. The sequence
> form is kept because a differently-named table is a different claim and reads
> better as a name.
>
> **One key, and `*` is the only metacharacter.** A second `sections:` key
> would be two spellings of one thing under `deny_unknown_fields`, and every
> reader would have to handle both.
>
> A *glob* would make `?`, `[`, `]`, `{` and `}` special. Measured: **21**
> distinct `section:` values are declared across every `manifest.yaml` under
> `~/dev` and **none carries a glob metacharacter**, and of the 2,802 headings
> in 417 `type: TestMatrix` documents exactly **one** carries `[`/`]` (a
> markdown link) while `?`, `{`, `}` and `*` appear in none. So globset would
> not change the meaning of any declaration that exists today — this is a
> hazard the design forecloses, not one it was observed to hit, and it is
> claimed as no more than that.
>
> An earlier draft of this note cited `Edge Cases [deferred]` as a real
> ecosystem heading. **It is not.** It appears nowhere in `~/dev` and was
> invented to support the conclusion. The conclusion survives on the census
> above; the example did not exist.
>
> A name carrying no `*` is the heading exactly, which is what makes "a target
> declaring one section does not start matching others" a property of the
> design rather than a test.
>
> **The reference declaration widens with the target.** `traces-to` reads
> `Traces To` off the rows `test-case` mints. A section the target reaches and
> the reference does not is a row whose id exists and whose stated coverage
> nothing reads, so the criteria it answers for report unreferenced while the
> matrix looks healthy. Both take `SectionNames`; so does an obligation
> source's, which scans the same declared tables.
>
> **The realised delta is 42× smaller than the estimate, and the reason is the
> id column.** `#272` predicted Δtotal ≈ +3,514 rows and Δbacked ≈ +455, from
> CR-117's census of ids stranded by an absent section. **[RAN]** over the 241
> repositories `scripts/corpus.py` enumerates, before and after, with the same
> binary and only the declaration changed: **total 19,938 → 20,021 (+83)** and
> **backed 5,036 → 5,037 (+1)**, moving row backing 25.26% → 25.16%. Two
> repositories moved; a third, `filament-ide`, holds 221 more but is
> `SUPERSEDED` and correctly outside the population.
>
> The estimate counted ids whose section is absent. Minting one needs the
> declared `id_column` too, and the ecosystem's stranded rows overwhelmingly
> fail on **both**: 5,732 ids across 120 repositories sit under a non-declared
> heading in a table whose id column is `Test Case ID`, `Test Case` or `ID`.
> `agent-ix/identity` — the ticket's own 606-id example — was measured with
> `section: "*"`, matching every heading in the document: it mints **zero**
> test-case ids and reports 33 `id-column-matches-nothing` findings instead.
> Not one of its ~30 headings carries a `Test ID` column, so no widening of
> `section:` can reach a single one of those ids. That is precisely the second
> pass CR-117's message was written to warn about, now walked and counted; the
> remaining population is an `id_column` ticket, not this one.

> **CR-101 note (2026-08-22):** AC-31 is new — the overfit check.
> `agent-ix/quire-rs#237`; epic `agent-ix/quoin#197`.
>
> An improvement tuned against `filament-ide-rs` might be an improvement to the
> **engine**, or an improvement to `filament-ide-rs`. One corpus cannot tell
> those apart, and the whole metric-integrity programme would be worth little
> if its gains turned out to be one repository's.
>
> **A distribution, not an average.** The statistic that answers the question is
> *concentration*: what fraction of the total gain came from a single
> repository. A change lifting four repositories by five rows each and a change
> lifting one by twenty produce the same ecosystem total and mean opposite
> things. The script reports the number and names the repository; it does not
> pass or fail on it, because whether concentration is overfitting depends on
> what changed and a script cannot know that.
>
> **Gains and regressions are never netted.** A change that lifts most
> repositories while breaking one is a different fact from one that lifts them
> all, and a single signed total erases it.
>
> **A moving population is reported.** A sweep that silently shrank its own
> population would show every remaining repository improving — so the
> comparison is over the intersection and says how many entries dropped or
> appeared.
>
> **Unreadable is not zero.** A repository the engine could not read is recorded
> as unreadable, for the reason this entire programme exists: scored as 0 it is
> indistinguishable from a repository with nothing in it.
>
> `workflow_dispatch` only. 241 repositories is minutes of work, and a gate that
> runs on every push is a gate somebody disables.

> **CR-099 note (2026-08-22):** AC-30 is new — the corpus benchmark.
> `agent-ix/quire-rs#231`, implementing the engine half of `agent-ix/quoin`
> FR-043; epic `agent-ix/quoin#197`.
>
> `coverage_baseline`'s byte-diff pattern extended from one fixture to the whole
> corpus. It **extends** the existing sweep tooling rather than replacing it:
> `sweep_coverage.py` and `ac_corpus_sweep.py` still do the walking; what is new
> is the pinned manifest, the score report and the ratchet.
>
> **Ratchet, not threshold.** A hand-picked threshold invites the number to be
> tuned to it — which is how `ac:unclassifiable` came to pass 99.2% of corpus
> cells (CR-019). A baseline moves only through `make bench-update`, whose diff
> belongs in the pull request.
>
> **Three refusals, and each is the point.** A corpus entry that cannot be read
> is skipped loudly; a metric the payload cannot supply is omitted **with its
> reason**; a run that can score nothing at all exits non-zero. Scoring any of
> those as `0` would reproduce the silent-zero defect *inside the thing built to
> catch it*.
>
> The two absences are distinguished, because they mean different things: a
> missing `binding_census` key means the engine predates FR-050-AC-27 and the
> score would be measuring the toolchain's **version**; an empty one means the
> corpus genuinely has no evidence symbols.
>
> **The sentinel does not depend on the code path it checks.** `hollow-denominator`
> is the engine's own report (FR-063-AC-5); the sentinel counts hollow metrics
> the engine did **not** flag, so a regression in that diagnostic cannot hide
> itself.
>
> **It caught two things on its first live run, both real.** The pinned tier-2
> corpus had moved — `filament-ide-rs` is at `16eca41`, not the adjudicated
> `fc5d644` — and the SHA gate refused to score it rather than answering a
> different question with the same key. And the ratchet failed on `dead_tags`
> 0 → 1: a Python docstring in the benchmark's own test file opened with
> `FR-043-AC-9`, a **quoin** requirement id, which `python-docstring-id` bound
> as a quire-rs trace tag. The reference was reworded; the number is back to 0.

> **CR-098 note (2026-08-22):** AC-29 is new — the regression corpus is data.
> `agent-ix/quire-rs#232` and `agent-ix/quire-rs#233`, carrying
> `agent-ix/quire-rs#234`; epic `agent-ix/quoin#197`.
>
> `tests/fixtures/filament_core/graph_cases.json` — an 18-case
> `{name, tags, input, expect}` array behind one parameterized test — was the
> **only** data-driven scenario corpus in this repository. Everything else was
> hand-authored directory convention, which is why every new regression cost a
> new `.rs` file and why the six battletest failure families had nowhere to
> land.
>
> **`expect` asserts absence as well as presence.** `absent_diagnostic_reasons`
> is the half a fixture usually forgets, and it is the half that catches a check
> firing on healthy input — the failure mode that killed two diagnostics during
> CR-094 and that a presence-only corpus cannot express. The
> `marker-form-declared` case exists purely as the control for
> `marker-form-mismatch`: same tree, declared spelling, nothing fires.
>
> **`issue_ref` is required, not decorative** (`agent-ix/quire-rs#234`). A
> fixture whose origin is unrecorded becomes a fixture nobody dares change,
> which is how a corpus rots into a set of assertions everybody works around.
> `every_case_is_attributed_and_uniquely_named` enforces it.
>
> **Every field of `expect` is optional**, so a case asserts what it is about
> and stays silent on the rest. A corpus where each case pins the whole envelope
> fails forty cases on one unrelated change, and is then relaxed wholesale.
>
> **Directory corpora stay for what needs them.** A case here has no filesystem
> topology beyond the paths it lists, so anything about the walk, exclusion
> globs or symlinks still belongs in a real fixture tree. The claim is narrower:
> a scenario expressible as data should not cost a file.
>
> **Writing the six cases found no engine defect, and one fixture-authoring
> trap.** The first draft declared only an `acceptance-criterion` trace target,
> so a symbol binding `TC-001` left `backed` at 0 while `unbacked_rows` was
> empty — which reads as an inconsistency and is not one: `totals` counts minted
> **targets** bound, `unbacked_rows` counts **reference rows** unsatisfied, and
> a row is satisfied by its own id or any it references. The fixture module now
> declares both targets, as the real ISO module does.

> **CR-095 note (2026-08-22):** AC-28 is new — the catch-all is split out of the
> headline. `agent-ix/quire-rs#230`, epic `agent-ix/quoin#197`.
>
> `quire properties` headlined `515/951 criteria extractable (54%)` over
> `agent-ix/filament-ide-rs` — 951 criteria across 274 spec files, under
> `quire 0.29.0` / engine `v0.42.0` / `spec-artifacts-process v0.23.0`. **440 of
> those 515 were the `universal` catch-all.** Excluding it, the specifically
> shaped set was **78/951 = 8%**.
>
> Both numbers are true. 54% reads as *"half this specification is
> property-testable"*, and 8% is the honest figure for *"the classifier said
> what property to write"* — and only the first is what a reader takes from a
> summary line and repeats. The fix is one clause, and it is worth an AC
> because the misreading is what the line invites.
>
> **What the split is, and is not.** Three shapes are excluded and for three
> different reasons: `universal` is the catch-all and adds nothing beyond
> `extractable` itself, `example` is `not-extractable` by construction, and
> `unclassified` means no signal fired. **This is not a quality ranking and must
> not become a gate** — a `universal` criterion is very often the right thing to
> write. It is a *reading-list* distinction: 78 specifically-shaped criteria out
> of 951 is a tractable set to sit down with, and `idempotence` on an
> `FR-029-AC-1` pointed straight at a property worth writing.
>
> **Grounding is reported per shape because the two halves were disjoint in the
> wrong direction.** A classification record carries a shape *and* a
> decomposition (`domain` / `precondition` / `oracle`), and measured on that
> corpus **65 of the 67 specific-shape non-`example` records carried zero
> spans**, while every span-bearing record but nine was `universal`. So the
> shapes that told a reader the most arrived with nothing a generator could be
> driven from, and the catch-all arrived with the decomposition. Per-shape rates
> make that readable from the payload instead of from a bespoke sweep.
>
> **Deliberately not a widening of the classifier.** `agent-ix/quire-rs#45`
> settled 31.3% recall at 93.3% precision as a measured ceiling and closed it as
> answered; nothing here trades that away. The remaining half of
> `agent-ix/quire-rs#228` — *why* a plainer `SHALL` clause defeats the span
> extractor when a `universal` one does not — stays open as its own question.

> **CR-093 note (2026-08-22):** AC-27 is new — the report states the premise its
> percentage rests on. `agent-ix/quire-rs#227`, epic `agent-ix/quoin#197`.
>
> `quire coverage` printed `555/2389 rows backed (23%)` over
> `agent-ix/filament-ide-rs` while its declared tag patterns matched **0 of
> 1,292** Rust evidence symbols. Correct arithmetic, meaningless number, and no
> signal anywhere in the payload that the traceability model could read the
> repository at all. A census that cannot say *"I may be measuring less than you
> think"* invites exactly the three SpecReviews (SR-150/151/152) that cited it.
>
> **Unconditional, and that breaks this FR's own convention on purpose.** Every
> other list here is skipped when empty so a conformant repository's payload
> stays byte-identical to a pre-field engine's — `no_symbol_rows`,
> `undeclared_statuses`, `shared_trace_ids`, `excluded_source_files` all do it.
> Those are defect lists: empty means nothing to say. This one is not a defect
> list. `1,292 candidates, 1,290 bound` is a reassurance no previous version of
> this payload could give, and a premise that only appears when it fails is one
> a reader cannot lean on when it holds. So `binding_census` is present for
> every repository with source symbols, and the checked-in FR-050-AC-20 baseline
> was regenerated to match — the diff is the census block and nothing else.
> FR-050-AC-7 is untouched: the guarantee there is that repeated runs agree, and
> they do.
>
> **CR-135 note (2026-08-26):** MP-201 now owns the 5% observation definition
> (`coverage.binding-read-v1`). FND-201 found that the old message crossed from
> observation into diagnosis by calling a marker-form mismatch the likeliest
> explanation without comparative evidence. The engine retains the factual
> boundary and both counts, but now says it cannot distinguish sparse tagging
> from a marker-form mismatch and tells the reader what to inspect. The plan is
> at `observe`; the boundary is neither a target nor a gate (`quire-rs#275`).
>
> **Two reasons, not one threshold.** `no-symbol-bound` is unambiguous — every
> candidate walked, every declared pattern missed — and needs no judgement.
> `low-symbol-binding` exists because at 3% a tail of genuinely untagged tests
> and a near-miss pattern look identical from inside the engine, so it reports
> both counts and names the forms rather than asserting which. The observation boundary is
> **5%**, and it is deliberately not a coverage target: an unbound candidate is
> usually a real untagged test, and a repository mid-migration sits well under
> any number worth calling healthy. Below it, the two explanations require
> inspection outside the engine.
>
> **Why the existing channel did not cover it.** `diagnostics` was already
> populated and read — 33 records on that corpus, all
> `uncatalogued-verification-method` — so the mechanism, the schema slot and the
> rendering all existed. There was simply no diagnostic class for "the extractor
> ran and matched nothing". And `untracked_symbols` is not this either: it
> reports symbols that bound to an id no row declares, i.e. symbols that
> *matched* a pattern. A symbol matching no pattern was invisible to every
> output surface.

> **CR-089 note (2026-08-21):** AC-26 is new — coverage records say which
> authored line they are about. `agent-ix/quire-rs#210`.
>
> No coverage record carried a line, so a consumer could not render
> `path:line: message` the way `validate` does, and an editor or agent could
> not jump to the offending matrix row — the blocker for the lint-shaped
> output in the companion `quire-cli` issue. The loss was two layers down:
> `parse_table` discarded line positions at parse, so `ScannedRow` had
> nothing to carry. Recovered without new parsing: `parse_table_with_lines`
> returns each row's content-relative line, and `rows_of` converts through
> `body_line_offset` + the section's `start_line` — the `to_doc_line`
> arithmetic, hand-verified against authored fixtures. Deliberately **not**
> `ears::abs_line`: measured against the real files, the grammar findings'
> line is one short for exactly this shape (`make validate` reports
> NFR-015's `effective` on line 20; it sits on 21) — a latent grammar-layer
> defect noted on #210 and out of its scope. `VerifiesRelation` now carries
> the symbol's declaration line, so `untracked_symbols` points at the tagged
> test. CR-086's dedup is preserved by comparing without `line`: two
> byte-identical duplicate rows still collapse to one record, which carries
> the first duplicate's line — letting the line distinguish them would have
> quietly reopened that decision. Contract: additive optional `line` on the
> five `$defs` (FR-055-CON-3), tc856 exercises them, TC-957 pins
> omitted-never-null in both directions, and the CR-057 byte-golden regen
> diff is the reviewed record of every recovered line. AC-24 and AC-25 are new — what `source_exclude`
> subtracts is observable, and an invalid glob is loud. `agent-ix/quire-rs#215`.
>
> CR-085 shipped the key verified safe as written, and the agreed direction was
> minimal prevention, maximal observability: a bad glob cannot be fully
> prevented, so it must be loud, not silent. Three silences remained. The walk
> bare-`continue`d on a match, so an over-broad glob silently dropped
> legitimate backing and the report read as a coverage regression
> indistinguishable from tests that were never written — AC-24 makes the
> subtraction a count on the report (`SymbolExtraction.excluded_source_files` →
> `SymbolGraph` → `CoverageReport`, skip-zero for AC-7 byte-identity; the
> human-rendered census line is `quire-cli`'s half and lands with its batch).
> `ExcludeSet::compile` dropped an uncompilable pattern and applied the rest —
> partial filtering with no diagnostic for any caller not routed through
> `TraceabilityModel::validate`, and `extract_tree_scoped` is `pub` over
> `&[String]` — AC-25 makes the compile seam a `Result` refusing the whole
> list as one unit, mirroring what validation does at load. And the load-time
> error for an invalid `source_exclude` read "invalid `exclude` pattern",
> with tc945's `contains("source_exclude")` satisfied by the location prefix
> alone; the message now names the key it checks and tc945 asserts the noun. AC-23 is new — one test-case id names one
> source symbol, and an id shared by several is a reported defect.
> `agent-ix/quire-rs#216`.
>
> v0.41.0 shipped two instances in this very crate — TC-943 tagged on two test
> fns (`src/symbols/typescript.rs`, CR-084) and TC-944 on two
> (`src/symbols/mod.rs`, CR-085) — and no surface reported either: the matrix
> lists each id once, the row is backed by *any* one of its binders, and so the
> row stays green while the other test rots or is deleted. The id has stopped
> naming which evidence backs the row.
>
> The policy decision #216 asked for: **1:1 for status-carrying row ids, not a
> declared N:1 convention.** The trace id is the join key between matrix row
> and evidence; an id that names a set cannot say which member satisfied it.
> The scoping is measured, not assumed: unscoped, the check fired on 100+ ids
> in this repository alone, overwhelmingly acceptance-criterion ids that are
> N:1 **by design** (TC-941 and TC-942 both bind FR-050-AC-21) — a rule
> misreading correct data. Scoped to ids whose rows carry a status — the rows
> whose green can rot — it reports 51 ids here, sampled 10/10 real: all are
> the older several-fns-per-row authoring convention (facet splits like
> TC-609 ×6, cross-surface parity like TC-528 bound from Rust and Python).
> Those 51 stay visible as advisory corpus debt; unification on 1:1 is
> enforcement work gated on its own measurement, per the promotion rule.
> Advisory-first, as CR-083 did it: a report list that does not affect
> `totals` and is not gated by `--strict` in this revision. The two shipped
> instances are re-idded in the same change (TC-948, TC-949) and no longer
> appear in the list.

> **CR-085 note (2026-08-20):** AC-22 is new — a declared `source_exclude:`
> scopes the **source-symbol walk**. `agent-ix/quire-rs#199`.
>
> `quire coverage` walked the code tree with exactly one exclusion, the document
> root, and that exclusion is the caller's argument rather than anything a module
> can say. So a repository whose fixtures deliberately contain trace tags
> reported them as untracked symbols forever. This crate is the case in hand:
> `tests/fixtures/coverage_baseline/scope/src/lib.rs` carries `#[trace("TC-999")]`
> on a test that nothing declares, because *being unbacked is what it tests*. The
> fixture cannot change, so nothing on the authoring side could fix it.
>
> **A new key, not a widening of `exclude`, and the reason is measurable.**
> Every existing `exclude:` — model-level and per-declaration alike — is applied
> to a **document** path; none has ever been shown a source file. Meanwhile
> `spec-artifacts-process` FR-004-AC-9 *requires* every trace target to exclude
> `tests/**`, and **194 of this crate's ~458 `#[trace(` markers live under
> `tests/`** — a share that is near total in every Python and TypeScript
> repository in the ecosystem. A single key meaning both would delete the
> evidence tree and read as a catastrophic coverage regression. `tests/**` must
> therefore never appear on `source_exclude`; the declared value anchors at the
> fixture directory (`tests/fixtures/**`), and `globset` anchors a pattern at the
> start unless it opens with `**/`, so it cannot reach `src/tests/fixtures/`.
>
> **This does not violate CR-045.** That note forbids *relocating* the two roots:
> "`spec/` is convention, not configuration: no manifest key and no flag
> relocates it." `source_exclude` subtracts *within* the code root and can do
> nothing else. Both roots still derive from one `--scope`, neither is nameable,
> and the document root's exclusion remains the caller's non-configurable
> argument — the globs are a second filter applied after it. TC-944 asserts the
> one-way property directly: declaring `spec/**` as a source glob neither
> un-excludes the document root nor admits anything under it.
>
> Applied **per file, after `language_of`**, never as a directory prune:
> `tests/fixtures/**` does not match the directory `tests/fixtures` itself, so
> glob pruning in `filter_entry` would be unreliable in precisely the case the
> key exists for. The `ExcludeSet` is compiled once outside the walk, for the
> reason CR-060 gave it its own type.
>
> **Scope, measured rather than quoted.** #199 says #198 "took the other 14",
> leaving `TC-999` as the last untracked symbol. That was stale on arrival:
> measured on the pinned engine, repo-root self-coverage reports **1** untracked
> symbol, not 4 — #198 landed and had already removed three `concat!`-fixture
> cases. So this change takes quire-rs from 1 to **0**. The three the earlier
> count included are trace-shaped ids inside string fixtures in real `src/`
> files, which no path glob can reach; they are not this ticket's.
>
> `spec/reviews/SR-048-wave-b-gap-analysis.md` triages `TC-999` as a permanent
> finding. It stops being reported from a repo-root scope once a module declares
> the glob, so that triage line is superseded and now says so.

> **CR-083 note (2026-08-20):** A status value the model's vocabulary classes as
> nothing is reported as its own defect (AC-21). `agent-ix/quire-rs#192`.
>
> `StatusClass::Unknown` has existed since the class was introduced and was
> computed and discarded on every run: the only consumer asked
> `class_of(value) == Complete`, `Unknown` compared false, and the row fell out
> of the report. A value the module's structural contract **admits** and its
> traceability model **does not class** was therefore exempt from the status-lie
> check by construction — not because the row was honest, but because the engine
> had an opinion, formed it, and threw it away.
>
> This is not hypothetical, and CR-015 already predicted it: it recorded the
> matrix vocabularies as "declared in two places … and drifting". Measured on
> `~/dev` today, over rows in the one locator that declares
> `column_patterns.Status`: **20 rows across 5 repositories** carry a status the
> model classes as nothing — 18 `⚠️` and 2 `🟡`. `spec-artifacts-process` admits
> `⚠️` at `manifest.yaml:261` and declares no class for it at `:825`; `🟡` is
> admitted by neither and fails validation already.
>
> **Placement is the substance of this change.** The classification runs *above*
> the `is_backed` early-continue, alongside the row's `document`. Vocabulary
> drift is a property of the declaration, not of the row's evidence, so a check
> that only ever saw unbacked rows would report a subset and read as complete.
> TC-942 is that assertion: a backed row with an undeclared status is reported,
> and it is the test that fails if the block is ever moved back down.
>
> **`--strict` deliberately does not gate on this list, in this release.** At the
> moment it ships, repositories across the ecosystem would flip red on an engine
> bump for a condition none of them has been told about yet. Promotion is a
> separate, measured, user-gated decision once the corpus is clean — the same
> advisory-first sequence FR-042 used. A gate is deferred here, never lowered.
>
> The record carries the authored string verbatim and **no class**: having none
> is the entire finding, and two undeclared glyphs in one corpus are
> distinguishable only by the value. `StatusClass` is not serialized.
>
> **CR-062 note (2026-08-17):** AC-15 and AC-19 are narrowed: the `document:`
> origin is **deleted**, and `archetype:` is the single required origin for a
> trace target and a document reference alike. `agent-ix/quire-rs#74`.
>
> The form existed for one reason, recorded verbatim in `traceability.rs` at the
> time: "`spec/tests.md` is on `DEFAULT_SKIP`, so archetype binding alone cannot
> see the file 184 repos call their Test Matrix." Type-driven corpus membership
> (#73, v0.26.0) deleted that premise. Two ways to acquire a minting document
> then bought nothing and cost coverage, because path binding **enumerates**:
> the module declared three near-identical targets, one per filename the
> ecosystem happens to use (`spec/tests.md`, `spec/matrix.md`, `spec/evals.md`),
> and reached nothing nested. A correctly authored matrix at
> `spec/<module>/matrix/tests.md` minted zero ids.
>
> **Measured** across `~/dev`, 238 repositories, worktrees deduped
> (`scripts/sweep_coverage.py`, agent-ix/quire-rs#78), with this change plus the
> matching `spec-artifacts-process` collapse: dead trace tags fall from **1,401
> occurrences / 1,052 distinct ids to 1,207 / 873**. The whole change is one
> repository — `filament-ide-rs`, **214 → 20** dead tags, rollup 17/850 →
> **473/2,184** rows backed — because it is the only repository in the ecosystem
> authoring nested module matrices. It is also the shape the ecosystem is moving
> toward, which is what makes enumeration the wrong contract rather than merely
> an inelegant one.
>
> Collapsing the *references* matters as much as the target: rebinding
> `test-case` alone leaves 49 dead tags in that repo, because `traces-to` and
> `functional-coverage` were path-bound too and could not read the nested
> matrices they describe. Nine declarations become three.
>
> Three consequences, each deliberate:
>
> 1. **`exclude:` is now load-bearing, not optional.** Archetype binding is what
>    lets a fixture matrix mint phantom ids — a fixture that exercises the
>    `TestMatrix` contract legitimately *is* `type: TestMatrix`. This is the
>    concern that kept path binding alive through CR-038 (67 phantom ids out of
>    `tests/fixtures/testmatrix/*.md`, 50 of them reported "backed"), and it is
>    answered by exclusion rather than by enumeration.
> 2. **A minting document that cannot be read is now reported *better*.** The
>    off-corpus reader returned `None` and emitted nothing until CR-054; the walk
>    emits `DocumentUnreadable` / `MissingUuid`. So AC-19's
>    `unreadable-declared-document` and `absent-declared-document` reasons are
>    withdrawn — CR-059 shipped them in v0.27.0 for a code path this change
>    deletes, which was the right call for the interim and is dead now.
>    `archetype-matches-nothing` is the surviving reason, and a misspelled
>    archetype is the surviving shape of the same fault.
> 3. **A mistyped matrix now mints nothing.** Under path binding, frontmatter was
>    irrelevant to minting. Measured before landing: 14 of 184 ecosystem matrices
>    were untyped or mistyped, **6 of them real matrices** carrying a Test Case
>    Summary (agent-ix/quire-rs#75). Left alone, this change would have taken
>    repositories minting zero test-case ids from 154 to 159. All six were
>    corrected first, and the measurement re-run: **153**. Zero matrices in the
>    ecosystem are frontmatter-less, which is the case that could have gone
>    silently invisible, and it is empty.
>
> **CR-060 note (2026-08-16):** AC-13 and AC-15 gain a **model-level**
> `exclude:`. CR-038 put `exclude:` on trace targets and document references,
> and `declared_tables::scan` applies it. The CR-028 criteria counts walk the
> corpus on their own axis — frontmatter `type` → archetype → grammar binding —
> and had no exclusion to apply at all, because criteria classification is not
> a declared target and there was nothing to hang one on.
>
> So a document under a declared-excluded path still contributed to
> `CoverageReport.criteria` and to `totals.criteria` / `totals.property_shaped`.
> Deliberately malformed fixture data inflated the criteria denominator, and its
> body was parsed during coverage despite the declaration saying it is not
> corpus data — the same class CR-038 fixed for trace targets, where scanning
> `spec-artifacts-process` by archetype minted 67 test-case ids out of
> `tests/fixtures/testmatrix/*.md`.
>
> The fix is not to union the declared excludes into the criteria walk. That
> couples two axes CR-028 kept orthogonal, and it silently promotes a
> per-declaration statement — "these documents mint no ids *for me*" — into a
> global one. **Which paths hold test data is a property of the repository**, so
> it is declared once, at the model: `traceability.exclude`. Both the criteria
> walk and every declaration read it, and a per-declaration `exclude:` keeps its
> narrower meaning unchanged.
>
> It scopes **traceability only**, and the AC says so rather than saying "not
> corpus data", which would promise more than it delivers. An excluded document
> is still a document: `validate_bundle` schema- and grammar-checks it like any
> other. Being outside the coverage rollup is not a licence to be malformed in
> ways nobody reports — and a deliberately malformed fixture is usually
> malformed in exactly one axis on purpose.
>
> Consequences worth stating. It merges across modules as a **union**, not
> first-wins like every named entry: a path one module calls non-corpus must not
> become corpus because another loaded first, and the set does not depend on
> load order (NFR-006). It does not make a model *declared* — a module saying
> only "these paths are not corpus data" has declared nothing to reconcile
> against. And the globs are now compiled **once** per model rather than per
> pattern per question, because the criteria walk asks about every document in
> the corpus and a glob build per document would land on the NFR-015 walk.
>
> This changes the report for repositories that declare `exclude:` and have
> criteria under the excluded paths, so it is a deliberate, reviewable diff
> against the AC-20 baseline — whose companion test pinned this leak on purpose
> so that closing it could not be absorbed. Closes agent-ix/quire-rs#124.

> **CR-059 note (2026-08-16):** AC-19 is amended: an **absent** declared
> document and a **present but unreadable** one are no longer the same finding.
> CR-054 made a declared `document:` that fails to open a reported diagnostic
> instead of a swallowed `.ok()?`, and flattened `io::Error` to a string at the
> point of the read — so `NotFound` became indistinguishable from permission
> denied, an IO error, or a directory where a file was expected.
>
> Measured: running the v0.26.0 engine against this repository produced **six**
> such diagnostics, all of one shape — `spec/evals.md` and `spec/matrix.md`,
> named by three declarations each. `spec-artifacts-process` declares those two
> auxiliary sources, this repository's matrix is `spec/tests.md`, and it has
> neither. The declarations are **optional by convention**: a module shipped
> across 200+ repositories names the auxiliary documents any of them *might*
> have. So the diagnostic was technically true and practically noise, and it
> would have fired on most repositories on that module the moment they upgraded.
> This is the finding that made SR-007 CONDITIONAL rather than PASS.
>
> The two failures are different facts. `NotFound` over an optional fleet-wide
> declaration is the **ordinary** case, and it gets the rule
> `archetype-matches-nothing` already uses and for the same reason — reported
> only when the model minted nothing at all, which is the shape a typo in the
> one document that mattered produces. Anything else is **always** wrong: the
> file is there and the ids vanished anyway, which is precisely the CR-045
> silent-un-minting class CR-054 was filed about, and it is reported either way.
>
> Two machine reasons, not one: `unreadable-declared-document` keeps its
> meaning narrowed to the always-wrong case, and `absent-declared-document` is
> new. Both consumers read the one `scan_reason` vocabulary, so `quire validate`
> cannot drift from `quire coverage` — the property CR-054 established, now
> asserted on the validate side too, which had no test at all.
>
> This changes the report for repositories that declare absent auxiliary
> documents, so it is a deliberate, reviewable diff against the AC-20 baseline.
> Which is what that baseline is for. Closes agent-ix/quire-rs#129.

> **CR-049 note (2026-08-15):** The *Declaration-driven body selection*
> section and AC-18 are new. The model was always a projection the engine
> discarded — `reconcile` reads one named section of one named archetype,
> one named column per trace target, and was handed a full parse of every
> markdown file in the tree. With the header/body split (CR-046) and the
> lazy body tier (CR-047), the selection the model declares now bounds what
> is parsed: `declared_tables::scan` and `criteria_counts` decide on the
> header tier (frontmatter `type`, then the archetype's grammar binding)
> and only then touch `body()`. This changes what is *parsed*, never what
> is *reconciled* — byte-identity of the report (AC-7) is the whole gate.
> `quire properties` remains glob-driven and builds no corpus; if it grows
> a corpus-driven mode, its selection is "archetypes with a grammar
> binding", the same read `criteria_counts` uses
> (agent-ix/quire-rs#94, umbrella #90).

> **CR-045 note (2026-08-15):** The *Two roots, one scope* section and AC-17
> are new, and the phantom `[--source <DIR>]...` flag is withdrawn from the
> invocation line — it was specified but never implemented, and the decided
> design derives both roots from the one `--scope` instead of adding a second
> flag (umbrella agent-ix/quire-rs#90: `spec/` is convention, not
> configuration). Before this note, `quire coverage` handed `--scope` to both
> `Spec::from_path` and the symbol extractor, so the corpus walk read the
> whole repository — see the CR-045 note in
> [FR-024](./FR-024-parallel-repo-walk.md) for the measured blast radius.
> `--scope` stays the report's relativization base; only what is *traversed*
> changes, never what is *reconciled* (AC-7 byte-identity is the gate).

> **CR-041 note (2026-08-14):** A status lie is a row claiming evidence it does
> not have. Some rows cannot have that evidence *by their own declared method*:
> an agent-behaviour eval is verified by running an agent against a live
> scenario, an inspection by a person reading code. Neither produces a symbol a
> trace tag could attach to, so reporting them as lies asserts something the
> declared method makes impossible.
>
> Measured in `quoin`, where it bites hardest: of 55 status lies, **40 are
> agent-behaviour evals** whose answerable ids are an eval id and a user-story
> id — neither of which any source symbol can ever back (agent-ix/quoin#65).
> quire-rs's own matrix has exactly one such row, which is why the gap was
> invisible from here.
>
> AC-16 adds `vocabularies.no_source_symbol` plus the `test_type_column` it is
> read from. Which methods produce code stays **module-declared**: the engine
> knowing that "Eval" or "Inspection" is special would be the hardcoded
> semantics FR-050 exists to avoid. The exemption changes the **verdict and
> never the facts** — an exempted row stays in `unbacked_rows` and the
> backed/total counts are untouched; only the lie is withdrawn, and a
> `no_symbol_rows` entry says which declared value withdrew it.
>
> `no_symbol_rows` is absent-by-default and skipped when empty, so a module
> declaring no such vocabulary serializes byte-identically to one written before
> the field existed (FR-050-AC-7). Declaring a value outside the `test_type`
> vocabulary, or omitting `test_type_column`, fails module load — a typo there
> would silently exempt nothing, which is the failure mode this whole programme
> keeps finding.

> **CR-038 note (2026-08-13):** A trace target had no way to say which paths it
> covers, and `archetype` + `document` was rejected as an incoherent pair. Both
> limits fall out of the same fact: `spec/tests.md` is on the corpus walk's
> `DEFAULT_SKIP`, so `archetype: TestMatrix` cannot see the file 184 repos call
> their Test Matrix, while every matrix that is *not* named `tests.md` — test
> fixtures included — is in the corpus and mints ids.
>
> Measured cost of having neither: scanning `spec-artifacts-process` by
> archetype minted 67 test-case ids from `tests/fixtures/testmatrix/*.md`, of
> which 50 read as **backed**, because a fixture reusing `TC-017` collides with
> the real one. A phantom backed row is precisely the falsehood this rollup
> exists to catch. The workaround that shipped instead binds by `document:`
> path — nine entries where two would do, one of them a filename that exists in
> a single repo and is dead weight in every other.
>
> AC-15 adds `exclude:` globs and allows the pair. Which paths hold test data
> stays **module-declared**: an engine that knew `tests/fixtures/` was special
> would be exactly the hardcoded semantics this model exists to avoid.
> `exclude` is absent-by-default and skipped on serialization, so a model that
> declares none is byte-identical to one written before the field existed
> (FR-050-AC-7).
>
> **Still open:** archetype binding remains blind to `tests.md` itself; naming
> it as the entry's `document` is what closes the gap. Whether the walk should
> stop skipping a file whose frontmatter names a registered archetype is a
> separate call with ecosystem-wide reach, tracked in agent-ix/quire-rs#63.
>
> **CR-035 note (2026-08-13):** FR-050-AC-9 covers *"no module declares a
> model"*. Nothing covered *"a model is declared and matched no rows"*, so the
> command had no reason to treat it as anything — and it treated it as success:
> `(backed * 100).checked_div(total).unwrap_or(100)` printed `0/0 rows backed
> (100%)`, and `--strict` fires only on non-empty `unbacked_rows` /
> `status_lies`, which are both empty when nothing matched. A gate wired to that
> command passed vacuously. It is how the ecosystem-wide failure in
> `spec-artifacts-process` (agent-ix/spec-artifacts-process#22) went unnoticed
> for nine days.
>
> The two states are opposites and must not render alike: `100%` on an empty
> denominator means "found nothing", not "all covered". AC-14 makes zero matched
> rows a distinct report line and a non-zero `--strict` exit.
>
> **Scope:** the fix is CLI-only. `CoverageReport` gains no field — a consumer
> reading `--json` already tests `totals.total == 0`, and a field carrying the
> same fact would be a second source for it. FR-050-AC-7's byte-identical
> guarantee is therefore untouched, which is the point of not adding one.
>
> **CR-028 note (2026-08-07):** `CoverageReport` grows a `criteria` field and
> `CoverageTotals` grows two counts — a criteria count and a property-shaped
> count — carrying the classification
> [FR-052](./FR-052-acceptance-criteria-property-classification.md) computes.
> The rollup gains a grammar dependency and **no acceptance-criteria knowledge**:
> it walks the already-path-sorted corpus, asks FR-052 for per-document counts,
> and skips any document yielding none, so a non-requirement corpus produces an
> empty list and output identical to today's. The counts are declaration-free by
> necessity — the process module declares exactly one trace target and no
> document references over criteria rows, so there is no group to hang a
> declared column on, and a declaration-driven column would ship structurally
> present and permanently unpopulated.
>
> The FR-050-AC-7 byte-identical guarantee therefore now covers content this FR
> does not otherwise describe. That is the point of stating it here rather than
> in FR-052: the ordering discipline is this FR's, the new fields sort into the
> same determinism block as every other, and `#[serde(default)]` on all three
> keeps a report written by an older engine round-tripping.
>
> FR-050-CON-1 is unaffected. A count is not a verdict, and a low
> property-shaped count is not a failing corpus — CR-020 already recorded that
> StR criteria legitimately score low. Judgment stays in the consuming workflow.

> **CR-015 note:** The ecosystem sweep behind FR-003 (report:
> `spec-artifacts-process/reports/2026-08-04-tests-md-sweep.md`) found the
> matrix vocabularies declared in two places — a module's `column_choices` and
> this model's status vocabulary — and drifting. It also found the authored
> corpus using forms the model could not express: statuses carrying the reason
> they are partial (`⚠️ scale evidence deferred`), a retired class, and
> `Traces To` cells written as ranges or carrying parenthetical qualifiers. This
> amendment makes the model the single source for column vocabularies, classes
> statuses by leading marker, and moves range expansion and annotation stripping
> into declared, default-off normalizations so the engine gains no behaviour a
> module has not asked for.

## Dependencies

- **Upstream**: [FR-051](./FR-051-source-symbol-extraction.md) (the symbol graph and trace tags), [FR-027](./FR-027-whole-spec-query-api.md) (corpus queries), [FR-014](./FR-014-module-activation.md) (manifest loading), [FR-010](./FR-010-query-api.md) (table extraction)
- **Upstream (added CR-028)**: [FR-052](./FR-052-acceptance-criteria-property-classification.md) (the per-criterion property classification the `criteria` counts summarize)
- **Downstream**: [FR-049](./FR-049-verification-reference-integrity.md) (reference declarations reused by bundle validation); `spec-artifacts-iso` declares the ISO model (follow-up change in that module); the `gap-analysis` workflow replaces its grep step with `quire coverage`; [FR-065](./FR-065-controlled-corpus-contract.md) (the controlled corpus whose cases assert against this payload — a case's `expect` is written against the shape published by FR-055)

> **CR-103 note (2026-08-22):** `agent-ix/quire-rs#237`, reopened.
> SR-054 FND-005 — **the ratchet's corpus was one language.**
>
> `bench/manifest.json` listed `self` and `filament-ide-rs`, both Rust, and the
> second is skipped whenever it sits off its pin. So the benchmark that exists
> to catch a check going wrong measured exactly one Rust repository, which is
> why CR-102's two false-positive classes reached a release: the TypeScript
> misread needed a TypeScript corpus to be visible, and the count-shaped
> hollow-denominator needed a repository where `coverage.implements` reads
> zero. Neither existed.
>
> **`quoin` (TypeScript) and `spec-artifacts-process` (Python) are now scored,
> and each is the corpus that exposed one of those defects.** They are working
> trees this repository does not control, so they carry a `metrics` allowlist
> and are scored on the two **gates** only. Ratcheting their `backed_pct` would
> move whenever somebody writes a spec row and train everyone to run
> `bench-update` reflexively, which is how a ratchet stops being one.
>
> **`skeptic.suspicion_rate` is the guard, and it was verified end to end** —
> not by reasoning about it. Reverting the CR-102 guard-list fix and re-running:
>
> ```
> !! quoin/skeptic.suspicion_rate: 99.1 (baseline 0.0) [percent of evidence symbols]
>      regressed against 0.0
> $ echo $?   # 1
> ```
>
> Restored, it reads `0.0` and the run is green. Current baselines: `quoin`
> 0.0, `spec-artifacts-process` 0.0, `self` 0.21 — the two genuine
> TC-1596-shaped positives in this crate's parser suite.
>
> **The determinism test was also rewritten.** SR-014 FND-003: it called a pure
> function twice and compared, which cannot fail short of deliberately
> injecting a clock. It now asserts the absence of any time-varying field on a
> scored row, which is the guard it was reaching for — verified by adding
> `generated_at` to a row and watching it fail.
