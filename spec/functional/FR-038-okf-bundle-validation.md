---
id: FR-038
title: "OKF Bundle Validation: Strict vs Okf Postures + Index Completeness"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-037"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-025"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-026"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-032"
    type: "requires"
    cardinality: "1:1"
---

## Description

`quire-rs` SHALL validate a whole **bundle** — a directory tree of authored
markdown documents loaded as a `Spec` corpus ([FR-025](./FR-025-spec-corpus-model.md)) — under one of two
postures, answering two different questions:

```rust
pub enum BundlePosture { Strict, Okf }

pub struct BundleFinding { pub path: PathBuf, pub message: String, pub reason: &'static str }
pub struct BundleReport { pub errors: Vec<BundleFinding>, pub warnings: Vec<BundleFinding> }
impl BundleReport { pub fn is_valid(&self) -> bool; }   // == errors.is_empty()

pub fn validate_bundle(spec: &Spec, registry: &Registry, posture: BundlePosture, root: &Path) -> BundleReport;
pub fn validate_bundle_at(root: &Path, registry: &Registry, posture: BundlePosture) -> BundleReport;
```

`validate_bundle_at` loads `root` into a `Spec` ([FR-025](./FR-025-spec-corpus-model.md)) then calls
`validate_bundle`. A bundle `is_valid()` for its posture iff it has no hard
errors. `BundlePosture`, `BundleReport`, `BundleFinding`, `validate_bundle`, and
`validate_bundle_at` are exported from the crate root.

`index.md` keeps its archetype and is validated like any other
document. Corpus membership follows frontmatter presence, regardless of filename
([FR-024](./FR-024-parallel-repo-walk.md)).

### `type` is required in BOTH postures

Every document in a bundle MUST carry a non-empty `type` regardless of posture.
A document with a missing or empty `type` is a **hard error** (reason
`frontmatter`) under both Strict and Okf. This is the OKF-adoption change: an
untyped corpus document was previously surfaced only as a non-fatal
`Diagnostic::UntypedArtifact` warning by the `Spec` indexer ([FR-024-AC-6](./FR-024-parallel-repo-walk.md) /
[FR-027-AC-9](./FR-027-whole-spec-query-api.md)); the bundle validator now **promotes "untyped" to an error**. The
indexer still records its warning diagnostic for coverage audits; the new strict
bundle validator is the layer that rejects the document.

### Strict posture

`Strict` answers "is this one of *our* archetype-conformant specs?" For every
document it requires:

- a **known** `type` — one with a registered archetype; an unregistered type is
  an error (reason `unknown-type`);
- satisfaction of the base concept contract ([FR-037](./FR-037-base-concept-schema.md) `validate_base_concept`, so
  mistyped `description`/`tags` are rejected) **and** the document's archetype:
  frontmatter schema + `body_extraction` + heading uniqueness via the existing
  `validate_document` ([FR-032](./FR-032-validate-document.md));
- resolvable `ix://` references — a dangling reference is an error (reason
  `dangling-reference`);
- index completeness (below).

### Okf posture (permissive)

`Okf` answers "can we read this *foreign* OKF bundle?" `type` is **still**
required and non-empty, but:

- **unknown** types are tolerated (warning, reason `unknown-type`);
- broken `ix://` / relative references degrade to warnings;
- archetype body contracts are **not** enforced;
- index gaps degrade to warnings.

### Index completeness

