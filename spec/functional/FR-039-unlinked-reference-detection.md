---
id: FR-039
title: "Unlinked Reference Detection and Autofix Suggestions"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-025"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-026"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-008"
    type: "requires"
    cardinality: "1:1"
---

## Description

Per [ADR 0007](../assets/adr/0007-internal-relative-path-links.md), intra-bundle
references are authored as relative-path links so the OKF graph is read from
explicit links, never from a runtime scan of prose. To migrate existing specs
and to keep new ones honest, `quire-rs` SHALL **detect bare artifact-id tokens
in prose that are not links** and, where it can resolve them unambiguously
inside the loaded corpus, emit the **exact relative-path link it would apply**.
Detection is **advisory** — it never blocks extraction, validation
([FR-032](./FR-032-validate-document.md)), or sync; it produces findings a CLI
surfaces as warnings and an opt-in autofix consumes.

This is the inverse of [FR-026](./FR-026-intra-spec-reference-resolution.md):
[FR-026](./FR-026-intra-spec-reference-resolution.md) harvests edges from links that already exist; FR-039 finds references that
*should be* links and tells the author (or the autofix) how to make them so.
Bare prose codes are **never** turned into edges directly (ADR 0007); they become
edges only after FR-039's suggestion is applied.

### API

```rust
pub struct UnlinkedReference {
    pub path: PathBuf,            // document the token was found in
    pub source: ArtifactId,      // that document's own id
    pub token: String,           // matched id, e.g. "FR-008" or "FR-008-CON-4"
    pub byte_span: Range<usize>, // span in the document's on-disk text
    pub fix: UnlinkedFix,
}

pub enum UnlinkedFix {
    /// `token`'s parent id resolves to exactly one in-bundle artifact.
    /// `suggested_link` is the exact Markdown to splice over `byte_span`.
    AutoFix { suggested_link: String },
    /// Not safely fixable: resolves to nothing in-bundle, or to >1 artifact.
    WarnOnly { reason: UnlinkedReason },
}

pub enum UnlinkedReason { Unresolved, Ambiguous }

pub fn unlinked_references(spec: &Spec) -> Vec<UnlinkedReference>;
```

`unlinked_references` is exported from the crate root and operates over a loaded
`Spec` ([FR-025](./FR-025-spec-corpus-model.md)) so it can consult the id index
and the path→id index ([FR-026](./FR-026-intra-spec-reference-resolution.md)). Results are **deterministic**: sorted by
`(path, byte_span.start)`, independent of document/thread ordering ([NFR-006](../non-functional/NFR-006-determinism.md)).

### Token grammar and parent resolution

A candidate token matches `(FR|NFR|StR|US|IT|TC)-\d+(-(AC|CON)-\d+)?` — a
**known artifact-id prefix** followed by a number, optionally with an
acceptance-criterion or constraint suffix. The prefix set is explicit on
purpose: a generic `[A-Z]{2,4}-\d+` over-matches standards and notes that look
like ids but are not artifacts (`ISO-8601`, `IMPL-4`, `CR-002`) and bare sub-ids
(`CON-1`, `AC-2`) whose parent is ambiguous. `-AC-`/`-CON-` are matched only as a
suffix of a parent id ([FR-008-CON-4](./FR-008-byte-exact-slicing.md)), never standalone. `StR` is intentionally
mixed-case. The **parent id**
is the token with any `-(AC|CON)-\d+` suffix stripped ([FR-008-CON-4](./FR-008-byte-exact-slicing.md) →
[FR-008](./FR-008-byte-exact-slicing.md)); resolution and the suggested link's destination key off the parent id.
The suggested link's **visible label is the full token** and its **destination
is the parent artifact's file**, as a path relative to the source document's
directory (anchors to a specific AC/CON row are future work, ADR 0007):

```
FR-008-CON-4   →   [FR-008-CON-4](./FR-008-byte-exact-slicing.md)
```

### Classification — three buckets

For each candidate occurrence:

