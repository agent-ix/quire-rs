// FR-058 ecosystem sweep (agent-ix/quire-rs#85): how many findings does the
// upward-trace check land, before anyone decides its severity.
//
// The declaration comes from `tests/fixtures/traceability/required-relations`
// rather than from the iso module, because the iso hop cannot land until an
// engine carrying `required_relations` is released — `TraceabilityModel` is
// `deny_unknown_fields` on the Rust side and `additionalProperties: false` in
// the iso schema, so a module declaring the key against today's release fails
// to load. The corpus being measured is the real one either way; only the
// carrier of the declaration is a fixture.
//
// Run: cargo run --release --example fr058_upward_trace_sweep
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use quire_rs::{validate_bundle_at, BundlePosture, Registry};

const EXCLUDED: [&str; 2] = ["filament-ide", "electron-hello"];

fn main() {
    let module = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/traceability/required-relations");
    let registry = Registry::load_module(&module).expect("load the required-relations fixture");

    let mut per_check: BTreeMap<String, usize> = BTreeMap::new();
    let mut per_repo: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut examples: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut repos_with_findings = 0usize;
    let mut scanned = 0usize;

    for repo in sweep_repos() {
        let name = repo
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let report = validate_bundle_at(&repo.join("spec"), &registry, BundlePosture::Okf);
        scanned += 1;

        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for f in report.errors.iter().chain(report.warnings.iter()) {
            let reason = f.reason.to_string();
            if !matches!(
                reason.as_str(),
                "orphan-fr" | "unimplemented-str" | "cyclic-refines"
            ) {
                continue;
            }
            *counts.entry(reason.clone()).or_default() += 1;
            *per_check.entry(reason.clone()).or_default() += 1;
            let ex = examples.entry(reason).or_default();
            if ex.len() < 5 {
                ex.push(format!("{name}: {}", f.message));
            }
        }
        if !counts.is_empty() {
            repos_with_findings += 1;
            per_repo.insert(name, counts);
        }
    }

    println!("repos scanned: {scanned}");
    println!("repos with at least one finding: {repos_with_findings}");
    let total: usize = per_check.values().sum();
    println!("total findings: {total}");
    for (check, n) in &per_check {
        println!("  trace:{check}: {n}");
    }

    println!("\n--- per-repo, top 20 by finding count ---");
    let mut rows: Vec<(usize, &String, &BTreeMap<String, usize>)> = per_repo
        .iter()
        .map(|(name, counts)| (counts.values().sum::<usize>(), name, counts))
        .collect();
    rows.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(b.1)));
    for (n, name, counts) in rows.iter().take(20) {
        let detail: Vec<String> = counts.iter().map(|(k, v)| format!("{k}={v}")).collect();
        println!("  {name}: {n}  [{}]", detail.join(", "));
    }

    println!("\n--- samples ---");
    for (check, ex) in &examples {
        println!("[{check}]");
        for e in ex {
            println!("   {e}");
        }
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
    let mut repos: Vec<_> = std::fs::read_dir(&workspace)
        .unwrap_or_else(|e| panic!("read workspace {workspace}: {e} (set IX_WORKSPACE)"))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join("spec").is_dir())
        .collect();
    repos.sort();
    repos.retain(|p| {
        let n = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
        !n.contains("-task") && n != "worktrees" && !EXCLUDED.contains(&n)
    });
    repos
}
