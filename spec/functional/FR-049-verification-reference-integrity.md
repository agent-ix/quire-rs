---
id: FR-049
title: "Verification-Reference Integrity"
type: FR
relationships:
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-038"
    type: "extends"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/functional/FR-050"
    type: "requires"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/spec/usecase/US-017"
    type: "implements"
---
# FR-049: Verification-Reference Integrity

## Description

Bundle validation ([FR-038](./FR-038-okf-bundle-validation.md)) SHALL check
that document references minted inside table cells — e.g. the ISO AC
`Verification` cell annotation `Test (TC-035)` — resolve to a real target, the
same way `ix://` edges are checked for dangling targets today
(`dangling-reference` in `src/corpus/validate.rs`).

The engine SHALL NOT hardcode the annotation pattern, the referencing column,
or the target archetype: all three come from the active module's declared
**traceability model** ([FR-050](./FR-050-declarative-coverage-computation.md)
`traceability:` manifest section). For each *document-reference declaration* in
that model (archetype + section + column + annotation pattern + target kinds),
`validate_bundle` SHALL extract every referenced id from the declared cells.
The engine SHALL resolve each extracted id against the bundle's **resolution
set** for the declaration's target kinds.

The resolution set for a target kind SHALL be the union of:

- ids of bundle documents whose archetype matches the declared target
  archetype; and
- trace ids minted by a declared auxiliary trace source — a file (e.g. the
  repo Test Matrix, `spec/tests.md`) that the corpus walk excludes as a
  non-artifact ([FR-038](./FR-038-okf-bundle-validation.md)) but that the
  model declares as minting trace ids. The engine SHALL harvest such files
  with a targeted scan, following the glossary-harvester pattern
  ([FR-044](./FR-044-project-glossary-lexicon.md)).

If a referenced id resolves to no member of the resolution set, then
`validate_bundle` SHALL report a `dangling-trace-reference` finding carrying
the referencing document path, the unresolved id, and the declaration that
minted the reference. The finding SHALL be posture-degradable exactly like
`dangling-reference`: an error under the `Strict` posture and a warning under
`Okf`.

When the active modules declare no traceability model, the check SHALL emit no
findings.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-049-AC-1 | A bundle whose AC `Verification` cell references a `TC` id present in the bundle (a TC document or a declared trace-source row) validates with no `dangling-trace-reference` finding. | Test (TC-724) |
| FR-049-AC-2 | A `Verification` cell referencing a TC id absent from the resolution set yields one `dangling-trace-reference` finding carrying the document path and the unresolved id. | Test (TC-725) |
| FR-049-AC-3 | The finding is posture-degradable: the same dangling reference is an error under `Strict` and a warning under `Okf`. | Test (TC-726) |
| FR-049-AC-4 | The annotation pattern and referencing column come from the declared model: a fixture module declaring a different pattern/column resolves references by its own declaration, with no ISO-specific behavior in the engine. | Test (TC-727) |
| FR-049-AC-5 | A declared auxiliary trace source outside the corpus walk (a `tests.md`-style matrix) contributes its minted trace ids to the resolution set via a targeted scan. | Test (TC-728) |
| FR-049-AC-6 | With no traceability model declared by any active module, `validate_bundle` emits zero `dangling-trace-reference` findings for any input. | Test (TC-729) |
| FR-049-AC-7 | A cell bearing multiple annotations (`Test (TC-035, TC-036)`) resolves each id independently and reports only the unresolved ones. | Test (TC-730) |
| FR-049-AC-8 | Findings are deterministic: repeated validation of the same bundle yields the same findings in the same order. | Test (TC-731) |
| FR-049-AC-9 | With the corpus walked from a document root nested under the scope, model-declared `document:` paths and `exclude:` globs resolve against the **reference root** (the scope), not the document root: a `document: spec/tests.md` target mints when `validate_bundle` receives the two roots separately, and un-mints — every reference to it dangling — when the roots are conflated (CR-045). | Test (TC-814) |

> **CR-045 note (2026-08-15):** `validate_bundle` gains the same two-root
> split `compute_coverage` has always had via its `root` parameter: a
> `document_root` (locates the root `index.md`) and a `reference_root` (the
> base for the model's `document:`/`exclude:` paths, which modules author
> against the repository scope — `spec/tests.md`). Found while landing the
> #91 CLI derivation: bounding the OKF bundle root to `<scope>/spec` with a
> single conflated root silently un-minted every path-bound trace target,
> surfacing as 123 new `dangling-trace-reference` findings on this repo's
> own spec. `validate_bundle_at` keeps single-root semantics for
> self-contained bundles (agent-ix/quire-rs#91, umbrella #90).

## Dependencies

- **Upstream**: [FR-050](./FR-050-declarative-coverage-computation.md) (the declared traceability model), [FR-038](./FR-038-okf-bundle-validation.md) (bundle validation and postures), [FR-025](./FR-025-spec-corpus-model.md) (the corpus)
- **Downstream**: `spec-artifacts-iso` declares the ISO `Verification`-cell model (follow-up change in that module); the `gap-analysis` workflow consumes the findings
