---
id: FR-047
title: "Acceptance-Criteria Grammar"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-042"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-043"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-044"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-014"
    type: "implements"
---
# FR-047: Acceptance-Criteria Grammar

## Description

The `iso-spec-core` grammar bundle SHALL include an **acceptance-criteria
grammar** (`ac`) registered on the [FR-042](./FR-042-requirement-grammar-check.md)
framework alongside EARS. The `ac` grammar SHALL bind to the `Criteria` column
of **every requirement archetype whose module contract declares a
binding-criteria table**, under that archetype's own section heading and with
its own sub-id kind (CR-020):

| Archetype | Section | Sub-id | Reference the criterion is judged against |
|---|---|---|---|
| FR | `## Acceptance Criteria` | `-AC-` | the specification (verification) |
| NFR | `## Acceptance Criteria` (optional) | `-AC-` | the specification (verification) |
| StR | `## Validation Criteria` | `-VC-` | the stakeholder's real need (validation) |

`StR` keeps its own heading and sub-id kind: ISO/IEC/IEEE 29148 separates
*validation* from *verification* at the artifact level, and only the table shape
is unified. `US` and `IT` are **not** bound — a user story's
`## Acceptance Examples (Illustrative)` are non-binding by design and `IT`
declares no criteria table, so an `## Acceptance Criteria` heading on either is
a structural finding for the module's `forbidden_section` lint rule, not an
input to this grammar. It SHALL additionally bind to criteria supplement
subsections (a `### <doc-id>-<kind>-N` heading whose body supplements its table
row), matched under the archetype's own sub-id kind.

The `ac` grammar SHALL treat every non-empty `Criteria` cell as one statement,
unlike the EARS segmenter's modal-verb filter: an acceptance criterion with no
`shall` is still a criterion and is still checked. The `ac` grammar SHALL skip
fenced code blocks and blockquotes inside supplement sections, per the FR-042
skip rules.

An acceptance criterion is a **verification statement**, not an obligation: the
requirement already stated the obligation, and the criterion states what is
observed under what input. The **assertion** is therefore the single canonical
acceptance-criteria shape — it carries the test oracle directly, whereas an
obligation restates the requirement one level down and a Given/When/Then cell is
a second rendering of the same assertion (CR-013).

The `ac` grammar SHALL classify each statement into exactly one shape. The
classification is **structural**, used to locate the outcome clause the checks
below read; only the `assertion` shape is canonical:

- `assertion` — the statement asserts an outcome directly, e.g. "A finding whose
  key is absent from the merged map defaults to warning". Its outcome clause is
  the whole statement. **Canonical.**
- `obligation` — the statement matches an EARS pattern per the FR-042
  classifier, i.e. it states an obligation rather than an observation. Its
  outcome clause is the response clause after the modal verb.
- `given-when-then` — the statement is structured as Given/When/Then clauses (a
  leading `Given`/`When` clause and a `Then`/result clause, in prose or bullet
  form). Its outcome clause is the `Then` clause, so a GWT cell's other checks
  still run.
- `unstructured` — none of the above **and no predicate marker at all**: no
  modal or copula, no inflected or irregular verb form, no elided-copula
  predication, no declared observable-result verb, and no concrete-object signal
  (CR-014, CR-019).

For each statement, the `ac` grammar SHALL emit a finding when the statement
violates a check:

1. **unclassifiable** — the statement carries **no predicate marker at all**: no
   modal or copula, no inflected verb form, no irregular past form, no
   elided-copula predication (an existential/quantifier head or a predicative
   adjective — CR-019), no declared observable-result verb, and no
   concrete-object signal. A bare noun phrase (`Structural evaluation`,
   `Type Check`), a bolded heading (`**Key Generation**`) or a dangling prose
   fragment (`Current validation note:`) authored into a `Criteria` column
   asserts nothing and cannot be tested. The check is deliberately *structural*
   — it does not ask whether the outcome is a good one, only whether the cell
   states an outcome (CR-014).

   **What this check is and is not (CR-019).** The predicate test is a
   disjunction of morphological and lexical markers, and one of its branches
   (`\b\w+(s|ed|ing)\b`) cannot distinguish a verb from a plural noun. Measured
   over 36,580 `Criteria` cells, **99.6% satisfy it**, and **23.4% satisfy it
   only through that branch**. It is accordingly a **bare-fragment heuristic** —
   it reliably catches material that is not a sentence at all — and **not** a
   test that a finite verb is present. FR-047 does not claim, and the check does
   not establish, that a passing cell states a testable outcome. The 5,187 → 87
   drop CR-014 reported is predominantly the test becoming easier to satisfy,
   not a precision improvement; the honest statement of CR-014's effect is that
   it exchanged one signal for a much narrower one. See CR-019.
