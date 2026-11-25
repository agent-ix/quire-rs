# PLAT-1873 synthetic baseline

## Current behavior before this change

Source and existing test inspection at `quire-rs` origin/main `e38d5823` found:

- `BundleIndex` and its `BundleArtifact`/`ImportedArtifact` entries carry package, id, names and kind, but no target declaration source locus.
- `src/semantic/target.rs::resolve_target` accepts a qualified identity when its package is in `SemanticModule.imports`; it returns `Target::Imported` with only the identity. It does not check whether an explicitly loaded package inventory contains that id or what kind it declares.
- `resolve_reference` admits that imported result without applying the construct or `allowed_links` kind rule.
- `tests/it/semantic_systems.rs::systems_references_resolve_qualified_and_imported_names_of_the_admitted_kind` is the synthetic baseline: an imported `ix://agent-ix/fleet/dispatcher` resolves with no imported artifact inventory, as the explicitly unchecked legacy case.
- The same source has no representation for a successful reference's target source location. Existing semantic diagnostics can carry only the reference `sourceSpan`.

The PLAT-1873 regression case extends that synthetic setup with an explicit `importedArtifacts` inventory containing a target id, object kind and `sourceSpan`. Before implementation, that case has no target location output, does not reject a wrong-kind target, and does not distinguish an id absent from the selected inventory from an existing imported identity. This baseline is from source and test inspection; the focused pre-change test was not run.
