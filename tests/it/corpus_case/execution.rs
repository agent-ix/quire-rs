//! Execute the real coverage and validation paths for a loaded corpus case.

use std::path::Path;

use super::loading::{corpus_root, load_module_path};
use super::Case;

/// Run the real coverage path over a case's `input/`, **in place**.
///
/// No tempdir, no copy, no materialisation: the directory the harness reads is
/// the directory an operator reproduces with. The module is the SHARED one the
/// case names under `corpus/modules/`, and the code walk excludes `spec/`
/// exactly as the CLI does — which is what makes a `tests/` topology
/// expressible, since the input tree is now real rather than three hardcoded
/// directories.
pub fn run(case: &Case) -> quire_rs::CoverageReport {
    let input = case.input();
    // Resolve the shared module through the case declaration so `module` is a
    // load-bearing field (FR-065-AC-18; history in #266).
    let module = corpus_root().join("modules").join(&case.meta.module);
    // A module id names either one module (`manifest.yaml`) or a search path
    // containing several modules (FR-065-AC-18; history in #292).
    let registry = if module.join("manifest.yaml").is_file() {
        quire_rs::Registry::load_module(&module).unwrap_or_else(|e| {
            panic!(
                "{}: module `{}` failed to load from {}: {e}",
                case.meta.id,
                case.meta.module,
                module.display()
            )
        })
    } else {
        load_module_path(&module, &case.meta.id)
    };
    let spec = quire_rs::Spec::from_path(&input.join("spec"));
    let model = registry.traceability().cloned().unwrap_or_default();
    let extraction = quire_rs::symbols::extract_tree_scoped(
        &input,
        // `module/` is not excluded: the per-case copies are gone, and the
        // CLI invocation a case documents has no such exclusion either.
        &[Path::new("spec")],
        &model.source_exclude,
    );
    let graph = quire_rs::symbols::trace::bind(&extraction, &model);
    quire_rs::coverage::compute(&spec, &registry, &graph, &input)
        .unwrap_or_else(|e| panic!("{}: compute failed: {e}", case.meta.id))
}

/// Every `quire validate` finding over a case's own spec tree, as one string.
///
/// A second command, because a structural defect is not a coverage one — and
/// running only `coverage` is how a fixture came to assert exactly what the
/// healthy control asserts.
pub(super) fn validate_report(case: &Case) -> String {
    let input = case.input();
    let module = corpus_root().join("modules").join(&case.meta.module);
    let registry = if module.join("manifest.yaml").is_file() {
        quire_rs::Registry::load_module(&module)
    } else {
        quire_rs::Registry::load_from(&[&module])
    }
    .unwrap_or_else(|e| panic!("{}: module load failed: {e}", case.meta.id));

    let report = quire_rs::validate_bundle_at(
        &input.join("spec"),
        &registry,
        quire_rs::BundlePosture::Strict,
    );
    // Errors AND warnings: a structural defect's severity is the module's
    // decision, and a case asserting the finding should not also have to know
    // which bucket the declaration put it in.
    report
        .errors
        .iter()
        .chain(report.warnings.iter())
        .map(|f| format!("{}: [{}] {}", f.path.display(), f.reason, f.message))
        .collect::<Vec<_>>()
        .join("\n")
}
