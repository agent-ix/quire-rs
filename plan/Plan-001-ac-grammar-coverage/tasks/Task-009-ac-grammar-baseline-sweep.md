---
id: Task-009
title: "Corpus cleanup — AC-grammar baseline sweep (user-gated promotion)"
type: Task
status: completed
track: C
priority: P2
relationships:
  - target: ix://agent-ix/quire-rs/Task-003
    type: depends_on
  - target: ix://agent-ix/quire-rs/FR-047
    type: references
  - target: ix://agent-ix/quire-rs/TC-714
    type: verifies
---
# Task-009: Corpus cleanup — AC-grammar baseline sweep (user-gated promotion)

## Scope

**Corpus/quality work class — own branch (`chore/ac-grammar-baseline`), never
mixed with feature branches.** After Gate G1: run the `ac` grammar over this
repo's spec corpus, record the per-check baseline histogram (via the generic
`--summary`), fix mechanical advisory findings in this repo's ACs (wording
only — a finding that implies a requirement change gets a CR note instead),
and re-record the post-fix baseline. Precedent: the FR-042 EARS rollout
baseline (791 vague / 333 non-singular before enforcement).

## Subtasks
- [x] **Baseline capture.** Per-check counts over `spec/**/*.md`, committed to
  this task's Notes.
- [x] **Mechanical fixes.** Done, but not as scoped: reading the seven findings
  showed six of them were checker defects (a quoted keyword read as a use), so
  the fix landed in the checker as **CR-017** rather than in the prose. The one
  genuine wording change is FR-047-AC-1. See the CR-017 re-baseline below.
- [x] **Promotion proposal.** **Signed off and executed, 2026-08-07.**
  `ac:vacuous-outcome` and `ac:non-singular` are now `error`, declared in the
  `spec-artifacts-iso` v0.8.0 manifest's FR-048 `grammar_severity` map and
  reaching consumers via `quoin` v0.10.0. `DEFAULT_SEVERITY` is unchanged, so
  CON-1 holds: the engine still ships advisory and the module declares the
  promotion (the lever CON-2 sanctions).

  **The proposal's own figures were wrong and are corrected here.** They were
  taken over a 16,449-cell corpus that counted worktree duplicates. Re-derived
  on the deduplicated corpus — 4,448 docs / 192 repos / 11,022 cells at the
  start of the work:

  | Check | Proposal said | Actual baseline | Outcome |
  |---|---|---|---|
  | `ac:vacuous-outcome` | 31 / 0.19% | **44** / 0.4% | → `error`, corpus at **0** |
  | `ac:non-singular` | 121 / 0.74% | **48** / 0.4% | → `error`, corpus at **0** |
  | `ac:unclassifiable` | 82 / 0.50% | **44** / 0.4% | stays `warning` |
  | `ac:vague-response` | 101 / 0.61% | **109** / 1.0% | stays `warning` |
  | `ac:non-canonical-shape` | 3,458 / 21.0% | **1,099** / 10.0% | stays `warning` — agent-ix/quire-rs#29 |

  Getting to zero took two passes, recorded in
  `reports/2026-08-07-ac-promotion-triage.md`:

  1. **27 of the 92 findings were checker defects**, fixed as CR-024/025/026 and
     shipped in v0.17.0 — the positive/negative pair idiom was tied to its
     separator (19), `then` was counted outside a Given/When/Then criterion (4),
     `functions` fired as a noun (3), and a double-backtick span left the
     keywords it quoted unmasked (1). The CR-017 lesson held a second time:
     triage before editing prose.
  2. **The remaining 65 were real**, fixed across 29 repos, one PR each, each
     document validated before its PR opened.

  Final: **4,448 docs / 192 repos / 11,045 cells, both promoted checks at 0.**

## Deliverables
- Baseline report (before/after counts) in this task file; corpus wording fixes
  on the cleanup branch; a promotion proposal, signed off and executed —
  `spec-artifacts-iso` v0.8.0 declares both promoted checks, `quoin` v0.10.0
  carries the pin. Triage detail: `reports/2026-08-07-ac-promotion-triage.md`.

## Baseline (Gate G1 dry-run, 2026-08-04 — counts only, no corpus edits)

Swept `spec/**/*.md` with the `ac` grammar through the PyO3 surface at
Task-003 completion (44 FR documents; `ac` binds to FR only):

