# `ac` promotion triage — 92 findings across 36 repos

Stage 1 of the FR-047-CON-1 promotion of `ac:vacuous-outcome` and
`ac:non-singular` to `error`. Dated 2026-08-07 (the plan named a `-08-08` file;
this is the date the sweep actually ran).

## Measurement

Re-derived, not quoted. The installed `quire` wheel was **0.15.0** — stale
against the shipped 0.16.0 engine — so it was rebuilt from `98bc99a`
(`make wheel`, clean tree at v0.16.0) and reinstalled before the sweep.

```
python3 scripts/ac_corpus_sweep.py --root ~/dev \
    --module ~/dev/spec-artifacts-iso/spec_artifacts_iso --out ac-sweep.json
docs 4448  repos 192  cells 11023
```

| check | findings | rate | repos | issue #21 says |
|---|---|---|---|---|
| `ac:vacuous-outcome` | 44 | 0.4% | 16 | 31 |
| `ac:non-singular` | 48 | 0.4% | 21 | 121 |
| `ac:non-canonical-shape` | 1,099 | 10.0% | 50 | 3,458 / 21.0% |

The two promotion targets match the handoff figures exactly (44 / 48, 16 / 21
repos). `non-canonical-shape` confirms at 1,099 / 10.0%, not the 3,458 / 21.0%
in `quire-rs#21` — that figure counted worktree duplicates. **#21 needs
correcting** (Stage 4, step 6).

One measurement caveat for Stage 3: a finding's `line` is unreliable — it points
at table separator rows for several findings (e.g. `ix-cli-core/FR-007` reports
line 42, a `| --- |` row; `auth-py/FR-002` reports 84 for the AC-9 cell on 85).
Locate cells by their `-AC-n` id, not by the reported line.

## Buckets

| bucket | vacuous-outcome | non-singular | total |
|---|---|---|---|
| checker defect | 3 | 24 | **27** |
| mechanical (corpus edit) | 41 | 23 | **64** |
| document structure | 0 | 1 | **1** |
| authoring residue | 0 | 0 | **0** |

(Final, after the Stage 2 re-sweep measured the fixes. Triage predicted 1 + 23;
the extra three are two more `functions`-as-noun cells and one defect — D4 —
that only surfaced once the first three fixes were in the tree.)

No finding was classified residue at triage. Residue is decided per document in
Stage 3, when the owning FR body is read — a criterion is residue only if its
document genuinely states no concrete outcome.

---

## Checker defects (Stage 2)

Three, not one. The plan predicted D1; D2 and D3 surfaced during triage.

### D1 — `is_positive_negative_pair` is separator-bound (19 findings)

`obligation_count` (`src/grammar/ac.rs:380`) already implements the
positive/negative-pair idiom, but `is_positive_negative_pair` (`ac.rs:395`)
splits **only on `;` and ` while `** and then looks for a negative *word* in the
second half. The dominant corpus idiom joins with ` and `:

> ``When `enabled === false`, the task SHALL render `skipped` with reason
> `"disabled"` and SHALL NOT execute`` — `ix-ui/FR-005-AC-…`

One behaviour stated positively then negatively, counted as two obligations.
Same class as CR-017's mention-vs-use bug: the rule exists, its trigger is too
narrow.

Affected (repo / criterion gist):

| # | repo | criterion |
|---|---|---|
| 1 | auth | `ix local init` SHALL fail … and SHALL NOT create or overwrite any Secret |
| 3 | auth | BFF SHALL resolve tenant to `"T2"` and SHALL NOT read the legacy `"T1"` |
| 4 | auth-py | `kid` SHALL be passed verbatim … SHALL NOT sanitize, transform, retry, or fall back |
| 6 | auth-service | field SHALL be present …; otherwise it SHALL be omitted/null |
| 8 | cloud-manager-ui | SHALL render `<ForcedRotationScreen>` and SHALL NOT mount any protected route |
| 13 | cloudmanager-local-sync | SHALL set `git_url` but `github_url` SHALL be None |
| 19 | ix-agent-core | SHALL be captured as `HookResult(success=False…)` and SHALL NOT prevent subsequent hooks |
| 21 | ix-cli | SHALL reject with exit code 2 …. No Secret deletion SHALL occur. |
| 23 | ix-coder-workflows | SHALL transition to `directed_implementing` and SHALL NOT call any tracker queue API |
| 26 | ix-coder-workflows | `--dry-run` SHALL print the planned creates/updates and SHALL NOT call any write API |
| 27 | ix-coder-workflows | headers SHALL match `^##\s+TASK-…$`. Lowercase … SHALL NOT create tasks. |
| 28 | ix-coder-workflows | SHALL throw a structured error …, and SHALL NOT call any tracker API |
| 31 | ix-coder-workflows | SHALL fire on an exact label match. Substring matches … SHALL NOT trigger. |
| 32 | ix-coder-workflows | SHALL ALSO fire on an open dependency. Depending only on a closed item SHALL NOT trigger. |
| 35 | ix-ui | SHALL render `skipped` with reason `"disabled"` and SHALL NOT execute |
| 41 | scheduler-service | SHALL fail with a descriptive error and SHALL NOT make an HTTP request |
| 42 | scheduler-service | SHALL fail with `AUTH_FAILURE` and SHALL NOT retry |
| 43 | scheduler-service | SHALL mark the execution as `dead` and SHALL NOT retry further |
| 45 | ts-auth-sdk | SHALL be a no-op and SHALL NOT trigger another probe |

