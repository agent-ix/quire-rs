---
id: SR-055
title: "Code review — EPIC #264 session 2 (quire-rs 049b840..816e187, qa-corpus 3e73db2..2bc486d)"
type: SpecReview
analysis: code-review
scope: "tests/corpus_case/mod.rs, tests/corpus_cases.rs, src/traceability.rs, src/coverage.rs, scripts/audits/check_spec_structure.sh, spec/, qa-corpus: bounds.py, verify.py, corpus.yaml, scripts/"
review_set: subset
---

## Summary

First independent review of the session-2 phase of EPIC #264 — the phase that
answered an outside review by adding `case_schema`, `witness_channels`,
FR-065-AC-46/AC-47, TC-1043, and the two Python self-tests. Twelve findings,
two of them high; both highs are the *same defect class the phase was written
to close*, recurring inside the fix.

## Verdict

**FAIL** — `case_schema.variant_forbidden` is a declared contract with zero
consumers, shipped in the commit whose change record says the point is that
"the list lives in `corpus.yaml` and both readers are held to it"; and the
engine's status-lie detector is structurally inert on three of the five
matrices in `spec/tests.md`, ecosystem-wide.

## Method

Every finding below was reproduced against the working tree, not read off the
diff. Gates were run rather than assumed:

- `quire-rs`: `make ci` — **exit 0**, `spec_validate: 130 document(s), 0 failed,
  41 warning(s)` (the pre-phase baseline, unmoved).
- `qa-corpus`: `QUIRE=<workspace build> make ci` — schema-selftest 8/8,
  bounds 77/45/4/10, verify 77/77 over 35 pairs, parity-selftest 6/6.
- Engine probes used `/home/peter/.cargo-target/debug/quire`, reporting
  `engine 816e187`, 4 capability tokens.

FND-002 and FND-006 are measurements, reproduced below in Evidence.

## Findings

| ID      | Severity | Summary                                                                                  | Refs                                        | Escape Cause                        |
| ------- | -------- | ---------------------------------------------------------------------------------------- | ------------------------------------------- | ----------------------------------- |
| FND-001 | high     | `case_schema.variant_forbidden` is declared and read by nothing; `bounds.py` hardcodes the same set | qa-corpus `corpus.yaml:171`, `bounds.py:204` | correct-requirement-no-evidence     |
| FND-002 | high     | The status-lie check is inert on 3 of 5 matrices in `spec/tests.md`: declared column `Status`, authored column `Coverage Status` | `spec/tests.md:30,41,61`, `src/coverage.rs:1229` | implementation-bug-despite-evidence |
| FND-003 | medium   | `make new-case --module <non-ecosystem>` exits 0 writing a `case.yaml` `bounds.py` rejects | qa-corpus `scripts/new_case.py:100`, `scripts/schema_selftest.py:174` | correct-requirement-no-evidence     |
| FND-004 | medium   | AC-47 is enforced in the Rust reader only; `verify.py` silently drops an unknown declared channel | qa-corpus `verify.py:608`                   | wrong-requirement                   |
| FND-005 | medium   | `by_kind.forbidden` and `by_kind.values` are implemented but never mutation-verified       | qa-corpus `scripts/schema_selftest.py:53`   | correct-requirement-no-evidence     |
| FND-006 | medium   | AC-46 discriminates 5 of 35 pairs only through channels added after those pairs failed     | qa-corpus `corpus.yaml:394`                 | correct-requirement-no-evidence     |
| FND-007 | medium   | 22 FR-coverage rows read done while citing TCs the same file marks pending; FR-065's is one | `spec/tests.md:64-99,134`                   | missing-requirement                 |
| FND-008 | low      | `new_case.py`'s `meta["case"]` branch is provably unreachable                              | qa-corpus `scripts/new_case.py:128`         | correct-requirement-no-evidence     |
| FND-009 | low      | `check_spec_structure.sh` computes `prefix` and never uses it; 3 of the 5 planned gates were dropped with no ticket | `scripts/audits/check_spec_structure.sh:21` | missing-requirement                 |
| FND-010 | low      | Nothing asserts `CaseMeta`'s field set is a subset of what `case_schema` declares          | `tests/corpus_cases.rs:600`                 | missing-requirement                 |
| FND-011 | low      | A malformed `if_field_is_not` rule raises a bare `ValueError`, not a `CorpusError`         | qa-corpus `bounds.py:135`                   | correct-requirement-no-evidence     |
| FND-012 | low      | A 260-character comment line in a block wrapped at 80                                      | `src/coverage.rs:697`                       | missing-requirement                 |

