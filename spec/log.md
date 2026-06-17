---
type: log
title: "Update Log"
description: "Chronological log of structural changes to this bundle."
---
# Update Log

## History

* **2026-06-15** — Adopted OKF-compatible bundle structure with directory indexes.
* **2026-06-16** — Added FR-037 (base concept frontmatter schema: required non-empty `type` + optional typed `description`/`tags`; `validate_base_concept` / `validate_concept_shape`) and FR-038 (OKF bundle validation: Strict vs Okf postures, untyped-corpus-doc-is-error, index completeness + root `okf_version`). Added a CR note to FR-027 clarifying the bundle validator promotes the corpus-level non-fatal `UntypedArtifact` diagnostic to a hard error. Matrix: TC-590..596, TC-600..607 at 100% AC→TC coverage (309 ACs).
* **2026-06-16** — Extended FR-032 with composed type+object validation (CR note + AC-11..13): `validate_document_in_registry(registry, archetype, doc_text)` validates BOTH the `type` archetype AND the frontmatter `object:` archetype; resolved-object `body_extraction` failures merge into `errors`, an unknown `object:` is a `unknown-object-type` **warning**, and `ValidationResult` gains a typed `warnings: Vec<ValidationWarning>` field. The 2-arg `validate_document` stays the type-only path. Matrix: TC-610..613 at 100% AC→TC coverage (314 ACs).
* **2026-06-17** — Internal-links slice (ADR 0007: internal references are relative-path links, `ix://` is external-only). Added ADR 0007. Extended FR-026 with a third edge-stub source — relative-path body links resolved via a path→id index — plus index/log source-exclusion and cross-source dedup parity (CR note + AC-9..11). Added FR-039 (unlinked-reference detection & autofix suggestions): `unlinked_references(spec) -> Vec<UnlinkedReference>` classifies bare artifact-id tokens into AutoFix / WarnOnly / Ignore, carrying the exact relative-path link to splice; bare prose codes are never harvested directly. Matrix: TC-620..631 at 100% AC→TC coverage (326 ACs).
