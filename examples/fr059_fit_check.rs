//! [RAN] FR-059 fit-check (agent-ix/quire-rs#162).
//!
//! ONE check per sweep, as the ticket requires: running this alongside FR-058
//! would make both baselines unattributable. Uses a purpose-built module whose
//! only declaration is the vocabulary coverage, so nothing else can fire.
//!
//! ```text
//! cargo run --release --example fr059_fit_check
//! IX_WORKSPACE=/path/to/dev cargo run --release --example fr059_fit_check
//! ```
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn main() {
    let module = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/traceability/vocabulary-coverage-25010");
    let registry = quire_rs::Registry::load_module(&module)
        .unwrap_or_else(|e| panic!("load {}: {e}", module.display()));

    let mut per_value: BTreeMap<String, usize> = BTreeMap::new();
    let mut per_repo: BTreeMap<String, usize> = BTreeMap::new();
    let mut scanned = 0usize;
    let mut with_nfr = 0usize;

    for repo in sweep_repos() {
        let spec_dir = repo.join("spec");
        if !spec_dir.is_dir() {
            continue;
        }
        scanned += 1;
        let report =
            quire_rs::validate_bundle_at(&spec_dir, &registry, quire_rs::BundlePosture::Okf);
        let hits: Vec<&quire_rs::BundleFinding> = report
            .errors
            .iter()
            .chain(report.warnings.iter())
            .filter(|f| f.reason == "unowned-quality-characteristic")
            .collect();
        if hits.is_empty() {
            continue;
        }
        with_nfr += 1;
        let name = repo
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        per_repo.insert(name, hits.len());
        for h in hits {
            if let Some(v) = h.message.split('\'').nth(1) {
                *per_value.entry(v.to_string()).or_insert(0) += 1;
            }
        }
    }

    let total: usize = per_repo.values().sum();
    println!("repos with a spec/ tree scanned : {scanned}");
    println!("repos reporting >=1 finding     : {with_nfr}");
    println!("TOTAL findings                  : {total}");
    println!("\n-- per declared value (how many repos leave it unowned) --");
    for (value, n) in &per_value {
        println!("   {value:26} {n}");
    }
    let mut rows: Vec<(&usize, &String)> = per_repo.iter().map(|(k, v)| (v, k)).collect();
    rows.sort_by(|a, b| b.0.cmp(a.0).then(a.1.cmp(b.1)));
    println!("\n-- distribution --");
    let counts: Vec<usize> = rows.iter().map(|(n, _)| **n).collect();
    if !counts.is_empty() {
        let mut sorted = counts.clone();
        sorted.sort_unstable();
        println!(
            "   min={} median={} max={}",
            sorted[0],
            sorted[sorted.len() / 2],
            sorted[sorted.len() - 1]
        );
    }
    println!("\n-- top 10 repos --");
    for (n, name) in rows.iter().take(10) {
        println!("   {name}: {n}");
    }
}

fn sweep_repos() -> Vec<PathBuf> {
    let workspace = std::env::var("IX_WORKSPACE").unwrap_or_else(|_| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap_or(Path::new("."))
            .to_string_lossy()
            .into_owned()
    });
    let mut out: Vec<PathBuf> = std::fs::read_dir(&workspace)
        .unwrap_or_else(|e| panic!("read workspace {workspace}: {e} (set IX_WORKSPACE)"))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    out.sort();
    out
}
