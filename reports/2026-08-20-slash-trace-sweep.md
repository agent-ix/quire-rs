---
type: SpecReview
id: SR-050
title: "Slash-separated trace-id chains: census and normalization"
analysis: base
---

# Slash-separated trace-id chains — census

## Summary

A corpus-wide census of slash-joined trace-id chains — the authoring form the
comma-only legacy grammar silently truncates — with `quire-rs`'s auto-editable
lines normalized in the same commit. 214 GREEN chains corpus-wide before the
sweep; 55 swept here; the remaining classes are refused with the reason
recorded per line. The findings table at the end restates the census classes.

Harness: `scripts/slash_tag_sweep.py`, committed with this report. Engine:
`quire-cli` @ `quire-rs v0.41.0`, module `spec-artifacts-process` @ v0.22.0.
Corpus: `/home/peter/dev`, **239 repositories** enumerated by
`scripts/corpus.py`, **238 scanned** after the exclusion below. Excluded:
['ecaz'].

> **Correction note (2026-08-21, #217/#219).** This document is amended in
> place; the census tables stay as the harness reported them on 2026-08-20.
>
> * **Identity.** Shipped as `id: SR-049`, colliding with
>   `spec/reviews/SR-049-fr059-code-review.md` (undetected because `reports/`
>   is outside the doc root), and `type: Review` where the convention is
>   `SpecReview`. Re-identified as **SR-050**, the next free id across all
>   refs.
> * **Corpus attribution (cross-ref #219).** `scripts/corpus.py` enumerates
>   **239** repositories; **238** is `repos_scanned` after the `ecaz`
>   exclusion. The header labelled 238 as "enumerated by `scripts/corpus.py`",
>   conflating the two. The harness census now names them
>   `repos_enumerated` and `repos_scanned_after_exclusions`.
> * **Arithmetic.** The sweep commit's diff edits **55** lines in `quire-rs`,
>   not the 56 this report claimed below (corrected inline). The pre-edit
>   per-repo census that would settle the one-line discrepancy was never
>   captured — the committed JSON is post-edit only (`quire-rs green: 0`),
>   violating the harness docstring's own census rule. The harness now emits
>   `totals` (before) and `totals_after_write` in one artifact.
> * **Three GREEN edits were placebos.** `FR-024-AC-4`, `FR-025-AC-4` and
>   `FR-027-AC-6` sat on lines with a ` + <id>` tail
>   (`// TC-473 / FR-024-AC-4 + NFR-006:`); the rewritten comma list still
>   binds only the first id past such a tail, and all three were measured
>   unbacked on main. The classifier judged the slash-span, not the rewritten
>   line; rule **R7** now refuses GREEN when the rewritten line cannot bind.
>   The three lines are repaired in-tree (#217): measured with the same
>   binary/module before and after, `totals.backed` 883 → **886**,
>   `unbacked_rows` 282 → **279** with all three ids leaving the list, and
>   `untracked_symbols` unchanged at 1 (the deliberate TC-999 fixture).
>   `FR-025-AC-6` shares the line shape but its row is method Inspection; its
>   line is left as-is and now classifies AMBER under R7.

## The defect

Every legacy trace form joins an id list on a **comma** and nothing else. The
corpus writes a slash — `/// NFR-002-AC-4 / TC-577:` — and capture group 1 stops
at the first id. The second is dropped **inside the regex**, before any engine
code runs, so there is no diagnostic.

The loss is invisible by construction: a dropped id never becomes a relation, so
it can never appear in `untracked_symbols`. It shows only as the row it would
have backed reading unbacked.

## Census

Taken **before** any edit, over the whole corpus:

| class | lines | disposition |
|---|---:|---|
| **GREEN** — every id in the chain is a shape a trace target mints | **214** | auto-editable |
| **AMBER** — the chain contains an id nothing mints | 189 | refused; needs a decision per repo |
| **ELISION** — numeric shorthand (`FR-011-AC-6/7/8`) | 62 | refused; a different transform |
| **PROSE** — the line is not a tag line | 788 | refused; rewriting would mint a binding that does not exist |

`quire-rs`'s 55 GREEN lines are swept in this commit (the diff-verified count;
this report originally claimed 56 — see the correction note), and a re-run of
the harness reported **158** GREEN remaining. The per-repository table below is
that remaining population — the work still to do.

| class | lines | disposition |
|---|---:|---|
| **GREEN** — remaining after `quire-rs` | **158** | auto-editable |
| **AMBER** — the chain contains an id nothing mints | 189 | refused; needs a decision per repo |
| **ELISION** — numeric shorthand (`FR-011-AC-6/7/8`) | 62 | refused; a different transform |
| **PROSE** — the line is not a tag line | 788 | refused; rewriting would mint a binding that does not exist |

### GREEN by repository

| repo | green | amber |
|---|---:|---:|
| `filament-ide-rs` | 50 | 77 |
| `github-projects` | 17 | 0 |
| `filament-ui` | 16 | 1 |
| `identity` | 15 | 17 |
| `mcp-gateway` | 11 | 0 |
| `ix-agent-fastapi` | 7 | 0 |
| `auth-service` | 6 | 5 |
| `filament-analysis-worker` | 6 | 8 |
| `ts-plugin-kit` | 5 | 0 |
| `user-admin-ui` | 5 | 0 |
| `cloudmanager-local-sync` | 3 | 2 |
| `config-service` | 3 | 0 |
| `filament-core-service` | 3 | 7 |
| `permission-service` | 2 | 0 |
| `sync-github-service` | 2 | 0 |
| `ts-auth-ui` | 2 | 3 |
| `filament-editor-integration` | 1 | 0 |
| `ix-cli-core` | 1 | 1 |
| `quire-cli` | 1 | 0 |
| `quoin` | 1 | 0 |
| `ts-build-chain` | 1 | 0 |

### Why AMBER is refused

An id that matches the legacy pattern is not an id a trace target **mints**.
Adding one to a comma list produces an `untracked_symbol`, not coverage — the
trap #193 hit and backed out of, where binding six such ids would have taken
dead tags from 15 to 21. The shapes found:

| shape | count |
|---|---:|
| `FR-N-CON-N` | 71 |
| `FR-N` | 67 |
| `IT-N-SC-N` | 45 |
| `NFR-N` | 39 |
| `NFR-N-ATK-N` | 14 |
| `NFR-N-CON-N` | 4 |
| `US-N-EX-N` | 3 |
| `US-N-AC-N` | 3 |
| `NFR-N-INV-N` | 2 |
| `FR-N-B-N` | 1 |
| `StR-N-AC-N` | 1 |
| `NFR-N-VR-N` | 1 |

`US` and `StR` criteria are deliberately unminted by the module; `-CON-`,
`-SC-`, `-ATK-`, `-INV-` and bare requirement ids are minted by nothing.

### Why PROSE is the largest class

`# Pull Architecture (FR-010 / FR-011)` binds nothing today and must keep
binding nothing. So must a **wrapped sentence** whose continuation happens to
begin with an id:

```rust
/// Whether the body tier has been materialised (test observability,
/// TC-816/TC-817).
```

That line anchors correctly and is still prose — the `)` closes a parenthetical
opened on the previous line, which is the tell (rule R1b). Harmless where it was
found, since a production function cannot bind `verifies` at all (CR-061), but
the same shape inside a test's doc block would mint a binding nobody authored.

## Result — `quire-rs`, the reference implementation

Measured before and after with the same binary and module:

| measure | before | after |
|---|---:|---:|
| `totals.backed` | 793 | **839** |
| `totals.total` | 1164 | 1164 |
| `status_lies` | 0 | 0 |
| `untracked_symbols` | 1 | 1 |
| **new untracked symbols** | — | **0** |

**+46 ids bound, no test written and no source behaviour changed.** Every one of
those tags was already in the file; the grammar simply could not read past the
first. `make ci` green.

The zero on the last row is the gate (rule R7): a GREEN edit that minted a dead
tag would show up there, and the line is reverted and reclassified AMBER if it
ever does.

## Not swept

`ecaz` keeps its own trace vocabulary (`ADR-085 D8`, `FR-079/005-P1`) and would
produce a large, wrong-looking diff in a repository that has its own open
findings. It gets this report and an issue, not an edit.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | 214 GREEN slash-joined chains across the 238 scanned repositories drop every id after the first inside the legacy-form regex, with no diagnostic; 55 swept in `quire-rs` here, 158 remain for the #211 tail | ix://agent-ix/quire-rs/spec/functional/FR-051 |
| FND-002 | medium | 189 AMBER chains carry an id no trace target mints (`-CON-`, `-SC-`, `-ATK-`, bare requirement ids); rewriting one adds an `untracked_symbol`, not coverage — refused, needs a per-repo decision | ix://agent-ix/quire-rs/spec/functional/FR-051 |
| FND-003 | low | 62 ELISION chains use the numeric shorthand (`FR-011-AC-6/7/8`); comma replacement would corrupt them — a different transform, out of scope | ix://agent-ix/quire-rs/spec/functional/FR-051 |
| FND-004 | low | 788 PROSE lines hold a slash chain that no form binds today; rewriting would mint bindings nobody authored — counted and refused | ix://agent-ix/quire-rs/spec/functional/FR-051 |
| FND-005 | high | Three of the 55 `quire-rs` GREEN edits were placebos: a ` + <id>` tail keeps the rewritten comma list unbindable past the first id (`FR-024-AC-4`, `FR-025-AC-4`, `FR-027-AC-6` measured unbacked). Repaired in-tree and guarded by rule R7 (#217, correction note above) | ix://agent-ix/quire-rs/spec/functional/FR-024 |
