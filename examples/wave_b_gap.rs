//! Wave B gap analysis (SR-048), run with **this branch's** engine.
//!
//! Deliberately not the installed `quire` CLI: that is 0.22.0 against this
//! crate's 0.30.1, so it predates the reconciliation being audited and would
//! report on code that is not the code under review.
//!
//! ```text
//! cargo run --example wave_b_gap
//! QUIRE_PROCESS_MODULE=/path/to/spec_artifacts_process cargo run --example wave_b_gap
//! ```
use std::collections::BTreeSet;
use std::path::Path;

fn main() {
    let module = std::env::var("QUIRE_PROCESS_MODULE").unwrap_or_else(|_| {
        format!(
            "{}/../spec-artifacts-process/spec_artifacts_process",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    let registry = quire_rs::Registry::load_module(Path::new(&module))
        .unwrap_or_else(|e| panic!("load {module}: {e}"));
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let spec = quire_rs::Spec::from_path(root);
    let extraction = quire_rs::symbols::extract_tree(root);
    let model = registry.traceability().expect("model");
    let graph = quire_rs::symbols::trace::bind(&extraction, model);
    let report = quire_rs::coverage::compute(&spec, &registry, &graph, root).expect("coverage");
    let backed: BTreeSet<&str> = graph.backed_trace_ids();

    println!(
        "rollup: {} backed / {} rows",
        report.totals.backed, report.totals.total
    );

    // The engine reconciles these itself (FR-050); this reads its answer.
    let wave_b = |id: &str| (898..=909).any(|n| id == format!("TC-{n}"));
    println!("unbacked rows repo-wide: {}", report.unbacked_rows.len());
    println!(
        "no-symbol rows (CR-041 exempt by declared method): {}",
        report.no_symbol_rows.len()
    );
    println!(
        "untracked symbols (tagged test, no matrix row): {}",
        report.untracked_symbols.len()
    );
    println!("status lies repo-wide:   {}", report.status_lies.len());
    let wb_unbacked: Vec<&str> = report
        .unbacked_rows
        .iter()
        .filter_map(|r| r.row_id.as_deref())
        .filter(|id| wave_b(id))
        .collect();
    let wb_lies: Vec<&str> = report
        .status_lies
        .iter()
        .filter_map(|r| r.row_id.as_deref())
        .filter(|id| wave_b(id))
        .collect();
    println!("  of which Wave B unbacked: {wb_unbacked:?}");
    println!("  of which Wave B lies:     {wb_lies:?}");

    print!("Wave B backing:");
    for n in 898..=909 {
        let id = format!("TC-{n}");
        if !backed.contains(id.as_str()) {
            print!(" {id}=UNBACKED");
        }
    }
    println!(" all backed");
    println!("\n== the 15 untracked symbols (a tagged test with no matrix row) ==");
    for u in &report.untracked_symbols {
        println!("  {:?}", u);
    }
}