2. **non-singular** — the statement bundles more than one independent
   obligation: more than one `shall`, or more than one `Then` clause. A single
   criterion pairing one positive and one negative case of the same behavior
   (the `X yields a finding; Y yields none` idiom) SHALL count as one
   obligation. The idiom SHALL be recognized by the **second obligation**, not
   by the punctuation joining the two (CR-024): a directly negated modal
   (`SHALL NOT`, `SHALL never`) or a negation marker inside that obligation's
   own clause. `Then` clauses SHALL be counted only in a `given-when-then`
   criterion, and only when the criterion states no modal at all — elsewhere
   `then` sequences a precedence chain, not an obligation list (CR-024).
3. **vague-response** — the statement's outcome clause (the whole statement for
   an `assertion`, the response clause of an `obligation`, or the `Then` clause
   of a `given-when-then` statement) uses a vague verb per the FR-042
   object-aware machinery. The check SHALL reuse the merged module lexicon
   ([FR-043](./FR-043-module-concrete-lexicon.md)) and project glossary
   ([FR-044](./FR-044-project-glossary-lexicon.md)) exactly as the EARS
   `vague-response` check does — one vague-verb implementation, two grammars.
4. **vacuous-outcome** — the statement's outcome clause is headed by a
   **vacuous predicate** and carries nothing else to check: the engine SHALL
   ship a built-in vacuity set (`works`, `working`, `behaves`,
   `functions correctly`, `work correctly`, `is correct`, `is successful`,
   `is fine`, `is ok` — see CR-025 on why `functions` is qualified) that a
   module MAY extend via a `vacuous_predicates` registry in its
   `manifest.yaml`, merged first-wins with the built-in defaults at lowest
   precedence — the same pattern as the FR-043 `lexicon` and the
   `observable_verbs` registry. The finding SHALL be suppressed when the clause
   carries a concrete-object signal (a backticked identifier or a numeric
   bound), a lexicon term, or a declared **observable-result verb**: the engine
   SHALL ship a built-in observable-verb set and a module MAY extend it via an
   `observable_verbs` registry (ADR 0009), and those verbs are what tell a
   vacuous cell (`Navigation works`) from a substantive one (`Volumes are
   correctly mounted into the container`).

5. **non-canonical-shape** — the statement is `obligation`-shaped or
   `given-when-then`-shaped. The finding steers the author toward the canonical
   assertion shape, in the same spirit as the EARS `non-canonical-trigger`
   check; classification still succeeds, so the cell's other checks still run on
   its outcome clause. An `assertion` cell yields none.