## Evidence

### FND-001 — a contract with no code under it, again

`corpus.yaml:171` declares:

```yaml
  variant_forbidden:
  - case
  - mode
  - module
  - kind
  - pending
```

`rg -n variant_forbidden` over both repositories returns **one hit: the
declaration itself.** Neither reader consumes it. `bounds.py:204` enforces the
identical rule from a Python literal:

```python
            protected = {"case", "mode", "module", "kind", "pending"}
```

This is `result_record` — the contract this same phase *deleted* under #279 for
being declared and unimplemented — reintroduced in `801afd5`, the commit whose
change record (CR-126) states the principle it violates: *"a second
hand-written list in Python would be that defect one level up."*

The failure is silent in both directions. Editing `variant_forbidden` changes
no behaviour; editing the Python set makes the declaration false. Nothing
compares them.

### FND-002 — the status-lie detector cannot see three of five matrices

`spec-artifacts-process` declares one model-wide status vocabulary:

```yaml
  status:
    column: Status
    complete: ["✅"]
```

`src/coverage.rs:1229` reads it as `row.cell(&status.column)`, and a missing
column is `None`, which skips the check with no diagnostic. In
`quire-rs/spec/tests.md` the column is named `Status` in two tables (the TC
ledger, L148; the NFR table, L107) and **`Coverage Status`** in three (StR L30,
US L41, Functional Requirement Coverage L61).

Measured on a clean `git archive` of `816e187`:

```
BEFORE status_lies: 0   unbacked functional-coverage rows: 10
# rename the 3 `| Coverage Status |` header cells to `| Status |`
AFTER  status_lies: 6
    functional-coverage | FR-002 JsonValue merge-validate | ✅ Complete | line 64
    functional-coverage | FR-003 schema_for surfaces fs schema | ✅ Complete | line 65
    functional-coverage | FR-009 Slug-line ID | ✅ Complete | line 71
    functional-coverage | FR-013 Archetype loader | ✅ Complete | line 75
    functional-coverage | FR-014 Module activation | ✅ Complete | line 76
    functional-coverage | FR-016 Fallback locators | ✅ Complete | line 77
```

Six rows assert **✅ Complete** over trace ids nothing backs, and the detector
built to catch exactly that reports zero.

This is not a `quire-rs` document defect. The canonical template at
`quoin/skills/spec-matrix/assets/test-matrix-template.md:17,22,28` writes
`Coverage Status`, and **206 files under the dev root carry that header.** The
detector has therefore never fired on a stakeholder, user-story or
functional-coverage matrix in any repository.