- **Ignore** (no finding): the token is inside a fenced code block (```` ``` ````
  / `~~~`), inside frontmatter, already inside a Markdown link (as link text
  `[FR-008](…)` or inside a link destination such as [FR-008](./FR-008-byte-exact-slicing.md)), or it is a
  **self-reference** — its parent id equals the document's own id (the H1 title,
  the `id:` field, and the document's own `…-AC-*` / `…-CON-*` definition rows are
  definitions, not references).
- **Auto-fix**: a candidate (prose, table cell, or **inline-code span**) whose
  parent id resolves to **exactly one** loaded artifact other than the document
  itself → `UnlinkedFix::AutoFix { suggested_link }`.
- **Warn-only**: a candidate whose parent id resolves to **nothing** in the
  loaded set (`UnlinkedReason::Unresolved` — likely cross-repo and needing a
  manual `ix://` reference, a typo, or a not-yet-written artifact) or to **more
  than one** loaded artifact (`UnlinkedReason::Ambiguous`). No `suggested_link`;
  the autofix abstains.

### Inline-code conversion

When a candidate sits in an **inline-code span** (`` `[FR-008](./FR-008-byte-exact-slicing.md)` ``), the
`byte_span` covers the **entire span including the backticks**, and the
`suggested_link` is a plain Markdown link with the backticks removed —
`` `[FR-008](./FR-008-byte-exact-slicing.md)` `` → `[FR-008](./FR-008-byte-exact-slicing.md)` — so applying the fix
replaces the code span with a link rather than nesting a link inside code.
**Fenced** code blocks are never candidates.

A code span holding **more than one** artifact token (e.g. `` `FR-012/FR-013` ``)
is left **unconverted** — no fix is emitted for any of its tokens. Converting it
to a single link is ambiguous, and emitting one whole-span fix per token would
produce **overlapping** byte spans that corrupt on apply. Such tokens fall into
the Ignore bucket.

### Idempotence

Applying every `AutoFix` suggestion and re-running `unlinked_references` SHALL
yield **no** `AutoFix` findings for the converted tokens: a token that is now
link text or a link destination falls into the Ignore bucket. Warn-only findings
are unaffected by autofix (nothing is applied for them).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-039-AC-1 | A bare [FR-008](./FR-008-byte-exact-slicing.md) in an artifact's prose, where [FR-008](./FR-008-byte-exact-slicing.md) is loaded in the corpus, yields exactly one `UnlinkedReference` with `fix = AutoFix`, `byte_span` covering the token, and `suggested_link` `[FR-008](<rel-path-to-FR-008-file>)`. | Test |
| FR-039-AC-2 | A sub-id token [FR-008-CON-4](./FR-008-byte-exact-slicing.md) yields an `AutoFix` whose `suggested_link` label is the full token [FR-008-CON-4](./FR-008-byte-exact-slicing.md) and whose destination is the **parent** [FR-008](./FR-008-byte-exact-slicing.md) file's relative path. | Test |
| FR-039-AC-3 | An inline-code candidate `` `[FR-008](./FR-008-byte-exact-slicing.md)` `` yields an `AutoFix` whose `byte_span` covers the whole code span (backticks included) and whose `suggested_link` is a plain link with no backticks (`[FR-008](…)`). | Test |
| FR-039-AC-4 | A token inside a fenced ```` ``` ```` block, and a token appearing only in frontmatter, each yield **no** finding. | Test |
| FR-039-AC-5 | A token already inside a Markdown link — as link text `[FR-008](./FR-008-….md)` or inside an [FR-008](./FR-008-byte-exact-slicing.md) destination — yields no finding; re-running `unlinked_references` after applying all `AutoFix` suggestions yields no `AutoFix` finding for the converted tokens (idempotence). | Test |
| FR-039-AC-6 | In [FR-024](./FR-024-parallel-repo-walk.md)'s own document, its H1 [FR-024](./FR-024-parallel-repo-walk.md), its [FR-024](./FR-024-parallel-repo-walk.md) frontmatter, and its own [FR-024-AC-1](./FR-024-parallel-repo-walk.md) table rows yield no findings (self-reference), while a reference to a different artifact [FR-008](./FR-008-byte-exact-slicing.md) in the same document does yield a finding. | Test |
| FR-039-AC-7 | A token whose parent id is absent from the loaded set yields `fix = WarnOnly { reason: Unresolved }` with no `suggested_link`; the autofix abstains (nothing in the result is applied for it). | Test |
| FR-039-AC-8 | A token whose parent id maps to more than one loaded document (duplicate ids) yields `fix = WarnOnly { reason: Ambiguous }` with no `suggested_link`. | Test |
| FR-039-AC-9 | `unlinked_references` results are sorted by `(path, byte_span.start)` and are identical across repeated runs and thread counts for a given loaded set ([NFR-006](../non-functional/NFR-006-determinism.md) determinism). | Test |
| FR-039-AC-10 | A single inline-code span holding more than one artifact token (e.g. `` `FR-008/FR-009` ``) yields **no** finding for any of those tokens (avoids overlapping whole-span fixes); a single-token code span still yields its `AutoFix`. | Test |
| FR-039-AC-11 | Every declared sub-id kind resolves to its parent document: `-AC-`, `-CON-` and `-VC-` (StR's validation-criterion kind, CR-020) are each stripped to the parent id, a plain artifact id is unchanged, and a token that merely resembles a sub-id is not stripped. The token regex recognizes the same kinds, so a bare `StR-001-VC-2` in prose is matched whole and its `AutoFix` replaces the whole token — matching only `StR-001` would link the parent and leave a dangling `-VC-2`. | Test (TC-765) |

## Dependencies

- **Upstream**: [FR-025](./FR-025-spec-corpus-model.md) (corpus + id index), [FR-026](./FR-026-intra-spec-reference-resolution.md) (path→id index), [FR-008](./FR-008-byte-exact-slicing.md) (byte-exact spans), ADR 0007
- **Downstream**: quire-cli `fix` surface (applies `AutoFix` via writeback)