> **CR-013 note:** This replaces the original FR-047 decision that EARS is the
> canonical acceptance-criteria shape (and that GWT is the only non-canonical
> one). The Gate G1 baseline over this repo's own spec produced 340 `ac`
> findings across 44 FR documents, **322 of them `unclassifiable`** — every one
> a correct report that a declarative assertion is not an obligation. Only 18
> were substantive. Classifying the corpus by quantifier instead of style showed
> 50.8% of its 327 acceptance criteria are already property-shaped (42.5%
> universally quantified, 8.3% metamorphic). An AC's testability depends on
> whether it names an input and an observable outcome, not on which prose style
> it wears, and the assertion shape supplies the test oracle directly. The
> canonical shape is therefore the assertion; `unclassifiable` now means
> "structureless", and `non-canonical-shape` fires on obligations as well as
> GWT. Shape conformance is not made configurable: FR-048's per-check severity
> map (`ac:non-canonical-shape: off`) is the opt-out, as for every other check.
>
> An **ecosystem-wide survey** (199 repos, 3,253 requirement documents, 11,919
> acceptance criteria) confirms the choice is not a quire-rs idiosyncrasy:
> `assertion` is the dominant shape in 139 of 199 repos. Note the quantifier
> split differs sharply by repo: 24.9% of ecosystem ACs are property-shaped
> (quantified or metamorphic) against 50.8% in quire-rs itself.
>
> **Shape-share figures corrected (CR-022, 2026-08-06).** This note originally
> recorded `assertion` 66.6% / `unstructured` 29.2% / `obligation` 2.9% /
> `given-when-then` 1.3%, and concluded that non-canonical shapes were "~4.2% of
> the corpus (~506 cells, concentrated in 17 repos)". Those figures never agreed
> with the primary measurement: `non-canonical-shape` fires on exactly
> `obligation | given-when-then`, and the fit report of the same date and corpus
> records it at **2,047 findings = 17.2% of cells**. 2.9% + 1.3% cannot produce
> 17.2%. The distribution table was the wrong record — a check's own finding
> count is a direct census, not a sample. Re-measured on the current classifier
> over a deduplicated 192-repo, 16,449-cell corpus
> (`~/dev/reports/2026-08-06-ac-canonical-shape-sweep.md`):
>
> | Shape | Cells | Share |
> |---|---|---|
> | `assertion` | 12,909 | **78.5%** |
> | `obligation` | 3,176 | 19.3% |
> | `given-when-then` | 282 | 1.7% |
> | `unstructured` | 82 | 0.5% |
>
> `given-when-then` was roughly right; **`obligation` is 19.3%, not 2.9%**, so
> the conversion target is ~3,458 cells across 50 repos rather than ~506 across
> 17 — about seven times the scope agent-ix/quire-rs#21 was written against.
> Concentration is extreme: `ecaz` alone holds 2,413 of them and six repos hold
> 80%. The `unstructured` share fell from 29.2% to 0.5% across CR-014 and CR-019
> and is no longer the reason CON-1 keeps promotion gated; the 21.0%
> non-canonical share is.

> **CR-014 note:** An ecosystem fit check (report:
> `~/dev/reports/2026-08-04-ac-grammar-fit.md`) ran this grammar through the
> PyO3 binding over 5,027 requirement documents in 199 repos — 11,919
> acceptance-criteria cells — and found two of its checks unusable as specified.
> `no-observable-outcome` fired on **51% of all cells at ~35% sampled
> precision**, and `unclassifiable` on **43.5% at ~12%**. The cause is
> structural, not a tuning gap: observability in acceptance criteria lives in an
> open-ended verb space (1,201 distinct stems in the corpus; the built-in 13
> covered 14.5%), and an allowlist cannot close an open set — declaring 73
> corpus-mined verbs cut findings 60% but left precision at ~30%/~12%, still
> flagging *"Semantic search ranks by relevance"* and *"Cache does not exceed
> max_size entries"*.
>
> The tests are therefore inverted. `vacuous-outcome` detects a **closed** set of
> vacuous predicates instead of requiring membership of an open one, and
> `unclassifiable` asks the structural question — is there a predicate at all —
> instead of asking whether a verb is on a list.
>
> CR-014 changed two things at once — the two predicates *and* the binding — so
> its result has two figures, and they must be quoted with their condition:
>
> | Condition | `vacuous-outcome` | `unclassifiable` |
> |---|---|---|
> | Same 199-repo corpus, **FR-only** binding (as measured mid-change) | 25 (0.21%) | 39 (0.33%) |
> | Same corpus, **widened** binding — what shipped | **32** (~95% precision) | **87** (~80% precision) |
>
> Against 11,269 findings from the two retired predicates before the change
> (6,082 + 5,187), with no true positive lost in sampling. The **shipped**
> figures are the 32/87 row; they are what
> `~/dev/reports/2026-08-04-ac-grammar-fit.md` records and what CR-017's
> 3,956 → 3,949 total is measured against. `non-canonical-shape` rose 2,047 →
> 2,638 (~95% precision, the rise being the widened binding) and the low-volume
> checks are otherwise unchanged.
>
> The same check also settled the binding: `FR`-only reached 76.9% of AC-bearing
> documents, and sampled US and NFR criteria are the same shape as FR ones, so
> the grammar now binds to every requirement archetype carrying an
> `Acceptance Criteria` table. Bullet-form AC sections remain unsegmented by any
> grammar — recorded as future work, not addressed here.

