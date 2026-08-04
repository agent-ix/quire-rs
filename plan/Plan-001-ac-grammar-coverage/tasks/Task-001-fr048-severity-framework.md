---
id: Task-001
title: "FR-048 — per-check grammar severity framework"
type: Task
status: completed
track: A
priority: P0
relationships:
  - target: ix://agent-ix/quire-rs/FR-048
    type: references
  - target: ix://agent-ix/quire-rs/TC-716
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-717
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-718
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-719
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-722
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-723
    type: verifies
  - target: ix://agent-ix/quire-rs/TC-752
    type: verifies
---
# Task-001: FR-048 — per-check grammar severity framework

## Scope

The engine half of FR-048: a `grammar_severity` registry in `manifest.yaml`
(`<grammar>:<check>` → `off`|`warning`|`error`), merged first-wins across
modules with a `DuplicateGrammarSeverity` diagnostic (mirror the FR-043
`lexicon` merge), exposed via `Registry::grammar_severity()`, applied at
finding-emission time (absent key → `warning`; `off` → dropped before routing,
absent from `--summary` input), with the type-only `validate_document` path on
the all-default map. `--strict` global semantics untouched.

## Subtasks
- [x] **Manifest schema + loader.** Parse/validate `grammar_severity`;
  malformed entry (unknown level, non-string key) fails module load (TC-723).
- [x] **First-wins merge + diagnostic.** Cross-module merge and
  `DuplicateGrammarSeverity` on conflicting redeclaration only (TC-717).
- [x] **Registry accessor.** `grammar_severity()` returns the merged map (TC-716).
- [x] **Severity application.** Key findings by `grammar`+`check`; default
  `warning`; route per FR-042-AC-7; `off` drops pre-routing (TC-718/719/752).
- [x] **Type-only degradation.** All-default map on `validate_document` (TC-722).

## Deliverables
- Severity map types + merge in the registry/loader modules; emission-time
  application in `src/grammar`; unit tests tagged TC-716..719, TC-722,
  TC-723, TC-752.

## Implementation record (2026-08-04)

- `GrammarSeverityLevel` (`off`|`warning`|`error`), `GrammarSeverityMap`,
  `severity_key`/`severity_level`, `apply_severity`, and the shared
  `default_severity()` all-default map live in `src/grammar/mod.rs`.
  `apply_severity` drops `off` findings **before** routing, so a suppressed
  check reaches neither `warnings`/`errors` nor any later summary histogram.
- `Manifest::grammar_severity` (`src/loader/manifest.rs`) is a
  `BTreeMap<String, GrammarSeverityLevel>`; the enum deserializer rejects an
  unknown level, and `check_grammar_severity_keys` rejects a malformed key
  (YAML stringifies a non-string scalar key such as `12:` rather than failing,
  so the key shape is checked explicitly).
- Merge + `Diagnostic::DuplicateGrammarSeverity` reuse the FR-043 `merge_vocab`
  path in `src/loader/mod.rs`; `Registry::grammar_severity()` exposes the map.
  `load_strict` is intentionally **not** extended to escalate the diagnostic —
  FR-048 specifies it as non-fatal and no AC asks for the strict promotion.
- Applied at emission in `validate_document::run_grammar` (both entry points)
  and in the PyO3 `check_grammar` binding; the type-only path passes
  `default_severity()`.
- **TC-718 scope note:** the AC's end-to-end phrasing ("an unclassifiable
  criteria cell") needs the `ac` grammar from Task-002. The test lands here as
  the framework contract it actually pins — an `ac:unclassifiable` finding
  mapped to `error` routes to `errors` and clears `is_valid` while an unmapped
  `ears` finding stays a warning — and Task-002 exercises the same key
  end-to-end through the real classifier.
- `spec/tests.md` TC statuses stay 🚧 until Gate G1, when the whole Track A
  slice (including the `ac` grammar the TC prose describes) is real.

## Notes
- First on Track A: FR-047's routing contract builds on this.
- Deterministic map iteration (NFR-006): `BTreeMap`, not `HashMap`.
- Unblocks: Task-002 (ac grammar routing), Task-003 (CLI `--severity` helper).
