// agent-ix/quire-rs#89 / CR-067 edge-harvest sweep: measure the `ix://`-in-a-
// code-span defect, and the orphan counts the fix moves, over the whole ~/dev
// corpus BEFORE and AFTER the masking change (CR-014 / FR-029 discipline).
//
// Deliberately module-free: `Spec::from_path` harvests edges and resolves them
// against the loaded set with no archetype registry involved, so this
// measurement cannot be skewed by which module happens to be installed (the
// `--module` / `IX_FILAMENT_MODULES_PATH` trap that shaped every P1 sweep).
//
// Run: cargo run --release --example issue89_edge_sweep
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use quire_rs::corpus::{Resolution, Spec};
use quire_rs::diagnostic::Diagnostic;

/// Repos excluded from every sweep, with the reason recorded so the exclusion
/// is auditable rather than folklore.
const EXCLUDED: [(&str, &str); 2] = [
    (
        "filament-ide",
        "superseded by filament-ide-rs; its spec/ is frozen legacy",
    ),
    (
        "electron-hello",
        "not a git repo; drifts between measurements",
    ),
];

fn main() {
    let repos = sweep_repos();

    let mut total_edges = 0usize;
    let mut total_dangling = 0usize;
    let mut backtick_targets: Vec<(String, String, String)> = Vec::new();
    let mut per_repo: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    // Upward-trace orphans, the #85 baseline this change moves: an FR with no
    // outgoing edge to any StR, and an StR nothing points at.
    let mut orphan_frs = 0usize;
    let mut orphan_strs = 0usize;
    let mut fr_total = 0usize;
    let mut str_total = 0usize;

    for repo in &repos {
        let name = repo_name(repo);
        let spec = Spec::from_path(&repo.join("spec"));
        if spec.is_empty() {
            continue;
        }

        let edges = spec.edges().len();
        let dangling = spec
            .edges()
            .iter()
            .filter(|e| e.resolution == Resolution::Dangling)
            .count();
        total_edges += edges;
        total_dangling += dangling;
        per_repo.insert(name.clone(), (edges, dangling));

        for d in spec.diagnostics() {
            if let Diagnostic::DanglingReference {
                source,
                target,
                edge_type,
            } = d
            {
                // The defect's signature: a target that is punctuation, which
                // no artifact id can be. A lone backtick is the reported shape;
                // the wider test catches any sibling the same unmasked scan
                // could mint.
                if !target.chars().any(|c| c.is_alphanumeric()) {
                    backtick_targets.push((name.clone(), source.clone(), target.clone()));
                    let _ = edge_type;
                }
            }
        }

        let frs = spec.by_type("FR");
        let strs = spec.by_type("StR");
        fr_total += frs.len();
        str_total += strs.len();
        for fr in &frs {
            let up = spec
                .outgoing(&fr.id)
                .iter()
                .any(|e| e.target.starts_with("StR-"));
            if !up {
                orphan_frs += 1;
            }
        }
        for s in &strs {
            if spec.referencing(&s.id).is_empty() {
                orphan_strs += 1;
            }
        }
    }

    println!("repos scanned: {}", repos.len());
    for (name, reason) in EXCLUDED {
        println!("  excluded {name}: {reason}");
    }
    println!("edges: {total_edges}");
    println!("dangling edges: {total_dangling}");
    println!(
        "dangling edges with a non-alphanumeric target: {}",
        backtick_targets.len()
    );
    for (repo, source, target) in &backtick_targets {
        println!("  {repo}: {source} -> {target:?}");
    }
    println!("FR documents: {fr_total}; with no outgoing StR edge: {orphan_frs}");
    println!("StR documents: {str_total}; with nothing referencing them: {orphan_strs}");

    println!("\n--- per-repo (edges, dangling), non-zero dangling only ---");
    for (name, (edges, dangling)) in &per_repo {
        if *dangling > 0 {
            println!("  {name}: {edges} edges, {dangling} dangling");
        }
    }
}

/// Every `~/dev/<repo>/spec` bundle, worktree copies and `-task<N>` clones
/// dropped the way `scripts/sweep_coverage.py` does, plus the named exclusions.
fn sweep_repos() -> Vec<PathBuf> {
    let mut repos: Vec<_> = std::fs::read_dir("/home/peter/dev")
        .expect("read ~/dev")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join("spec").is_dir())
        .collect();
    repos.sort();
    repos.retain(|p| {
        let n = repo_name(p);
        !n.contains("-task")
            && n != "worktrees"
            && !EXCLUDED.iter().any(|(excluded, _)| *excluded == n)
    });
    repos
}

fn repo_name(p: &Path) -> String {
    p.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}