> **CR-020 note:** CR-014 widened the binding from `FR` alone to
> FR/NFR/US/StR/IT on a **cell census** — 11,476 FR cells, 340 NFR, 69 US, 20
> StR, 14 IT. A census measures what authors wrote, not what the contract
> declares. Checking `spec-artifacts-iso`, only **FR** declared an
> acceptance-criteria table at all: NFR's section was optional free prose, StR
> has no `Acceptance Criteria` section (its required section is
> `## Validation Criteria`), and a US carries `## Acceptance Examples
> (Illustrative)` whose skeleton states outright that nothing in it is binding.
> The non-FR counts were not adoption — 20 StR cells across 199 repos are 20
> improvised tables.
>
> That produced two failures in opposite directions. Reading US criteria as
> binding treats discovery context as verification criteria and inflates every
> number counting them; missing StR's required validation criteria hides binding
> criteria that genuinely exist. CR-014's stated premise — "the criteria of a
> user story … are verification statements of the same kind" — is **wrong for
> US**, and this note retracts it.
>
> The binding now follows the contract (spec-artifacts-iso#9, which gives StR
> and NFR real tables): per-archetype section + sub-id kind, FR/NFR under
> `Acceptance Criteria` with `-AC-`, StR under `Validation Criteria` with
> `-VC-`, and US/IT unbound. Two engine sites follow: `parent_id` and the token
> regex in `src/corpus/unlinked.rs` both learn `-VC-` (FR-039-AC-11) — the
> second was not in the issue's plan and was found by the test, since matching
> only `StR-001` would have linked the parent and left a dangling `-VC-2`; and a
> `forbidden_section` lint rule type (FR-036-AC-7) gives the module a way to
> flag an `## Acceptance Criteria` heading on a US, which
> `section_body_pattern` cannot express because a missing section is defined to
> produce no finding.
>
> **Baseline restated.** Every count taken since CR-014 mixed contract-conformant
> FR cells with improvised non-FR ones. Re-measured over the same 36,582 cells
> with `scripts/ac_corpus_sweep.py`: total `ac`+`ears` findings fall **16,792 →
> 15,247 (−9.2%)**, all of it material read from documents whose contract
> declares no binding criteria — `non-canonical-shape` 6,859 → 5,958,
> `vague-response` 315 → 286, `unclassifiable` 184 → 157, `vacuous-outcome`
> 51 → 40. The `ears` checks are unchanged, as expected: they bind on their own
> terms.
>
> Consequence for downstream work: StR criteria are validated by **demonstration**
> in an operational context rather than by quantifying over an input domain, so
> they will legitimately score low on property-extractability. The property-shape
> classifier (agent-ix/quire-rs#20) must not read that as a quality failure.

> **CR-019 note:** Both of CR-014's high-volume checks were re-measured against
> the open questions on agent-ix/quire-rs#18 and #19, over 36,580 `Criteria`
> cells in 197 repos, using the harness now committed at
> `scripts/ac_corpus_sweep.py` (the original run's script was never saved, which
> is why the decision could not be re-derived). Report:
> `~/dev/reports/2026-08-06-ac-check-remeasure.md`.
>
> **`no-observable-outcome` was restored and re-measured, and CR-014 stands.**
> All three mechanical defects that inflated its original 51%/~35% measurement
> are fixed, and the 73-verb vocabulary widening the fit report tested was in
> fact adopted — the built-in observable set is now 86 verbs. Repaired, the check
> fires on **14.8%** of cells, a 3.4× drop. But sampled precision **fell** to
> ~13% (4 of 30), because the mechanical fixes removed the easy true positives
> and the residual is almost entirely root cause 1 — the open verb space.
> `invalidates`, `clears`, `revokes`, `displays`, `exchanges`, `recovers`,
> `activates`, `delegates` are all legitimate observable outcomes absent from an
> 86-verb list, and the list cannot be finished. A recall frame of 30 unflagged
> cells found **0** untestable. The check does not reach usable precision even
> repaired, so it does not return, not even shipping `off`: a check nobody could
> defensibly enable is not worth the surface. The `remeasure` cargo feature keeps
> it buildable for measurement and is never enabled in a shipping build.
>
> **`unclassifiable` is not what its name claims, and FR-047 now says so.** The
> predicate test is satisfied by 99.6% of cells, 23.4% through the
> `\b\w+(s|ed|ing)\b` branch alone, which matches any plural noun. Tightening it
> to require a finite verb needs part-of-speech tagging, which is out of scope
> for a deterministic engine; the check is therefore documented honestly as a
> bare-fragment heuristic (check 1 above) rather than renamed, since the name
> `unclassifiable` describes the *shape* outcome accurately and only the prose
> overclaimed.
>
> **The false positives both issues required fixing are fixed.** A predication
> whose copula is elided — an existential/quantifier head or a predicative
> adjective — is now a predicate (`re_elided_copula`, FR-047-AC-14, TC-763).
> This removed 92 of 276 `unclassifiable` findings (−33%), and **every one was a
> false positive**: negative-existence assertions (`No refresh_token in response
> body`), uniqueness constraints (`Only one active password_reset token per
> user`), and comparatives (`Lockout response identical to normal failure`) are
> all perfectly testable. It also fixes the one residual false positive the
> original fit report called unfixable without part-of-speech tagging
> (*"…plugin management and template creation operate normally"*). Like
> `vacuous_predicates`, this is a **closed** set used to *suppress* a finding
> rather than an open one whose membership is required — the distinction CR-014
> turned on — and it is engine-built-in rather than module data because it is
> English grammar, not domain vocabulary.
>
> A `-able`/`-ible` predicative-adjective branch was tested and **not adopted**:
> it would remove 4 further findings, of which one is the template placeholder
> `[observable outcome]` that should stay flagged. Two true fixes against one
> regression is not a good trade for a broader rule.
>
> No `grammar_severity` default changed; promotion remains user-gated by CON-1.

> **CR-017 note:** Grammar-keyword detection confused **mention with use**. A
> criterion that quotes a keyword as example data — ``a statement with two
> `shall` clauses yields exactly one `non-singular` finding`` — was read as
> though it imposed an obligation. Every one of the seven `non-canonical-shape`
> findings standing against this repo's own spec after CR-014 was of that kind:
> FR-042, FR-043 and FR-047 describe a grammar, so they quote its keywords.
> Rewording them would have hidden a checker defect behind prose.
>
> Shape classification and obligation counting therefore read a **masked** copy
> in which each closed code span's contents are neutralized. The mask is
> deliberately narrow: the signal and lexicon checks still read the real words,
> so a backticked identifier still counts as a concrete-object signal (FR-042)
> and a backticked lexicon term still suppresses `vague-response`
> (FR-043-AC-3).
>
> Masking alone exposed a second defect it would otherwise have made worse.
> Supplement prose was segmented on `". "` without regard to code spans, so an
> embedded example (``` `EXPLAIN … SELECT ... ORDER BY col LIMIT 10` ```) was cut
> in half; the resulting fragment had unbalanced backticks, and the mask then
> paired them wrongly and swallowed the modal that followed. Sentence
> segmentation is now code-span aware, which is what makes the mask safe.
>
> Measured over the same 199-repo corpus as CR-014: **3,956 → 3,949** findings.
> Sixteen mention-only false positives are gone; the segmentation fix restores
> nine true positives the mask alone had hidden and surfaces one more that was
> previously split apart. **No new finding appears in any other check.** In this
> repo's own corpus the `ac` findings fall **10 → 2** — `non-canonical-shape`
> and `non-singular` both reach zero, leaving two `vague-response` findings on
> the PyO3-parity criteria of FR-042 and FR-047.
>
> **Closed by CR-021** (2026-08-06): `ears` does not adopt the mask, for now.
> Measured, it is one true fix against two regressions, because `ears`
> segmentation is line-based and a code span wrapping across source lines leaves
> the next line beginning inside an unterminated span. Adoption is conditional on
> paragraph-joining segmentation. The reasoning is recorded in
> [FR-042](./FR-042-requirement-grammar-check.md); the original open question
> below is superseded.
>
> Whether the same mention/use distinction should apply to the `ears` grammar is
> **open**: EARS statements quote keywords far less often, and the change is not
> made here so this slice ships one measured behaviour change rather than two.

> **CR-024 note (2026-08-07):** The corpus baseline sweep CON-1 requires
> before promotion found `non-singular` firing on **23 singular criteria out of
> 48** — a 48% false-positive rate that would have made the check unusable as an
> `error`. Both causes were checker defects, and both are the same shape as
> CR-017: a rule that was right in principle and too narrow in trigger. Per the
> CR-017 precedent, the checker is fixed rather than the prose.
>
> **The pair idiom was tied to its separator.** AC-3's positive/negative rule
> only fired when the halves were split by `;` or ` while `, so the form the
> corpus actually writes — ``the task SHALL render `skipped` with reason
> `"disabled"` and SHALL NOT execute`` — counted as two obligations. Nineteen
> criteria across eleven repos were flagged for stating one behaviour in two
> directions. Recognition now reads the **second obligation**: its modal is
> directly negated, or its clause (delimited at the last separator before that
> modal, so a `no` in the *first* obligation cannot suppress) carries a negation
> marker — `` `github_url` SHALL be None ``, `No Secret deletion SHALL occur`,
> `otherwise it SHALL be omitted`. Bare `not` was dropped from the marker set in
> the same pass: it is the commonest word in a criterion's condition
> (`when the record is not found …`), where it says nothing about the second
> obligation. The `count == 2` guard is unchanged and is what keeps this narrow
> — a three-obligation criterion is never suppressed however it is worded.
>
> **`Then` was counted outside a Given/When/Then criterion.** `obligation_count`
> took `max(shall_count, then_count)` regardless of shape, so a **precedence
> chain** scored as plural: ``resolves safeStorage first, then `GITHUB_TOKEN`,
> then `undefined` `` states one resolution rule and carries no modal verb at
> all. Four criteria (filament-ide, filament-ide-rs, quoin ×2) were flagged this
> way, none of them Given/When/Then-shaped. `then` now counts only for a
> `GivenWhenThen` shape, and a criterion's modals outvote it rather than tying
> with it — `max` let a narrative `then` win over the obligations actually
> stated.
>
> Neither change touches `DEFAULT_SEVERITY`; CON-1 still gates promotion.

> **CR-025 note (2026-08-07):** `vacuous-outcome` fired on a **noun**. The
> built-in vacuity set carried bare `functions`, which matched
> *"a spec requirement node can be traversed to the code **functions** that
> implement it and the tests that verify it via typed edges"* — a criterion
> naming a concrete traversal. This is the collision the set already anticipated
> for `work`, whose doc comment records that the corpus uses it as a noun far
> more often than as a predicate; `functions` is no different, and the fix is
> the same one: qualify it (`functions correctly`, `functions properly`,
> `functions as expected`, `functions independently`). Corpus impact is exactly
> two cells — the StR-013 criterion above stops firing, and
> py-observability's *"In-process metric collection functions independently of
> exporters"* keeps firing through the qualified form.

> **CR-026 note (2026-08-07):** CR-017's mask read only the **first** backtick
> of a run, so a ``double-tick`` span — the form used to quote a fragment that
> itself contains a code span — degenerated into an empty span and left the
> keywords inside it unmasked. The criterion quoting them was then read as
> though it used them. This was found by dogfooding: the AC-15 row added by
> CR-024 quotes ``the task SHALL render `skipped` and SHALL NOT execute`` and
> was itself flagged `non-singular` and `non-canonical-shape` by the very fix it
> documents. Span matching now follows CommonMark — a run of N backticks is
> closed by the next run of exactly N, a longer run is content, an unbalanced
> run still opens no span — and the mask stays byte-length-preserving, which
> `outcome_clause` depends on.

> **CR-027 note — CON-1 satisfied for two checks (2026-08-07).**
> `ac:vacuous-outcome` and `ac:non-singular` are **promoted to `error`**. CON-1
> requires a corpus baseline sweep plus explicit user sign-off; both are done,
> and the promotion is declared where CON-2 says it must be — the FR-048
> `grammar_severity` map in the `spec-artifacts-iso` manifest (v0.8.0), reaching
> consumers through `quoin` v0.10.0.
>
> **`DEFAULT_SEVERITY` is unchanged.** CON-1 forbids *shipping the engine
> default* promoted, not promotion itself; the engine still ships every `ac`
> check advisory and the module opts in. Nothing about this note licenses
> editing that constant.
>
> Measured over 4,448 docs / 192 repos, worktree-deduped:
>
> | check | baseline | after CR-024/025/026 | after the corpus sweep |
> |---|---|---|---|
> | `ac:vacuous-outcome` | 44 | 41 | **0** |
> | `ac:non-singular` | 48 | 24 | **0** |
>
> Of the 92 baseline findings, **27 were checker defects** (CR-024, CR-025,
> CR-026) and 65 were real, fixed across 29 repos with each document validated
> before its PR opened. The split is the whole lesson: a check with a 50%
> false-positive rate cannot be promoted, and the rate was invisible until the
> findings were read one by one. This is CR-017's lesson holding a second time —
> triage before editing prose.
>
> **The other three checks stay `warning`.** `ac:non-canonical-shape` is 1,099
> findings (10.0% of cells) and far too large to gate on — its sweep is
> agent-ix/quire-rs#29. `ac:unclassifiable` (44) and `ac:vague-response` (109)
> are unchanged in status; neither has been re-sampled for precision since
> CR-019, and promotion of either needs its own CON-1 pass.
>
> The figures in agent-ix/quire-rs#21 and in the CR-022 proposal were taken over
> a corpus that counted worktree duplicates and are corrected here and in
> `plan/Plan-001-ac-grammar-coverage/tasks/Task-009-ac-grammar-baseline-sweep.md`.
> Triage detail: `reports/2026-08-07-ac-promotion-triage.md`.

Each `ac` finding SHALL carry `grammar: "ac"`. The framework SHALL route `ac`
findings into `ValidationResult` by severity per FR-042. The rollout default
is explicit: every `ac` check ships advisory (`warning`) at most, and each
check is individually suppressible (`off`) or promotable per
[FR-048](./FR-048-per-check-grammar-severity.md).

`quire-cli` `validate --summary` SHALL surface findings for **any** grammar in
the active bundle. The summary parser SHALL group findings by the generic
prefix `[<grammar>:<check>]` — replacing the hardcoded `[ears:` prefix — so
the histogram covers every grammar and check.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-047-CON-1 | `ac` checks SHALL NOT ship promoted to `error` by default: promotion waits for a corpus baseline sweep and an explicit user gate, mirroring the FR-042 EARS rollout precedent | Operational | Inspection |
| FR-047-CON-2 | The canonical shape SHALL NOT be made configurable per module: a `preferred_shape`-style option would reintroduce the plurality this FR removes. Suppression uses the FR-048 per-check severity map, the same mechanism every other check uses (CR-013) | Architecture | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-047-AC-1 | A `Criteria` cell asserting an outcome is classified `assertion` (canonical), an obligation-shaped cell `obligation`, a `Given`/`When`/`Then` cell `given-when-then`, and a cell with no modal, no `Given`/`When`/`Then` structure, and no observable signal is classified `unstructured` and yields one `unclassifiable` finding. | Test (TC-707) |
| FR-047-AC-2 | A non-empty `Criteria` cell with no modal verb is still segmented and checked; an empty cell yields no statement. | Test (TC-708) |
| FR-047-AC-3 | A cell with two `shall` obligations or two `Then` clauses yields exactly one `non-singular` finding; the positive/negative pair idiom (`X yields a finding; Y yields none`) yields none. | Test (TC-709) |
| FR-047-AC-4 | A cell whose outcome clause uses a vague verb over an abstract object yields a `vague-response` finding; the same cell with the object present in the merged lexicon yields none. | Test (TC-710) |
| FR-047-AC-5 | A cell headed by a vacuous predicate with nothing else to check (`Navigation works`) yields a `vacuous-outcome` finding; the same predicate alongside a concrete-object signal, a lexicon term, or a declared observable-result verb (`Volumes are correctly mounted into the container`) yields none. | Test (TC-711) |
| FR-047-AC-6 | The `ac` grammar runs on the `Criteria` column of each archetype whose contract declares a binding-criteria table, under that archetype's own heading — FR/NFR under `Acceptance Criteria` with `-AC-` supplements, StR under `Validation Criteria` with `-VC-` supplements — and on nothing else: US and IT yield no findings even when they carry an improvised `Acceptance Criteria` table, an `Acceptance Criteria` section on an StR is not its binding section, an FR `Constraints` cell and an NFR `Statement` receive EARS findings only, and an archetype with no criteria table contributes nothing (CR-020). | Test (TC-712) |
| FR-047-AC-7 | An `ac` finding carries `grammar: "ac"`, a stable check id, the statement excerpt, a 1-based line number, the classified shape, and a severity, and routes into `ValidationResult` per its severity. | Test (TC-713) |
| FR-047-AC-8 | `quire validate --summary` histograms findings by the generic `[<grammar>:<check>]` prefix: a corpus emitting both `[ears:*]` and `[ac:*]` findings shows both in the summary. | Test (TC-714) |
| FR-047-AC-9 | The `ac` grammar entry point is exposed through the existing grammar PyO3 surface and returns the same findings as the in-process Rust call for a fixture document. | Test (TC-715) |
| FR-047-AC-10 | An `obligation`-shaped cell and a `given-when-then`-shaped cell each yield one `non-canonical-shape` finding while still classifying as that shape (their other checks run on their outcome clause); an `assertion` cell yields none. | Test (TC-751) |
| FR-047-AC-11 | Fenced code blocks and blockquotes inside a `### <doc-id>-AC-N` supplement section are skipped: statements inside them are not segmented and yield no `ac` findings, while the surrounding supplement prose is still checked. | Test (TC-754) |
| FR-047-AC-12 | Both vocabularies are module data: a module's `observable_verbs` registry merges first-wins over the built-in defaults (a module-added verb suppresses `vacuous-outcome` and gives the cell a predicate), a module's `vacuous_predicates` registry likewise extends the built-in vacuity set, and with no module declaration both built-in default sets apply unchanged. | Test (TC-757) |
| FR-047-AC-13 | A grammar keyword inside an inline code span is a mention, not a use: a cell reading ``a statement with two `shall` clauses yields one finding`` classifies `assertion` and yields no `non-canonical-shape` or `non-singular` finding, while the same cell with the modal unquoted classifies `obligation`; the span's contents are still read by the signal and lexicon checks, and an unbalanced backtick opens no span. | Test (TC-761) |
| FR-047-AC-14 | A predication whose copula is elided is a predicate: a cell headed by an existential or quantifier (`No refresh_token in response body`, `Only one active password_reset token per user at any time`) or carrying a predicative adjective (`No credential-material field present`, `Loki datasource visible in Grafana`, `Lockout response identical to normal failure`) classifies `assertion` and yields no `unclassifiable` finding, while a bare noun phrase, a bolded heading and a dangling prose fragment each still classify `unstructured` and yield one. | Test (TC-763) |
| FR-047-AC-15 | The positive/negative pair idiom is recognized by its second obligation rather than by a separator (CR-024): ``SHALL render `skipped` … and SHALL NOT execute``, `` SHALL set `git_url` but `github_url` SHALL be None ``, and `SHALL reject … . No Secret deletion SHALL occur.` each yield no `non-singular` finding, while two positive obligations joined the same way (`SHALL emit `A` and SHALL persist `B`.`) still yield one, a criterion whose *condition* contains `not` still yields one, and a three-obligation criterion yields one however it is worded. | Test (TC-775) |
| FR-047-AC-16 | `Then` counts as an obligation separator only in a `given-when-then` criterion that states no modal (CR-024): a precedence chain (``resolves safeStorage first, then `GITHUB_TOKEN`, then `undefined` ``) yields no `non-singular` finding, while a `Given`/`When`/`Then` cell with two `Then` clauses still yields one. | Test (TC-776) |
| FR-047-AC-17 | A vacuous predicate that is also a common noun does not fire on the noun (CR-025): a criterion reading `the code functions that implement it` yields no `vacuous-outcome` finding, while the qualified predicate (`functions independently of exporters`) still yields one. | Test (TC-777) |
| FR-047-AC-18 | A backtick run masks to its matching run (CR-026): a criterion quoting a modal inside a double-tick span classifies `assertion` and yields no `non-singular` or `non-canonical-shape` finding, while an unbalanced run opens no span and leaves a following unquoted modal counted. | Test (TC-778) |

## Dependencies

- **Upstream**: [FR-042](./FR-042-requirement-grammar-check.md) (the grammar framework and its section/table binding), [FR-043](./FR-043-module-concrete-lexicon.md) / [FR-044](./FR-044-project-glossary-lexicon.md) (the lexicon consumed by `vague-response`), [FR-010](./FR-010-query-api.md) (table extraction)
- **Downstream**: [FR-048](./FR-048-per-check-grammar-severity.md) (per-check severity promotion), the authoring and review workflows consume `ac` findings; the AC quality gate feeds the coverage rollup ([FR-050](./FR-050-declarative-coverage-computation.md))
