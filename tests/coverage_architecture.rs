//! Static dependency boundaries for the #360 coverage and corpus decomposition.

use ix_trace_rs::trace;

#[trace("TC-1059", "FR-050-CON-1")]
#[trace("TC-1059", "FR-050-CON-2")]
#[test]
fn tc1059_coverage_and_corpus_modules_keep_one_responsibility() {
    let declarations = include_str!("../src/coverage/declarations.rs");
    let diagnostics = include_str!("../src/coverage/diagnostics.rs");
    let binding = include_str!("../src/coverage/binding_diagnostics.rs");
    let reconcile = include_str!("../src/coverage/reconcile.rs");

    assert!(!declarations.contains("CoverageDiagnostic"));
    assert!(!diagnostics.contains("SymbolGraph"));
    assert!(!binding.contains("declared_tables"));
    assert!(!reconcile.contains("serde_json"));

    let metadata = include_str!("corpus_case/mod.rs");
    let loading = include_str!("corpus_case/loading.rs");
    let execution = include_str!("corpus_case/execution.rs");
    let grading = include_str!("corpus_case/grading.rs");

    assert!(!metadata.contains("coverage::compute"));
    assert!(!loading.contains("pub fn grade"));
    assert!(!loading.contains("coverage::compute"));
    assert!(!execution.contains("CaseExpect"));
    assert!(!grading.contains("read_to_string"));
}