**Proposed fix.** Drop the separator dependency. When the count is exactly 2,
treat it as one obligation if the **second** obligation is the negative face:
the second modal is directly negated (`shall not` / `shall never` /
`must not`), or its clause carries a negation marker
(`no|none|nothing|neither|never|not|otherwise`). Rows 13, 21 and 6 need the
clause form (`SHALL be None`, `No Secret deletion SHALL occur`,
`otherwise … omitted/null`); the rest are directly negated. Keeping the
`count == 2` guard is what keeps this narrow — a three-obligation criterion is
never suppressed (see #5, below, which stays mechanical).

### D2 — `then` counted outside a Given/When/Then criterion (4 findings)

`obligation_count` takes `max(shall_count, then_count)` regardless of shape, so
a **precedence chain** using `then` as a sequencer scores as multiple
obligations even with no modal verb at all:

| # | repo | criterion |
|---|---|---|
| 14 | filament-ide | `getGithubToken` resolves safeStorage first, then `GITHUB_TOKEN`/`GH_TOKEN`, then `undefined` |
| 15 | filament-ide-rs | node fill precedence is dim, then hover-center, then highlight-incident, then code layer, then object-type swatch |
| 39 | quoin | `--config-root`, then `IX_HOME`, then `~/.ix` select the config root in that precedence |
| 40 | quoin | `--org` takes precedence over `QUOIN_ORG`, then the stored configuration, then the `origin` remote |

None of the four is Given/When/Then-shaped (`classify` returns `Assertion` —
there is no `given`/`when` before the `then`), so `then` is not a consequent
marker in any of them.

**Proposed fix.** `then` is an obligation separator only in a
`GivenWhenThen`-shaped criterion, and the modal count wins when there is one:
`count = shall_count if shall_count > 0 else then_count (GWT only)`, replacing
the unconditional `max`. `check_statement` already has `shape` in hand at
`ac.rs:166`, so it is a parameter, not a re-classification.

### D3 — `functions` matches as a noun (1 finding)

`BUILTIN_VACUOUS_PREDICATES` (`src/grammar/mod.rs:180`) contains bare
`functions`. It fires on a noun:

> "A spec requirement node can be traversed to the code **functions** that
> implement it and the tests that verify it via typed edges."
> — `filament-ide-rs/spec/code-graph/stakeholder/StR-013…`

That criterion names a concrete traversal; nothing about it is vacuous. This is
the same noun-collision the set already handles for `work` — the doc comment at
`mod.rs:303` says bare `work` is excluded "because the corpus uses it far more
often as a noun".

**Proposed fix.** Mirror the `work` precedent: replace bare `functions` with the
qualified forms (`functions correctly`, `functions properly`,
`functions as expected`, `functions independently`). Corpus impact is exactly
two cells — StR-013 stops firing (correct), and
`py-observability/FR-009` ("In-process metric collection **functions
independently** of exporters") keeps firing via the qualified form.

### D4 — a double-backtick span left its contents unmasked (CR-026)

Found by dogfooding, after the fixes above shipped into the tree: the `AC-15`
row written to document CR-024 quotes
``the task SHALL render `skipped` and SHALL NOT execute`` and was itself flagged
`non-singular` and `non-canonical-shape`. CR-017's mask reads only the **first**
backtick of a run, so a double-tick span — the form used to quote a fragment
that itself contains a code span — degenerates into an empty span and leaves the
keywords inside it unmasked.

**Fix.** CommonMark run matching: a run of N backticks is closed by the next run
of exactly N, a longer run is content, an unbalanced run still opens no span.
The mask stays byte-length-preserving, which `outcome_clause` depends on.

### Post-fix counts — measured

Re-swept on the fixed engine (`make ci-python` rebuilds and installs the wheel):

| check | before | predicted | **measured** |
|---|---|---|---|
| `ac:non-singular` | 48 | 25 | **24** |
| `ac:vacuous-outcome` | 44 | 43 | **41** |

Both beat the prediction, and every difference is a false positive removed:

- `non-singular` 48 → 24. The 23 predicted defect findings all cleared; the 24th
  is `quire-rs`'s own `FR-047-AC-15`, cleared by D4.
- `vacuous-outcome` 44 → 41. D3 removed **three**, not one: besides the
  filament-ide-rs StR-013 criterion, `py_code` and `usul-code` both carry
  *"Local imports inside **functions** are handled"* — the same noun, missed at
  triage because the sweep's statement excerpt made it read as a predicate.

**No finding appears anywhere it did not before.** The full diff across all
grammars is: 24 `ac` findings gone, 0 new. Repo-level totals for `ears:*` are
unchanged.

---

## Corpus work (Stage 3)

### `non-singular` — 24 mechanical + 1 structural

Genuine multi-obligation cells; each splits into one criterion per obligation,
appending new `-AC-n` ids and updating every trace in the same PR.

| # | repo | why it is genuinely plural |
|---|---|---|
| 2 | auth | forbidden-verb list + "each attempt SHALL return 403" |
| 5 | auth-py | bounded time + return-or-raise + SHALL NOT hang (3 obligations) |
| 7 | auth-service | include `scope` + include `rotation_origin` + MUST NOT appear on normal branch |
| 9 | cloud-manager-ui | URL preserved + view mounts |
| 10 | cloud-manager-ui | `must_rotate=false` + cookie cleared + lands on `/login` |
| 11, 12 | cloud-manager-ui-services | icon mapping + fallback; view toggle + `ServiceCard` |
| 16 | filament-parser-lib | precedence + resulting `ArtifactType.NFR` value |
| 17 | gateway-bff-contract | two distinct poll scenarios in one cell |
| 18 | identity | default username rule + distinctness of two invites |
| 20 | ix-cli | delete + recreate within 30s + value differs |
| 24, 25 | ix-coder-workflows | gate + block-transition; spawnSync + record exit codes |
| 29, 30, 33 | ix-coder-workflows | render rule + doc obligation; configurable + inclusive comparison; embed marker + strategy set + fail on unknown |
| 34, 36, 37, 38 | ix-ui | glyph + bracketed text; render summary + propagate; unmount + reject; tick suppressed + frozen frame |
| 44 | scheduler-service | dispatch disabled + only legacy available |
| 46 | ts-auth-ui | renders `Error 400` + remains mounted |
| 47, 48 | workflow-worker-pool | JSON parsed / non-JSON as text; store returns URI / fetch returns data |

**Structural (1):** `ix-cli-core/spec/functional/FR-007-encrypted-file-fallback.md`
— the `FR-007-AC-2` supplement section has `AC-2a` and `AC-2b` authored as bold
bullets *inside* it, so the grammar reads all three as one statement. The fix is
document structure (promote `AC-2a`/`AC-2b` to their own
`### FR-007-AC-2a` sections), not a reword.

### `vacuous-outcome` — 43 mechanical

All are literal "works" claims; each needs the observable result its FR body
already states.

| repo | n |
|---|---|
| agent-duncan | 21 |
| spec-hierarchy, usul-code | 3 each |
| py_code, ui-data-table, workflow-execution | 2 each |
| auth-fastapi, catalog-service, chat-input, cloud-manager-ui-core, filament-ide-rs, filament-review-service, jest-results, local, py-observability, sync-filament | 1 each |

`agent-duncan` alone is 21 of 43 and is the whole first PR: 15 FR cells of the
form "X works" plus 6 StR validation criteria of the form
"X works (indicator: Y)" — for those the `(indicator: …)` suffix is what needs
the concrete outcome, per the plan.

## Next

Stage 2: implement D1, D2, D3 in `src/grammar/ac.rs` / `src/grammar/mod.rs`
with CR notes on FR-047 and new `TC-` rows, `make ci` + `make ci-python`, ship
v0.17.0, then **re-sweep and re-derive** before any corpus edit.
