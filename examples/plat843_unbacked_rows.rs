use quire_rs::symbols::{self, trace};
use quire_rs::traceability::SourceLanguage;
use quire_rs::{coverage, Registry, Spec};
use std::path::{Path, PathBuf};

/// Dumps `coverage::compute`'s `unbacked_rows`, one line per row, sorted for a
/// stable diff: `document\trow_id\ttarget_ids\treference`. Used to decompose
/// an aggregate `unbacked_rows` delta into the exact rows that moved, by
/// running this against two engine builds over the same repo and diffing the
/// output — no reimplementation of `coverage::compute`'s internals needed.
fn main() {
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("usage: plat843_unbacked_rows <repo_root> [module_path]"),
    );
    let module_path = std::env::args().nth(2).unwrap_or_else(|| {
        std::env::var("PLAT840_MODULE").unwrap_or_else(|_| {
            "/home/peter/dev/spec-artifacts-process/spec_artifacts_process".to_string()
        })
    });

    let registry = Registry::load_module(Path::new(&module_path))
        .unwrap_or_else(|e| panic!("load module {module_path}: {e}"));
    let model = registry
        .traceability()
        .unwrap_or_else(|| panic!("module {module_path} declares no traceability model"));
    let source_exclude_globs = model.source_exclude.clone();

    let spec = Spec::from_path(&root.join("spec"));
    let extraction =
        symbols::extract_tree_scoped(&root, &[Path::new("spec")], &source_exclude_globs);
    let rust_only = extraction
        .symbols
        .iter()
        .filter(|s| s.language == SourceLanguage::Rust)
        .count();
    eprintln!("rust symbols extracted: {rust_only}");

    let graph = trace::bind(&extraction, model);
    let report = coverage::compute(&spec, &registry, &graph, &root)
        .unwrap_or_else(|e| panic!("coverage::compute: {e}"));

    let mut lines: Vec<String> = report
        .unbacked_rows
        .iter()
        .map(|r| {
            format!(
                "{}\t{}\t{}\t{}",
                r.document,
                r.row_id.clone().unwrap_or_default(),
                r.target_ids.join(","),
                r.reference
            )
        })
        .collect();
    lines.sort();
    println!("unbacked_rows: {}", lines.len());
    for l in lines {
        println!("{l}");
    }
}