It is the same shape as the section-name mismatch (#270/#272) and the id-column
mismatch (#318), one column over: a declared name that matches nothing, and a
silent skip where FR-063 already requires *"not computed is a value, not a
zero"*.

### FND-003 — the on-ramp is still broken for one argument

`check_scaffolder()` was added this phase precisely because nothing ran the
scaffolder. It runs it three times and passes `--module ecosystem` every time
(`schema_selftest.py:174`). Reproduced on a clean checkout:

```
$ python3 scripts/new_case.py --mode minting --case probe-nonecosystem \
      --kind failure --module variants/no-implements-declaration --issue ...
scaffolded cases/minting/probe-nonecosystem
rc=0

$ bounds.validate_case(<the file it wrote>)
PROBLEM: `relaxation_ticket` is required here — FR-065-CON-3 — a variant
         binding must name the ticket it is sizing
```

The scaffolder exits 0 having written metadata the corpus rejects. The
conditional that requires `relaxation_ticket` is new in this phase, and
`new_case.py` was not updated for it.

### FND-004 — AC-47 in one reader, violated in the other

`tests/corpus_cases.rs:1508` requires every declared channel to be one
`CaseExpect::channel_names()` knows, and fails loudly otherwise. `verify.py:608`
does the opposite:

```python
            restricted = {k: v for k, v in live.items() if k in channels}
```

A channel name in `witness_channels` that no `expect.yaml` key matches is
silently absent from `channels`' effect — the restriction simply keeps fewer
keys, weakening AC-46 for that mode with no signal. That is the precise
behaviour AC-47 forbids ("rejected, rather than dropped"), in the reader that
FR-065 does not cite for AC-47.

FR-065 is honest about this — AC-47 cites `TC-1028` alone — so it is not a
status lie. It is the same one-reader asymmetry AC-42 carried until #337, in a
criterion written one commit later. `verify.py` already maintains the `KNOWN`
set (line 98) this needs.

### FND-005 — two implemented rules, never observed to reject anything

`schema_selftest.py`'s six mutations cover: missing required, duplicate id,
present-but-empty, unknown field, wrong type, and `pending` without
`pending_reason`. `validate_case` also implements `by_kind.forbidden`
("a `{kind}` case may not declare `{field}`") and `by_kind.values`
("`{field}: {value}` is not one of {allowed}"). Neither substring appears in
any mutation's expectation, so neither branch has been observed to fire — the
standard this file's own docstring sets.

### FND-006 — where AC-46's teeth actually are

Re-graded all 35 pairs under three narrowings of the declared channel sets:

```
35 (case, control) pairs graded

as-declared       0 pair(s) fail AC-46
minus validate    0 pair(s) fail AC-46
minus widened     5 pair(s) fail AC-46
                   tag-on-non-test-function-{python,rust,typescript} [attachment]
                   stale-name-correct-trace [detection]
                   test-name-id-in-fn-name  [detection]
minus both        5 pair(s) fail AC-46
```

Two readings, and both matter:

- The **stated weakening** — `validate_contains`/`validate_absent` admitted as a
  witness in every mode — costs nothing today. Zero pairs depend on it. The
  concern raised when it landed does not reproduce.
- The **two widenings made after seeing failures** — `attachment += metrics`,
  `detection += untracked_symbols, unbacked_rows` — carry 5 of 35 pairs
  entirely. Removing them, AC-46 collapses to AC-42 for those pairs.

The widenings were justified from the fixtures' own comments and that
justification holds on re-reading. The finding is not that they are wrong; it
is that **nothing records the dependency and nothing gates the next one.**
`witness_channels` is authored in the same file, by the same hand, as the cases
it grades, and AC-47 is a type check, not a meaning check. A future widening
made for the wrong reason is indistinguishable from these two.

### FND-007 — two status columns over one fact, 22 disagreements

`spec/tests.md` carries a per-TC `Status` column and a per-FR `Coverage Status`
column describing the same underlying facts. Nothing compares them. Measured:
**22 of the Functional Requirement Coverage rows are marked done while citing a
TC the same document marks 🚧.** The direction of the error varies — `TC-636`
carries a live `#[trace("TC-636")]` in `src/registry.rs` while its ledger row
says 🚧, so there the *ledger* is stale — which is exactly why a machine
comparison is worth more than another hand pass.

FR-065's own row (L134) is one of the 22: it reads `✅` while citing TC-1013,
TC-1014 and TC-1015, all three of which the ledger marks *"🚧 awaiting #268"*
and none of which exists in code. `816e187` (CR-131) is the last commit to touch
that line, and widened its range to `AC-1..47` without touching the mark.

FND-002 does not catch this class: the row is *backed* (its other cited TCs
bind), so no unbacked-row rule applies.

### FND-008 — an unreachable branch

```python
    case_id = f"{args.case}-control" if args.kind == "control" else args.case
    ...
    if args.kind == "failure" and args.case != case_id:
        meta["case"] = args.case
```

`case_id` differs from `args.case` only when `kind == "control"`, which the
first conjunct excludes. `meta["case"]` is never written, for any kind. Benign
— both readers derive `case` — but the comment above it describes behaviour the
code cannot perform, which is a finding on its own under the repo's Rust idiom
doc and applies equally here.

### FND-009 — the gate shipped at two fifths, silently

Wave 1.2 of the approved plan named five engine-independent structural gates:
conflict markers, index↔files completeness, **duplicate artifact ids**,
**unresolved local links**, **frontmatter schema validation**.
`check_spec_structure.sh` ships the first two. CR-124 is accurate — it says
"checks two things" — so there is no false claim; there is also no ticket for
the other three, so the narrowing is recorded nowhere a reader will find it.

Separately, `check_spec_structure.sh:21` computes `prefix` and never reads it.

### FND-010 — the untested direction of TC-1043

TC-1043 asserts `case_schema.required ∪ optional == case_schema.types` — both
sides of that equality are the corpus's. Nothing holds `CaseMeta`'s actual
field set to the declaration. Today they match exactly (15 fields), so this is
latent: a Rust-only field would be silently unreachable, since `bounds.py`'s
unknown-field rule means no case can carry it. The test's own docstring is
candid that the type half could not be falsified by mutation; this is the
adjacent gap.

## Notes

FND-002 and FND-007 are pre-existing conditions surfaced by reviewing this
phase, not defects it introduced — except that CR-131 edited the FR-065 row in
FND-007 and left the mark standing. They are recorded here because the phase's
stated purpose is detection integrity, and a detector that has never fired is
the thing this EPIC exists to find.