| Check | Count |
|---|---|
| `ac:unclassifiable` | 322 |
| `ac:no-observable-outcome` | 12 |
| `ac:non-canonical-shape` | 2 |
| `ac:non-singular` | 2 |
| `ac:vague-response` | 2 |
| **total** | **340** |

Heaviest documents: FR-011 (16), FR-013 (14), FR-047 (14), FR-032 (13),
FR-033 (13), FR-051 (13).

Read of the shape: the corpus authors acceptance criteria as declarative
assertions (`A manifest declaring X loads, and Registry::grammar_severity()
returns the merged map`) — neither EARS nor Given/When/Then, so
`unclassifiable` is a correct classification, not a classifier defect
(sampled against FR-042's own AC table). It does mean **`ac:unclassifiable`
would flag ~95% of the corpus if promoted**, which is the central input to the
promotion proposal below — the realistic first promotion candidates are the
low-count checks, with `unclassifiable` staying `warning` (or `off`) until the
corpus converges on EARS.

**Promotion remains user-gated (FR-047-CON-1): no `grammar_severity` default
has been authored, and none will be without explicit sign-off.**

## Re-baseline after CR-013 + CR-014 (2026-08-04)

The 340-finding baseline above measured a grammar whose canon and whose two
high-volume checks were both wrong — see `~/dev/reports/2026-08-04-ac-grammar-fit.md`.
After CR-013 (assertion is the canonical shape) and CR-014 (the vacuity and
predicate checks replace the allowlist ones, binding widened, inflector fixed):

| Check | Baseline | After |
|---|---|---|
| `ac:unclassifiable` | 322 | **0** |
| `ac:no-observable-outcome` → `ac:vacuous-outcome` | 12 | **0** |
| `ac:non-canonical-shape` | 2 | **7** |
| `ac:vague-response` | 2 | 2 |
| `ac:non-singular` | 2 | 1 |
| **total over 44 FR documents** | **340** | **10** |

The seven `non-canonical-shape` findings are obligation-shaped criteria in
FR-047, FR-042, and FR-043 — the documents that describe the grammar, written
before the canon changed. They are the entire remaining corpus debt, and fixing
them is a wording change to five-odd cells rather than the 322-cell rewrite the
original baseline implied.

Ecosystem-wide the same engine reports 3,956 findings across 5,027 requirement
documents, down from 14,487.

## Re-baseline after CR-017 (2026-08-05) — Track C

Diagnosing the seven `non-canonical-shape` findings above as "a wording change
to five-odd cells" was wrong. Every one of them quoted a keyword as **example
data** (``a statement with two `shall` clauses yields exactly one
`non-singular` finding``) — the documents that describe a grammar necessarily
quote its keywords. Rewording them would have hidden a classifier defect
(mention read as use) behind prose. CR-017 fixes the classifier instead; the
only corpus edit is FR-047-AC-1, where `Given/When/Then` was the one keyword
mentioned outside a code span and is now backticked like every other mention.

| Check | Baseline | After CR-014 | After CR-017 |
|---|---|---|---|
| `ac:unclassifiable` | 322 | 0 | **0** |
| `ac:vacuous-outcome` | 12 | 0 | **0** |
| `ac:non-canonical-shape` | 2 | 7 | **0** |
| `ac:non-singular` | 2 | 1 | **0** |
| `ac:vague-response` | 2 | 2 | **2** |
| **total** | **340** | **10** | **2** |

The two remaining findings are the PyO3-parity criteria of FR-042 and FR-047
("*the entry point is exposed through the … binding and returns the same
findings as the in-process Rust call*"). Both name a concrete outcome in the
second clause, so `vague-response` firing on *exposed* is a false positive of
the FR-042 vague-verb heuristic rather than corpus debt — recorded, not
reworded, and not a reason to touch the wording of a truthful criterion.

Ecosystem-wide over the same 199 repos: **3,956 → 3,949** findings, with no new
finding in any check (16 mention-only false positives removed; 9 true positives
restored that code-span-blind sentence segmentation had been splitting apart,
plus 1 newly surfaced).

**Promotion is still user-gated and still not done (FR-047-CON-1):** no
`grammar_severity` default has been authored. Reaching zero on four of five
checks in this corpus is not a licence to promote — the ecosystem still carries
2,632 `non-canonical-shape` findings.

## Notes
- Wider-ecosystem sweeps (other repos' corpora) are follow-up work owned by
  those repos — this task covers quire-rs only.
- Never tune the checker to the corpus from this branch; classifier bugs
  found here are Track A fixes.
