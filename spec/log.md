---
type: log
title: "Update Log"
description: "Chronological log of structural changes to this bundle."
---
# Update Log

## History

* **2026-06-15** — Adopted OKF-compatible bundle structure with directory indexes.
* **2026-06-16** — Added FR-037 (base concept frontmatter schema: required non-empty `type` + optional typed `description`/`tags`; `validate_base_concept` / `validate_concept_shape`) and FR-038 (OKF bundle validation: Strict vs Okf postures, untyped-corpus-doc-is-error, index completeness + root `okf_version`). Added a CR note to FR-027 clarifying the bundle validator promotes the corpus-level non-fatal `UntypedArtifact` diagnostic to a hard error. Matrix: TC-590..596, TC-600..607 at 100% AC→TC coverage (309 ACs).
