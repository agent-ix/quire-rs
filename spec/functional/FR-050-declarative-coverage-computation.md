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
| FR-050-AC-15 | A trace target or document reference MAY declare `exclude:` path globs, and MAY declare `archetype` and `document` together; excluded documents mint no ids and contribute no reference rows, and a declaration naming both scans the archetype's corpus documents and the auxiliary file in one entry. The model MAY additionally declare a model-level `exclude:` meaning "not corpus data for any purpose", which scopes every declaration in addition to its own and merges across modules as a union; its patterns are compile-checked like any other, and declaring it alone leaves the model undeclared. | Test (TC-801, TC-802, TC-826) |
| FR-050-AC-16 | A module MAY declare which test-type values mint no source symbol; an unbacked row carrying one is reported as a no-symbol row rather than a status lie, stays listed as unbacked, and a module declaring none reports exactly as before. | Test (TC-805) |
| FR-050-AC-17 | The two roots derive from one `--scope` and stay distinct: repo-root files (`README.md`, `CHANGELOG.md`, `plan/*.md`) are never read as documents, the code walk never enters the document root, the minted-id set over a compliant repo is byte-identical to a pre-split run, and a scope with no `spec/` directory exits with a diagnostic naming the missing root (CR-045). | Test (TC-809, TC-810, TC-811) |
| FR-050-AC-18 | During coverage computation, a corpus document whose archetype no trace target, document reference, or grammar binding names has its body left unmaterialised; a declared archetype's body is parsed; selection is decided on the header tier and never by filename; a module declaring no `traceability:` model still errors (`ModelUndeclared`) before any selection; and the report is byte-identical to a full-parse engine's (CR-049). | Test (TC-818, TC-738) |
| FR-050-AC-19 | A declaration that selects nothing is reported in `CoverageReport.diagnostics` and as a `quire validate` warning, never in silence: a declared auxiliary `document:` that is **present and cannot be read** is reported against every declaration naming it, with the path and the OS error, whether or not the model minted; a declared `document:` that is **absent**, and a declared `archetype:` no corpus document has, are reported when the model minted no id at all; and a model declaring no `trace_targets` is reported as minting nothing. The two document reasons are distinct machine tokens, and `quire coverage` and `quire validate` report the same token for the same finding. The list is empty — and the key absent — for a model whose declarations select, so FR-050-AC-7 byte-identity holds (CR-054, amended CR-059). | Test (TC-822, TC-825) |
| FR-050-AC-20 | The byte-identity property is gated by a checked-in baseline, not by inspection: a fixture corpus exercising minted ids, an auxiliary matrix, an `exclude:` glob, all three status classes, the `no_source_symbol` exemption, an untracked symbol, a dangling reference, an undeclared archetype and criteria classification has its report stored as `tests/fixtures/coverage_baseline/expected.json` and byte-diffed on every test run. Regeneration is a deliberate act (`make coverage-baseline-update`) whose diff is reviewed, and a companion test fails if the corpus stops exercising any of that surface (CR-057). | Test (TC-824) |

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
- **Downstream**: [FR-049](./FR-049-verification-reference-integrity.md) (reference declarations reused by bundle validation); `spec-artifacts-iso` declares the ISO model (follow-up change in that module); the `gap-analysis` workflow replaces its grep step with `quire coverage`