Folded into `validate_bundle`: for every directory containing an `index.md`,
every sibling artifact `.md` MUST appear in that index's `## Contents`. Only
`index.md` is exempt because an index cannot be a sibling of itself
(CR-044). Additionally, the
**bundle-root** `index.md` MUST declare `okf_version` in frontmatter; subdirectory
indexes need not. A missing sibling (reason `index-incomplete`) or a missing
root `okf_version` (reason `index-okf-version`) is an error under Strict and a
warning under Okf.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-038-AC-1 | Under `Strict`, a bundle containing a document with no `type` field is `!is_valid()`, with an error whose reason is `frontmatter` and whose message names `type` (untyped is a hard error, not a warning). | Test |
| FR-038-AC-2 | Under `Okf`, the same untyped document is still an error (`!is_valid()`, reason `frontmatter`) — `type` is required even in the permissive posture. | Test |
| FR-038-AC-3 | Under `Okf`, a document with an unregistered `type` and a dangling `ix://` reference yields `is_valid()` with warnings `unknown-type` and `dangling-reference`; under `Strict` the same bundle is `!is_valid()` with both as errors. | Test |
| FR-038-AC-4 | Under `Strict`, a bundle whose documents all carry a known, archetype-conformant `type`, whose root `index.md` lists every sibling and declares `okf_version`, validates with no errors (`is_valid()`). | Test |
| FR-038-AC-5 | A directory whose `index.md` omits a sibling artifact yields an `index-incomplete` finding naming the missing file — an error under `Strict`, a warning under `Okf`. A typed `tests.md` is a sibling artifact for this purpose; only `index.md` is exempt (CR-044). | Test |
| FR-038-AC-6 | A bundle-root `index.md` lacking `okf_version` in frontmatter yields an `index-okf-version` error under `Strict`. | Test |
| FR-038-AC-7 | A subdirectory `index.md` without `okf_version` does not produce an `index-okf-version` finding (only the bundle root must declare it); an otherwise-conformant nested bundle is `is_valid()` under `Strict`. | Test |
| FR-038-AC-8 | Under `Strict`, a document with a known `type` but a mistyped optional `description` (e.g. `description: 7`) is `!is_valid()` with an error naming `description` (the base concept contract, [FR-037](./FR-037-base-concept-schema.md), runs as part of bundle validation). | Test |
| FR-038-AC-9 | An `ix://<org>/<repo>/<...>/<id>` reference (at least three path segments) to an artifact absent from the bundle is NOT reported as `dangling-reference` when the bundle declares its package (`<org>/<repo>`) and the reference's authority — its first two path segments joined with `/` — differs from that package under ASCII case-insensitive comparison (GitHub org and repo names are case-insensitive). The same absent target under the declared package's own authority, any absent target when no package is declared, a two-segment `ix://<org>/<id>` reference (which names no repo, so has no authority), and a bare-id target are still `dangling-reference`. A target spelled both as a bare id and as a foreign `ix://` link is one edge and stays dangling. | Test (TC-1992) |
| FR-038-AC-10 | The authority-versus-package comparison of AC-9 folds ASCII case: with the declared package `acme/widgets`, an absent `ix://Acme/widgets/X` target (the bundle's own repo, mixed case) is still `dangling-reference`, an absent `ix://Other/Repo/X` target (foreign, mixed case) is still skipped, and with no package declared every absent target stays `dangling-reference`. `Spec::with_bundle_package` stores the package as given; the comparison alone folds, so no normalisation of the declared value is observable. | Test (TC-2151) |

> **PLAT-1056 note:** AC-9 is new. `quire validate --okf` reported every correct
> cross-repo `ix://` target as dangling, because this corpus cannot see a
> foreign repository. Skipping needs the bundle's own package declared; with
> none, nothing separates a foreign target from a broken local one, so it stays
> dangling.

> **PLAT-1357 note:** AC-10 is new. quire-cli lowercases the declared package, so a
> mixed-case link to the bundle's own repo was wrongly skipped as cross-repo
> under the byte-for-byte comparison of AC-9.

## Dependencies

- **Upstream**: [FR-037](./FR-037-base-concept-schema.md) (requires), [FR-025](./FR-025-spec-corpus-model.md) (requires), [FR-026](./FR-026-intra-spec-reference-resolution.md) (requires), [FR-032](./FR-032-validate-document.md) (requires)
- **Downstream**: none
