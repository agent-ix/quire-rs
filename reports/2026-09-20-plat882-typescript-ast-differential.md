# PLAT-882: TypeScript symbol-scanner AST-port differential, against PLAT-851's baseline

- Date: `2026-09-20`
- Baseline: `reports/2026-09-20-plat851-typescript-scanner-baseline.md` / the shared
  `2026-09-20-plat851-language-scanner-baseline.json` — `1,610` TypeScript symbols
  total (`165` `quire-rs`, `1,445` `filament-ide-rs`; every other repo in the
  corpus, `quire-code-rs`/`quire-contract-ir`/`quire-protocol`/`ecaz`, carries
  **zero** TypeScript — checked in that report, not assumed, so the
  with/without-`ecaz` split is a no-op for this language and is stated once
  here rather than repeated). `3` abandoned files, all `filament-ide-rs`, all
  the PLAT-163 brace-desync shape.
- Old engine: `quire-rs`'s own pristine tree at `08d39ea2ce50db35812f836df65cac18757daa4b`
  — the exact pre-port `src/symbols/typescript.rs` line-structural scanner,
  the same sha PLAT-840/843/845/851 all pinned as the "old engine" checkout.
  Reproduced independently here: `examples/plat843_audit_list.rs` run against
  this branch's own tree gives `165`/`1,445`, matching PLAT-851's published
  baseline exactly. `quire-rs`'s own 165 TypeScript symbols all come from the
  `corpus` submodule (`agent-ix/qa-corpus`, pinned at
  `7442f2770880a4ade303fb23d725804bdef454db` for every run in this report),
  not from any `.ts`/`.tsx` file this repo itself tracks — a
  `git diff 08d39ea2c bc31c2b -- '*.ts' '*.tsx'` pathspec diff (the check this
  report originally cited) cannot see the submodule's contents at all and so
  cannot fail regardless of what changed inside it, which makes it a vacuous
  check (PLAT-882 PR #481 review finding F7). The real guarantee is the
  gitlink: `git ls-tree 08d39ea2c corpus` and `git ls-tree bc31c2b corpus`
  both name the same `7442f277…` commit, and that commit is what every
  `quire-rs`-side count in this report was measured against.
- New engine: this PR's branch (`feat/plat882-typescript-ast`), tree-sitter
  port via `quire-rust-extraction`/`quire-code-parse`, unchanged pin
  (`57b83ba00431914060297bf94fcee31549c9b68b`).
- Target trees held constant: `quire-rs` measured against this branch's own
  worktree (TypeScript source unchanged from the old engine's pinned sha, see
  above); `filament-ide-rs` measured against a disposable `gh repo clone`
  pinned to PLAT-851's own recorded `head_sha`,
  `37c44d907ad419472faca240bd02a0fae9add7c0`. Both engines ran over
  byte-identical source trees per repo — every delta below is engine-only.
- Harness: `examples/plat843_audit_list.rs`, run once per engine per repo for
  the headline counts, per PLAT-843's own reproduction convention — no new
  extraction tooling. **One change to the tool itself, not to what it
  measures**: it used to hardcode a copy of `spec-artifacts-process`'s three
  `source_exclude` globs; it now reads them from the module via
  `Registry::load_module` (the same call `plat840_rust_baseline_sweep.rs`
  already makes), so a differential and a baseline run through this tool can
  no longer silently disagree because someone edited the module without
  remembering this copy existed. The three globs happened to already be
  byte-identical to the module's declared list, so this changes no number in
  this report — it changes what the next report cannot silently get wrong.
  Per-symbol enumeration used the same tool's `path\tqualified_name\tkind\tleading_line`
  output, diffed by exact-row **multiset** difference (`collections.Counter`
  subtraction in Python, the same technique `Counter`-based subtraction
  PLAT-843's own report used), never a raw line diff — a row is `old_only`/
  `new_only` only when its full `(path, qualified_name, kind, leading_line)`
  tuple has no counterpart on the other side. The full enumeration (`155`
  rows: every genuine identity change, split from the `308` rows that are a
  `leading_line`-only shift with no identity change) is
  `reports/2026-09-20-plat882-symbol-diff.tsv`, checked in beside this file.

## Headline

| Metric | Baseline (PLAT-851) | This PR (new engine) | Δ |
|---|---:|---:|---:|
| `typescript_symbols_total` (`quire-rs`) | 165 | 165 | **0** |
| `typescript_symbols_total` (`filament-ide-rs`) | 1,445 | 1,410 | **−35** |
| `typescript_symbols_total` (both repos) | 1,610 | 1,575 | **−35** |
| Abandoned TypeScript files (`filament-ide-rs`) | 3 | **0** | **−3** |
| TypeScript diagnostics, either repo | — | **0** | — |

**`quire-rs` is identical on identity and `leading_line`** — corrected wording
(review round 3, F1): the new engine's `(path, qualified_name, kind,
leading_line)` list matches the old engine's exactly, symbol for symbol —
165/165, zero rows on either side of that multiset diff. This report
previously called that "byte-identical", which overclaims: `(path,
qualified_name, kind, leading_line)` is 4 of the 7 columns
`plat843_audit_list` emits, and **`end_line` is not in that key**. Re-diffed
on all three span fields, `quire-rs` carries **2** identity-stable rows whose
`end_line` moved with everything else held constant — both
`coverage.ts::severityOf` (the `tag-on-non-test-function` fixture pair),
`11→12` and `15→16` — so "byte-identical" is false for the full record and
true only for identity + `leading_line`. See "The `end_line` class" below for
the full count and why these two, like the other 25 in `filament-ide-rs`, are
recoveries rather than defects. `quire-rs` has no TypeScript that exercises
any of the *identity-changing* shapes this port changes (no regex literals
with braces in its own `corpus/cases` TypeScript fixtures beyond the two
above, no `get`/`set` accessors, no `const NAME = (` false positives).

## The three previously-abandoned files, named, all recovered

All three files PLAT-840/851 measured as abandoned (a `{` inside a regex
literal desynchronising the old scanner's brace-depth counter,
`check_balanced` rejecting the whole file, every trace tag in it binding to
nothing) now parse cleanly and extract symbols. Verified directly — zero
TypeScript diagnostics from `SymbolExtraction::diagnostics` over either
repo's full tree, not inferred from the count moving:

| File | Symbols now extracted | Kind breakdown |
|---|---:|---|
| `ui/tests/e2e/tc-784-786-789-project-switcher.spec.ts` | 14 | — |
| `ui/tests/native/it-019-sync-native.spec.ts` | 7 | — |
| `ui/tests/mocks/handlers.ts` | 23 | — |
| **Total recovered** | **44** | 7 Container, 25 Function, 12 TestFunction |

PLAT-840 confirmed the escaped-brace-regex shape by direct source inspection
for the first two; the third (`handlers.ts`) it left **unmeasured** ("brace
counts exactly balanced (386/386) here... not re-investigated, still
unmeasured"). It is recovered too — its own regex-with-brace construct
(`bindings.match(/export type TokenStatusDto = \{([^}]*)\}/)`-shaped content
elsewhere in the same MSW handler file) hit the identical desync class PLAT-163
names; tree-sitter tokenizes every regex literal as its own grammar
construct in all three files, so a `{`/`}` inside one is never a code brace
to begin with, in any of them.

Recovering these three is what makes `filament-ide-rs`'s Container count rise
(317 → 324, **+7**, all from these files: the module-container symbol each
previously-abandoned file mints, plus real `describe`/`suite` containers
inside them) and its TestFunction count rise (431 → 443, **+12**, all real
`test`/`it` registrations these files always contained and never bound a tag
from). This is this ticket's headline result, closing PLAT-163's remaining
live exposure (measured at zero for Rust by PLAT-840; TypeScript was where it
was real).

## The other 111 changed identities (excluding the three recovered files), all named

`filament-ide-rs`'s remaining delta, outside the three recovered files: **95
lost** (all false-positive removals, not real losses) and **16 gained** (all
genuine recoveries), net **−79**, combined with the **+44** file recovery
above for the reported **−35** total. Every one of the 111 changed identities
(`(path, qualified_name, kind)`, ignoring `leading_line`) is enumerated in
`reports/2026-09-20-plat882-symbol-diff.tsv`; none are asserted by pattern
alone.

### Cause 1: `const NAME = (non-arrow expression)` false positives — 76 removed, `filament-ide-rs` only

The old scanner's `re_arrow_const` regex,
`^(?:const|let|var)\s+(NAME)\s*(?::[^=]+)?=\s*(?:async\s+)?\(`, matched the
**opening parenthesis** immediately after `=` and never checked for an
arrow (`=>`) at all — so `const dy = (positions[...] ?? 0) - (...);` (a
parenthesized arithmetic expression, not a function) matched exactly the
same as a real `const f = (x) => {...}`. Verified directly against Python's
`re` module on the exact pattern (empirical, not hand-derived): it matches
`camera(): { x }` at all for a different reason (Cause 2 below) but the
`const NAME = (` shape's own miss is a plain absent-arrow-check, confirmed
against every one of the 76 rows' own source line.

Representative rows (full list in the TSV, `side=old_only_false_positive_removed`):

- `ui/src/graph/cosmos/adapter.ts:324` — `const dy = (positions[a * 2 + 1] ?? 0) - (positions[b * 2 + 1] ?? 0);`
- `ui/src/graph/cosmos/adapter.ts:326` — `const needed = (sizes[a] ?? 0) + (sizes[b] ?? 0);`
- `ui/tests/native/tc-503-agent-session.spec.ts` — `const invoke = (\n  ...\n).__TAURI_INTERNALS__.invoke;` (the parenthesized value is a type-cast-and-property-access expression, not a function)
- `ui/tests/e2e/tc-971-graph-filters.spec.ts:196` — `const depths = (await linksRequests(page)).map((r) => r.params.depth);` — a genuine `=>` appears later in the line, inside `.map()`'s own callback, which is what makes this shape sharper than a plain "no `=>` anywhere" check would catch: the assigned value itself is the *result of a method call*, not the callback.

The new engine reads `variable_declarator.value.kind() == "arrow_function"`
directly off the tree — the AST cannot confuse a parenthesized expression
for a function the way text matching on "starts with `(`" can. This is the
same class of fix as PLAT-843's Rust Cause 1 (`macro_rules!` template text
lexically shaped like a declaration but not one): removing a symbol that was
never a real declaration is in scope under the port's own cause-based rule.

### Cause 2: `method_definition`-shaped false positives — 19 removed, `filament-ide-rs` only, split two ways

**Corrected attribution (PLAT-882 PR #481 review finding F4):** the original
version of this report attributed all 19 rows to interface member signatures.
4 of them are not — they are mocha `before(function () { ... })` hook calls,
whose `before(` + the passed function expression's own opening `{` matches
the old regex's `NAME(...) {` shape exactly the same way an interface member
does, for the same underlying reason (the regex has no notion of "is this a
declaration at all," only "does this line's shape match"), but the construct
is a **call**, not a signature. The count (19) and the fact that every row is
spurious were both already correct; only which of two false-positive shapes
each row belongs to was wrong for 4 of them.

**15 interface member signatures.** The old scanner's `re_method` regex,
matched against a **single line**, can satisfy its own `(?::\s*[^{]+)?`
return-type group by consuming as little as one whitespace character before
requiring a literal `{` — so `camera(): { x: number; y: number; ratio:
number } | null;`, an interface member signature whose return type is an
inline object type, matches the regex's own `NAME() { ` shape: the object
type's own opening `{` is read as if it were a method body opener. Confirmed
empirically (`re.match` in Python against the exact pattern and exact source
line, not inferred): `camera(): {` is captured in full as `match.group(0)`.

Representative rows (`side=old_only_false_positive_removed`, `kind=Function`):

- `ui/src/graph/GraphCanvas.tsx:268` — `camera(): { x: number; y: number; ratio: number } | null;` (an interface member, `interface GraphControls { ... }`)
- `ui/src/graph/GraphCanvas.tsx:395` — `collapseState(): {`
- `ui/src/graph/GraphCanvas.tsx:247` — `filters(): {`

The new engine only recognises a `method_definition` node — which
tree-sitter never emits for an `interface_body`'s `method_signature`
members, a structurally distinct node kind — so an interface's own type-only
declarations mint nothing. This adapter's module docs now state that rule
directly (`src/symbols/typescript.rs`'s "What survives byte-for-byte"
section, added per review finding F5 — it was true of the old regex's shape
by accident and is now a stated, tested rule:
`tc1923_an_interface_method_signature_mints_no_symbol`).

**4 `before(function () { ... })` mocha hook calls.**
`ui/tests/native/tc-1095-artifact-impact-native.spec.ts:84`,
`ui/tests/native/tc-1485-embedding-search-native.spec.ts:58`,
`ui/tests/native/tc-890-graph-extraction-native.spec.ts:231`,
`ui/tests/native/tc-981-graph-export-smoke.spec.ts:152` — each a
`before(function () {` mocha lifecycle hook, matched by the same
`NAME(...) {` shape with `before` read as the method name and the passed
function expression's parameter list read as the method's own. The new
engine mints nothing for a `call_expression` whose callee is not one of
`test`/`it`/`describe`/`suite` (`registration()` returns `None`, and `walk`'s
`call_expression` arm only recurses in that case) — `before` is not a
registration name this adapter recognises at all, so this is the same
"only a `method_definition` node mints" boundary, from the call side rather
than the declaration side.

### Cause 3: `get`/`set` accessor methods now recognised — 12 added, `filament-ide-rs` only

`re_method` requires the captured name to be the **first** token
(`^(?:modifiers)*(NAME)\s*\(`); `get foo() {}` does not match this shape —
`get` itself would have to be the captured identifier, and the next token is
`foo`, not `(`. tree-sitter's `method_definition` node covers an accessor
the same as an ordinary method (`get`/`set` are leading keyword tokens, not
a distinct node kind), so this is recognised for free, the same class of
recovery as Rust's `pub(in path)`/`unsafe impl` (PLAT-843).

All 12: `ui/src/graph/cosmos/adapter.ts` (`PointIndex.order`, `PointIndex.size`),
`ui/src/graph/cosmos/scene.ts` (`CosmosScene.ready`, `.frameCount`,
`.pointCount`, `.cameraFits`, `.cameraTaken`), `ui/src/graph/model.ts`
(`GraphAccumulator.nodesLoaded`, `.edgesLoaded`, `.progress`,
`.isLimitReached`), `ui/src/graph/presenter.ts` (`GraphPresenter.lastFrameCostMs`).

### Cause 4: multi-line method signatures now recognised — 4 added, `filament-ide-rs` only

`re_method` is matched against one physical line; a method whose parameter
list spans multiple lines (`static getDerivedStateFromProps(\n  props,\n
state,\n): ReturnType {`) never presents its own `NAME(...) {` on one line,
so the regex never sees it — the same defect class as Rust's PLAT-69
(multi-line attribute continuation), now closed for a TypeScript method's
own signature. Confirmed by direct comparison: the same class's
single-line sibling, `static getDerivedStateFromError(error: Error):
ErrorBoundaryState {`, was already extracted by both engines — unaffected,
proving the delta is specifically about the line count, not about `static`
or React's lifecycle-method naming.

All 4: `ui/src/ErrorBoundary.tsx` (`ErrorBoundary.getDerivedStateFromProps`),
`ui/tests/mocks/graph-harness.ts` (`focus`, `loadProject`,
`loadProjectWithCeiling` — `method_definition` nodes inside a mock object
literal, the same node kind a class body uses).

### The 308 identity-stable rows: a `leading_line`-only shift, not an identity change

`308` rows share an identical `(path, qualified_name, kind)` on both engines
and differ **only** in `leading_line`. **307 by exactly `−1`** (new engine one
line earlier), checked exhaustively (not sampled) once this section's own
claim was put under review — the original version of this report generalised
"always `−1`" from a sample rather than checking every row, which was wrong
for one of them: **one row is `−13`**,
`ui/src/graph/GraphCanvas.tsx`'s `resolveCarriedLayout`, where **two**
directly-adjacent `/** ... */` blocks (no blank line or code between them)
precede the same declaration. The old scanner's per-line walk breaks
immediately on hitting the *first* block's own `/**` opening line (see
below) — which does not merely cost that one line the way it does for every
single-block case, it also cuts the walk off from ever reaching the *second*
(nearer) block's own opening line and the first block entirely, orphaning it
from any declaration's span under the old engine. The new engine's sibling
walk has no such cliff: two adjacent `comment` siblings with no gap between
them join the same leading span exactly as one would, so it recovers both
blocks whole. Confirmed directly against source (not inferred from the
count): every sampled `−1` case is a single-line `/** ... */` JSDoc comment
immediately preceding the declaration —
e.g. `ui/src/graph/cosmos/scene.ts:481`, `/** The multiplier the current
camera implies, clamped to the fixed bounds. */` directly above
`fittedPointScale()`. The old scanner's `is_annotation()` recognised `//`,
`@`, and `*`-prefixed lines as leading annotation but **not** a line
starting `/**` (two slashes then two asterisks is not `//`, and the line
does not start with a bare `*`), so a JSDoc block's own opening line was
excluded from the leading span even though its later `*`-prefixed lines and
closing `*/` were included. tree-sitter represents the whole `/** ... */`
block as one `comment` sibling node regardless of line count, so
[`leading_span`]'s sibling walk includes its true start — the same class of
fix as PLAT-846 (Rust block doc comments). This does not change any
symbol's identity (`leading_line` is not part of `Symbol::compute_id`), only
the span a trace tag's binding search reads — a strict widening (recovers a
tag written on the JSDoc's own opening line; loses nothing).

**Methodology audit, prompted by finding the above:** checked every other
"sampled"/"representative" claim in this report for the same failure mode —
stating a sampled check as a universal one. One other instance existed and
was already corrected in this same review round: the original Cause 2
picked 3 "representative rows" out of 19 and generalised "all 19 are
interface member signatures" from them, which was wrong for 4 (mocha
`before(function () {})` hook calls, not interface members — PLAT-882 PR
#481 review finding F4, see Cause 2's own corrected section, which now
names every one of the 19 rather than sampling). Every other exhaustive
claim in this report (the 76 `const NAME = (...)` rows, the file-recovery
counts, the widened-edge zero-occurrence claims) was independently
re-derived over the full set by the PR #481 reviewer's own separate
measurement and matched, so those are not re-audited here a third time.

### A second span defect, found writing this review's own span assertions, fixed the same way

Restoring `tc803_one_reading_decides_whether_delimiters_are_code`'s span
assertions (PLAT-882 PR #481 review finding F3) surfaced a real bug the
original version of this port shipped: a `comment` sibling **trailing** on
the same line as the statement before a declaration (`const re = /a/; //
note\ntest(...)`) was read as *the following declaration's own* leading
annotation, because the sibling-walk only checked "is the nearest preceding
sibling a comment," not "does that comment start its own line." The old
line-structural scanner never had this failure mode — a line whose own
trimmed text does not begin `//`/`/*`/`*` was never an annotation line at
all, trailing or not. Fixed in [`leading_span`] (an added check: a comment
sibling only joins the span when it does not itself start on the same row
its own preceding sibling ends on), with `tc1924_a_trailing_comment_does_
not_leak_into_the_next_declarations_span` (`FR-051-AC-14`) pinning both the
defect and its standalone-comment control. Re-verified after the fix: this
does not change any of the counts above — `quire-rs` is still 165/165 on
identity + `leading_line` (no trailing-same-line-comment-then-declaration
shape exists in its own tree), and `filament-ide-rs`'s 308 `leading_line`-
only-shift rows are still 308 (no row in the real corpus exercises this
specific pattern; see the previous section's own exhaustive re-check for the
one row that pattern search did turn up, which was a different defect in the
same function).

## The `end_line` class: 27 rows, unreported until review round 3

**F1 (review round 3).** Every count and claim above keys the differential on
`(path, qualified_name, kind, leading_line)` — 4 of the 7 columns
`plat843_audit_list` emits, kept for continuity with PLAT-843's own
reproduction instructions (see this report's own harness note). `end_line`
is not in that key, so a symbol whose identity and `leading_line` are both
unchanged but whose `end_line` moved was invisible to every diff in this
report — exactly the shape PLAT-868's own review finding F2 widened this
tool to expose, because a delta in a field the tool doesn't emit is
structurally unmeasurable, and a delta in a field it emits but the diff
doesn't key on is measurable but silently dropped. This report repeated the
second failure mode, not the first.

Re-diffed on all three span fields (`leading_line`, `line`, `end_line`)
together: **27 identity-stable rows carry an `end_line` change — 2 in
`quire-rs`, 25 in `filament-ide-rs`** (15 of the 25 also carry a
`leading_line` shift and are already counted in the 308; the other 10 have
`end_line` as their only span change). Every one was read. Every one is a
strict recovery: the new engine reports the declaration's real closing
brace; the old regex-based scanner truncated at some earlier line, almost
always because the declaration's own signature or an intervening construct
spanned multiple lines and the old scanner's own per-line matching lost
track. None relocates a binding — the global `verifies` relation set
(keyed `(path, symbol, trace_id, provenance)`) is identical in membership
before and after accounting for these 27 rows.

`quire-rs`'s own 2: both `coverage.ts::severityOf` (the
`tag-on-non-test-function`/`-control` fixture pair), `end 11→12` and
`15→16`.

`filament-ide-rs`'s 25 (`path`, `qualified_name`, `end_line` old→new):

| Path | Symbol | `end_line` |
|---|---|---:|
| `ui/src/graph/GraphCanvas.tsx` | `hasBothLayers` | 510 → 519 |
| `ui/src/graph/clustering.ts` | `shouldRecomputeClusters` | 229 → 238 |
| `ui/src/graph/cosmos/separation.ts` | `pushApart` | 243 → 311 |
| `ui/src/graph/data.ts` | `collectGraphExport` | 447 → 468 |
| `ui/src/graph/data.ts` | `fetchGraphExportPage` | 401 → 404 |
| `ui/src/graph/data.ts` | `fetchLinks` | 410 → 413 |
| `ui/src/graph/data.ts` | `graphExportRequest` | 304 → 324 |
| `ui/src/graph/data.ts` | `linksRequest` | 335 → 362 |
| `ui/src/graph/impact.ts` | `fetchArtifactImpact` | 198 → 208 |
| `ui/src/graph/model.ts` | `GraphAccumulator.record` | 435 → 457 |
| `ui/src/graph/model.ts` | `focusNeighborhood` | 523 → 528 |
| `ui/src/graph/model.ts` | `loadProjectGraph` | 498 → 512 |
| `ui/src/plansync/GanttChart.tsx` | `buildLanes` | 118 → 153 |
| `ui/tests/e2e/tc-1176-graph-node-type-color.spec.ts` | `expectedFill` | 45 → 51 |
| `ui/tests/e2e/tc-1185-graph-layer-regions.spec.ts` | `group` | 148 → 149 |
| `ui/tests/e2e/tc-1185-graph-layer-regions.spec.ts` | `spread` | 154 → 156 |
| `ui/tests/e2e/tc-125-nfr-032-gantt-perf.spec.ts` | `assertBudget` | 121 → 126 |
| `ui/tests/e2e/tc-1661-graph-repo-filter.spec.ts` | `hiddenOfTarget` | 89 → 91 |
| `ui/tests/e2e/tc-449-inner-loop-benchmark.spec.ts` | `TC-449: @perf @TC-449 @NFR-023 saved spec reruns against warm mock IPC` | 28 → 38 |
| `ui/tests/e2e/tc-795-switcher-round-trip.spec.ts` | `emitProjectsChanged` | 33 → 38 |
| `ui/tests/e2e/tc-967-graph-search.spec.ts` | `offset` | 199 → 208 |
| `ui/tests/e2e/tc-979-graph-performance.spec.ts` | `assertBudget` | 129 → 136 |
| `ui/tests/mocks/graph-fixtures.ts` | `hit` | 539 → 560 |
| `ui/tests/native/tc-710-gantt-perf.spec.ts` | `clickTestId` | 89 → 92 |
| `ui/tests/native/tc-710-gantt-perf.spec.ts` | `waitForTestId` | 72 → 75 |

**This is a reporting gap, not a defect** — the change was real, correct,
and always covered by `verifies`-set equality; it was simply never named as
its own class. Named and counted here so it does not recur unmeasured a
second time, per PLAT-868's own F2 precedent.

**Independently hit twice.** PLAT-868 (the Python port, landed in parallel)
found the identical defect in `python.rs`: a trailing `x = 1  # note`
comment read as the *following* declaration's own leading annotation, same
root cause (tree-sitter gives a trailing comment its own sibling node) and
the same fix (require the comment to start its own line). Two language
adapters made the same mistake independently and each was caught
independently by its own port's review, which is worth noting on its own —
and it means the own-line check now exists as near-duplicate logic in two
adapters rather than once in a shared place. That is a real follow-up
(`leading_span`'s trailing-annotation rule belongs in whatever seam both
adapters already share, if one exists, or a new shared helper otherwise),
but it is **not done in this PR** — this port's own scope is `typescript.rs`
only, per the ticket's own "do not touch `python.rs`" instruction, and two
independently-landing PRs are the wrong place to restructure a shared seam
neither can see the other's final shape of. Naming it here so the next
person doesn't have to rediscover it.

## Widened grammar edges: CR-176's own named carve-outs, resolved and measured at zero

CR-176's note (landed with PLAT-843) explicitly named two TypeScript-adapter
scan bounds — AC-18's bounded title lookahead and AC-21's same-line `{`
requirement — as decisions belonging to this port, "to be widened only as
its own change with its own **[RAN]** numbers," not assumed safe. This port
resolves both, plus a third edge that widens as a direct structural
consequence (see `spec/functional/FR-051-source-symbol-extraction.md`'s new
`CR-180` note for the full spec-level record):

| Widened edge | Real occurrences recovered in this corpus |
|---|---:|
| A registration's title written arbitrarily far down the file (no more `TITLE_LOOKAHEAD_LINES` window; superseded by "a registration needs a callback argument") | **0** |
| A `describe(...)` whose callback `{` falls on a later physical line still parents its members | **0** |
| Whitespace before a modifier chain's `.` (`it .skip(...)`) | **0** |

`grep -rn "describe($" ui --include=*.ts --include=*.tsx` over `filament-ide-rs`
and the equivalent over `quire-rs`'s `corpus/`: zero matches either way — no
wrapped `describe(` call exists in the measured corpus. None of the 155
enumerated identity-changed rows are attributable to any of these three
edges (all are Cause 1–4 above or a file recovery); this is a structural
consequence of the AST no longer needing a window, checked, not merely
argued from the mechanism.

**Two narrowings ride along with the same change** (PLAT-882 PR #481 review
finding F8), not named in the original version of this report:
`it.todo('x')`/`test.skip('name')`-shaped calls with no callback argument
registered under the old regex and register nothing here (superseded by the
same callback-argument requirement above); and a comma-separated
`const a = () => {}, b = () => {}` statement now mints every arrow-valued
declarator, where the old regex's single capture group matched only the
first. Both measured at **zero** occurrences in this corpus — the no-callback
shape by the same pattern search used for the widened edges above, across
both repositories including `quire-rs`'s own fixtures; the multi-declarator
shape by a narrower single-line-only search (see the `CR-180` note's own
caveat on that one). See the spec's own `CR-180` note for the full record.

## Binding numbers (PLAT-882 PR #481 review: "add the binding numbers")

Every number above is a **symbol** count. None of it says whether coverage
went up or down — that is a **binding** question, answered by `binding_census`
(candidate symbols, how many carry a trace tag, how many of those bind) and
`coverage::compute`'s `unbacked_rows`. Measured directly (old engine: this
repo's `src/symbols/typescript.rs` swapped back to the pre-port file at
`origin/main`, same tree otherwise; new engine: this branch), via
`examples/plat840_rust_baseline_sweep` for `filament-ide-rs`
(`PLAT840_PATH_FILAMENT_IDE_RS` pointed at a disposable clone pinned to the
same `head_sha`, `37c44d907ad419472faca240bd02a0fae9add7c0`, this report
already uses) and `examples/plat843_unbacked_rows` for `quire-rs`'s own spec:

| Metric (`filament-ide-rs`, TypeScript `binding_census`) | Old engine | New engine | Δ |
|---|---:|---:|---:|
| `candidates` | 431 | 443 | **+12** |
| `tagged` | 394 | 406 | **+12** |
| `bound` | 385 | 392 | **+7** |
| `tagged_not_bound` | 9 | 14 | +5 |

| Metric (`quire-rs`'s own spec) | Old engine | New engine | Δ |
|---|---:|---:|---:|
| `plat843_unbacked_rows` (`examples/plat843_unbacked_rows`) | 404 | 404 | **0** |

**Correction, re-measured after rebasing onto `daaafaf` (PLAT-868/#479
merged):** this row originally read `404 → 402` (**−2**). Re-run from
scratch on the rebased tree — same harness, same swap technique, old and
new engine's row sets diffed directly (not just their counts) — the two
`unbacked_rows` outputs are now **identical, row for row**, zero rows on
either side of the diff. `404 → 402` does not reproduce, and in hindsight it
was already in tension with this same report's own `165/165` identity-and-
`leading_line`-identical claim for `quire-rs`'s TypeScript symbols: if the
symbols `coverage::compute` reads are identical on every field that
computation reads (identity, `leading_line` — `end_line` is not one of
them), the coverage computed from them cannot differ. Whatever produced the original `−2` is
not reproducible from the port itself; it most likely reflects a measurement
taken against a different tree state at the time (this repo's own spec has
moved under multiple merges since), not a defect in either number's
arithmetic. Stated plainly rather than quietly overwritten, per the standing
instruction to disclose a correction rather than silently fix it.

The `filament-ide-rs` `binding_census` table above **did** re-verify
unchanged, re-run against the same disposable pinned clone: `431/394/385/9`
→ `443/406/392/14`, identical to the original figures. That repo is
untouched by `quire-rs`'s own rebase, so its numbers were never expected to
move, and they didn't.

Every number here moves in the favourable direction or is unchanged: more
candidates, more tagged, more bound, and `quire-rs`'s own unbacked-row count
does not regress. **Zero tags lost** — `tagged` only rises, `bound` only
rises, and `tagged_not_bound` rising by 5 is candidates newly *seen* (symbols
the old engine never extracted at all, so it could not report their tag as
anything, bound or not) rather than any candidate moving from bound to
unbound. The `−35` symbol-count headline above is a false-positive removal,
not a coverage loss — this is what settles that question directly rather
than by inference from the symbol count alone.

## Tests retired

| Retired | Asserted (why it dies) | Successor | Successor asserts |
|---|---|---|---|
| `tc803_one_lex_serves_every_consumer` | Internal state of the deleted single-pass lexer (`lex`, `check_balanced`, `LexedLine.delta`) | `tc803_one_reading_decides_whether_delimiters_are_code` (same TC-803, same `FR-051-AC-14`) | Outcome: a brace inside a block comment, inside a carried template literal, and after an unterminated quote-shaped regex are content to `parse`, over three adversarial fixtures — **and now also the span** (`leading_line`/`line`/`end_line`) of the registration that follows all three, restored per PLAT-882 PR #481 review finding F3 (the first version of this successor dropped the span half of `FR-051-AC-14`'s own claim) |

Confirmed to fail first: reverting the adapter's parse path to a naive
`source.matches('{').count() == source.matches('}').count()` text check
rejects all three adversarial fixtures in the successor test (an imbalance
is reported on every one), which is exactly the false rejection the
property forbids.

**New tests added in the PLAT-882 PR #481 review round** (none retire
anything; each pins a property the original port shipped without a test
for, or a genuine defect the review round's own test-writing surfaced):
`tc1920` (F1, multi-line template title), `tc1921`/`tc1922`/`tc1923` (F2,
the three mutation-killers above), `tc1924` (a real `leading_span` bug
found writing F3's restored span assertions — see its own section above),
`tc1039_a_late_brace_describe_still_parents_its_members` (F6, AC-21's own
new clause had no test).

`tc798_comment_stripping_is_string_aware`, `tc799_template_literal_state_carries_across_lines`
and `tc799_braces_inside_a_multiline_literal_do_not_unbalance` keep their TC
ids and their outcome-level assertions (`parse`'s own result) unchanged; each
lost only a trailing mechanism-level sub-assertion that called the deleted
`lex_line`/`ScanState` directly — a mechanical deletion following the type
these tests inspected being deleted, not a retirement of the TC itself (its
outcome claim is intact and still exercised).

`TC-1881` (`FR-051-AC-25`), reserved but unimplemented since PLAT-851/840,
is now implemented and green:
`tc1881_a_brace_inside_a_regex_literal_is_content`, the exact `{n,m}`-quantifier
and escaped-brace-regex shapes this ticket exists to fix, both with a
same-file-minus-the-construct control.

No `tc943`/`tc948`/`tc1039` (FR-051-AC-18/21) tests were retired — all pass
unchanged against the new engine, since they assert through `parse`'s public
outcome, which this port preserves for every shape they cover. `tc961`
(FR-051-AC-18's pinned edges) keeps its own id; one of its pinned edges
(whitespace before `.`) moved from "outside" to "admitted" per the CR-180
note above, and the test's own assertion was updated to match, in place,
with the reasoning inline.

## Gates

Re-run in full after rebasing onto `main`'s tip `daaafaf` (PLAT-868/`#479`
merged), under a dedicated `CARGO_TARGET_DIR` (`.worktrees/plat882-target`,
not the machine-wide shared default) and the shared `/tmp/quire-heavy-check.lock`
for every heavy build, batched into as few lock acquisitions as the work
allowed rather than one per command.

- `cargo fmt --check`: clean.
- `cargo clippy --locked --all-targets -- -D warnings` (default features, which now include `typescript-symbols`): clean.
- `cargo clippy --locked --all-features --all-targets -- -D warnings`: clean — every feature combination compiles, including `python` + `wasm` + all three `*-symbols` together (see the `leading_block` note below, which this run is what settles it for).
- `cargo test --locked --lib`: 658 passed, 0 failed (23 of them `symbols::typescript::tests::*`, up one this round — `tc1925`, the E5-corrected isolating case).
- `cargo test --locked` (default features, full workspace, including `tests/trace_dogfood.rs`): all green.
- `cargo check --no-default-features --features typescript-symbols` (`make check-typescript-symbols`): clean.
- `cargo check --no-default-features --features python-symbols` (`make check-python-symbols`): clean.
- `cargo check --features python` (`make check-python`): clean.
- `cargo check --target wasm32-unknown-unknown --no-default-features --features wasm` (`make check-wasm`): clean — **only after a fix this round**, see below.
- `make deny` / `make deny-grammars`: clean.
- `make audit-unsafe` / `make audit-property` / `make audit-static`: clean.
- `scripts/validate_spec.py`, run directly against disposable clones pinned to `quality/validation-stack-lock.json`'s exact locked revisions (`spec-artifacts-process@e6ea515`, `spec-artifacts-iso@a60ee12`) rather than through `make validate`: `172` documents, `0` failed, `41` warnings. `make validate`'s own default relative paths (`../spec-artifacts-process`, `../spec-artifacts-iso`) resolve from this worktree's own directory, one level too shallow (`.worktrees/plat882/../spec-artifacts-process` instead of the real sibling of `quire-rs` itself) — a worktree-layout artifact, not something this branch changed; the same workaround an earlier round of this same review used.
- `make check-engine QUIRE_CLI=/home/peter/dev/quire-cli`: `OK`, 14 capability tokens (one advisory, pre-existing and unrelated: `Cargo.toml`'s `version` field trails the latest tag, `agent-ix/quire-rs#282`).

**Two real findings this round, both fixed in-PR:**

- **`leading_block` was dead code, now deleted.** `src/symbols/mod.rs`'s shared line-structural helper (the 1-based first line of a declaration's leading comment/attribute run) had no callers left once this port lands — `rust.rs` (PLAT-843) and `python.rs` (PLAT-868) had already stopped calling it, and this PR's own tree-sitter `leading_span` in `typescript.rs` replaces its last caller. Confirmed dead under **every** feature combination, not just the default build's clippy leg: a whole-tree `grep` (source, tests, benches, examples, `fuzz/`) found no caller anywhere, and `cargo clippy --all-features --all-targets` compiled clean with it removed. **What the deletion removes:** the pre-port `leading_block` computed a span by scanning raw text lines backward for a caller-supplied `is_annotation(&str) -> bool` predicate; each adapter's own tree-sitter port (`rust.rs`'s and `python.rs`'s span logic, and this PR's `leading_span` for TypeScript) now computes the equivalent span by walking `prev_sibling()` over the parsed tree instead, so the behaviour is not lost, only relocated to be per-adapter and grammar-driven rather than shared and text-driven. Nothing in this repo (or the `iso`/traceability fixtures) references it outside two now-stale prose comments, left as historical pointers (`python.rs:1037`, `trace.rs:1885`).
- **Deleting `leading_block` broke `cargo check --features wasm`, fixed.** Under `--no-default-features --features wasm`, all three `*-symbols` features are off (same C-toolchain cross-compilation reason each one is gated for), so every match arm in `symbols::mod::extend_with_file`'s `parsed = match language { ... }` becomes its `#[cfg(not(feature = "..."))]` fallback — before this PR, `typescript::parse` still ran unconditionally there, giving the compiler a concrete `Ok(Vec<RawSymbol>)` arm to infer `parsed`'s type from even under `wasm`; gating TypeScript the same way `rust-symbols`/`python-symbols` already are removed that last unconditional arm, leaving nothing to infer `Vec<RawSymbol>` from (`E0282: type annotations needed`). Fixed with an explicit `let parsed: Result<Vec<RawSymbol>, String> = match language { ... }` annotation, with the reasoning recorded inline so a future all-`#[cfg]`-arms match doesn't rediscover this the same way.

**Mutation verification (PLAT-882 PR #481 review finding F2), re-run from scratch on the rebased tree, each applied alone to `src/symbols/typescript.rs`, confirmed against a saved clean copy after reverting, run via `cargo test --locked --lib symbols::typescript`:**

- **E4** (reintroduce a 3-line title-lookahead window in `registration()`): `tc1921_a_far_title_with_a_callback_still_registers` **fails** (the only failure, exit 101); reverted, 23/23 pass again (exit 0).
- **E6** (also match `"method_signature"` in `walk`'s `method_definition` arm): `tc1923_an_interface_method_signature_mints_no_symbol` **fails** (the only failure, exit 101); reverted, 23/23 pass again.
- **E5, corrected. The original round's "`tc1922` fails (the only failure)" claim is false and is corrected here, not left standing.** "Accept a non-arrow `parenthesized_expression` value too, in `mint_arrow_const_declarators`" — widening only the `value.kind() != "arrow_function"` check — leaves all tests green, `tc1922` included: the very next line, `if value.child_by_field_name("parameters").is_none() { continue; }`, is an **independent, already-existing guard**, and `parenthesized_expression` never carries a `parameters` field regardless of what it wraps, so this second guard rejects `tc1922`'s `(a + b)` fixture on its own either way. Two explanations were possible — the mutated line is load-bearing for some input the fixture doesn't exercise, or it is genuinely dead code and should go — and this was settled as a question of fact, not asserted either way: a throwaway debug probe over nine candidate declarator-value shapes found that **`function_expression` and `generator_function` values do carry a `parameters` field** (`const f = function() {}`, `const f = function*() {}` — checked directly against the live tree, not assumed from the grammar's docs), so only the first guard's `arrow_function` check rejects them. The line is load-bearing, not dead. **TC-1925** is the isolating fixture this needed and `tc1922` could not provide: a `const f = function() {}` / `const g = function*() {}` pair, which clears the second guard (has a `parameters` field) and depends entirely on the first. Verified as a real mutation kill — widening the first guard to also accept `function_expression` fails exactly `tc1925` (the only failure) and leaves `tc1922` green (untouched by that specific widening, since `parenthesized_expression` isn't `function_expression`); reverted, 23/23 pass again.
